//! Turning a jobs folder into a [`Snapshot`], by reading it and nothing else.
//!
//! **The directory a job sits in is its state.** There is no lock file, no
//! pause flag, no status file the tools have to agree on — just where the
//! folder is:
//!
//! | on disk | means |
//! |---|---|
//! | `TARGET.job` at the top level | dropped, not yet picked up |
//! | `_ready/<date>-<name>/` | staged as a folder, waiting for a slot |
//! | `_running/<date>-<name>/` | running |
//! | `_paused/<date>-<name>/` | suspended — the process group is stopped |
//! | `_ok/<date>-<name>/` | finished clean |
//! | `_failed/<date>-<name>/` | finished badly, or was stopped |
//!
//! That makes the schema readable in Finder, editable by hand, and — because
//! moving a folder is the only gesture — it is also how the queue is
//! *commanded*. Anything that can write to the folder can pause, stop or
//! requeue a job by dragging it, and the filesystem's own permissions are the
//! access control. The runner is the only thing that touches a process.
//!
//! Over SMB every read here is a network round trip, so `_ok` and `_failed` —
//! the two that grow without bound — are cached against the directory's own
//! mtime and only re-read when something has actually changed.

use std::cmp::Reverse;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom};
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// How many finished jobs to carry in a snapshot.
const RECENT_MAX: usize = 8;

/// How long a claimed job must go without printing before silence counts
/// against it. Long enough to sit through a quiet stretch of an encode.
const SILENT_STALL: Duration = Duration::from_secs(10 * 60);

/// Only the tail of a log is read — enough for the last line however long the
/// file has grown, and one small read per poll over the network.
const TAIL_BYTES: u64 = 4096;

/// Longer lines are cut to this before they reach a menu.
const MAX_LINE: usize = 160;

/// A directory whose mtime is this recent is re-read rather than served from
/// cache: mtime has one-second granularity, so a change landing in the same
/// second as the cached read would otherwise be invisible until the next one.
const CACHE_SETTLE: Duration = Duration::from_secs(2);

/// What the runner leaves in a run folder so local clients can find the
/// process. It is state the runner emits, not a channel anyone writes to —
/// commands are folder moves.
pub const STATUS_FILE: &str = ".status";

/// A jobs folder — `~/jobs` locally, `/Volumes/Jobs` over a share.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Root {
    path: PathBuf,
}

impl Root {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    /// The local jobs folder, honouring `$JOBS_DIR`.
    pub fn local() -> Self {
        let path = std::env::var_os("JOBS_DIR").map(PathBuf::from).unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_default()
                .join("jobs")
        });
        Self::new(path)
    }

    /// The top level, where `.job` files are dropped.
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn dir(&self, state: State) -> PathBuf {
        self.path.join(state.dir_name())
    }

    pub fn ready(&self) -> PathBuf {
        self.dir(State::Ready)
    }

    pub fn running(&self) -> PathBuf {
        self.dir(State::Running)
    }

    pub fn paused(&self) -> PathBuf {
        self.dir(State::Paused)
    }

    pub fn ok(&self) -> PathBuf {
        self.dir(State::Ok)
    }

    pub fn failed(&self) -> PathBuf {
        self.dir(State::Failed)
    }

    /// Every state directory, for creating them up front.
    pub fn state_dirs(&self) -> [PathBuf; 5] {
        [
            self.ready(),
            self.running(),
            self.paused(),
            self.ok(),
            self.failed(),
        ]
    }

    /// A short name for menus: the folder's own last component.
    pub fn label(&self) -> String {
        self.path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| self.path.to_string_lossy().to_string())
    }
}

/// Where a job is, which is all there is to say about what it is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Ready,
    Running,
    Paused,
    Ok,
    Failed,
}

