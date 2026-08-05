# job-server

A drop-folder job queue for macOS, with a menu bar face. Drop a `NAME.job`
shell script into `~/jobs` and it runs — one at a time, first `*.job`
alphabetically — with everything it produced filed away when it finishes.

This is the default runner. Three other pieces share the same folder:

| | |
|---|---|
| [`job-server-cli`](../job-server-cli) | the same contract in bash, triggered by launchd instead of a resident process |
| [`job-monitor`](../job-monitor) | watches this folder from another machine over SMB; runs nothing |
| [`job-core`](../job-core) | the shared library — naming rules, the folder observer, the icon |

Only one *runner* may be active per folder — `job-server` and `job-server-cli`
share `$JOBS_DIR/.lock`, so nothing runs twice even if both are installed, but
there is no reason to have both. Any number of monitors can watch.

Install: `setup/install/install-job-server.sh` (builds to `~/.local/bin/job-server`,
creates `~/jobs` + `_running/` + `_done/` + `_err/`, loads the resident
`com.jayu.job-server` agent, and retires `job-server-cli` and the pre-rename
`com.jayu.job-folder` / `com.jayu.job-runner` agents if they are installed).

## The contract (for anything that wants to queue work)

Write an executable shell script ending in `.job` into `~/jobs` (locally, or
over the mounted `jobs` share — `send-job` in `config/zsh/bin/` does exactly
this, copying into `/Volumes/jobs` (or `$JOBS_DIR`) and shipping data files
first so the watcher never fires on a half-copied job).

Lifecycle of `video.mp4.job` (dropped in beside `video.mp4`):

| stage | location |
|---|---|
| queued | `~/jobs/video.mp4.job` (+ `~/jobs/video.mp4`) |
| running | `_running/20260729-120000-video/` — the job file and its target file are moved in; the move out of the top level *is* the lock, so a job can't be picked up twice |
| stdout | `<run folder>/video.log` (only if the job writes to stdout) |
| stderr | `<run folder>/video.error.log` (only if the job writes to stderr) |
| succeeded | the whole run folder moves to `_done/20260729-120000-video/` |
| failed | the whole run folder moves to `_err/20260729-120000-video/` |

Everything a run produced stays together in one folder: the job script, the
file it operated on, and its logs. A silent job leaves no `.log`/`.error.log`
behind — each file is created on that stream's first line of output, and both
are written live so a running job can be tailed. `_done` vs `_err` is decided
by the job's exit status alone (stderr output on its own is not a failure —
plenty of tools log there). Run-folder name collisions get `-2`, `-3` appended
rather than clobbering.

Jobs run cd'd into their run folder with:

- `TARGET_FILE` — the job filename minus `.job` (`video.mp4.job` =>
  `video.mp4`). It sits in the CWD, so a bare filename is enough.
- `JOB_NAME` — `TARGET_FILE` minus its extension (`video`). Names the logs.
- `JOB_FILE`, `JOB_RUN_DIR` — the running job's path and its run folder.
- `JOB_DIR` — the jobs root. Any *other* data file shipped alongside stays
  there, so reference it as `"$JOB_DIR/name"`.
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

## The folder is the state

The menu bar display reads nothing from the running process. Every poll
re-derives what it shows from the folder itself, through `job_core`'s observer:
top-level `*.job` is the queue, `_running/*` is what is running, `_done`/`_err`
are the outcomes, `.paused` holds the queue, and `.lock`'s mtime is the
runner's heartbeat.

That is why restarting mid-job shows the job still running rather than an
empty queue — and it is the whole reason a monitor on another machine can show
the same thing without talking to this process at all.

## Menu

Running job and elapsed time, the queue, and the last few outcomes (click one
to reveal its run folder in Finder). Below that: open the jobs folder, open
`_err`, tail the running job's log, clear the error badge, and pause the queue.
Pausing writes `.paused`; the runner finishes the job in flight and then stops
claiming new ones.

A one-line status trail for every job is appended to
`~/Library/Logs/job-server.log`. Yazi styles the job artifacts
(`config/yazi/theme.toml`).
