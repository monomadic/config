//! The job loop: a Rust port of `utils/job-server-cli/job-server-cli`, running
//! in-process instead of behind a launchd WatchPaths trigger.
//!
//! The on-disk contract is deliberately identical, so `.job` scripts and
//! `send-job` work against either implementation:
//!
//!   1. `TARGET.job` — and the target file beside it, if there is one — is
//!      moved into a dated run folder `_running/<date>-<JOB_NAME>` to claim
//!      it. The scan only matches top-level `*.job`, so a claimed job can
//!      never be picked up twice. One job runs at a time, guarded by the same
//!      `$JOBS_DIR/.lock` directory the shell version uses.
//!   2. It runs cd'd into the run folder with `TARGET_FILE`, `JOB_NAME`,
//!      `JOB_FILE`, `JOB_DIR`, `JOB_RUN_DIR` exported and `TERM=dumb` /
//!      `NO_COLOR=1` / `CLICOLOR=0` set. stdout goes to `$JOB_NAME.log`,
//!      stderr to `$JOB_NAME.error.log`, each created only once that stream
//!      produces output, so a silent job leaves no empty artifacts behind.
//!   3. On exit 0 the whole run folder is moved to `_done/`; otherwise to
//!      `_err/` — job file, target file and logs together.
//!
//! This module only ever *writes* the folder. What the menu shows is read back
//! off disk by `job_core::Observer`, so the display cannot drift from the
//! truth, survives a restart mid-job, and is the same view `job-monitor` gets
//! from another machine.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use job_core::clock;
use job_core::observe::{Root, job_name, target_file};

const POLL: Duration = Duration::from_secs(1);
const SETTLE: Duration = Duration::from_secs(2);
const LOCK_STALE: Duration = Duration::from_secs(60 * 60);
const LOCK_HEARTBEAT: Duration = Duration::from_secs(60);

pub fn root() -> Root {
    Root::local()
}

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

fn log_file() -> PathBuf {
    std::env::var_os("JOB_SERVER_LOG")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join("Library/Logs/job-server.log"))
}

/// Where this machine records which failures the user has already seen. Kept
/// out of the jobs folder itself: acknowledgement is per-viewer, and whoever is
/// looking may not even have write access to the folder.
pub fn ack_file() -> PathBuf {
    home().join(".config/job-server/ack")
}

pub fn read_ack() -> i64 {
    fs::read_to_string(ack_file())
        .ok()
        .and_then(|text| text.trim().parse::<i64>().ok())
        .unwrap_or(0)
}

pub fn acknowledge_errors() {
    let path = ack_file();
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_secs() as i64)
        .unwrap_or(0);
    let _ = fs::write(path, now.to_string());
}