impl State {
    pub fn dir_name(self) -> &'static str {
        match self {
            State::Ready => "_ready",
            State::Running => "_running",
            State::Paused => "_paused",
            State::Ok => "_ok",
            State::Failed => "_failed",
        }
    }

    /// The state a directory name denotes, for reading a move back off disk.
    pub fn from_dir_name(name: &str) -> Option<State> {
        match name {
            "_ready" => Some(State::Ready),
            "_running" => Some(State::Running),
            "_paused" => Some(State::Paused),
            "_ok" => Some(State::Ok),
            "_failed" => Some(State::Failed),
            _ => None,
        }
    }

    /// The state implied by a folder's location, whatever moved it there.
    pub fn of(dir: &Path) -> Option<State> {
        dir.parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| State::from_dir_name(&name.to_string_lossy()))
    }
}

/// What the runner published about a job it is running.
#[derive(Clone, Copy, Debug)]
pub struct Status {
    /// The job's process group — signal this, not the shell's pid, or the
    /// encoder underneath carries on regardless.
    pub pgid: i32,
}

/// A job that hasn't finished: staged, running, or suspended.
#[derive(Clone, Debug)]
pub struct Run {
    pub name: String,
    pub dir: PathBuf,
    pub state: State,
    pub started: Option<SystemTime>,
    /// The last non-empty line the job printed, if it has printed anything.
    pub last_line: Option<String>,
    /// When that line landed.
    pub last_output: Option<SystemTime>,
    /// A percentage parsed out of the last line, 0..1.
    pub progress: Option<f64>,
    /// Present once the runner has spawned the job, and only meaningful on the
    /// machine that spawned it — `host` is recorded alongside so a monitor
    /// watching over SMB doesn't test a pid against its own process table.
    pub status: Option<Status>,
    pub local: bool,
}

impl Run {
    /// The live stdout log, once the job has written a first line — it is
    /// created lazily, so a silent job never has one.
    pub fn log_path(&self) -> Option<PathBuf> {
        let path = self.dir.join(format!("{}.log", self.name));
        path.is_file().then_some(path)
    }

    /// Time since the job last printed anything.
    pub fn silent_for(&self) -> Option<Duration> {
        self.last_output
            .map(|at| SystemTime::now().duration_since(at).unwrap_or_default())
    }

    /// Time since the job started, clamped at zero — the timestamp comes from
    /// the machine that owns the folder, whose clock may run ahead of ours.
    pub fn elapsed(&self) -> Option<Duration> {
        self.started
            .map(|started| SystemTime::now().duration_since(started).unwrap_or_default())
    }

    /// Whether the process is still there. Definitive, but only answerable on
    /// the machine running it: `kill(pgid, 0)` against another host's pid
    /// would be answering a different question entirely.
    pub fn alive(&self) -> Option<bool> {
        if !self.local {
            return None;
        }
        let pgid = self.status?.pgid;
        // Signal 0 performs the permission and existence checks without
        // sending anything.
        Some(unsafe { libc::killpg(pgid, 0) } == 0)
    }

    /// A job in `_running` that nothing is actually running. Prefers the
    /// process check and falls back to output silence when the folder belongs
    /// to another machine.
    pub fn is_stalled(&self) -> bool {
        if self.state != State::Running {
            return false;
        }
        match self.alive() {
            Some(alive) => !alive,
            None => {
                let quiet = self.silent_for().or_else(|| self.elapsed());
                quiet.is_some_and(|since| since > SILENT_STALL)
            }
        }
    }
}

/// A job that has finished and been filed away.
#[derive(Clone, Debug)]
pub struct Outcome {
    pub name: String,
    pub dir: PathBuf,
    pub ok: bool,
    /// Seconds since the epoch, from the run folder's ctime — the moment it
    /// was moved into `_ok` or `_failed`.
    pub finished: i64,
    pub started: Option<SystemTime>,
}

impl Outcome {
    /// How long the job ran for.
    pub fn took(&self) -> Option<Duration> {
        let started = self.started?.duration_since(UNIX_EPOCH).ok()?.as_secs() as i64;
        (self.finished > started).then(|| Duration::from_secs((self.finished - started) as u64))
    }

