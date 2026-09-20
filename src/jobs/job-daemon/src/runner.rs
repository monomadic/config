//! The job loop.
//!
//! A job is a folder, and the folder it sits in is its state. Nothing else —
//! no lock file, no pause flag, no status protocol the tools have to agree on:
//!
//!   1. `TARGET.job` is dropped at the top level, with its target file beside
//!      it. The runner stages both into `_ready/<date>-<name>/`, claiming it
//!      by `mkdir` — which fails if the folder exists, so two runners racing
//!      the same job can't both win.
//!   2. When a slot frees, `_ready/<x>` is promoted to `_running/<x>` with
//!      `RENAME_EXCL`, and the job runs cd'd into it with `TARGET_FILE`,
//!      `JOB_NAME`, `JOB_FILE`, `JOB_DIR` and `JOB_RUN_DIR` exported, and
//!      `TERM=dumb` / `NO_COLOR=1` / `CLICOLOR=0` set. stdout goes to
//!      `$JOB_NAME.log`, stderr to `$JOB_NAME.error.log`, each created only
//!      once that stream produces output.
//!   3. On exit 0 the folder moves to `_ok/`, otherwise `_failed/`.
//!
//! And because the folder is the state, moving it is how the queue is
//! commanded. The runner watches its own run folder: moved to `_paused` it
//! suspends the process group, moved back it resumes, moved anywhere else it
//! terminates. That works from Finder, from a script, or from a menu bar app
//! on another machine — no socket, no daemon protocol, and the filesystem's
//! own permissions decide who may do it.
//!
//! This module only ever *writes* the folder. What the UIs show is read back
//! off disk by `job_core::Observer`, so the display cannot drift from the
//! truth and is the same view a monitor gets from another machine.

use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use job_core::clock;
use job_core::observe::{Root, STATUS_FILE, State, hostname, job_name, target_file};

use crate::watch::{FolderWatch, rename_exclusive};

const POLL: Duration = Duration::from_secs(1);

/// How many jobs may run at once. Encodes are the common case and they are
/// each happy to saturate several cores, so this is deliberately small — the
/// point is to keep a short job from queueing behind a two-hour one, not to
/// run the machine flat out.
const DEFAULT_CONCURRENCY: usize = 2;
const MAX_CONCURRENCY: usize = 8;
const SETTLE: Duration = Duration::from_secs(2);
/// How long a stopped job gets to exit on its own before it is killed.
const TERM_GRACE: Duration = Duration::from_secs(10);

pub fn root() -> Root {
    Root::local()
}

/// How much to yield to everything else. 0 is normal priority; higher numbers
/// mean "run this only when nothing else wants the CPU".
///
/// Deliberately opt-in. launchd's `ProcessType = Background` used to impose
/// nice 19 and throttled I/O on the whole queue, which turned encodes into a
/// tenth of their speed for no stated reason — priority is a decision about a
/// job, so it is made here where it can be seen and changed.
pub fn niceness() -> i32 {
    std::env::var("JOB_NICE")
        .ok()
        .and_then(|raw| raw.trim().parse::<i32>().ok())
        .unwrap_or(0)
        .clamp(0, 20)
}

pub fn concurrency() -> usize {
    std::env::var("JOB_CONCURRENCY")
        .ok()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .unwrap_or(DEFAULT_CONCURRENCY)
        .clamp(1, MAX_CONCURRENCY)
}

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

/// One trail for the whole toolchain: the daemon and the manager are the same
/// loop, so their lines belong in the same file.
fn log_file() -> PathBuf {
    std::env::var_os("JOB_LOG")
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join("Library/Logs/jobs.log"))
}

fn log(line: &str) {
    let path = log_file();
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    // One write of one complete line: `writeln!` formats in fragments, and
    // with several jobs running at once their fragments interleave in the
    // file. O_APPEND makes a single write atomic; two of them are not.
    let entry = format!("{} {line}\n", clock::timestamp());
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = file.write_all(entry.as_bytes());
    }
}

