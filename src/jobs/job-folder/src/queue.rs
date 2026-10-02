//! The queue, in memory, in the same process as the menu that shows it.
//!
//! A *job folder* is any directory under the jobs root that holds a `job.sh`.
//! The script is the workflow; the folder around it is its queue:
//!
//! | on disk | |
//! |---|---|
//! | `job.sh` | run once per input, with `$INPUT` and friends set |
//! | `input/` | drop files here — each one is a run, in arrival order |
//! | `output/` | `$OUTPUT_DIR`, the script's to fill |
//! | `done/` | inputs whose run succeeded, moved here afterwards |
//! | `failed/` | inputs whose run failed or was stopped |
//! | `stdout.log`, `stderr.log` | every run's output, appended; a run that prints adds a header first |
//!
//! An input stays in `input/` while it runs, so `$INPUT_DIR` never moves under
//! the script, and anything still in `input/` when the app starts is simply
//! queued again: a file that never got moved out is a run that never finished,
//! which is all the state worth recovering. Retrying is dragging a file back.
//!
//! The intended way in is to copy to the jobs root first and then `mv` into
//! `input/` — a rename on one volume, so the file arrives whole. The checks in
//! [`Jobs::consider`] are light insurance for a direct copy.
//!
//! Whether a file in `input/` is queued, held, running or suspended is not
//! written anywhere — the only thing that needs to know is holding it here, as
//! a `Vec<Job>` behind a mutex. Pause is a `SIGSTOP` on the way back from the
//! click; reordering is moving an element.
//!
//! Files in one job folder run one at a time, oldest first, so a workflow's
//! logs never interleave and a folder of drops is worked through in order.
//! The Workers setting caps how many *folders* run at once.

use std::collections::{HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use job_core::clock;
use job_core::progress::parse_progress;

pub const SCRIPT: &str = "job.sh";
pub const INPUT: &str = "input";
pub const OUTPUT: &str = "output";
pub const DONE: &str = "done";
pub const FAILED: &str = "failed";
pub const STDOUT_LOG: &str = "stdout.log";
pub const STDERR_LOG: &str = "stderr.log";

/// How often the scheduler looks at itself. Only the queue's own bookkeeping —
/// starting a job when a slot frees, escalating a stop that was ignored — runs
/// on this; every command is applied the moment it is pressed.
const TICK: Duration = Duration::from_millis(250);

/// How often the job folders are scanned for new inputs.
const SCAN: Duration = Duration::from_secs(1);

/// How long an input's size and mtime must hold still before it is taken to
/// have finished arriving. Short, because inputs are expected to arrive by
/// rename; `$JOB_SETTLE` (seconds) overrides it for copying straight into
/// `input/` from a slow share.
const DEFAULT_SETTLE: Duration = Duration::from_secs(2);

/// How long a stopped job gets to exit on its own before it is killed.
const TERM_GRACE: Duration = Duration::from_secs(10);

const DEFAULT_CONCURRENCY: usize = 2;
const MAX_CONCURRENCY: usize = 8;

/// Longer log lines are cut before they reach a menu.
const MAX_LINE: usize = 160;

/// What a job is doing. Held here and nowhere else.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Waiting for a slot.
    Queued,
    /// Waiting, and skipped when a slot frees, until you say otherwise.
    Held,
    Running,
    /// Its process group is stopped. It keeps its slot: pausing an encode to
    /// get the machine back, only for the queue to start the next one in its
    /// place, would be the opposite of what was asked.
    Paused,
    Finished {
        ok: bool,
    },
}

impl Phase {
    pub fn active(self) -> bool {
        matches!(self, Phase::Running | Phase::Paused)
    }

    pub fn waiting(self) -> bool {
        matches!(self, Phase::Queued | Phase::Held)
    }

    pub fn finished(self) -> bool {
        matches!(self, Phase::Finished { .. })
    }
}

/// One input through one job folder's script.
#[derive(Clone, Debug)]
pub struct Job {
    pub id: u64,
    /// The job folder: the one holding `job.sh`.
    pub dir: PathBuf,
    /// The input file — in `input/` until the run ends, then in `done/` or
    /// `failed/`.
    pub input: PathBuf,
    pub phase: Phase,
    pub queued_at: SystemTime,
    pub started: Option<SystemTime>,
    pub finished: Option<SystemTime>,
    /// Parsed out of the last line the job printed, 0..1.
    pub progress: Option<f64>,
    pub last_line: Option<String>,
    pub last_output: Option<SystemTime>,
    /// The job's process group, while it has one. Signal this rather than the
    /// shell's pid, or the encoder underneath carries on regardless.
    pub pgid: Option<i32>,
    pub exit: Option<i32>,
    /// Set when the job ended for a reason its exit status doesn't explain —
    /// stopped by hand, or never started at all.
    pub note: Option<String>,
    /// When SIGTERM was sent, so an ignored stop can be escalated.
    stopping: Option<Instant>,
}

impl Job {
    /// The job folder's name — the workflow.
    pub fn workflow(&self) -> String {
        file_name(&self.dir)
    }

    pub fn file(&self) -> String {
        file_name(&self.input)
    }

    /// `workflow · file`, for a row.
    pub fn label(&self) -> String {
        format!("{} · {}", self.workflow(), self.file())
    }

    pub fn log_path(&self) -> Option<PathBuf> {
        let path = self.dir.join(STDOUT_LOG);
        path.is_file().then_some(path)
    }

    pub fn elapsed(&self) -> Option<Duration> {
        let started = self.started?;
        let end = self.finished.unwrap_or_else(SystemTime::now);
        Some(end.duration_since(started).unwrap_or_default())
    }