    pub fn ago(&self) -> Duration {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|since| since.as_secs() as i64)
            .unwrap_or(0);
        Duration::from_secs((now - self.finished).max(0) as u64)
    }
}

/// Everything the UIs render, as of one poll.
#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    /// The folder this is a snapshot of, so a row can work out where its
    /// buttons would move a job to.
    pub root: Option<Root>,
    /// False when the folder isn't there — an unmounted share, mostly. Kept
    /// distinct from "no jobs", because rendering a disconnected folder as an
    /// empty queue is a lie that reads as "nothing to do".
    pub connected: bool,
    /// `.job` files sitting at the top level, not yet staged. Normally empty:
    /// a running queue picks them up within a poll. A pile here means nothing
    /// is watching the folder.
    pub inbox: Vec<String>,
    /// Staged, running and paused jobs, in that order.
    pub jobs: Vec<Run>,
    pub recent: Vec<Outcome>,
    /// Failures newer than the acknowledgement timestamp handed to [`Observer::poll`].
    pub errors: usize,
}

impl Snapshot {
    pub fn in_state(&self, state: State) -> impl Iterator<Item = &Run> {
        self.jobs.iter().filter(move |job| job.state == state)
    }

    pub fn running(&self) -> impl Iterator<Item = &Run> {
        self.in_state(State::Running)
    }

    pub fn is_busy(&self) -> bool {
        !self.jobs.is_empty() || !self.inbox.is_empty()
    }

    /// Any claimed job nobody is working on.
    pub fn stalled(&self) -> bool {
        self.jobs.iter().any(Run::is_stalled)
    }
}

/// Reads a [`Root`] into a [`Snapshot`], holding the caches between polls.
///
/// Poll from a background thread, always: a `read_dir` on a dead SMB mount
/// blocks until the mount times out, and doing that on the main thread is how
/// a menu bar app comes to beachball.
pub struct Observer {
    root: Root,
    local: bool,
    ok: DirCache,
    failed: DirCache,
}

impl Observer {
    pub fn new(root: Root) -> Self {
        // Whether this is the machine running the jobs, decided once: it is
        // what makes the process checks meaningful.
        let local = root == Root::local();
        Self {
            root,
            local,
            ok: DirCache::default(),
            failed: DirCache::default(),
        }
    }

    pub fn root(&self) -> &Root {
        &self.root
    }

    /// `ack` is the epoch-seconds cutoff below which failures are considered
    /// already seen — the caller keeps that, since a monitor acknowledges
    /// errors on its own machine rather than writing to somebody's share.
    pub fn poll(&mut self, ack: i64) -> Snapshot {
        if !self.is_connected() {
            self.ok.clear();
            self.failed.clear();
            return Snapshot::default();
        }

        let ok = self.ok.read(&self.root.ok(), true);
        let failed = self.failed.read(&self.root.failed(), false);

        let mut recent: Vec<Outcome> = ok.iter().chain(failed.iter()).cloned().collect();
        recent.sort_by_key(|outcome| Reverse(outcome.finished));
        recent.truncate(RECENT_MAX);

        // Running first, then paused, then the queue — the order they are
        // worth reading in.
        let mut jobs = self.read_state(State::Running);
        jobs.extend(self.read_state(State::Paused));
        jobs.extend(self.read_state(State::Ready));

        Snapshot {
            root: Some(self.root.clone()),
            connected: true,
            inbox: self.read_inbox(),
            jobs,
            errors: failed.iter().filter(|outcome| outcome.finished > ack).count(),
            recent,
        }
    }

    /// The folder exists *and* looks like a jobs folder. A share that failed to
    /// mount can leave an empty `/Volumes/Jobs` behind, and reporting that as a
    /// healthy, idle queue would be worse than saying nothing.
    fn is_connected(&self) -> bool {
        self.root.path.is_dir()
            && self
                .root
                .state_dirs()
                .iter()
                .any(|dir| dir.is_dir())
    }