pub fn set_paused(paused: bool) {
    let marker = root().paused_marker();
    if paused {
        let _ = fs::write(&marker, "");
    } else {
        let _ = fs::remove_file(&marker);
    }
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

/// Top-level `*.job` files, alphabetically — the same scan order the shell
/// version's glob produces. A claimed job has moved into `_running/`, so it
/// can't match again.
fn scan_jobs(root: &Root) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(root.path()) else {
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

/// True once the file's size has held steady across SETTLE (i.e. a copy into
/// the folder has finished).
fn is_stable(path: &Path) -> bool {
    let size = |p: &Path| fs::metadata(p).map(|m| m.len()).ok();
    let Some(first) = size(path) else { return false };
    thread::sleep(SETTLE);
    size(path) == Some(first)
}

/// A non-colliding directory path: appends `-2`, `-3`, ... if taken, so an
/// earlier run's folder is never clobbered.
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

/// Copy a child stream to `path`, creating the file only when the first
/// bytes arrive — a stream that stays silent never gets a file at all.
/// Flushed per chunk so a running job's log can be tailed.
fn pump(mut stream: impl Read, path: PathBuf) {
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
}

struct Lock {
    dir: PathBuf,
}

impl Lock {
    /// The same `mkdir`-based lock the shell version uses, so the two can
    /// never run a job at the same time. A lock left behind by a killed run
    /// (older than LOCK_STALE) is stolen.
    fn acquire(root: &Root) -> Option<Lock> {
        let dir = root.lock();
        if fs::create_dir(&dir).is_ok() {
            return Some(Lock { dir });
        }
        let stale = fs::metadata(&dir)
            .and_then(|meta| meta.modified())
            .map(|modified| modified.elapsed().unwrap_or_default() > LOCK_STALE)
            .unwrap_or(false);
        if stale {
            let _ = fs::remove_dir_all(&dir);
            if fs::create_dir(&dir).is_ok() {
                return Some(Lock { dir });
            }
        }
        None
    }

    /// Touch the lock so a job that outlives LOCK_STALE isn't declared dead by
    /// another runner — or reported as stalled by a monitor.
    fn heartbeat(&self) {
        let beat = self.dir.join("heartbeat");
        let _ = fs::write(&beat, "");
        let _ = fs::remove_file(&beat);
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

fn run_one(root: &Root, job: &Path, lock: &Lock) {
    let name = job_name(job);
    let target = target_file(job);

    // Claim the job by moving it — and its target file, if present — out of
    // the scan set and into a dated run folder, which becomes the CWD.
    let _ = fs::create_dir_all(root.running());
    let run_dir = uniq_dir(root.running().join(format!("{}-{name}", clock::file_stamp())));
    if fs::create_dir(&run_dir).is_err() {
        log(&format!("SKIP {name} (could not create run folder)"));
        return;
    }
    let job_file = run_dir.join(format!("{target}.job"));
    if fs::rename(job, &job_file).is_err() {
        let _ = fs::remove_dir(&run_dir);
        log(&format!("SKIP {name} (could not claim job file)"));
        return;
    }
    let target_path = root.path().join(&target);
    if target_path.exists() && fs::rename(&target_path, run_dir.join(&target)).is_err() {
        log(&format!("WARN could not move target file {target}"));
    }

    let log_path = run_dir.join(format!("{name}.log"));
    let err_path = run_dir.join(format!("{name}.error.log"));
    log(&format!("RUN  {name}"));

    if let Ok(meta) = fs::metadata(&job_file) {
        let mut perms = meta.permissions();
        perms.set_mode(perms.mode() | 0o100);
        let _ = fs::set_permissions(&job_file, perms);
    }
    let executable = fs::metadata(&job_file).is_ok_and(|m| m.permissions().mode() & 0o111 != 0);

    // TARGET_FILE lets a job reference the file it was named after without
    // knowing the folder scheme (foo.mp4.job => TARGET_FILE=foo.mp4,
    // JOB_NAME=foo). TERM=dumb / NO_COLOR make well-behaved tools drop
    // progress-bar redraws and ANSI color.
    let mut command = if executable {
        Command::new(&job_file)
    } else {
        let mut command = Command::new("/bin/bash");
        command.arg(&job_file);
        command
    };
    let spawned = command
        .current_dir(&run_dir)
        .env("JOB_NAME", &name)
        .env("TARGET_FILE", &target)
        .env("JOB_FILE", &job_file)
        .env("JOB_DIR", root.path())
        .env("JOB_RUN_DIR", &run_dir)
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
            finish(root, &name, 127, &run_dir);
            return;
        }
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let log_writer = thread::spawn(move || {
        if let Some(stream) = stdout {
            pump(stream, log_path);
        }
    });
    let err_writer = thread::spawn(move || {
        if let Some(stream) = stderr {
            pump(stream, err_path);
        }
    });

    // Poll rather than block so the lock can be kept warm under long jobs.
    let mut last_beat = Instant::now();
    let code = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status.code().unwrap_or(1),
            Ok(None) => {
                thread::sleep(POLL);
                if last_beat.elapsed() >= LOCK_HEARTBEAT {
                    lock.heartbeat();
                    last_beat = Instant::now();
                }
            }
            Err(_) => break 1,
        }
    };

    let _ = log_writer.join();
    let _ = err_writer.join();
    finish(root, &name, code, &run_dir);
}

/// Move the whole run folder — job file, target file and logs — to `_done`
/// or `_err` according to the exit status.
fn finish(root: &Root, name: &str, code: i32, run_dir: &Path) {
    let dest_dir = if code == 0 {
        log(&format!("DONE {name} (exit 0)"));
        root.done()
    } else {
        log(&format!("FAIL {name} (exit {code})"));
        root.err()
    };

    let _ = fs::create_dir_all(&dest_dir);
    let folder = run_dir.file_name().unwrap_or_default().to_os_string();
    if fs::rename(run_dir, uniq_dir(dest_dir.join(&folder))).is_err() {
        log(&format!("WARN could not move run folder for {name}"));
    }
}

pub fn spawn() {
    thread::spawn(move || {
        let root = root();
        for dir in [root.path().to_path_buf(), root.running(), root.done(), root.err()] {
            let _ = fs::create_dir_all(dir);
        }
        loop {
            if root.paused_marker().exists() || scan_jobs(&root).is_empty() {
                thread::sleep(POLL);
                continue;
            }

            let Some(lock) = Lock::acquire(&root) else {
                thread::sleep(POLL);
                continue;
            };

            // Drain the folder before releasing the lock, so files that
            // arrive mid-run are still picked up (sequentially).
            loop {
                if root.paused_marker().exists() {
                    break;
                }
                let Some(job) = scan_jobs(&root).into_iter().next() else {
                    break;
                };
                if !is_stable(&job) {
                    log(&format!("WAIT {} (still uploading)", job_name(&job)));
                    break;
                }
                // The target file may still be copying in even once the job
                // file itself settled.
                let target = root.path().join(target_file(&job));
                if target.is_file() && !is_stable(&target) {
                    log(&format!("WAIT {} (target still uploading)", job_name(&job)));
                    break;
                }
                run_one(&root, &job, &lock);
            }
            drop(lock);
            thread::sleep(POLL);
        }
    });
}
