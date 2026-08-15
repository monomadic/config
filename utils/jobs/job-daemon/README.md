# job-daemon

The jobs queue with no UI. Drop a `NAME.job` shell script into `~/jobs` and it
runs — two at a time, first `*.job` alphabetically — with everything it
produced filed away when it finishes.

```bash
job-daemon           # stay resident and run jobs as they arrive
job-daemon --once    # drain the queue and exit
```

`--once` is how it is normally installed: a launchd WatchPaths agent fires it
when something lands in the folder, it drains, and it exits. launchd does the
watching, so nothing polls and no process sits around between jobs. The
resident mode watches the folder itself, for a machine where you would rather
have one long-lived process than a spawn per job.

```bash
setup/install/install-job-daemon.sh
```

This is the only thing that runs jobs *under the folder protocol* —
[`job-folder`](../job-folder) is a separate app that runs its own queue in
memory, and the two must not share a folder. [`job-monitor`](../job-monitor) is
the UI and deliberately does not depend on this crate: a binary with no job loop
linked into it cannot claim a job, which is what makes watching a shared folder
from another machine safe. Install both — they don't conflict.

The LaunchAgent watches two paths, not one: the top level fires when a `.job`
is dropped, and `_ready` fires when a folder is moved back into the queue —
releasing a held job, or requeueing a failed one — which the top level would
never see. `$JOBS_DIR`, `$JOB_LOG` and `$JOB_CONCURRENCY` are written into the
agent, since launchd inherits almost nothing.

Jobs run at normal priority. `JOB_NICE` (0–20) makes the queue yield to
whatever you are doing in the foreground; it can only ever *raise* niceness,
since lowering it needs root. This is deliberately the runner's decision and
not launchd's: the agent used to carry `ProcessType = Background`, which
imposes nice 19 *and* throttled disk I/O on everything the queue spawns, and
turned a Topaz encode into 0.05x with an eight-hour ETA. If a queue exists to
encode video, throttling it is backwards.

Two jobs run at once by default (`JOB_CONCURRENCY`, 1–8). There is no
folder-wide lock: exclusive promotion is per-job, so two runners on one folder
would be safe rather than merely discouraged — they would simply share the
queue.

## The contract (for anything that wants to queue work)

Write an executable shell script ending in `.job` into `~/jobs` (locally, or
over the mounted `jobs` share — `send-job` in `config/zsh/bin/` does exactly
this, copying into `/Volumes/jobs` (or `$JOBS_DIR`) and shipping data files
first so the watcher never fires on a half-copied job).

For Topaz encodes there is a layer above that: `topaz-job` turns a video plus a
preset file into the `.job` script and then sends it through `send-job`, trying
the mounted share, then iCloud, then a local jobs folder. The mpv Topaz
workflow (`config/mpv/scripts/topaz-workflow-current.lua`) uses it for ⌘S /
⌘⇧S / ⌘R and knows nothing about job files itself.

**A job is a folder, and the folder it sits in is its state.** No lock file, no
pause flag, no status protocol — just where it is:

| location | means |
|---|---|
| `~/jobs/video.mp4.job` (+ `video.mp4`) | dropped, not yet picked up |
| `_ready/20260729-120000-video/` | staged as a folder, waiting for a slot |
| `_running/20260729-120000-video/` | running |
| `_paused/20260729-120000-video/` | suspended — its process group is stopped |
| `_ok/20260729-120000-video/` | finished clean |
| `_failed/20260729-120000-video/` | finished badly, or was stopped |

Staging is the claim: `mkdir` of the `_ready` folder fails if it exists, so two
runners racing the same job can't both win. Promotion from `_ready` to
`_running` uses `renamex_np(RENAME_EXCL)` for the same reason — plain
`rename(2)` silently clobbers its destination, which would make it useless as a
claim.

Inside the folder: the `.job` script, the target file it operates on,
`video.log` (stdout) and `video.error.log` (stderr), each created only on that
stream's first line of output, and both written live so a running job can be
tailed. `_ok` vs `_failed` is decided by exit status alone — stderr output on
its own is not a failure, plenty of tools log there.

## Moving a folder is how you command it

The runner holds an open descriptor on its own run folder and watches it with
kqueue. Move the folder and the kernel tells it, `F_GETPATH` says where to, and
the destination is the verb:

| move a running job to | happens |
|---|---|
| `_paused` | `SIGSTOP` to the process group — the encode freezes and gives back the CPU |
| `_running` | `SIGCONT` — it picks up where it left off |
| `_failed` or `_ok` | `SIGTERM`, then `SIGKILL` after ten seconds |
| `_ready` | terminated and requeued |

Nothing polls, there is no socket, and it works from Finder, a shell, or the
pause and stop buttons in either menu bar app — including one on another
machine over SMB — the filesystem's own permissions are the
access control. If a job finishes while its folder has been moved, the runner
leaves it where you put it rather than overruling you.

A job that ends up in `_failed` can be edited and dragged back to `_ready` to
run again. That is the whole recovery story; there is no separate verb for it.

## Jobs it lost

A run folder only ever gets stranded in `_running` when the *runner* goes —
killed, crashed, or the machine restarted mid-job. While it is alive it
supervises its own children and files them away itself. So it looks once, at
startup: any folder in `_running` whose `.status` names this host and a process
group that no longer exists is filed to `_failed`, with `REAP` in the log.
Nothing had ever cleaned those up, and the monitor went on reporting each one
as **not running** forever.

The claim is deliberately narrow. Another machine's job on a shared folder is
not ours to judge, a folder promoted a moment ago has no `.status` yet, and
only `ESRCH` counts as gone — `killpg` failing any other way says something
about us, and the answer here is acted on by moving somebody's encode out of
the queue.

The runner writes one file into a running job's folder: `.status`, carrying the
process group id and the host that owns it. That is state it emits, not a
channel anyone writes to — it lets a local client renice or signal a job
without being its parent, and the hostname stops a monitor on another machine
testing that pid against its own process table.

Jobs run cd'd into their run folder with:

- `TARGET_FILE` — the job filename minus `.job` (`video.mp4.job` =>
  `video.mp4`). It sits in the CWD, so a bare filename is enough.
- `JOB_NAME` — `TARGET_FILE` minus its extension (`video`). Names the logs.
- `JOB_FILE`, `JOB_RUN_DIR` — the running job's path and its run folder.
- `JOB_DIR` — the jobs root. Any *other* data file shipped alongside stays at
  the top level, so reference it as `"$JOB_DIR/name"`.
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

Everything the UIs show is re-derived from the folder every poll, through
`job-core`'s observer — nothing reads this process's memory. That is why
restarting mid-job shows the job still running rather than an empty queue, and
why a monitor on another machine can show the same thing without talking to
this process at all.

A one-line status trail for every job is appended to is appended to
`~/Library/Logs/jobs.log` (`$JOB_LOG`). Yazi styles the job artifacts
(`config/yazi/theme.toml`).
