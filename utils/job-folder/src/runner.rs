//! The job loop: a Rust port of `utils/job-runner/job-runner`, running
//! in-process instead of behind a launchd WatchPaths trigger.
//!
//! The on-disk contract is deliberately identical, so `.job` scripts and
//! `send-job` work against either implementation:
//!
//!   1. `NAME.job` is renamed to `NAME.job.running` to claim it — the scan
//!      only matches `*.job`, so a claimed job can never be picked up twice.
//!      One job runs at a time, guarded by the same `$JOBS_DIR/.lock`
//!      directory the shell version uses.
//!   2. It runs cd'd into `$JOBS_DIR` with `JOB_NAME`, `JOB_FILE`, `JOB_DIR`
//!      exported and `TERM=dumb` / `NO_COLOR=1` / `CLICOLOR=0` set.
//!      stdout goes to `NAME.job.log`, stderr to `NAME.job.errors`, each
//!      created only once that stream produces output, so a silent job
//!      leaves no empty artifacts behind.
//!   3. On exit 0 the job lands in `_done/` as `NAME.job.done`; otherwise in
//!      `_err/` as `NAME.job.err`. Logs follow the job; an `.errors` file
//!      always lands in `_err/`.

use std::collections::VecDeque;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::clock;

const POLL: Duration = Duration::from_secs(1);
const SETTLE: Duration = Duration::from_secs(2);
const LOCK_STALE: Duration = Duration::from_secs(60 * 60);
const LOCK_HEARTBEAT: Duration = Duration::from_secs(60);
const RECENT_MAX: usize = 6;

pub struct RunningJob {
    pub name: String,
    pub started: Instant,
}

pub struct RecentJob {
    pub name: String,
    pub ok: bool,
    pub finished: Instant,
    pub artifact: PathBuf,
}

#[derive(Default)]
pub struct State {
    pub running: Option<RunningJob>,
    pub queued: Vec<String>,
    pub errors: usize,
    pub paused: bool,
    pub recent: VecDeque<RecentJob>,
}

pub fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

pub fn jobs_dir() -> PathBuf {
    std::env::var_os("JOBS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join("jobs"))
}

pub fn done_dir() -> PathBuf {
    jobs_dir().join("_done")
}

pub fn err_dir() -> PathBuf {
    jobs_dir().join("_err")
}

fn lock_dir() -> PathBuf {
    jobs_dir().join(".lock")
}

pub fn paused_marker() -> PathBuf {
    jobs_dir().join(".paused")
}

fn ack_file() -> PathBuf {
    home().join(".config/job-folder/ack")
}

fn log_file() -> PathBuf {
    std::env::var_os("JOB_FOLDER_LOG")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join("Library/Logs/job-folder.log"))
}

/// The running job's live stdout log, if the job has produced output yet.
pub fn running_log_path(name: &str) -> Option<PathBuf> {
    let path = jobs_dir().join(format!("{name}.job.log"));
    path.is_file().then_some(path)
}

fn log(line: &str) {
    let path = log_file();
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = writeln!(file, "{} {line}", clock::timestamp());
    }
}

/// Failures the user hasn't acknowledged: `_err/*.job.err` newer than the
/// timestamp written by "Clear error badge". Counting from disk (rather than
/// a counter in memory) means the badge survives a restart.
fn unacknowledged_errors() -> usize {
    let ack = fs::read_to_string(ack_file())
        .ok()
        .and_then(|text| text.trim().parse::<u64>().ok())
        .unwrap_or(0);

    let Ok(entries) = fs::read_dir(err_dir()) else {
        return 0;
    };
    entries
        .flatten()
        .filter(|entry| {
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                return false;
            };
            if !name.ends_with(".job.err") {
                return false;
            }
            entry
                .metadata()
                .and_then(|meta| meta.modified())
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .is_some_and(|since| since.as_secs() > ack)
        })
        .count()
}

pub fn acknowledge_errors() {
    let path = ack_file();
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_secs())
        .unwrap_or(0);
    let _ = fs::write(path, now.to_string());
}

pub fn set_paused(paused: bool) {
    let marker = paused_marker();
    if paused {
        let _ = fs::write(&marker, "");
    } else {
        let _ = fs::remove_file(&marker);
    }
}

