# job-server-cli

The job queue in bash: same folder, same contract, no menu bar, no resident
process. A launchd WatchPaths LaunchAgent fires it when something lands in
`~/jobs`; it drains the folder and exits.

The full contract — run folders, `$TARGET_FILE` / `$JOB_NAME`, logs, `_done`
vs `_err` — is documented once, in [`job-server`](../job-server). Everything
there applies here unchanged; a `.job` script cannot tell which one ran it.

Install: `setup/install/install-job-server-cli.sh` (copies the script to
`~/.local/bin/job-server-cli`, creates `~/jobs` + `_running/` + `_done/` +
`_err/`, loads the `com.jayu.job-server-cli` agent). `setup/macos/server.sh`
runs it automatically.

## Which one to install

`job-server` is the default: it shows what is happening, and its heartbeat
lets a monitor tell a live runner from an abandoned one. Reach for
`job-server-cli` when there is no reason for a process to sit resident — a
headless box, a machine with no user logged in to draw a menu bar — or when
you want the queue driven purely by launchd.

Install one, not both. They share `$JOBS_DIR/.lock`, so running both is
harmless rather than dangerous, but it is redundant.

One caveat if you pair this with `job-monitor`: the shell runner takes the
lock and holds it without touching it again, so it has no heartbeat. A monitor
therefore can't tell a long job here from an abandoned one until the lock
passes the one-hour stale threshold.

A one-line status trail for every job is appended to
`~/Library/Logs/job-server-cli.log`.
