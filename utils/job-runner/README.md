# job-runner

A drop-folder job queue for macOS. A launchd WatchPaths LaunchAgent watches
`~/jobs`; drop a `NAME.job` shell script in and it runs — one at a time, first
`*.job` alphabetically — with its artifacts filed away when it finishes.

Install: `setup/install/install-job-runner.sh` (copies this script to
`~/.local/bin/job-runner`, creates `~/jobs` + `_done/` + `_err/`, and loads the
`com.jayu.job-runner` agent). `setup/macos/server.sh` runs it automatically.

## The contract (for anything that wants to queue work)

Write an executable shell script ending in `.job` into `~/jobs` (locally, or
over the mounted `jobs` share — `send-job` in `config/zsh/bin/` does exactly
this, copying into `/Volumes/jobs` (or `$JOBS_DIR`) and shipping data files
first so the watcher never fires on a half-copied job).

Lifecycle of `NAME.job`:

| stage | file |
|---|---|
| queued | `NAME.job` |
| running | `NAME.job.running` (rename = the lock; can't be picked up twice) |
| stdout | `NAME.job.log` (only if the job writes to stdout) |
| stderr | `NAME.job.errors` (only if the job writes to stderr) |
| succeeded | `_done/NAME.job.done` + its `.log`, if any |
| failed | `_err/NAME.job.err` + its `.log`, if any |

A silent job leaves no `.log`/`.errors` behind — each file is created on that
stream's first line of output, and both are written live so a running job can
be tailed. An `.errors` file always lands in `_err/`, even for a job that
exited 0. Name collisions in `_done`/`_err` get a timestamp inserted
(`NAME.20260729-120000.job.done`) rather than clobbering.

Jobs run cd'd into `~/jobs` with:

- `JOB_NAME` — the job filename minus `.job`. Name the job after its target
  (`video.mp4.job`) and the script can just use `"$JOB_NAME"`.
- `JOB_FILE`, `JOB_DIR` — the running job's path and the jobs dir.
- `TERM=dumb`, `NO_COLOR=1`, `CLICOLOR=0` — so tools log plain lines instead
  of progress-bar redraws. Pass quiet flags for stubborn tools
  (`ffmpeg -nostats`, `yt-dlp --no-progress`).

These three variables, plus the fact that stdout is a pipe rather than a tty,
are the whole "you are headless" signal — the runner deliberately passes no
per-tool flags, so a tool that wants to behave differently in a job detects
them itself. `topaz-encode` is the worked example: under a job it drops ANSI
colour and its in-place progress bar for a timestamped transcript with a
progress line every 5%, skips every prompt (a partial output resumes
automatically), quotes the encoder log into the transcript when an encode
fails, and reports failure through the exit status alone. Nothing in the
`.job` file needs to say so; `--log` forces the same behaviour in a terminal.

A one-line status trail for every job is appended to
`~/Library/Logs/job-runner.log`. Yazi styles the job file states
(`config/yazi/theme.toml`).
