# job-folder

The jobs queue and its menu bar in one process. Drop a `NAME.job` script into
`~/jobs` and it runs — same contract as [`job-daemon`](../job-daemon), same rows
as [`job-monitor`](../job-monitor) — but the queue itself lives in memory, in
the app you are looking at.

```bash
setup/install/install-job-folder.sh
```

Builds the crate, assembles `~/Applications/Job Folder.app`, and ad-hoc signs
it. No LaunchAgent: **the queue runs while the app is open and stops when you
quit it.** Add it to Login Items if you want it always on.

## Why

The daemon-and-monitor pair keeps a job's state in the folder it sits in, so
that a runner here and a monitor on another machine can agree without ever
talking to each other. It is a good design and it buys a real thing — you can
watch and command an encode queue across the LAN over nothing but SMB.

It also costs exactly what you would expect, once you are sitting at the machine
doing the work:

- Pause is a `rename` that the runner has to notice.
- The row you pressed doesn't change until a poll comes round — 2s locally, up
  to a minute through the SMB directory cache.
- Queue order is the alphabet, so reordering means renaming folders.
- "Is it still running?" is answered by *inference* — silence, a heartbeat, a
  pid in a `.status` file — because nothing watching the folder is the job's
  parent.

This app makes the other trade. One process runs the jobs and draws the menu, so
the queue is a `Vec<Job>` behind a mutex:

| | |
|---|---|
| pause | `SIGSTOP` to the process group, on the way back from the click |
| resume | `SIGCONT`, same |
| stop | `SIGTERM`, then `SIGKILL` after ten seconds |
| hold | a queued job is skipped until you say otherwise |
| ↑ | move a queued job to the front — a `Vec` splice |
| retry | a finished job goes back to the end of the queue |

Every one of those is true the instant it is pressed, and the menu **stays open
and redraws itself** — while the shape of the list holds, each row is handed a
fresh spec rather than the menu being rebuilt under your pointer.

There is no "not running" state and no "no output 41m" warning, because neither
is a question here: we are holding the child. A job that is in the list is
running; one that isn't has already become an outcome.

## What it gives up

**The network.** Nothing on disk says what any job is doing, so no second
machine can watch this queue, and there is no dragging a folder to `_paused`
from Finder. If you want that, run `job-daemon` and watch it with
`job-monitor` — that is what they are for. Run one or the other on a given
folder, never both.

**Everything but the payload, on quit.** Quitting sends `SIGTERM` to every
running job (an encode nobody is watching, with no row left to stop it from, is
worse than one that stopped). Whatever hadn't finished is still in `ready/`, and
starts again from the top next launch.

## The folder

Two directories, and neither is a state machine:

| | |
|---|---|
| `~/jobs/TARGET.job` (+ `TARGET`) | dropped, not yet picked up |
| `~/jobs/ready/<date>-<name>/` | the job's payload while it is the queue's: the script, its target file, its logs |
| `~/jobs/done/<date>-<name>/` | the same folder once it has finished, however it finished |

No `.status`, no `_paused`, no `_ok` and `_failed` — whether a folder in
`ready/` is queued, running, held or suspended is not written down anywhere,
because the only thing that needs to know is holding it in memory. `done/` is
where payloads go, not a verdict: which of them failed is in the menu, and in
the exit status the job already reported.

`$JOBS_DIR` moves the root, `$JOB_CONCURRENCY` (1–8, default 2) and `$JOB_NICE`
(0–20) work as they do in the daemon, and the concurrency is also the
**Workers** submenu — how many jobs may run at once, changed live.

The drop folder is still watched, and deliberately so: `send-job`, `topaz-job`
and the mpv Topaz workflow queue work by writing a file, from this machine or
over a mounted share, and none of them should have to know a process exists. A
dropped file is staged once its size has held steady for two seconds.

## Jobs run exactly as the daemon runs them

Same environment, to the letter, so a `.job` script written for one works under
the other: `TARGET_FILE`, `JOB_NAME`, `JOB_FILE`, `JOB_DIR`, `JOB_RUN_DIR`, and
`TERM=dumb` / `NO_COLOR=1` / `CLICOLOR=0`. stdout goes to `$JOB_NAME.log` in the
run folder *and* into the row's last-line display; stderr goes to
`$JOB_NAME.error.log` and stays out of the row — plenty of tools log there, and
a warning is not what the job is doing. Exit status alone decides pass or fail.

Each job leads its own process group, which is what makes pause and stop reach
the encoder underneath rather than the shell wrapping it.

## Preferences

`~/.config/job-folder/` holds the icon style, the notification mute and the
concurrency — how you like the app, kept out of the folder that holds the work.