    pub fn since_finished(&self) -> Option<Duration> {
        Some(SystemTime::now().duration_since(self.finished?).unwrap_or_default())
    }
}

/// Something worth a banner, left for the UI thread to post. The queue does not
/// talk to Notification Centre itself: it runs on its own threads, and posting
/// from them would be the one part of this design that has to care which thread
/// it is on.
#[derive(Clone, Debug)]
pub enum Event {
    Finished { name: String, ok: bool },
}

pub struct Queue {
    /// Every job the app knows about, in order. For the ones waiting, the order
    /// *is* the priority: the scheduler takes the first startable entry, so
    /// moving an element is all reordering the queue amounts to.
    pub jobs: Vec<Job>,
    pub concurrency: usize,
    /// The whole queue held: running jobs carry on, nothing new starts.
    pub paused: bool,
    pub events: Vec<Event>,
    next_id: u64,
}

impl Queue {
    #[cfg(test)]
    pub fn running(&self) -> usize {
        self.jobs.iter().filter(|job| job.phase.active()).count()
    }

    pub fn queued(&self) -> usize {
        self.jobs.iter().filter(|job| job.phase.waiting()).count()
    }

    pub fn failures(&self) -> usize {
        self.jobs
            .iter()
            .filter(|job| job.phase == Phase::Finished { ok: false })
            .count()
    }

    fn find(&mut self, id: u64) -> Option<&mut Job> {
        self.jobs.iter_mut().find(|job| job.id == id)
    }

    fn index_of(&self, id: u64) -> Option<usize> {
        self.jobs.iter().position(|job| job.id == id)
    }

    /// True if `input` is already a job that hasn't finished.
    fn has_input(&self, input: &Path) -> bool {
        self.jobs
            .iter()
            .any(|job| !job.phase.finished() && job.input == input)
    }
}

/// One of the buttons on a row, or one of the items under it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verb {
    /// Suspend a running job, or hold a queued one back.
    Pause,
    /// The way back from either.
    Resume,
    Stop,
    /// To the front of the queue.
    Top,
    /// Run a finished job again.
    Retry,
}

impl Verb {
    fn code(self) -> u64 {
        match self {
            Verb::Pause => 0,
            Verb::Resume => 1,
            Verb::Stop => 2,
            Verb::Top => 3,
            Verb::Retry => 4,
        }
    }

    fn from_code(code: u64) -> Option<Verb> {
        Some(match code {
            0 => Verb::Pause,
            1 => Verb::Resume,
            2 => Verb::Stop,
            3 => Verb::Top,
            4 => Verb::Retry,
            _ => return None,
        })
    }
}

/// A row button carries one integer back to the app, so a job and a verb are
/// packed into one. Ids are a counter, so the room this leaves is not a limit
/// anything can reach.
pub fn token(id: u64, verb: Verb) -> u64 {
    (id << 3) | verb.code()
}

pub fn untoken(token: u64) -> Option<(u64, Verb)> {
    Verb::from_code(token & 0b111).map(|verb| (token >> 3, verb))
}

/// What an input looked like when last seen: if this holds still for the
/// settle window, the file has finished arriving.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Fingerprint {
    size: u64,
    modified: Option<SystemTime>,
}

impl Fingerprint {
    fn of(path: &Path) -> Option<Self> {
        let meta = fs::metadata(path).ok()?;
        meta.is_file().then(|| Self {
            size: meta.len(),
            modified: meta.modified().ok(),
        })
    }
}

/// The scanner's memory: files seen in an `input/` but not yet queued.
#[derive(Default)]
struct Arrivals {
    /// When each candidate's fingerprint last changed.
    pending: HashMap<PathBuf, (Fingerprint, Instant)>,
    /// Inputs whose run ended but which could not be moved out of `input/`.
    /// Skipped until they go, or they would be run again every scan.
    stuck: HashSet<PathBuf>,
}

/// The queue and the folder it draws its work from.
pub struct Jobs {
    pub root: PathBuf,
    settle: Duration,
    state: Mutex<Queue>,
    arrivals: Mutex<Arrivals>,
}

impl Jobs {
    /// Start the two threads that keep the queue moving: one scanning the job
    /// folders, one starting what they find.
    pub fn start(root: PathBuf) -> Arc<Self> {
        Self::start_with(root, configured_settle())
    }

    fn start_with(root: PathBuf, settle: Duration) -> Arc<Self> {
        let _ = fs::create_dir_all(&root);
        // Absolute, because it ends up in every script's environment.
        let root = fs::canonicalize(&root).unwrap_or(root);

        let jobs = Arc::new(Self {
            root,
            settle,
            state: Mutex::new(Queue {
                jobs: Vec::new(),
                concurrency: configured_concurrency(),
                paused: false,
                events: Vec::new(),
                next_id: 1,
            }),
            arrivals: Mutex::new(Arrivals::default()),
        });

        let scheduler = Arc::clone(&jobs);
        thread::spawn(move || {
            loop {
                scheduler.schedule();
                thread::sleep(TICK);
            }
        });

        let scanner = Arc::clone(&jobs);
        thread::spawn(move || {
            loop {
                scanner.scan();
                thread::sleep(SCAN);
            }
        });

        jobs
    }

