//! The queue, in memory, in the same process as the menu that shows it.
//!
//! `job-daemon` keeps a job's state in the folder it sits in, so that a runner
//! here and a monitor on another machine can agree without talking. That is the
//! right trade for a shared queue and the wrong one for a queue you are
//! standing in front of: every command is a `rename` somebody else has to
//! notice, and every answer waits on a poll.
//!
//! Here the process that runs the jobs is the process that draws the menu, so
//! the model is a `Vec<Job>` behind a mutex. Pause is a `SIGSTOP` on the way
//! back from the click. Reordering the queue is moving an element. Nothing is
//! written down to be read back, so nothing can disagree.
//!
//! The disk keeps only what a job genuinely needs on it:
//!
//! | on disk | |
//! |---|---|
//! | `TARGET.job` at the top level | dropped, not yet picked up |
//! | `ready/<date>-<name>/` | the job's payload while it is ours: the script, its target file, its logs |
//! | `done/<date>-<name>/` | the same folder once the job has finished, however it finished |
//!
//! Two folders, and neither is a state machine. Whether a job in `ready/` is
//! queued, running, held or suspended is not written anywhere, because the only
//! thing that needs to know is holding it in memory. There is no `.status`, no
//! `_paused`, and no way to command the queue by dragging — that is the price
//! of this design, and [`README.md`](../README.md) says so plainly.
//!
//! What survives a crash is the payload. Anything left in `ready/` when the app
//! next starts is queued again from the top: the folder is a job that didn't
//! get to finish, which is all the state worth recovering.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

use job_core::clock;
use job_core::observe::{job_name, parse_progress, target_file};

/// Where a job's payload lives while it is the queue's, and where it goes when
/// it is finished with. Deliberately unprefixed, and deliberately not the
/// daemon's `_ready`: the two tools mean different things by a folder, and a
/// name that could be mistaken for the other's is a queue running twice.
pub const READY: &str = "ready";
pub const DONE: &str = "done";

/// How often the scheduler looks at itself. Only the queue's own bookkeeping —
/// starting a job when a slot frees, escalating a stop that was ignored — runs
/// on this; every command is applied the moment it is pressed.
const TICK: Duration = Duration::from_millis(250);

/// How often the inbox is scanned for dropped `.job` files. This is the one
/// thing that still has to be discovered from the filesystem, because the whole
/// point of the drop folder is that anything can write to it.
const SCAN: Duration = Duration::from_secs(1);

/// A dropped file whose size holds steady this long is taken to have finished
/// copying. `send-job` ships data files first, but a plain `cp` over the network
/// is the case this exists for.
const SETTLE: Duration = Duration::from_secs(2);

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

