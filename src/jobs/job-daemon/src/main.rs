//! job-daemon — the job queue with no face on it.
//!
//! Two ways to run, and the choice is the installer's:
//!
//!   `job-daemon`          resident. Watches the folder itself and runs jobs
//!                         as they arrive. Installed with KeepAlive.
//!   `job-daemon --once`   drain the queue and exit. Installed as a launchd
//!                         WatchPaths handler, so launchd does the watching
//!                         and this process only exists while there is work.
//!
//! `--once` is the event-driven one: nothing polls, launchd wakes it when the
//! folder changes. The resident mode polls, which is the price of not having a
//! launchd trigger behind it.
//!
//! This is the only thing that runs jobs. `job-monitor` is the UI, and links
//! no job loop at all — it commands the queue by moving folders, which is all
//! any command in this system is.

use std::process::ExitCode;

use job_daemon::runner;

const USAGE: &str = "\
job-daemon — run the jobs dropped into $JOBS_DIR (default ~/jobs)

usage:
  job-daemon           stay resident and run jobs as they arrive
  job-daemon --once    drain the queue and exit (launchd WatchPaths handler)
  job-daemon --help

environment:
  JOBS_DIR                 the jobs folder
  JOB_CONCURRENCY          how many jobs may run at once (default 2, max 8)
  JOB_LOG                  the status trail (default ~/Library/Logs/jobs.log)
";

fn main() -> ExitCode {
    match std::env::args().nth(1).as_deref() {
        None => {
            runner::spawn();
            // The loop lives on its own thread so it is the same code path
            // the manager drives; here there is nothing else to do but wait.
            loop {
                std::thread::park();
            }
        }
        Some("--once") => {
            let drained = runner::drain_once();
            // Nothing to do is a perfectly good outcome for a trigger that
            // fired on a file that turned out not to be a job.
            if drained > 0 {
                eprintln!("job-daemon: ran {drained} job(s)");
            }
            ExitCode::SUCCESS
        }
        Some("--help" | "-h") => {
            print!("{USAGE}");
            ExitCode::SUCCESS
        }
        Some(other) => {
            eprintln!("job-daemon: unknown argument {other}\n");
            print!("{USAGE}");
            ExitCode::FAILURE
        }
    }
}