    fn lock(&self) -> MutexGuard<'_, Queue> {
        self.state.lock().unwrap_or_else(|err| err.into_inner())
    }

    /// Read the queue for as long as the closure runs. Kept short: the menu
    /// builds from this on the main thread while jobs are writing to it.
    pub fn read<T>(&self, with: impl FnOnce(&Queue) -> T) -> T {
        with(&self.lock())
    }

    pub fn take_events(&self) -> Vec<Event> {
        std::mem::take(&mut self.lock().events)
    }

    /// Every job folder under the root, alphabetically.
    pub fn job_folders(&self) -> Vec<PathBuf> {
        let Ok(entries) = fs::read_dir(&self.root) else {
            return Vec::new();
        };
        let mut dirs: Vec<PathBuf> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| !is_hidden(path) && path.join(SCRIPT).is_file())
            .collect();
        dirs.sort();
        dirs
    }

    /// One pass over every job folder: queue what has finished arriving, and
    /// forget queued inputs that have been taken away.
    fn scan(&self) {
        let folders = self.job_folders();

        // A queued file dragged out of `input/` — or a whole job folder
        // removed — is a dequeue. Only waiting jobs: a running one's input
        // going missing is the script's business, and it will say so.
        self.lock()
            .jobs
            .retain(|job| !job.phase.waiting() || job.input.is_file());

        let mut seen = HashSet::new();
        for dir in &folders {
            // A folder that has just gained a `job.sh` gets the rest of its
            // shape, so the first thing anyone sees in it is where to drop.
            for sub in [INPUT, OUTPUT] {
                let _ = fs::create_dir(dir.join(sub));
            }
            let mut candidates: Vec<(PathBuf, Option<SystemTime>)> = fs::read_dir(dir.join(INPUT))
                .into_iter()
                .flatten()
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| is_candidate(path))
                .map(|path| {
                    let modified = fs::metadata(&path).and_then(|meta| meta.modified()).ok();
                    (path, modified)
                })
                .collect();
            // Oldest first, so a batch dropped together queues in the order it
            // was written rather than the alphabet.
            candidates.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
            for (path, _) in candidates {
                seen.insert(path.clone());
                self.consider(dir, path);
            }
        }

        // Forget anything no longer there, so it is judged afresh if it returns.
        let mut arrivals = self.arrivals.lock().unwrap_or_else(|err| err.into_inner());
        arrivals.pending.retain(|path, _| seen.contains(path));
        arrivals.stuck.retain(|path| seen.contains(path));
    }

    /// Queue `path` once it has finished arriving.
    fn consider(&self, dir: &Path, path: PathBuf) {
        if self.lock().has_input(&path) {
            return;
        }
        let Some(now) = Fingerprint::of(&path) else { return };

        let settled = {
            let mut arrivals = self.arrivals.lock().unwrap_or_else(|err| err.into_inner());
            if arrivals.stuck.contains(&path) {
                return;
            }
            match arrivals.pending.get(&path) {
                Some((before, since)) if *before == now => since.elapsed() >= self.settle,
                _ => {
                    arrivals.pending.insert(path.clone(), (now, Instant::now()));
                    self.settle.is_zero()
                }
            }
        };
        // Held still for long enough — and nothing on this machine is still
        // writing it. The second check catches a copy that has stalled rather
        // than finished, which is the one a quiet file can't tell apart.
        if !settled || open_for_writing(&path) {
            return;
        }

        self.arrivals
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .pending
            .remove(&path);
        self.enrol(dir.to_path_buf(), path);
    }

    /// Add an input to the back of the queue, unless it is already in it.
    fn enrol(&self, dir: PathBuf, input: PathBuf) -> Option<u64> {
        let mut queue = self.lock();
        if queue.has_input(&input) {
            return None;
        }
        let id = queue.next_id;
        queue.next_id += 1;
        queue.jobs.push(Job {
            id,
            dir,
            input,
            phase: Phase::Queued,
            queued_at: SystemTime::now(),
            started: None,
            finished: None,
            progress: None,
            last_line: None,
            last_output: None,
            pgid: None,
            exit: None,
            note: None,
            stopping: None,
        });
        Some(id)
    }

    /// Start whatever the free slots allow, and escalate any stop that has been
    /// ignored for long enough.
    fn schedule(self: &Arc<Self>) {
        let mut starting = Vec::new();
        {
            let mut queue = self.lock();
            for job in queue.jobs.iter_mut() {
                if let Some(since) = job.stopping
                    && since.elapsed() > TERM_GRACE
                {
                    if let Some(pgid) = job.pgid {
                        signal(pgid, libc::SIGKILL);
                    }
                    job.stopping = None;
                }
            }

            if !queue.paused {
                // One run per job folder at a time: its logs are shared, and a
                // folder of drops is a list to work through, not a race.
                let mut busy: HashSet<PathBuf> = queue
                    .jobs
                    .iter()
                    .filter(|job| job.phase.active())
                    .map(|job| job.dir.clone())
                    .collect();
                let concurrency = queue.concurrency;
                for job in queue.jobs.iter_mut() {
                    if busy.len() >= concurrency {
                        break;
                    }
                    if job.phase != Phase::Queued || busy.contains(&job.dir) {
                        continue;
                    }
                    job.phase = Phase::Running;
                    job.started = Some(SystemTime::now());
                    busy.insert(job.dir.clone());
                    starting.push(job.id);
                }
            }
        }
        for id in starting {
            let jobs = Arc::clone(self);
            thread::spawn(move || run(jobs, id));
        }
    }

    /// Apply a row button. Runs on the main thread, straight off the click, and
    /// every branch of it is a memory write, a signal, or one `rename`.
    pub fn command(self: &Arc<Self>, id: u64, verb: Verb) {
        match verb {
            Verb::Pause => {
                let mut queue = self.lock();
                let Some(job) = queue.find(id) else { return };
                match job.phase {
                    Phase::Running => {
                        if let Some(pgid) = job.pgid {
                            signal(pgid, libc::SIGSTOP);
                        }
                        job.phase = Phase::Paused;
                    }
                    // Nothing to signal yet: holding it is the same intent one
                    // step earlier.
                    Phase::Queued => job.phase = Phase::Held,
                    _ => {}
                }
            }
            Verb::Resume => {
                let mut queue = self.lock();
                let Some(job) = queue.find(id) else { return };
                match job.phase {
                    Phase::Paused => {
                        if let Some(pgid) = job.pgid {
                            signal(pgid, libc::SIGCONT);
                        }
                        job.phase = Phase::Running;
                    }
                    Phase::Held => job.phase = Phase::Queued,
                    _ => {}
                }
            }
            Verb::Stop => {
                let finish_now = {
                    let mut queue = self.lock();
                    let Some(job) = queue.find(id) else { return };
                    match job.phase {
                        Phase::Running | Phase::Paused => {
                            if let Some(pgid) = job.pgid {
                                // A suspended process can't act on SIGTERM, so
                                // wake it first.
                                if job.phase == Phase::Paused {
                                    signal(pgid, libc::SIGCONT);
                                }
                                signal(pgid, libc::SIGTERM);
                                job.phase = Phase::Running;
                                job.stopping = Some(Instant::now());
                                job.note = Some("stopping".to_string());
                                // The thread waiting on the child files it away
                                // when it goes.
                                false
                            } else {
                                true
                            }
                        }
                        Phase::Queued | Phase::Held => true,
                        Phase::Finished { .. } => false,
                    }
                };
                if finish_now {
                    self.finish(id, false, Some("stopped".to_string()));
                }
            }
            Verb::Top => {
                let mut queue = self.lock();
                let Some(index) = queue.index_of(id) else { return };
                if queue.jobs[index].phase.waiting() {
                    let job = queue.jobs.remove(index);
                    // Held or not: sending a job to the front is also how you
                    // say you want it, so it stops being skipped.
                    queue.jobs.insert(0, Job {
                        phase: Phase::Queued,
                        ..job
                    });
                }
            }
            Verb::Retry => self.retry(id),
        }
    }

    /// Put a finished job's input back in `input/`, at the back of the queue —
    /// exactly what dragging it back from Finder does, minus the settle wait.
    fn retry(&self, id: u64) {
        // Held across the rename, so the scanner can't see the file land in
        // `input/` before the job does and queue it a second time.
        let mut queue = self.lock();
        let Some(index) = queue
            .index_of(id)
            .filter(|index| queue.jobs[*index].phase.finished())
        else {
            return;
        };
        let (dir, input) = (queue.jobs[index].dir.clone(), queue.jobs[index].input.clone());

        let back = uniq_file(dir.join(INPUT).join(input.file_name().unwrap_or_default()));
        if fs::rename(&input, &back).is_err() {
            queue.jobs[index].note = Some("could not requeue".to_string());
            return;
        }

        let mut job = queue.jobs.remove(index);
        job.input = back;
        job.phase = Phase::Queued;
        job.queued_at = SystemTime::now();
        job.started = None;
        job.finished = None;
        job.progress = None;
        job.last_line = None;
        job.last_output = None;
        job.exit = None;
        job.note = None;
        queue.jobs.push(job);
    }

    /// File a job away: its input moves to `done/` or `failed/`, and the row
    /// becomes an outcome.
    fn finish(&self, id: u64, ok: bool, note: Option<String>) {
        let Some((dir, input)) = ({
            let queue = self.lock();
            queue
                .jobs
                .iter()
                .find(|job| job.id == id && !job.phase.finished())
                .map(|job| (job.dir.clone(), job.input.clone()))
        }) else {
            return;
        };

        let bin = dir.join(if ok { DONE } else { FAILED });
        let _ = fs::create_dir_all(&bin);
        let destination = uniq_file(bin.join(input.file_name().unwrap_or_default()));
        let moved = !input.exists() || fs::rename(&input, &destination).is_ok();
        if !moved {
            self.arrivals
                .lock()
                .unwrap_or_else(|err| err.into_inner())
                .stuck
                .insert(input.clone());
        }

        let mut queue = self.lock();
        let Some(job) = queue.find(id) else { return };
        job.phase = Phase::Finished { ok };
        job.finished = Some(SystemTime::now());
        job.pgid = None;
        job.stopping = None;
        job.note = if moved {
            note
        } else {
            Some("could not move input".to_string())
        };
        if moved && destination.exists() {
            job.input = destination;
        }
        let name = job.label();
        queue.events.push(Event::Finished { name, ok });
    }

    /// Forget the finished jobs. Their inputs stay in `done/` and `failed/` —
    /// this is the list being cleared, not the work.
    pub fn clear_finished(&self) {
        self.lock().jobs.retain(|job| !job.phase.finished());
    }

    pub fn set_paused(&self, paused: bool) {
        self.lock().paused = paused;
    }

    pub fn set_concurrency(&self, concurrency: usize) {
        self.lock().concurrency = concurrency.clamp(1, MAX_CONCURRENCY);
    }

    /// Stop everything, on the way out.
    ///
    /// A queue that lives in one process dies with it, so quitting has to say
    /// so to the jobs as well: an orphaned encode nothing is watching would go
    /// on burning the machine for hours with no row left to stop it from. Its
    /// input is still in `input/`, and runs again next launch.
    pub fn shutdown(&self) {
        let mut queue = self.lock();
        for job in queue.jobs.iter_mut().filter(|job| job.phase.active()) {
            if let Some(pgid) = job.pgid {
                signal(pgid, libc::SIGCONT);
                signal(pgid, libc::SIGTERM);
            }
        }
    }
}