pub fn prepare(root: &Root) {
    let _ = fs::create_dir_all(root.path());
    for dir in root.state_dirs() {
        let _ = fs::create_dir_all(dir);
    }
    reap_orphans(root);
}

/// File away run folders this machine claimed and then lost.
///
/// A job only ever becomes an orphan when the *runner* goes: while it is alive
/// it supervises its own children and files them away itself. So the moment to
/// look is startup — which is where this is called from, rather than on a
/// timer that would spend the rest of the day reading a directory to be told
/// nothing has changed.
///
/// The claim is deliberately narrow: only folders whose `.status` names this
/// host, and only when the process group it names is gone for good. Another
/// machine's jobs in a shared folder are not ours to judge, and a folder
/// promoted a moment ago by a concurrent runner has no `.status` yet — both
/// read as "leave it alone".
fn reap_orphans(root: &Root) {
    let Ok(entries) = fs::read_dir(root.running()) else {
        return;
    };
    for dir in entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
    {
        if !job_core::observe::orphaned_here(&dir) {
            continue;
        }
        let name = job_core::observe::run_folder_name(&dir);
        log(&format!("REAP {name} (in _running with nothing running it)"));
        let _ = fs::remove_file(dir.join(STATUS_FILE));
        file_away(root, &dir, &name, false);
    }
}

