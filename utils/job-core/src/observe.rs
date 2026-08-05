//! Turning a jobs folder into a [`Snapshot`], by reading it and nothing else.
//!
//! Every piece of state the UIs show is derivable from the tree, which is what
//! makes a remote monitor possible at all — and what makes a restarted
//! `job-server` pick up exactly where it left off instead of forgetting the job
//! it is in the middle of running:
//!
//! | on disk | means |
//! |---|---|
//! | `TARGET.job` at the top level | queued |
//! | `_running/<date>-<name>/` | running (started = the folder's birth time) |
//! | `_done/<date>-<name>/` | succeeded (finished = its ctime, set by the move) |
//! | `_err/<date>-<name>/` | failed |
//! | `.paused` | the queue is held |
//! | `.lock` | a runner holds the folder; its mtime is the heartbeat |
//!
//! Over SMB every one of those reads is a network round trip, so `_done` and
//! `_err` — the two that grow without bound — are cached against the
//! directory's own mtime and only re-read when something has actually changed.

use std::cmp::Reverse;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// How many finished jobs to carry in a snapshot.
const RECENT_MAX: usize = 8;

/// A runner that hasn't touched `.lock` in this long has died; it matches the
/// threshold at which a runner steals another's lock, so the two agree on when
/// an abandoned folder is genuinely abandoned.
const LOCK_STALE: Duration = Duration::from_secs(60 * 60);

/// A directory whose mtime is this recent is re-read rather than served from
/// cache: mtime has one-second granularity, so a change landing in the same
/// second as the cached read would otherwise be invisible until the next one.
const CACHE_SETTLE: Duration = Duration::from_secs(2);

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

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn running(&self) -> PathBuf {
        self.path.join("_running")
    }

    pub fn done(&self) -> PathBuf {
        self.path.join("_done")
    }

    pub fn err(&self) -> PathBuf {
        self.path.join("_err")
    }

    pub fn lock(&self) -> PathBuf {
        self.path.join(".lock")
    }

    pub fn paused_marker(&self) -> PathBuf {
        self.path.join(".paused")
    }

    /// A short name for menus: the folder's own last component.
    pub fn label(&self) -> String {
        self.path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| self.path.to_string_lossy().to_string())
    }
}

/// A job the runner currently has claimed.
#[derive(Clone, Debug)]
pub struct Run {
    pub name: String,
    pub dir: PathBuf,
    pub started: Option<SystemTime>,
}

impl Run {
    /// The live stdout log, once the job has written a first line — it is
    /// created lazily, so a silent job never has one.
    pub fn log_path(&self) -> Option<PathBuf> {
        let path = self.dir.join(format!("{}.log", self.name));
        path.is_file().then_some(path)
    }

    /// Time since the job started. `None` when the folder's birth time is
    /// unreadable; clamped at zero, since the timestamp comes from the machine
    /// that owns the folder and its clock may be a little ahead of ours.
    pub fn elapsed(&self) -> Option<Duration> {
        self.started
            .map(|started| SystemTime::now().duration_since(started).unwrap_or_default())
    }
}

/// A job that has finished and been filed away.
#[derive(Clone, Debug)]
pub struct Outcome {
    pub name: String,
    pub dir: PathBuf,
    pub ok: bool,
    /// Seconds since the epoch, from the run folder's ctime — the moment it
    /// was moved into `_done` or `_err`.
    pub finished: i64,
}

impl Outcome {
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
    /// False when the folder isn't there — an unmounted share, mostly. Kept
    /// distinct from "no jobs", because rendering a disconnected folder as an
    /// empty queue is a lie that reads as "nothing to do".
    pub connected: bool,
    pub paused: bool,
    /// A job is claimed but no runner is holding the lock: the runner died
    /// mid-job and the run folder has been left behind.
    pub stalled: bool,
    pub queued: Vec<String>,
    pub running: Vec<Run>,
    pub recent: Vec<Outcome>,
    /// Failures newer than the acknowledgement timestamp handed to [`Observer::poll`].
    pub errors: usize,
}

impl Snapshot {
    pub fn is_busy(&self) -> bool {
        !self.running.is_empty() || !self.queued.is_empty()
    }
}

/// Reads a [`Root`] into a [`Snapshot`], holding the caches between polls.
///
/// Poll from a background thread, always: a `read_dir` on a dead SMB mount
/// blocks until the mount times out, and doing that on the main thread is how
/// a menu bar app comes to beachball.
pub struct Observer {
    root: Root,
    done: DirCache,
    err: DirCache,
}

impl Observer {
    pub fn new(root: Root) -> Self {
        Self {
            root,
            done: DirCache::default(),
            err: DirCache::default(),
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
            self.done.clear();
            self.err.clear();
            return Snapshot::default();
        }

        let done = self.done.read(&self.root.done(), true);
        let err = self.err.read(&self.root.err(), false);

        let mut recent: Vec<Outcome> = done.iter().chain(err.iter()).cloned().collect();
        recent.sort_by_key(|outcome| Reverse(outcome.finished));
        recent.truncate(RECENT_MAX);

        let running = self.read_running();
        Snapshot {
            connected: true,
            paused: self.root.paused_marker().exists(),
            stalled: !running.is_empty() && self.lock_is_dead(),
            queued: self.read_queued(),
            running,
            errors: err.iter().filter(|outcome| outcome.finished > ack).count(),
            recent,
        }
    }