/// Run one job to its end. One thread per job, which is also the thread that
/// waits on it — there is no supervision loop, because there is nothing to
/// watch for: a command reaches the process directly.
fn run(jobs: Arc<Jobs>, id: u64) {
    let Some((dir, input)) = jobs.read(|queue| {
        queue
            .jobs
            .iter()
            .find(|job| job.id == id)
            .map(|job| (job.dir.clone(), job.input.clone()))
    }) else {
        return;
    };

    let script = dir.join(SCRIPT);
    let input_dir = input.parent().map(Path::to_path_buf).unwrap_or_default();
    let input_file = file_name(&input);
    let input_name = input
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| input_file.clone());
    let output_dir = dir.join(OUTPUT);
    let _ = fs::create_dir_all(&output_dir);

    // One log per stream per folder, shared by every run. A run's header goes
    // in only with its first output, so a quiet stream stays empty.
    let header = format!("=== {} {input_file} ===\n", clock::timestamp());
    let out_log = Log::new(dir.join(STDOUT_LOG), header.clone());
    let err_log = Log::new(dir.join(STDERR_LOG), header);

    // Run as-is if it's executable, so its shebang counts; through bash if not,
    // so a script saved without +x still works.
    let executable = fs::metadata(&script).is_ok_and(|meta| meta.permissions().mode() & 0o111 != 0);
    let mut command = if executable {
        Command::new(&script)
    } else {
        let mut command = Command::new("/bin/bash");
        command.arg(&script);
        command
    };
    command
        .current_dir(&dir)
        .env("INPUT", &input)
        .env("INPUT_DIR", &input_dir)
        .env("INPUT_FILE", &input_file)
        .env("INPUT_NAME", &input_name)
        .env("OUTPUT_DIR", &output_dir)
        .env("JOB_DIR", &dir)
        .env("TERM", "dumb")
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        // Its own process group, so pause and stop reach the encoder underneath
        // and not just the shell wrapping it.
        .process_group(0);

    let nice = niceness();
    if nice > 0 {
        // SAFETY: setpriority is async-signal-safe, which is the bar for what
        // may run between fork and exec. Only ever raises the value: lowering
        // it needs root, and failing quietly is better than refusing to run.
        unsafe {
            command.pre_exec(move || {
                libc::setpriority(libc::PRIO_PROCESS, 0, nice);
                Ok(())
            });
        }
    }

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            jobs.finish(id, false, Some(format!("could not start: {err}")));
            return;
        }
    };

    // The process group id is the job's own pid, since it leads the group.
    let pgid = child.id() as i32;
    {
        let mut queue = jobs.lock();
        let Some(job) = queue.find(id) else { return };
        job.pgid = Some(pgid);
        // Paused between the slot opening and the fork returning: the click
        // beat the process into existence, so honour it now that there is
        // something to signal.
        if job.phase == Phase::Paused {
            signal(pgid, libc::SIGSTOP);
        }
        if job.stopping.is_some() {
            signal(pgid, libc::SIGTERM);
        }
    }

    // stdout is the job talking: it goes to the log *and* into the model, so
    // the row can show the last line without anything reading the file back.
    // stderr is kept beside it but stays out of the row — plenty of tools log
    // there, and a warning is not what the job is doing.
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let watcher = Arc::clone(&jobs);
    let out_pump = thread::spawn(move || {
        if let Some(stream) = stdout {
            pump(stream, out_log, Some((watcher, id)));
        }
    });
    let err_pump = thread::spawn(move || {
        if let Some(stream) = stderr {
            pump(stream, err_log, None);
        }
    });

    let code = child.wait().ok().and_then(|status| status.code()).unwrap_or(1);
    let _ = out_pump.join();
    let _ = err_pump.join();

    let stopped = jobs.read(|queue| {
        queue
            .jobs
            .iter()
            .find(|job| job.id == id)
            .is_some_and(|job| job.stopping.is_some() || job.note.as_deref() == Some("stopping"))
    });

    if let Some(job) = jobs.lock().find(id) {
        job.exit = Some(code);
    }
    // Exit status alone decides — stderr output on its own is not a failure. A
    // job we stopped is the one case the status can't speak for.
    jobs.finish(
        id,
        code == 0 && !stopped,
        stopped.then(|| "stopped".to_string()),
    );
}

