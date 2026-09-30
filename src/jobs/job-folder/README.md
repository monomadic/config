# job-folder

A job queue where every workflow is a folder. Put a `job.sh` in a directory
under `~/jobs`, drop files into its `input/`, and each one runs through the
script. The queue and its menu bar live in one process.

```bash
scripts/install/install-job-folder.sh
```

Builds the crate, assembles `~/Applications/Job Folder.app`, and ad-hoc signs
it. No LaunchAgent: **the queue runs while the app is open and stops when you
quit it.** Add it to Login Items if you want it always on.

## A job folder

```
~/jobs/interpolate-60fps/
  job.sh        the workflow — run once per input
  input/        drop files here; each is a run, oldest first
  output/       $OUTPUT_DIR, the script's to fill
  done/         inputs whose run succeeded
  failed/       inputs whose run failed or was stopped
  stdout.log    every run's stdout, appended under a header
  stderr.log    every run's stderr, same
```

Any non-hidden directory under the root with a `job.sh` counts, picked up live.
`input/` and `output/` are created for you; `done/` and `failed/` appear when
first needed.

`job.sh` runs with its folder as the working directory and:

| | |
|---|---|
| `$INPUT` | the input's absolute path |
| `$INPUT_DIR` | the directory it is in (the folder's `input/`) |
| `$INPUT_FILE` | its file name |
| `$INPUT_NAME` | its file name without the extension |
| `$OUTPUT_DIR` | the folder's `output/` |
| `$JOB_DIR` | the job folder itself |

plus `TERM=dumb`, `NO_COLOR=1`, `CLICOLOR=0`. If `job.sh` is executable it runs
as-is (its shebang counts); if not, through `/bin/bash`. Exit status alone
decides pass or fail. The last line of stdout shows in the menu row, and a
percentage in it drives the progress bar.

```bash
#!/bin/bash
set -euo pipefail
ffmpeg -nostdin -i "$INPUT" -vf minterpolate=fps=60 "$OUTPUT_DIR/$INPUT_NAME.60fps.mkv"
```

## The queue

- **One run per folder at a time**, oldest input first — a folder's logs never
  interleave. The **Workers** submenu (and `$JOB_CONCURRENCY`, 1–8, default 2)
  caps how many folders run at once.
- **An input stays in `input/` while it runs**, then moves to `done/` or
  `failed/`. Anything left in `input/` when the app starts is queued again.
  Retry is dragging a file back into `input/` (or the row's retry button).
- **Dragging a queued file out of `input/` dequeues it.**
- Whether a file is queued, held, running or paused is held in memory only.
  Row buttons act immediately: pause/resume are `SIGSTOP`/`SIGCONT` to the job's
  process group, stop is `SIGTERM` then `SIGKILL` after ten seconds, ↑ moves a
  queued file to the front.
- Quitting sends `SIGTERM` to every running job — an encode nothing is watching
  is worse than one that stopped. Its input is still in `input/`.

`$JOBS_DIR` moves the root, `$JOB_NICE` (0–20) lowers the jobs' priority.

## Getting files in whole

The intended way in is to copy to `~/jobs` first, then `mv` into the job
folder's `input/`. That last step is a rename on one volume, so the file
appears complete in a single step.

A file dropped straight into `input/` is still covered by some light checks.
It is only queued once all of these hold:

1. **It is visible and not a download in progress.** Dotfiles, `._*`, and
   names ending `.part`, `.partial`, `.crdownload`, `.download`, `.tmp`,
   `.temp` are skipped. `rsync` writes to a hidden temporary and renames it
   when complete, so an `rsync` into `input/` is safe too.
2. **Its size and mtime have held still** for `$JOB_SETTLE` seconds (default
   2). Raise it if you copy straight in from a slow share.
3. **No process of this user has it open for writing** (`lsof`). This catches a
   stalled Finder or `cp` copy. Writers belonging to other users, such as the
   SMB server, are invisible without root; (2) covers those.

Inputs are files; directories in `input/` are ignored. A symlink to a file
counts — `send-job --link` queues that way — and filing it into `done/` or
`failed/` moves the link, never the file it points at.

## Preferences

`~/.config/job-folder/` holds the icon style, the notification mute and the
concurrency — how you like the app, kept out of the folder that holds the work.