    /// Top-level `*.job` files, alphabetically — dropped but not yet staged.
    fn read_inbox(&self) -> Vec<String> {
        let Ok(entries) = fs::read_dir(&self.root.path) else {
            return Vec::new();
        };
        let mut inbox: Vec<String> = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.is_file()
                    && path
                        .file_name()
                        .is_some_and(|name| name.to_string_lossy().ends_with(".job"))
            })
            .map(|path| job_name(&path))
            .collect();
        inbox.sort();
        inbox
    }

    fn read_state(&self, state: State) -> Vec<Run> {
        let Ok(entries) = fs::read_dir(self.root.dir(state)) else {
            return Vec::new();
        };
        let mut jobs: Vec<Run> = entries
            .flatten()
            .filter(|entry| entry.path().is_dir())
            .map(|entry| {
                let dir = entry.path();
                let started = entry
                    .metadata()
                    .ok()
                    .and_then(|meta| meta.created().or_else(|_| meta.modified()).ok());
                let name = run_folder_name(&dir);
                let (last_line, last_output) = read_tail(&dir.join(format!("{name}.log")));
                let progress = last_line.as_deref().and_then(parse_progress);
                Run {
                    status: read_status(&dir),
                    name,
                    dir,
                    state,
                    started,
                    last_line,
                    last_output,
                    progress,
                    local: self.local,
                }
            })
            .collect();
        jobs.sort_by(|a, b| a.dir.cmp(&b.dir));
        jobs
    }
}

/// One directory's listing, held between polls and re-read only when the
/// directory's mtime says an entry was added or removed.
#[derive(Default)]
struct DirCache {
    mtime: Option<SystemTime>,
    entries: Vec<Outcome>,
}

impl DirCache {
    fn clear(&mut self) {
        self.mtime = None;
        self.entries.clear();
    }

    /// Returns a copy: the list is capped at [`RECENT_MAX`], so cloning it
    /// costs nothing next to the round trips the cache is there to avoid.
    fn read(&mut self, dir: &Path, ok: bool) -> Vec<Outcome> {
        let mtime = fs::metadata(dir).and_then(|meta| meta.modified()).ok();
        let settled = mtime.is_some_and(|mtime| {
            mtime.elapsed().map(|age| age > CACHE_SETTLE).unwrap_or(false)
        });
        if mtime.is_some() && mtime == self.mtime && settled {
            return self.entries.clone();
        }

        self.mtime = mtime;
        self.entries = match fs::read_dir(dir) {
            Err(_) => Vec::new(),
            Ok(entries) => {
                let mut outcomes: Vec<Outcome> = entries
                    .flatten()
                    .filter(|entry| entry.path().is_dir())
                    .map(|entry| {
                        let dir = entry.path();
                        let meta = entry.metadata().ok();
                        Outcome {
                            name: run_folder_name(&dir),
                            finished: meta.as_ref().map(|meta| meta.ctime()).unwrap_or(0),
                            started: meta.and_then(|meta| meta.created().ok()),
                            dir,
                            ok,
                        }
                    })
                    .collect();
                outcomes.sort_by_key(|outcome| Reverse(outcome.finished));
                outcomes.truncate(RECENT_MAX);
                outcomes
            }
        };
        self.entries.clone()
    }
}

/// Read the runner's `.status`. Only trusted when it names this host — a pid
/// from another machine means nothing to `killpg` here.
fn read_status(dir: &Path) -> Option<Status> {
    let text = fs::read_to_string(dir.join(STATUS_FILE)).ok()?;
    let mut pgid = None;
    let mut host = None;
    for line in text.lines() {
        match line.split_once('=') {
            Some(("pgid", value)) => pgid = value.trim().parse::<i32>().ok(),
            Some(("host", value)) => host = Some(value.trim().to_string()),
            _ => {}
        }
    }
    (host == Some(hostname())).then_some(Status { pgid: pgid? })
}