/// One run's half of a shared log file. Nothing is written — not the header,
/// not even the file — until the run actually prints something, so a log only
/// grows when there is something in it to read, and every block in it is
/// headed by the run it came from.
struct Log {
    path: PathBuf,
    header: String,
    file: Option<File>,
}

impl Log {
    fn new(path: PathBuf, header: String) -> Self {
        Self { path, header, file: None }
    }

    fn write(&mut self, bytes: &[u8]) {
        if self.file.is_none() {
            let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&self.path) else {
                return;
            };
            // A blank line between blocks, but none at the top of a new file.
            let gap = if file.metadata().is_ok_and(|meta| meta.len() > 0) { "\n" } else { "" };
            let _ = file.write_all(format!("{gap}{}", self.header).as_bytes());
            self.file = Some(file);
        }
        if let Some(file) = self.file.as_mut() {
            let _ = file.write_all(bytes);
        }
    }
}

/// Copy a stream to its log and — for stdout — feed each complete line into
/// the job's row as it lands.
///
/// Carriage returns end a line like newlines: a tool redrawing a progress bar in
/// place writes `\r`, and what it just drew is the interesting part.
fn pump(mut stream: impl Read, mut log: Log, mut model: Option<(Arc<Jobs>, u64)>) {
    let mut buffer = [0u8; 8192];
    let mut partial = String::new();
    loop {
        match stream.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(read) => {
                log.write(&buffer[..read]);
                let Some((jobs, id)) = model.as_mut() else {
                    continue;
                };
                partial.push_str(&String::from_utf8_lossy(&buffer[..read]));
                let Some(cut) = partial.rfind(['\n', '\r']) else {
                    continue;
                };
                let complete: String = partial.drain(..=cut).collect();
                let line = complete
                    .split(['\n', '\r'])
                    .map(str::trim)
                    .rfind(|line| !line.is_empty())
                    .map(clip);
                if let Some(line) = line {
                    let mut queue = jobs.lock();
                    if let Some(job) = queue.find(*id) {
                        job.progress = parse_progress(&line).or(job.progress);
                        job.last_line = Some(line);
                        job.last_output = Some(SystemTime::now());
                    }
                }
            }
        }
    }
}