/// Top-level `*.job` files, alphabetically — the same scan order the shell
/// version's glob produces. In-flight and finished names (`*.job.running`,
/// `*.job.done`, `*.job.err`) no longer end in `.job`, so they can't match.
fn scan_jobs() -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(jobs_dir()) else {
        return Vec::new();
    };
    let mut jobs: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file() && path.file_name().is_some_and(|n| n.to_string_lossy().ends_with(".job"))
        })
        .collect();
    jobs.sort();
    jobs
}

fn job_name(path: &Path) -> String {
    path.file_name()
        .map(|n| n.to_string_lossy().trim_end_matches(".job").to_string())
        .unwrap_or_default()
}

/// True once the file's size has held steady across SETTLE (i.e. a copy into
/// the folder has finished).
fn is_stable(path: &Path) -> bool {
    let size = |p: &Path| fs::metadata(p).map(|m| m.len()).ok();
    let Some(first) = size(path) else { return false };
    thread::sleep(SETTLE);
    size(path) == Some(first)
}

/// A non-colliding path: if `path` exists, insert a timestamp before the
/// `.job.<suffix>` tail so an earlier run's artifact is never clobbered.
fn uniq_dest(path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }
    let Some(dir) = path.parent() else { return path };
    let base = path.file_name().unwrap_or_default().to_string_lossy().to_string();
    let Some((stem, tail)) = base.split_once(".job.") else {
        return path;
    };
    dir.join(format!("{stem}.{}.job.{tail}", clock::file_stamp()))
}

/// Copy a child stream to `path`, creating the file only when the first
/// bytes arrive — a stream that stays silent never gets a file at all.
/// Returns whether anything was written. Flushed per chunk so a running
/// job's log can be tailed.
fn pump(mut stream: impl Read, path: PathBuf) -> bool {
    let mut file: Option<File> = None;
    let mut buffer = [0u8; 8192];
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
            }
        }
    }
    file.is_some()
}

struct Lock;

impl Lock {
    /// The same `mkdir`-based lock the shell version uses, so the two can
    /// never run a job at the same time. A lock left behind by a killed run
    /// (older than LOCK_STALE) is stolen.
    fn acquire() -> Option<Lock> {
        let dir = lock_dir();
        if fs::create_dir(&dir).is_ok() {
            return Some(Lock);
        }
        let stale = fs::metadata(&dir)
            .and_then(|meta| meta.modified())
            .map(|modified| modified.elapsed().unwrap_or_default() > LOCK_STALE)
            .unwrap_or(false);
        if stale {
            let _ = fs::remove_dir_all(&dir);
            if fs::create_dir(&dir).is_ok() {
                return Some(Lock);
            }
        }
        None
    }

    /// Touch the lock so a job that outlives LOCK_STALE isn't declared dead
    /// by another instance.
    fn heartbeat(&self) {
        let beat = lock_dir().join("heartbeat");
        let _ = fs::write(&beat, "");
        let _ = fs::remove_file(&beat);
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(lock_dir());
    }
}

fn run_one(job: &Path, state: &Arc<Mutex<State>>) {
    let jobs = jobs_dir();
    let name = job_name(job);
    let running = jobs.join(format!("{name}.job.running"));
    let log_path = jobs.join(format!("{name}.job.log"));
    let err_path = jobs.join(format!("{name}.job.errors"));

    // Claim the job by renaming it out of the scan set before running.
    if fs::rename(job, &running).is_err() {
        log(&format!("SKIP {name} (could not mark running)"));
        return;
    }
    log(&format!("RUN  {name}"));

    if let Ok(meta) = fs::metadata(&running) {
        let mut perms = meta.permissions();
        perms.set_mode(perms.mode() | 0o100);
        let _ = fs::set_permissions(&running, perms);
    }
    let executable = fs::metadata(&running).is_ok_and(|m| m.permissions().mode() & 0o111 != 0);

    if let Ok(mut state) = state.lock() {
        state.running = Some(RunningJob {
            name: name.clone(),
            started: Instant::now(),
        });
    }

    // JOB_NAME lets a job reference its target without knowing the rename
    // scheme (foo.mp4.job => JOB_NAME=foo.mp4). TERM=dumb / NO_COLOR make
    // well-behaved tools drop progress-bar redraws and ANSI color.
    let mut command = if executable {
        Command::new(&running)
    } else {
        let mut command = Command::new("/bin/bash");
        command.arg(&running);
        command
    };
    let spawned = command
        .current_dir(&jobs)
        .env("JOB_NAME", &name)
        .env("JOB_FILE", &running)
        .env("JOB_DIR", &jobs)
        .env("TERM", "dumb")
        .env("NO_COLOR", "1")
        .env("CLICOLOR", "0")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    let mut child = match spawned {
        Ok(child) => child,
        Err(err) => {
            log(&format!("FAIL {name} (could not start: {err})"));
            finish(&name, 127, &running, false, false, state);
            return;
        }
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let log_writer = thread::spawn(move || stdout.is_some_and(|s| pump(s, log_path)));
    let err_writer = thread::spawn(move || stderr.is_some_and(|s| pump(s, err_path)));

    // Poll rather than block so the lock can be kept warm under long jobs.
    let mut last_beat = Instant::now();
    let code = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status.code().unwrap_or(1),
            Ok(None) => {
                thread::sleep(POLL);
                if last_beat.elapsed() >= LOCK_HEARTBEAT {
                    Lock.heartbeat();
                    last_beat = Instant::now();
                }
            }
            Err(_) => break 1,
        }
    };

    let wrote_log = log_writer.join().unwrap_or(false);
    let wrote_err = err_writer.join().unwrap_or(false);
    finish(&name, code, &running, wrote_log, wrote_err, state);
}