/// This machine's name, as recorded in a run folder's status.
pub fn hostname() -> String {
    let mut buffer = [0i8; 256];
    let ok = unsafe { libc::gethostname(buffer.as_mut_ptr(), buffer.len()) } == 0;
    if !ok {
        return String::new();
    }
    let bytes: Vec<u8> = buffer
        .iter()
        .take_while(|byte| **byte != 0)
        .map(|byte| *byte as u8)
        .collect();
    String::from_utf8_lossy(&bytes).to_string()
}

/// The last non-empty line of a log, and when it was written.
///
/// Only the final [`TAIL_BYTES`] are read, so this costs the same on a log of
/// ten lines and one of ten million. Carriage returns split like newlines: a
/// tool redrawing a progress bar in place writes `\r`, and the interesting
/// text is the segment after the last one, not the whole accumulated line.
fn read_tail(path: &Path) -> (Option<String>, Option<SystemTime>) {
    let Ok(mut file) = File::open(path) else {
        return (None, None);
    };
    let Ok(meta) = file.metadata() else {
        return (None, None);
    };
    let modified = meta.modified().ok();

    let len = meta.len();
    if len > TAIL_BYTES && file.seek(SeekFrom::End(-(TAIL_BYTES as i64))).is_err() {
        return (None, modified);
    }
    let mut buffer = Vec::new();
    if file.read_to_end(&mut buffer).is_err() {
        return (None, modified);
    }

    let text = String::from_utf8_lossy(&buffer);
    let line = text
        .split(['\n', '\r'])
        .map(str::trim)
        .rfind(|line| !line.is_empty())
        .map(|line| {
            if line.chars().count() > MAX_LINE {
                let cut: String = line.chars().take(MAX_LINE - 1).collect();
                format!("{cut}…")
            } else {
                line.to_string()
            }
        });
    (line, modified)
}

/// A percentage anywhere in a line, as a 0..1 fraction — `45%`, `at 45.5%`,
/// `[ 45% ]`. The last one wins, since a line that carries several is most
/// likely counting up to the one nearest its end.
fn parse_progress(line: &str) -> Option<f64> {
    let bytes = line.as_bytes();
    let mut found = None;
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'%' {
            continue;
        }
        let mut start = index;
        let mut seen_digit = false;
        let mut seen_dot = false;
        while start > 0 {
            let previous = bytes[start - 1];
            if previous.is_ascii_digit() {
                seen_digit = true;
            } else if previous == b'.' && !seen_dot && seen_digit {
                seen_dot = true;
            } else {
                break;
            }
            start -= 1;
        }
        if seen_digit && let Ok(value) = line[start..index].parse::<f64>() {
            found = Some((value / 100.0).clamp(0.0, 1.0));
        }
    }
    found
}

/// A job file's target: its name minus `.job` (`foo.mp4.job` => `foo.mp4`).
pub fn target_file(path: &Path) -> String {
    let base = path.file_name().unwrap_or_default().to_string_lossy();
    base.strip_suffix(".job").unwrap_or(&base).to_string()
}

/// A job's name: the target minus its own extension (`foo.mp4.job` => `foo`).
/// A job file with no inner extension keeps its whole name.
pub fn job_name(path: &Path) -> String {
    let target = target_file(path);
    match target.rsplit_once('.') {
        Some((stem, _)) if !stem.is_empty() => stem.to_string(),
        _ => target,
    }
}