fn clip(line: &str) -> String {
    if line.chars().count() > MAX_LINE {
        let cut: String = line.chars().take(MAX_LINE - 1).collect();
        format!("{cut}…")
    } else {
        line.to_string()
    }
}

fn signal(pgid: i32, signal: i32) {
    unsafe {
        libc::killpg(pgid, signal);
    }
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| name.to_string_lossy().starts_with('.'))
}

/// Suffixes that mean "still being written". Dotfiles are skipped too, which is
/// what makes `rsync` safe for free: it writes a hidden temporary and renames
/// it into place only once it is whole.
const PARTIAL: &[&str] = &[".part", ".partial", ".crdownload", ".download", ".tmp", ".temp"];

/// A regular, visible file that isn't a download in progress.
fn is_candidate(path: &Path) -> bool {
    if is_hidden(path) || !path.is_file() {
        return false;
    }
    let name = file_name(path).to_lowercase();
    !PARTIAL.iter().any(|suffix| name.ends_with(suffix))
}

/// True if any process this user can see has `path` open for writing — a
/// Finder copy, a `cp`, an encoder still producing it. Writers belonging to
/// other users (the SMB server's, when this machine hosts the share) are
/// invisible without root; the settle window is what covers those.
fn open_for_writing(path: &Path) -> bool {
    let Ok(output) = Command::new("/usr/sbin/lsof")
        .args(["-F", "a", "--"])
        .arg(path)
        .stderr(Stdio::null())
        .output()
    else {
        return false;
    };
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .any(|line| line.starts_with('a') && (line.contains('w') || line.contains('u')))
}

/// The jobs folder: `$JOBS_DIR`, else `~/jobs`.
pub fn root() -> PathBuf {
    std::env::var_os("JOBS_DIR").map(PathBuf::from).unwrap_or_else(|| {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_default()
            .join("jobs")
    })
}

/// How much to yield to everything else: 0 is normal priority, and this can
/// only ever raise it.
fn niceness() -> i32 {
    std::env::var("JOB_NICE")
        .ok()
        .and_then(|raw| raw.trim().parse::<i32>().ok())
        .unwrap_or(0)
        .clamp(0, 20)
}

fn configured_concurrency() -> usize {
    std::env::var("JOB_CONCURRENCY")
        .ok()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .unwrap_or(DEFAULT_CONCURRENCY)
        .clamp(1, MAX_CONCURRENCY)
}

fn configured_settle() -> Duration {
    std::env::var("JOB_SETTLE")
        .ok()
        .and_then(|raw| raw.trim().parse::<f64>().ok())
        .filter(|secs| secs.is_finite() && *secs >= 0.0)
        .map(Duration::from_secs_f64)
        .unwrap_or(DEFAULT_SETTLE)
}

pub fn max_concurrency() -> usize {
    MAX_CONCURRENCY
}