fn finish(
    name: &str,
    code: i32,
    running: &Path,
    wrote_log: bool,
    wrote_err: bool,
    state: &Arc<Mutex<State>>,
) {
    let jobs = jobs_dir();
    let ok = code == 0;
    let (dest_dir, dest_job) = if ok {
        log(&format!("DONE {name} (exit 0)"));
        (done_dir(), done_dir().join(format!("{name}.job.done")))
    } else {
        log(&format!("FAIL {name} (exit {code})"));
        (err_dir(), err_dir().join(format!("{name}.job.err")))
    };

    let _ = fs::create_dir_all(&dest_dir);
    let dest_job = uniq_dest(dest_job);
    // NAME[.ts].job — the artifact name minus its final suffix.
    let stem = dest_job
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| name.to_string());

    if fs::rename(running, &dest_job).is_err() {
        log(&format!("WARN could not move {name} job artifact"));
    }
    if wrote_log {
        let source = jobs.join(format!("{name}.job.log"));
        let _ = fs::rename(source, uniq_dest(dest_dir.join(format!("{stem}.log"))));
    }
    if wrote_err {
        let _ = fs::create_dir_all(err_dir());
        let source = jobs.join(format!("{name}.job.errors"));
        let _ = fs::rename(source, uniq_dest(err_dir().join(format!("{stem}.errors"))));
    }

    if let Ok(mut state) = state.lock() {
        state.running = None;
        state.recent.push_front(RecentJob {
            name: name.to_string(),
            ok,
            finished: Instant::now(),
            artifact: dest_job,
        });
        state.recent.truncate(RECENT_MAX);
    }
}

pub fn spawn(state: Arc<Mutex<State>>) {
    thread::spawn(move || {
        for dir in [jobs_dir(), done_dir(), err_dir()] {
            let _ = fs::create_dir_all(dir);
        }
        loop {
            let paused = paused_marker().exists();
            let jobs = scan_jobs();

            if let Ok(mut state) = state.lock() {
                state.paused = paused;
                state.errors = unacknowledged_errors();
                state.queued = jobs.iter().map(|path| job_name(path)).collect();
            }

            if paused || jobs.is_empty() {
                thread::sleep(POLL);
                continue;
            }

            let Some(lock) = Lock::acquire() else {
                thread::sleep(POLL);
                continue;
            };

            // Drain the folder before releasing the lock, so files that
            // arrive mid-run are still picked up (sequentially).
            loop {
                if paused_marker().exists() {
                    break;
                }
                let Some(job) = scan_jobs().into_iter().next() else {
                    break;
                };
                if !is_stable(&job) {
                    log(&format!("WAIT {} (still uploading)", job_name(&job)));
                    break;
                }
                run_one(&job, &state);
                if let Ok(mut state) = state.lock() {
                    state.errors = unacknowledged_errors();
                    state.queued = scan_jobs().iter().map(|path| job_name(path)).collect();
                }
            }
            drop(lock);
            thread::sleep(POLL);
        }
    });
}