/// Top-level `*.job` files, alphabetically.
fn scan_inbox(root: &Root) -> Vec<PathBuf> {
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

/// Staged job folders, alphabetically — which is date order, so the queue runs
/// oldest first. Renaming a folder reorders the queue.
fn scan_ready(root: &Root) -> Vec<PathBuf> {
    let Ok(entries) = fs::read_dir(root.ready()) else {
        return Vec::new();
    };
    let mut ready: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    ready.sort();
    ready
}

/// True once the file's size has held steady across SETTLE (i.e. a copy into
/// the folder has finished).
fn is_stable(path: &Path) -> bool {
    let size = |p: &Path| fs::metadata(p).map(|m| m.len()).ok();
    let Some(first) = size(path) else { return false };
    thread::sleep(SETTLE);
    size(path) == Some(first)
}

/// A non-colliding directory path: appends `-2`, `-3`, ... if taken.
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

/// Copy a child stream to `path`, creating the file only when the first bytes
/// arrive — a stream that stays silent never gets a file at all.
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

/// Stage a dropped job as a folder: `mkdir` the destination (atomic, fails if
/// it exists) and move the job file and its target in. From here on the job is
/// a unit — one folder to move, edit, or hand back to the queue.
fn stage(root: &Root, job: &Path) -> bool {
    let name = job_name(job);
    let target = target_file(job);

    let dir = uniq_dir(root.ready().join(format!("{}-{name}", clock::file_stamp())));
    if fs::create_dir(&dir).is_err() {
        log(&format!("SKIP {name} (could not stage)"));
        return false;
    }
    if fs::rename(job, dir.join(format!("{target}.job"))).is_err() {
        let _ = fs::remove_dir(&dir);
        log(&format!("SKIP {name} (could not claim job file)"));
        return false;
    }
    let beside = root.path().join(&target);
    if beside.exists() && fs::rename(&beside, dir.join(&target)).is_err() {
        log(&format!("WARN could not stage target file {target}"));
    }
    true
}

/// Take the oldest staged job by promoting it into `_running`. `RENAME_EXCL`
/// makes that the claim: two runners can race and only one moves the folder.
fn promote(root: &Root) -> Option<PathBuf> {
    for ready in scan_ready(root) {
        let folder = ready.file_name()?.to_os_string();
        let running = root.running().join(&folder);
        if rename_exclusive(&ready, &running) {
            return Some(running);
        }
    }
    None
}

/// What a job's folder moving means.
enum Move {
    Pause,
    Resume,
    /// Moved somewhere final — the destination is where the user wants it, so
    /// the runner stops the job and leaves the folder alone.
    Stop,
}

fn command_for(dir: &Path) -> Option<Move> {
    match State::of(dir) {
        Some(State::Paused) => Some(Move::Pause),
        Some(State::Running) => Some(Move::Resume),
        // Ready, ok, failed, or dragged clean out of the tree.
        _ => Some(Move::Stop),
    }
}

fn signal(pgid: i32, signal: i32) {
    unsafe {
        libc::killpg(pgid, signal);
    }
}

fn run_one(root: &Root, run_dir: PathBuf) {
    let name = job_core::observe::run_folder_name(&run_dir);
    let job_file = match find_job_file(&run_dir) {
        Some(path) => path,
        None => {
            log(&format!("FAIL {name} (no .job file in its folder)"));
            file_away(root, &run_dir, &name, false);
            return;
        }
    };
    let target = target_file(&job_file);

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
    // progress-bar redraws and ANSI colour.
    let mut command = if executable {
        Command::new(&job_file)
    } else {
        let mut command = Command::new("/bin/bash");
        command.arg(&job_file);
        command
    };
    command
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
        // Its own process group, so pausing or stopping the job reaches the
        // encoder underneath and not just the shell wrapping it.
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
    let spawned = command.spawn();

    let mut child = match spawned {
        Ok(child) => child,
        Err(err) => {
            log(&format!("FAIL {name} (could not start: {err})"));
            file_away(root, &run_dir, &name, false);
            return;
        }
    };

    // The process group id is the job's own pid, since it leads the group.
    // Published so a local client can renice or signal it without being the
    // parent; the host is recorded so a monitor on another machine knows the
    // pid means nothing to it.
    let pgid = child.id() as i32;
    let _ = fs::write(
        run_dir.join(STATUS_FILE),
        format!("pgid={pgid}\nhost={}\n", hostname()),
    );

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

    let outcome = supervise(&mut child, &run_dir, pgid, &name);

    let _ = log_writer.join();
    let _ = err_writer.join();

    // Wherever the folder is now is where it belongs: if it was moved while
    // running, the user has already said where they want it.
    let watched = outcome.final_dir.unwrap_or(run_dir);
    let _ = fs::remove_file(watched.join(STATUS_FILE));
    if State::of(&watched) == Some(State::Running) {
        file_away(root, &watched, &name, outcome.code == 0);
    } else {
        log(&format!("MOVE {name} left where it was put"));
    }
}

struct Outcome {
    code: i32,
    /// Where the folder ended up, if it was moved while the job ran.
    final_dir: Option<PathBuf>,
}

/// Wait for the job, acting on its folder moving in the meantime.
///
/// This is the whole command channel: no socket, no polling of a control file,
/// just a descriptor on the folder and the kernel telling us it moved.
fn supervise(child: &mut Child, run_dir: &Path, pgid: i32, name: &str) -> Outcome {
    let watch = FolderWatch::open(run_dir);
    let mut current = run_dir.to_path_buf();
    let mut paused = false;
    let mut stopping: Option<Instant> = None;

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return Outcome {
                    code: status.code().unwrap_or(1),
                    final_dir: Some(current),
                };
            }
            Err(_) => {
                return Outcome {
                    code: 1,
                    final_dir: Some(current),
                };
            }
            Ok(None) => {}
        }

        // A stopped job that ignored SIGTERM gets SIGKILL, which it cannot.
        if let Some(since) = stopping
            && since.elapsed() > TERM_GRACE
        {
            log(&format!("KILL {name} (did not exit on its own)"));
            signal(pgid, libc::SIGKILL);
            stopping = None;
        }

        let Some(watch) = watch.as_ref() else {
            // Without a watch there is nothing to react to; just wait it out.
            thread::sleep(POLL);
            continue;
        };
        let Some(moved) = watch.moved(POLL) else {
            continue;
        };
        if moved == current {
            continue;
        }
        current = moved;

        match command_for(&current) {
            Some(Move::Pause) if !paused => {
                log(&format!("STOP {name} (moved to _paused)"));
                signal(pgid, libc::SIGSTOP);
                paused = true;
            }
            Some(Move::Resume) if paused => {
                log(&format!("CONT {name} (moved back to _running)"));
                signal(pgid, libc::SIGCONT);
                paused = false;
            }
            Some(Move::Stop) => {
                log(&format!("TERM {name} (moved out of _running)"));
                // A suspended process can't act on SIGTERM, so wake it first.
                if paused {
                    signal(pgid, libc::SIGCONT);
                    paused = false;
                }
                signal(pgid, libc::SIGTERM);
                stopping = Some(Instant::now());
            }
            _ => {}
        }
    }
}