/// A non-colliding file path: `clip.mov` becomes `clip-2.mov`, `clip-3.mov`, …
fn uniq_file(path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }
    let parent = path.parent().map(Path::to_path_buf).unwrap_or_default();
    let stem = path
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_default();
    let ext = path
        .extension()
        .map(|ext| format!(".{}", ext.to_string_lossy()))
        .unwrap_or_default();
    for n in 2..1000 {
        let candidate = parent.join(format!("{stem}-{n}{ext}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(tag: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!("job-folder-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&base).unwrap();
        fs::canonicalize(&base).unwrap()
    }

    fn workflow(base: &Path, name: &str, script: &str) -> PathBuf {
        let dir = base.join(name);
        fs::create_dir_all(dir.join(INPUT)).unwrap();
        fs::write(dir.join(SCRIPT), script).unwrap();
        dir
    }

    fn wait_for(what: &str, mut done: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(20);
        while !done() {
            assert!(Instant::now() < deadline, "timed out waiting for {what}");
            thread::sleep(Duration::from_millis(50));
        }
    }

    #[test]
    fn a_token_survives_the_round_trip() {
        for id in [1u64, 2, 17, 1_000_000] {
            for verb in [Verb::Pause, Verb::Resume, Verb::Stop, Verb::Top, Verb::Retry] {
                assert_eq!(untoken(token(id, verb)), Some((id, verb)));
            }
        }
    }

    #[test]
    fn partial_and_hidden_files_are_not_inputs() {
        let base = scratch("candidates");
        let skipped = [".clip.mov.Xa81", "clip.mov.part", "a.crdownload", "._clip.mov"];
        for name in skipped.iter().chain(["clip.mov"].iter()) {
            fs::write(base.join(name), "x").unwrap();
        }
        fs::create_dir(base.join("helpers")).unwrap();
        assert!(is_candidate(&base.join("clip.mov")));
        assert!(!is_candidate(&base.join("helpers")));
        for name in skipped {
            assert!(!is_candidate(&base.join(name)), "{name}");
        }
        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn collisions_get_a_number_before_the_extension() {
        let base = scratch("uniq");
        fs::write(base.join("clip.mov"), "").unwrap();
        fs::write(base.join("clip-2.mov"), "").unwrap();
        assert_eq!(uniq_file(base.join("clip.mov")), base.join("clip-3.mov"));
        let _ = fs::remove_dir_all(&base);
    }

    /// The whole contract in one test: a file dropped in `input/` runs with the
    /// documented environment, logs are appended, and the input is filed away.
    #[test]
    fn a_dropped_input_runs_and_is_filed_away() {
        let base = scratch("run");
        let dir = workflow(
            &base,
            "echo",
            "#!/bin/bash\n\
             echo \"$INPUT|$INPUT_DIR|$INPUT_FILE|$INPUT_NAME|$OUTPUT_DIR|$PWD\" > \"$OUTPUT_DIR/$INPUT_NAME.env\"\n\
             echo 'encoding 45% eta 1:00'\n\
             echo oops >&2\n",
        );
        let jobs = Jobs::start_with(base.clone(), Duration::from_millis(300));
        fs::write(dir.join(INPUT).join("clip.mov"), "payload").unwrap();

        wait_for("the job to finish", || {
            jobs.read(|queue| queue.jobs.first().is_some_and(|job| job.phase.finished()))
        });

        jobs.read(|queue| {
            let job = &queue.jobs[0];
            assert_eq!(job.phase, Phase::Finished { ok: true });
            assert_eq!(job.progress, Some(0.45));
            assert_eq!(job.label(), "echo · clip.mov");
            assert_eq!(job.input, dir.join(DONE).join("clip.mov"));
        });
        assert!(!dir.join(INPUT).join("clip.mov").exists());

        let input = dir.join(INPUT);
        let env = fs::read_to_string(dir.join(OUTPUT).join("clip.env")).unwrap();
        assert_eq!(
            env.trim(),
            format!(
                "{}|{}|clip.mov|clip|{}|{}",
                input.join("clip.mov").display(),
                input.display(),
                dir.join(OUTPUT).display(),
                dir.display(),
            )
        );
        let out = fs::read_to_string(dir.join(STDOUT_LOG)).unwrap();
        assert!(out.contains("clip.mov ===") && out.contains("encoding 45%"));
        // Each log holds its own stream and nothing of the other's, and starts
        // with its header rather than a blank line.
        assert!(out.starts_with("=== "), "got {out:?}");
        let err = fs::read_to_string(dir.join(STDERR_LOG)).unwrap();
        assert!(err.contains("oops"));
        assert!(!err.contains("encoding 45%"), "stdout leaked into stderr.log:\n{err}");
        assert!(!out.contains("oops"), "stderr leaked into stdout.log:\n{out}");

        let _ = fs::remove_dir_all(&base);
    }

    /// A symlink is an input like any other — `send-job --link` depends on it —
    /// and filing it away moves the link, never the file it points at.
    #[test]
    fn a_linked_input_runs_and_only_the_link_moves() {
        let base = scratch("link");
        let dir = workflow(&base, "w", "#!/bin/bash\ncat \"$INPUT\" > \"$OUTPUT_DIR/$INPUT_FILE\"\n");
        let source = base.join("source.mov");
        fs::write(&source, "original").unwrap();
        let jobs = Jobs::start_with(base.clone(), Duration::ZERO);
        std::os::unix::fs::symlink(&source, dir.join(INPUT).join("source.mov")).unwrap();

        wait_for("the linked job to finish", || {
            jobs.read(|queue| queue.jobs.first().is_some_and(|job| job.phase.finished()))
        });
        jobs.read(|queue| assert_eq!(queue.jobs[0].phase, Phase::Finished { ok: true }));
        let filed = dir.join(DONE).join("source.mov");
        assert!(fs::symlink_metadata(&filed).unwrap().file_type().is_symlink());
        assert_eq!(fs::read_to_string(&source).unwrap(), "original");
        assert_eq!(fs::read_to_string(dir.join(OUTPUT).join("source.mov")).unwrap(), "original");
        // It printed nothing, so neither log gained so much as a header.
        assert!(!dir.join(STDOUT_LOG).exists() && !dir.join(STDERR_LOG).exists());
        let _ = fs::remove_dir_all(&base);
    }

    /// A failing run files its input under `failed/`, and retry puts it back.
    #[test]
    fn failures_are_filed_and_can_be_retried() {
        let base = scratch("fail");
        let dir = workflow(&base, "broken", "exit 3\n");
        let jobs = Jobs::start_with(base.clone(), Duration::ZERO);
        fs::write(dir.join(INPUT).join("a.wav"), "x").unwrap();

        wait_for("the failure", || {
            jobs.read(|queue| queue.jobs.first().is_some_and(|job| job.phase.finished()))
        });
        let id = jobs.read(|queue| {
            let job = &queue.jobs[0];
            assert_eq!(job.phase, Phase::Finished { ok: false });
            assert_eq!(job.exit, Some(3));
            assert_eq!(job.input, dir.join(FAILED).join("a.wav"));
            job.id
        });

        jobs.set_paused(true);
        jobs.command(id, Verb::Retry);
        jobs.read(|queue| {
            let job = queue.jobs.last().unwrap();
            assert_eq!(job.phase, Phase::Queued);
            assert_eq!(job.input, dir.join(INPUT).join("a.wav"));
        });
        jobs.set_paused(false);
        let _ = fs::remove_dir_all(&base);
    }

    /// A file that keeps growing is not queued until it stops.
    #[test]
    fn a_growing_file_waits_until_it_settles() {
        let base = scratch("settle");
        let dir = workflow(&base, "w", "true\n");
        let jobs = Jobs::start_with(base.clone(), Duration::from_millis(1500));
        jobs.set_paused(true);

        let path = dir.join(INPUT).join("big.mov");
        let started = Instant::now();
        while started.elapsed() < Duration::from_secs(3) {
            let mut file = OpenOptions::new().create(true).append(true).open(&path).unwrap();
            file.write_all(b"chunk").unwrap();
            drop(file);
            thread::sleep(Duration::from_millis(400));
            assert_eq!(jobs.read(|queue| queue.jobs.len()), 0, "queued while still growing");
        }
        wait_for("the settled file to queue", || jobs.read(|queue| queue.jobs.len() == 1));
        let _ = fs::remove_dir_all(&base);
    }

    /// One run per job folder at a time; different folders run side by side.
    #[test]
    fn one_run_per_folder() {
        let base = scratch("serial");
        let slow = "#!/bin/bash\nsleep 1\n";
        let a = workflow(&base, "a", slow);
        let b = workflow(&base, "b", slow);
        let jobs = Jobs::start_with(base.clone(), Duration::ZERO);
        jobs.set_paused(true);
        jobs.set_concurrency(4);
        for name in ["1", "2"] {
            fs::write(a.join(INPUT).join(name), "").unwrap();
        }
        fs::write(b.join(INPUT).join("1"), "").unwrap();
        wait_for("all three to queue", || jobs.read(|queue| queue.jobs.len() == 3));

        jobs.set_paused(false);
        wait_for("two to start", || jobs.read(|queue| queue.running() == 2));
        thread::sleep(Duration::from_millis(300));
        jobs.read(|queue| {
            assert_eq!(queue.running(), 2);
            let dirs: HashSet<_> = queue
                .jobs
                .iter()
                .filter(|job| job.phase.active())
                .map(|job| job.dir.clone())
                .collect();
            assert_eq!(dirs.len(), 2, "never two from the same folder");
        });
        wait_for("everything to finish", || {
            jobs.read(|queue| queue.jobs.iter().all(|job| job.phase.finished()))
        });
        let _ = fs::remove_dir_all(&base);
    }

    /// Commands are answered in the model, not on the disk, so they are true
    /// the instant they are pressed — and dragging a queued file out dequeues it.
    #[test]
    fn commands_land_immediately() {
        let base = scratch("commands");
        let dir = workflow(&base, "w", "true\n");
        let jobs = Jobs::start_with(base.clone(), Duration::ZERO);
        jobs.set_paused(true);
        for name in ["a", "b", "c"] {
            fs::write(dir.join(INPUT).join(name), "").unwrap();
            thread::sleep(Duration::from_millis(20));
        }
        wait_for("three queued", || jobs.read(|queue| queue.jobs.len() == 3));
        let ids: Vec<u64> = jobs.read(|queue| queue.jobs.iter().map(|job| job.id).collect());

        jobs.command(ids[0], Verb::Pause);
        assert_eq!(jobs.read(|queue| queue.jobs[0].phase), Phase::Held);

        jobs.command(ids[2], Verb::Top);
        assert_eq!(jobs.read(|queue| queue.jobs[0].id), ids[2]);

        // Stopping something that never started files it under failed/.
        jobs.command(ids[1], Verb::Stop);
        jobs.read(|queue| {
            let job = queue.jobs.iter().find(|job| job.id == ids[1]).unwrap();
            assert_eq!(job.phase, Phase::Finished { ok: false });
            assert_eq!(job.note.as_deref(), Some("stopped"));
            assert_eq!(job.input, dir.join(FAILED).join("b"));
        });

        fs::remove_file(dir.join(INPUT).join("a")).unwrap();
        wait_for("the removed input to leave the queue", || {
            jobs.read(|queue| !queue.jobs.iter().any(|job| job.id == ids[0]))
        });
        let _ = fs::remove_dir_all(&base);
    }

    /// A pause reaches the process itself, and the queue does not quietly start
    /// something else in the freed slot.
    #[test]
    fn pausing_suspends_the_process_group() {
        let base = scratch("pause");
        let tick = "#!/bin/bash\nfor i in $(seq 1 200); do echo tick; sleep 0.1; done\n";
        let a = workflow(&base, "a", tick);
        let b = workflow(&base, "b", tick);
        let jobs = Jobs::start_with(base.clone(), Duration::ZERO);
        jobs.set_concurrency(1);
        fs::write(a.join(INPUT).join("x"), "").unwrap();
        wait_for("the first job to start", || {
            jobs.read(|queue| queue.jobs.first().is_some_and(|job| job.pgid.is_some()))
        });
        fs::write(b.join(INPUT).join("y"), "").unwrap();
        wait_for("the second to queue", || jobs.read(|queue| queue.jobs.len() == 2));

        let id = jobs.read(|queue| queue.jobs[0].id);
        let pgid = jobs.read(|queue| queue.jobs[0].pgid).unwrap();

        jobs.command(id, Verb::Pause);
        assert_eq!(jobs.read(|queue| queue.jobs[0].phase), Phase::Paused);
        assert_eq!(unsafe { libc::killpg(pgid, 0) }, 0, "still there, just stopped");

        thread::sleep(TICK * 4);
        assert_eq!(jobs.read(|queue| queue.jobs[1].phase), Phase::Queued);

        jobs.command(id, Verb::Resume);
        assert_eq!(jobs.read(|queue| queue.jobs[0].phase), Phase::Running);

        jobs.command(id, Verb::Stop);
        wait_for("the stop", || jobs.read(|queue| queue.jobs[0].phase.finished()));
        jobs.read(|queue| {
            assert_eq!(queue.jobs[0].phase, Phase::Finished { ok: false });
            assert_eq!(queue.jobs[0].note.as_deref(), Some("stopped"));
        });

        jobs.shutdown();
        let _ = fs::remove_dir_all(&base);
    }
}