    /// The folder exists *and* looks like a jobs folder. A share that failed to
    /// mount can leave an empty `/Volumes/Jobs` behind, and reporting that as a
    /// healthy, idle queue would be worse than saying nothing.
    fn is_connected(&self) -> bool {
        self.root.path.is_dir()
            && (self.root.running().is_dir() || self.root.done().is_dir() || self.root.err().is_dir())
    }

    /// Top-level `*.job` files, alphabetically — the runner's scan order. A
    /// claimed job has moved into `_running/`, so it never shows up here.
    fn read_queued(&self) -> Vec<String> {
        let Ok(entries) = fs::read_dir(&self.root.path) else {
            return Vec::new();
        };
        let mut queued: Vec<String> = entries
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
        queued.sort();
        queued
    }

    fn read_running(&self) -> Vec<Run> {
        let Ok(entries) = fs::read_dir(self.root.running()) else {
            return Vec::new();
        };
        let mut running: Vec<Run> = entries
            .flatten()
            .filter(|entry| entry.path().is_dir())
            .map(|entry| {
                let dir = entry.path();
                let started = entry
                    .metadata()
                    .ok()
                    .and_then(|meta| meta.created().or_else(|_| meta.modified()).ok());
                Run {
                    name: run_folder_name(&dir),
                    dir,
                    started,
                }
            })
            .collect();
        running.sort_by(|a, b| a.dir.cmp(&b.dir));
        running
    }

    /// True when nothing is holding the lock, or the holder stopped
    /// heartbeating long enough ago that a runner would steal it. A runner
    /// killed cleanly removes the lock on its way out; one killed outright
    /// leaves it behind to go stale.
    fn lock_is_dead(&self) -> bool {
        match fs::metadata(self.root.lock()) {
            Err(_) => true,
            Ok(meta) => meta
                .modified()
                .map(|beat| beat.elapsed().unwrap_or_default() > LOCK_STALE)
                .unwrap_or(false),
        }
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
                        Outcome {
                            name: run_folder_name(&dir),
                            finished: entry.metadata().map(|meta| meta.ctime()).unwrap_or(0),
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

    /// Build a jobs tree in a scratch directory and read it back, which is the
    /// whole contract in one test: what the runner writes is what a monitor on
    /// another machine sees.
    #[test]
    fn a_folder_reads_back_as_a_snapshot() {
        let base = std::env::temp_dir().join(format!("job-core-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let root = Root::new(&base);
        for dir in [root.running(), root.done(), root.err()] {
            fs::create_dir_all(dir).unwrap();
        }

        fs::write(base.join("clip.mov.job"), "#!/bin/bash\n").unwrap();
        fs::write(base.join("later.mp4.job"), "#!/bin/bash\n").unwrap();
        let run = root.running().join("20260804-001607-video");
        fs::create_dir_all(&run).unwrap();
        fs::write(run.join("video.log"), "working\n").unwrap();
        fs::create_dir_all(root.done().join("20260804-001500-earlier")).unwrap();
        fs::create_dir_all(root.err().join("20260804-001530-broken")).unwrap();

        let mut observer = Observer::new(root.clone());
        let snapshot = observer.poll(0);

        assert!(snapshot.connected);
        assert!(!snapshot.paused);
        assert_eq!(snapshot.queued, vec!["clip".to_string(), "later".to_string()]);
        assert_eq!(snapshot.running.len(), 1);
        assert_eq!(snapshot.running[0].name, "video");
        assert!(snapshot.running[0].log_path().is_some());
        // No .lock beside a claimed job: the runner is gone.
        assert!(snapshot.stalled);
        assert_eq!(snapshot.recent.len(), 2);
        assert_eq!(snapshot.errors, 1);

        // Acknowledging past the failure clears the badge without touching it.
        let future = snapshot.recent.iter().map(|o| o.finished).max().unwrap() + 1;
        assert_eq!(observer.poll(future).errors, 0);

        // A live runner is holding the folder, so nothing is stalled.
        fs::create_dir_all(root.lock()).unwrap();
        assert!(!observer.poll(0).stalled);

        fs::write(root.paused_marker(), "").unwrap();
        assert!(observer.poll(0).paused);

        // A folder that isn't there reads as disconnected, never as idle.
        fs::remove_dir_all(&base).unwrap();
        let gone = observer.poll(0);
        assert!(!gone.connected);
        assert!(gone.queued.is_empty());
    }

    #[test]
    fn run_folders_lose_their_timestamp() {
        assert_eq!(run_folder_name(Path::new("/j/_done/20260804-001607-video")), "video");
        // The collision suffix stays: it is part of how the folder is named.
        assert_eq!(run_folder_name(Path::new("/j/_done/20260804-001607-video-2")), "video-2");
        // Anything not matching the shape is left alone rather than sliced.
        assert_eq!(run_folder_name(Path::new("/j/_done/handmade")), "handmade");
        assert_eq!(run_folder_name(Path::new("/j/_done/2026-08-04-video")), "2026-08-04-video");
    }
}