/// The job name inside a run folder: `20260804-001607-video` => `video`. The
/// timestamp is fixed-width, so this is a slice rather than a parse; anything
/// that doesn't match the shape is shown as-is.
pub fn run_folder_name(dir: &Path) -> String {
    let base = dir.file_name().unwrap_or_default().to_string_lossy().to_string();
    let bytes = base.as_bytes();
    let shaped = bytes.len() > 16
        && bytes[..8].iter().all(u8::is_ascii_digit)
        && bytes[8] == b'-'
        && bytes[9..15].iter().all(u8::is_ascii_digit)
        && bytes[15] == b'-';
    if shaped { base[16..].to_string() } else { base }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_split_target_and_job() {
        let job = Path::new("/jobs/video.mp4.job");
        assert_eq!(target_file(job), "video.mp4");
        assert_eq!(job_name(job), "video");

        let bare = Path::new("/jobs/backup.job");
        assert_eq!(target_file(bare), "backup");
        assert_eq!(job_name(bare), "backup");

        let dotted = Path::new("/jobs/set.one.mov.job");
        assert_eq!(job_name(dotted), "set.one");
    }

    #[test]
    fn a_folders_parent_is_its_state() {
        assert_eq!(
            State::of(Path::new("/j/_running/20260804-001607-video")),
            Some(State::Running)
        );
        assert_eq!(
            State::of(Path::new("/j/_paused/20260804-001607-video")),
            Some(State::Paused)
        );
        assert_eq!(State::of(Path::new("/j/elsewhere/thing")), None);
    }

    /// Build a jobs tree in a scratch directory and read it back, which is the
    /// whole contract in one test: where a folder sits is what it is doing.
    #[test]
    fn a_folder_reads_back_as_a_snapshot() {
        let base = std::env::temp_dir().join(format!("job-core-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let root = Root::new(&base);
        for dir in root.state_dirs() {
            fs::create_dir_all(dir).unwrap();
        }

        fs::write(base.join("clip.mov.job"), "#!/bin/bash\n").unwrap();
        let run = root.running().join("20260804-001607-video");
        fs::create_dir_all(&run).unwrap();
        fs::write(run.join("video.log"), "encoding 45.5% eta 1:52\n").unwrap();
        fs::create_dir_all(root.ready().join("20260804-001700-later")).unwrap();
        fs::create_dir_all(root.paused().join("20260804-001500-held")).unwrap();
        fs::create_dir_all(root.ok().join("20260804-001500-earlier")).unwrap();
        fs::create_dir_all(root.failed().join("20260804-001530-broken")).unwrap();

        let mut observer = Observer::new(root.clone());
        let snapshot = observer.poll(0);

        assert!(snapshot.connected);
        assert_eq!(snapshot.inbox, vec!["clip".to_string()]);
        assert_eq!(snapshot.jobs.len(), 3);
        assert_eq!(snapshot.running().count(), 1);

        let running = snapshot.running().next().unwrap();
        assert_eq!(running.name, "video");
        assert!(running.log_path().is_some());
        assert_eq!(running.progress, Some(0.455));
        assert_eq!(snapshot.in_state(State::Paused).next().unwrap().name, "held");
        assert_eq!(snapshot.in_state(State::Ready).next().unwrap().name, "later");
        assert_eq!(snapshot.recent.len(), 2);
        assert_eq!(snapshot.errors, 1);

        // No .status and a log written a moment ago: nothing to declare dead.
        assert!(!snapshot.stalled());

        // Acknowledging past the failure clears the badge without touching it.
        let future = snapshot.recent.iter().map(|o| o.finished).max().unwrap() + 1;
        assert_eq!(observer.poll(future).errors, 0);

        // A folder that isn't there reads as disconnected, never as idle.
        fs::remove_dir_all(&base).unwrap();
        let gone = observer.poll(0);
        assert!(!gone.connected);
        assert!(gone.jobs.is_empty());
    }

    #[test]
    fn run_folders_lose_their_timestamp() {
        assert_eq!(run_folder_name(Path::new("/j/_ok/20260804-001607-video")), "video");
        // The collision suffix stays: it is part of how the folder is named.
        assert_eq!(run_folder_name(Path::new("/j/_ok/20260804-001607-video-2")), "video-2");
        // Anything not matching the shape is left alone rather than sliced.
        assert_eq!(run_folder_name(Path::new("/j/_ok/handmade")), "handmade");
        assert_eq!(run_folder_name(Path::new("/j/_ok/2026-08-04-video")), "2026-08-04-video");
    }
}