#[derive(Clone, Debug)]
pub struct Job {
    pub id: u64,
    pub name: String,
    /// The payload folder: under `ready/` until it finishes, `done/` after.
    pub dir: PathBuf,
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
    pub fn log_path(&self) -> Option<PathBuf> {
        let path = self.dir.join(format!("{}.log", self.name));
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
    /// *is* the priority: the scheduler takes the first [`Phase::Queued`] entry
    /// it finds, so moving an element is all reordering the queue amounts to.
    pub jobs: Vec<Job>,
    pub concurrency: usize,
    /// The whole queue held: running jobs carry on, nothing new starts.
    pub paused: bool,
    pub events: Vec<Event>,
    next_id: u64,
}

impl Queue {
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

/// The queue and the folder it draws its work from.
pub struct Jobs {
    pub root: PathBuf,
    state: Mutex<Queue>,
}

impl Jobs {
    /// Prepare the folder, take back anything a previous run left behind, and
    /// start the two threads that keep the queue moving.
    pub fn start(root: PathBuf) -> Arc<Self> {
        let _ = fs::create_dir_all(root.join(READY));
        let _ = fs::create_dir_all(root.join(DONE));

        let jobs = Arc::new(Self {
            root,
            state: Mutex::new(Queue {
                jobs: Vec::new(),
                concurrency: configured_concurrency(),
                paused: false,
                events: Vec::new(),
                next_id: 1,
            }),
        });

        jobs.reclaim();

        let scheduler = Arc::clone(&jobs);
        thread::spawn(move || {
            loop {
                scheduler.schedule();
                thread::sleep(TICK);
            }
        });

        let watcher = Arc::clone(&jobs);
        thread::spawn(move || {
            loop {
                watcher.ingest_inbox();
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

    pub fn ready_dir(&self) -> PathBuf {
        self.root.join(READY)
    }

    pub fn done_dir(&self) -> PathBuf {
        self.root.join(DONE)
    }

    /// Everything a previous run was in the middle of. A folder in `ready/` is
    /// a job that never finished — whether it was queued, running or suspended
    /// when the app went is not recorded anywhere, and putting it back in the
    /// queue is the only answer that can't be wrong.
    fn reclaim(&self) {
        let Ok(entries) = fs::read_dir(self.ready_dir()) else {
            return;
        };
        let mut dirs: Vec<PathBuf> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.is_dir() && find_job_file(path).is_some())
            .collect();
        // Alphabetical is date order, so a recovered queue keeps the order it
        // was dropped in.
        dirs.sort();
        for dir in dirs {
            self.enrol(dir);
        }
    }

    /// Add a payload folder to the back of the queue.
    fn enrol(&self, dir: PathBuf) -> u64 {
        let mut queue = self.lock();
        let id = queue.next_id;
        queue.next_id += 1;
        queue.jobs.push(Job {
            id,
            name: job_core::observe::run_folder_name(&dir),
            dir,
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
        id
    }

    /// Stage every `.job` file that has finished landing in the drop folder.
    ///
    /// This is the one thing still discovered from the filesystem, and it stays
    /// that way on purpose: `send-job`, `topaz-job` and the mpv Topaz workflow
    /// all queue work by writing a file, from this machine or across a mounted
    /// share, and none of them should have to know a process exists.
    fn ingest_inbox(&self) {
        for job_file in scan_inbox(&self.root) {
            if !is_stable(&job_file) {
                continue;
            }
            let target = self.root.join(target_file(&job_file));
            if target.is_file() && !is_stable(&target) {
                continue;
            }
            self.stage(&job_file);
        }
    }

    /// Move a dropped job and its target file into a folder of their own.
    fn stage(&self, job_file: &Path) {
        let name = job_name(job_file);
        let target = target_file(job_file);
        let dir = uniq_dir(self.ready_dir().join(format!("{}-{name}", clock::file_stamp())));
        if fs::create_dir(&dir).is_err() {
            return;
        }
        if fs::rename(job_file, dir.join(format!("{target}.job"))).is_err() {
            let _ = fs::remove_dir(&dir);
            return;
        }
        let beside = self.root.join(&target);
        if beside.exists() {
            let _ = fs::rename(&beside, dir.join(&target));
        }
        self.enrol(dir);
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
                let mut running = queue.running();
                let concurrency = queue.concurrency;
                for index in 0..queue.jobs.len() {
                    if running >= concurrency {
                        break;
                    }
                    if queue.jobs[index].phase != Phase::Queued {
                        continue;
                    }
                    queue.jobs[index].phase = Phase::Running;
                    queue.jobs[index].started = Some(SystemTime::now());
                    starting.push(queue.jobs[index].id);
                    running += 1;
                }
            }
        }
        for id in starting {
            let jobs = Arc::clone(self);
            thread::spawn(move || run(jobs, id));
        }
    }

    /// Apply a row button. Runs on the main thread, straight off the click, and
    /// every branch of it is either a memory write or a signal — which is the
    /// entire reason this app exists.
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

    /// Put a finished job back in the queue: its payload moves out of `done/`
    /// and it goes to the back, where a job queued now belongs.
    fn retry(&self, id: u64) {
        let Some((index, dir)) = ({
            let queue = self.lock();
            queue
                .index_of(id)
                .filter(|index| queue.jobs[*index].phase.finished())
                .map(|index| (index, queue.jobs[index].dir.clone()))
        }) else {
            return;
        };

        let folder = dir.file_name().unwrap_or_default().to_os_string();
        let back = uniq_dir(self.ready_dir().join(&folder));
        if fs::rename(&dir, &back).is_err() {
            let mut queue = self.lock();
            if let Some(job) = queue.find(id) {
                job.note = Some("could not requeue".to_string());
            }
            return;
        }

        let mut queue = self.lock();
        let mut job = queue.jobs.remove(index);
        job.dir = back;
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

    /// File a job away: it stops being work and becomes an outcome, and its
    /// folder moves to `done/`.
    fn finish(&self, id: u64, ok: bool, note: Option<String>) {
        let Some(dir) = ({
            let queue = self.lock();
            queue
                .jobs
                .iter()
                .find(|job| job.id == id && !job.phase.finished())
                .map(|job| job.dir.clone())
        }) else {
            return;
        };

        let folder = dir.file_name().unwrap_or_default().to_os_string();
        let destination = uniq_dir(self.done_dir().join(&folder));
        let moved = fs::rename(&dir, &destination).is_ok();

        let mut queue = self.lock();
        let Some(job) = queue.find(id) else { return };
        job.phase = Phase::Finished { ok };
        job.finished = Some(SystemTime::now());
        job.pgid = None;
        job.stopping = None;
        job.note = note;
        if moved {
            job.dir = destination;
        }
        let name = job.name.clone();
        queue.events.push(Event::Finished { name, ok });
    }

    /// Forget the finished jobs. Their folders stay in `done/` — this is the
    /// list being cleared, not the work.
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
    /// on burning the machine for hours with no row left to stop it from. What
    /// it was working on stays in `ready/`, and starts again next launch.
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
    let Some((name, dir)) = jobs.read(|queue| {
        queue
            .jobs
            .iter()
            .find(|job| job.id == id)
            .map(|job| (job.name.clone(), job.dir.clone()))
    }) else {
        return;
    };

    let Some(job_file) = find_job_file(&dir) else {
        jobs.finish(id, false, Some("no .job file in its folder".to_string()));
        return;
    };
    let target = target_file(&job_file);

    if let Ok(meta) = fs::metadata(&job_file) {
        let mut perms = meta.permissions();
        perms.set_mode(perms.mode() | 0o100);
        let _ = fs::set_permissions(&job_file, perms);
    }
    let executable = fs::metadata(&job_file).is_ok_and(|meta| meta.permissions().mode() & 0o111 != 0);

    let mut command = if executable {
        Command::new(&job_file)
    } else {
        let mut command = Command::new("/bin/bash");
        command.arg(&job_file);
        command
    };
    // The same contract job-daemon runs jobs under, to the letter: a `.job`
    // script written for one has to work under the other, or the drop folder
    // stops being a shared interface.
    command
        .current_dir(&dir)
        .env("JOB_NAME", &name)
        .env("TARGET_FILE", &target)
        .env("JOB_FILE", &job_file)
        .env("JOB_DIR", &jobs.root)
        .env("JOB_RUN_DIR", &dir)
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

    // stdout is the job talking: it goes to the log file *and* into the model,
    // so the row can show the last line without anything reading the file back.
    // stderr is kept beside it but stays out of the row — plenty of tools log
    // there, and a warning is not what the job is doing.
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let out_path = dir.join(format!("{name}.log"));
    let err_path = dir.join(format!("{name}.error.log"));
    let watcher = Arc::clone(&jobs);
    let out_pump = thread::spawn(move || {
        if let Some(stream) = stdout {
            pump(stream, out_path, Some((watcher, id)));
        }
    });
    let err_pump = thread::spawn(move || {
        if let Some(stream) = stderr {
            pump(stream, err_path, None);
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
    // Exit status alone decides, exactly as it does in the daemon — stderr
    // output on its own is not a failure. A job we stopped is the one case the
    // status can't speak for.
    jobs.finish(
        id,
        code == 0 && !stopped,
        stopped.then(|| "stopped".to_string()),
    );
}

/// Copy a stream to `path`, creating the file only when the first bytes arrive,
/// and — for stdout — feeding each complete line into the job's row as it lands.
///
/// Carriage returns end a line like newlines: a tool redrawing a progress bar in
/// place writes `\r`, and what it just drew is the interesting part.
fn pump(mut stream: impl Read, path: PathBuf, mut model: Option<(Arc<Jobs>, u64)>) {
    let mut file: Option<File> = None;
    let mut buffer = [0u8; 8192];
    let mut partial = String::new();
    loop {
        match stream.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(read) => {
                if file.is_none() {
                    file = File::create(&path).ok();
                }
                if let Some(handle) = file.as_mut() {
                    let _ = handle.write_all(&buffer[..read]);
                    let _ = handle.flush();
                }
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

/// The jobs folder: `$JOBS_DIR`, else `~/jobs`.
pub fn root() -> PathBuf {
    std::env::var_os("JOBS_DIR").map(PathBuf::from).unwrap_or_else(|| {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_default()
            .join("jobs")
    })
}

/// How much to yield to everything else, as in the daemon: 0 is normal
/// priority, and this can only ever raise it.
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

pub fn max_concurrency() -> usize {
    MAX_CONCURRENCY
}

/// Top-level `*.job` files, alphabetically.
fn scan_inbox(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(root) else {
        return Vec::new();
    };
    let mut jobs: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().ends_with(".job"))
        })
        .collect();
    jobs.sort();
    jobs
}

fn find_job_file(dir: &Path) -> Option<PathBuf> {
    fs::read_dir(dir).ok()?.flatten().find_map(|entry| {
        let path = entry.path();
        path.file_name()
            .is_some_and(|name| name.to_string_lossy().ends_with(".job"))
            .then_some(path)
    })
}

/// True once the file's size has held steady across [`SETTLE`].
fn is_stable(path: &Path) -> bool {
    let size = |path: &Path| fs::metadata(path).map(|meta| meta.len()).ok();
    let Some(first) = size(path) else { return false };
    thread::sleep(SETTLE);
    size(path) == Some(first)
}

/// A non-colliding directory path: appends `-2`, `-3`, … if taken.
fn uniq_dir(path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }
    for n in 2..1000 {
        let candidate = PathBuf::from(format!("{}-{n}", path.display()));
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
        base
    }

    #[test]
    fn a_token_survives_the_round_trip() {
        for id in [1u64, 2, 17, 1_000_000] {
            for verb in [Verb::Pause, Verb::Resume, Verb::Stop, Verb::Top, Verb::Retry] {
                assert_eq!(untoken(token(id, verb)), Some((id, verb)));
            }
        }
    }

    /// The whole contract in one test: a dropped file becomes a folder, the
    /// folder becomes a queue entry, and the queue entry runs, reports and is
    /// filed away — with nothing on disk ever saying what state it was in.
    #[test]
    fn a_dropped_job_runs_and_is_filed_away() {
        let base = scratch("run");
        let jobs = Jobs::start(base.clone());
        // Held while the staging half is checked, or the queue does its job too
        // quickly to catch it: a slot is free, so the scheduler would have the
        // thing running before the assertions got there.
        jobs.set_paused(true);

        fs::write(
            base.join("clip.mov.job"),
            "#!/bin/bash\necho 'encoding 45% eta 1:00'\nsleep 0.2\n",
        )
        .unwrap();
        fs::write(base.join("clip.mov"), "payload").unwrap();

        // Staged by the app's own watcher: job and target travel together into
        // one folder, and the drop folder is left clean.
        let deadline = Instant::now() + Duration::from_secs(20);
        while jobs.read(|queue| queue.jobs.is_empty()) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(50));
        }
        let staged = jobs
            .read(|queue| queue.jobs.first().map(|job| job.dir.clone()))
            .expect("the dropped job should have been staged");
        assert!(staged.join("clip.mov.job").is_file());
        assert!(staged.join("clip.mov").is_file());
        assert!(!base.join("clip.mov.job").exists());
        assert_eq!(jobs.read(|queue| queue.queued()), 1);

        // Nothing in the folder says "queued" — that is the point.
        assert!(!staged.join(".status").exists());

        let id = jobs.read(|queue| queue.jobs[0].id);
        jobs.set_paused(false);
        let deadline = Instant::now() + Duration::from_secs(20);
        loop {
            let done = jobs.read(|queue| queue.jobs[0].phase.finished());
            if done || Instant::now() > deadline {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }

        jobs.read(|queue| {
            let job = &queue.jobs[0];
            assert_eq!(job.id, id);
            assert_eq!(job.phase, Phase::Finished { ok: true });
            assert_eq!(job.progress, Some(0.45));
            assert_eq!(job.last_line.as_deref(), Some("encoding 45% eta 1:00"));
            // The payload moved to done/, logs and all.
            assert!(job.dir.starts_with(base.join(DONE)));
            assert!(job.dir.join("clip.log").is_file());
        });
        assert_eq!(jobs.read(|queue| queue.jobs.len()), 1);

        let _ = fs::remove_dir_all(&base);
    }

    /// Commands are answered in the model, not on the disk, so they are true
    /// the instant they are pressed.
    #[test]
    fn commands_land_immediately() {
        let base = scratch("commands");
        let jobs = Jobs::start(base.clone());
        jobs.set_paused(true);

        for name in ["a", "b", "c"] {
            let dir = base.join(READY).join(format!("20260101-00000{name}-{name}"));
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join(format!("{name}.job")), "#!/bin/bash\ntrue\n").unwrap();
            jobs.enrol(dir);
        }
        let ids: Vec<u64> = jobs.read(|queue| queue.jobs.iter().map(|job| job.id).collect());

        jobs.command(ids[0], Verb::Pause);
        assert_eq!(jobs.read(|queue| queue.jobs[0].phase), Phase::Held);

        // The third job to the front, and asking for it releases the hold.
        jobs.command(ids[2], Verb::Top);
        assert_eq!(jobs.read(|queue| queue.jobs[0].id), ids[2]);
        jobs.command(ids[0], Verb::Resume);
        assert_eq!(
            jobs.read(|queue| queue.jobs.iter().find(|job| job.id == ids[0]).unwrap().phase),
            Phase::Queued
        );

        // Stopping something that never started is an outcome, not a deletion:
        // the payload is still there to look at, in done/.
        jobs.command(ids[1], Verb::Stop);
        jobs.read(|queue| {
            let job = queue.jobs.iter().find(|job| job.id == ids[1]).unwrap();
            assert_eq!(job.phase, Phase::Finished { ok: false });
            assert_eq!(job.note.as_deref(), Some("stopped"));
            assert!(job.dir.starts_with(base.join(DONE)));
        });

        // And it can be put back, at the end of the queue.
        jobs.command(ids[1], Verb::Retry);
        jobs.read(|queue| {
            let job = queue.jobs.last().unwrap();
            assert_eq!(job.id, ids[1]);
            assert_eq!(job.phase, Phase::Queued);
            assert!(job.dir.starts_with(base.join(READY)));
        });

        let _ = fs::remove_dir_all(&base);
    }

    /// A pause reaches the process itself, and the queue does not quietly start
    /// something else in the freed slot.
    #[test]
    fn pausing_suspends_the_process_group() {
        let base = scratch("pause");
        let jobs = Jobs::start(base.clone());
        jobs.set_concurrency(1);

        for name in ["first", "second"] {
            let dir = base.join(READY).join(format!("20260101-000000-{name}"));
            fs::create_dir_all(&dir).unwrap();
            fs::write(
                dir.join(format!("{name}.job")),
                "#!/bin/bash\nfor i in $(seq 1 200); do echo tick; sleep 0.1; done\n",
            )
            .unwrap();
            jobs.enrol(dir);
        }

        let deadline = Instant::now() + Duration::from_secs(10);
        while jobs.read(|queue| queue.jobs[0].pgid.is_none()) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(50));
        }
        let id = jobs.read(|queue| queue.jobs[0].id);
        let pgid = jobs.read(|queue| queue.jobs[0].pgid).expect("the job should have started");

        jobs.command(id, Verb::Pause);
        assert_eq!(jobs.read(|queue| queue.jobs[0].phase), Phase::Paused);
        assert_eq!(unsafe { libc::killpg(pgid, 0) }, 0, "still there, just stopped");

        // The slot stays taken: pausing is for the machine, and back-filling it
        // with the next encode would defeat the whole gesture.
        thread::sleep(TICK * 4);
        assert_eq!(jobs.read(|queue| queue.jobs[1].phase), Phase::Queued);

        jobs.command(id, Verb::Resume);
        assert_eq!(jobs.read(|queue| queue.jobs[0].phase), Phase::Running);

        jobs.command(id, Verb::Stop);
        let deadline = Instant::now() + Duration::from_secs(20);
        while !jobs.read(|queue| queue.jobs[0].phase.finished()) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(50));
        }
        jobs.read(|queue| {
            assert_eq!(queue.jobs[0].phase, Phase::Finished { ok: false });
            assert_eq!(queue.jobs[0].note.as_deref(), Some("stopped"));
        });

        jobs.shutdown();
        let _ = fs::remove_dir_all(&base);
    }
}