fn find_job_file(dir: &Path) -> Option<PathBuf> {
    fs::read_dir(dir).ok()?.flatten().find_map(|entry| {
        let path = entry.path();
        path.file_name()
            .is_some_and(|name| name.to_string_lossy().ends_with(".job"))
            .then_some(path)
    })
}

/// Move a finished run folder to `_ok` or `_failed`.
fn file_away(root: &Root, run_dir: &Path, name: &str, ok: bool) {
    let dest_dir = if ok {
        log(&format!("DONE {name} (exit 0)"));
        root.ok()
    } else {
        log(&format!("FAIL {name}"));
        root.failed()
    };

    let _ = fs::create_dir_all(&dest_dir);
    let folder = run_dir.file_name().unwrap_or_default().to_os_string();
    if fs::rename(run_dir, uniq_dir(dest_dir.join(&folder))).is_err() {
        log(&format!("WARN could not file {name} away"));
    }
}

/// Stage everything dropped in the inbox, then run the queue down with up to
/// `concurrency` jobs in flight.
fn drain(root: &Root) -> usize {
    for job in scan_inbox(root) {
        if !is_stable(&job) {
            log(&format!("WAIT {} (still uploading)", job_name(&job)));
            continue;
        }
        // The target file may still be copying in even once the job file has
        // settled.
        let target = root.path().join(target_file(&job));
        if target.is_file() && !is_stable(&target) {
            log(&format!("WAIT {} (target still uploading)", job_name(&job)));
            continue;
        }
        stage(root, &job);
    }

    let gate = Mutex::new(());
    let ran = AtomicUsize::new(0);
    thread::scope(|scope| {
        let workers: Vec<_> = (0..concurrency())
            .map(|_| {
                scope.spawn(|| {
                    loop {
                        let claimed = {
                            let _held = gate.lock();
                            promote(root)
                        };
                        let Some(run_dir) = claimed else { break };
                        run_one(root, run_dir);
                        ran.fetch_add(1, Ordering::Relaxed);
                    }
                })
            })
            .collect();
        for worker in workers {
            let _ = worker.join();
        }
    });

    ran.into_inner()
}

/// Drain the queue and return: the launchd WatchPaths path, where the process
/// only exists while there is work.
pub fn drain_once() -> usize {
    let root = root();
    prepare(&root);
    drain(&root)
}

/// Whether the resident loop is claiming new jobs. In-process and deliberately
/// not on disk: a queue that came back paused after a reboot would be a queue
/// that silently does nothing.
static PAUSED: AtomicBool = AtomicBool::new(false);

pub fn is_paused() -> bool {
    PAUSED.load(Ordering::Relaxed)
}

pub fn set_paused(paused: bool) {
    PAUSED.store(paused, Ordering::Relaxed);
}

pub fn spawn() {
    thread::spawn(move || {
        let root = root();
        prepare(&root);
        loop {
            if is_paused() {
                thread::sleep(POLL);
                continue;
            }
            if scan_inbox(&root).is_empty() && scan_ready(&root).is_empty() {
                thread::sleep(POLL);
                continue;
            }
            drain(&root);
            thread::sleep(POLL);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A queue that has lost its runner — killed, crashed, or rebooted mid-job
    /// — leaves the run folder sitting in `_running` with a dead pid in its
    /// status, where nothing ever cleaned it up and every UI went on reporting
    /// it as a job that isn't running.
    #[test]
    fn startup_files_away_a_job_this_machine_lost() {
        let base = std::env::temp_dir().join(format!("job-daemon-reap-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let root = Root::new(&base);
        for dir in root.state_dirs() {
            fs::create_dir_all(dir).unwrap();
        }
        // The trail is the one thing here that isn't scoped to the temp root:
        // without this the test files its fake REAP lines into the real
        // ~/Library/Logs/jobs.log, alongside a machine's actual history.
        //
        // SAFETY: single-threaded at this point, and no other test in this
        // binary reads the environment.
        unsafe { std::env::set_var("JOB_LOG", base.join("jobs.log")) };

        // A process group that certainly existed and certainly doesn't now:
        // spawned as its own leader, then waited on so the pid is released.
        let mut child = Command::new("/bin/sh")
            .arg("-c")
            .arg("exit 0")
            .process_group(0)
            .spawn()
            .unwrap();
        let dead = child.id() as i32;
        child.wait().unwrap();

        let orphan = root.running().join("20260101-000000-orphan");
        fs::create_dir_all(&orphan).unwrap();
        fs::write(
            orphan.join(STATUS_FILE),
            format!("pgid={dead}\nhost={}\n", hostname()),
        )
        .unwrap();

        // Another machine's job on a shared folder: not ours to judge, whatever
        // that pid means here.
        let theirs = root.running().join("20260101-000000-theirs");
        fs::create_dir_all(&theirs).unwrap();
        fs::write(theirs.join(STATUS_FILE), format!("pgid={dead}\nhost=elsewhere\n")).unwrap();

        // Just promoted by a concurrent runner, no status written yet.
        let fresh = root.running().join("20260101-000000-fresh");
        fs::create_dir_all(&fresh).unwrap();

        reap_orphans(&root);

        assert!(!orphan.exists(), "the orphan should have been filed away");
        assert!(
            root.failed().join("20260101-000000-orphan").is_dir(),
            "and filed to _failed, where a stopped job belongs"
        );
        assert!(theirs.is_dir(), "another host's job must be left alone");
        assert!(fresh.is_dir(), "a job with no status yet must be left alone");

        let _ = fs::remove_dir_all(&base);
    }

    #[test]
    fn staging_makes_a_job_a_folder() {
        let base = std::env::temp_dir().join(format!("job-daemon-stage-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let root = Root::new(&base);
        prepare(&root);

        fs::write(base.join("clip.mov.job"), "#!/bin/bash\n").unwrap();
        fs::write(base.join("clip.mov"), "payload").unwrap();
        assert!(stage(&root, &base.join("clip.mov.job")));

        // Job and target travel together, and the inbox is left clean.
        let staged: Vec<_> = fs::read_dir(root.ready()).unwrap().flatten().collect();
        assert_eq!(staged.len(), 1);
        let dir = staged[0].path();
        assert!(dir.join("clip.mov.job").is_file());
        assert!(dir.join("clip.mov").is_file());
        assert!(!base.join("clip.mov.job").exists());

        // Promotion is exclusive: the second attempt on the same folder fails
        // rather than clobbering, which is what makes it a claim.
        let promoted = promote(&root).unwrap();
        assert_eq!(State::of(&promoted), Some(State::Running));
        assert!(promote(&root).is_none());

        let duplicate = root.running().join(promoted.file_name().unwrap());
        fs::create_dir_all(root.ready().join("20260101-000000-clip")).unwrap();
        assert!(!rename_exclusive(
            &root.ready().join("20260101-000000-clip"),
            &duplicate
        ));

        let _ = fs::remove_dir_all(&base);
    }
}
