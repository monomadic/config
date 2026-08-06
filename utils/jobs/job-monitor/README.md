# job-monitor

A read-only menu bar view of one or more jobs folders — normally a
[`job-daemon`](../job-daemon) queue on another machine, mounted over SMB at
`/Volumes/Jobs`. It shows what is running, what is queued, and what finished,
and posts a notification when something completes or fails.

It is a normal `.app` you launch and quit, not a LaunchAgent.

```bash
setup/install/install-job-monitor.sh
```

Builds the crate, assembles `~/Applications/Job Monitor.app`, ad-hoc signs it,
and seeds `~/.config/job-monitor/roots`. No agent is installed and nothing
starts at login; open it when you care, quit it when you don't.

## Why a separate app

A `--remote` flag on the runner would be one typo away from a second machine
*claiming and running* jobs off the share. The `.lock` mkdir would probably
save you; over SMB, "probably" is not the guarantee you want on an encode
queue. A binary with no runner compiled into it cannot claim a job however it
is launched.

The bundle earns its keep twice over: `UNUserNotificationCenter` refuses to
work without a bundle identifier, and a bundle is what makes it a normal app
with a Quit item rather than a service.

The only thing it ever writes to the share is a folder move, and only when you
press a button. Acknowledged errors and the notification mute live in
`~/.config/job-monitor/`, on your machine, because how *you* read a shared
folder is nobody else's business.

## Watched folders

In order of precedence:

| | |
|---|---|
| `$JOB_MONITOR_ROOTS` | colon-separated paths, for a one-off run |
| `~/.config/job-monitor/roots` | one path per line, `#` comments, `~` expanded |
| `/Volumes/Jobs` | the default |

Each folder gets its own polling thread, so a share that has gone away blocks
only its own updates. With more than one, the menu grows a labelled section
per folder.

## The rows

Each job is a row in the style of `free-disk-space-widget`'s volumes: state
symbol on the left with a small caption under it (elapsed, queue position, or
how long a finished job took), the name with a value right-aligned, and a bar
underneath. Click a row to open its run folder. Job rows share one height;
section headers and the actions below them stay ordinary menu items.

Each job row carries its own controls on the right, in the style of the
eject button on a volume row:

| button | on | does |
|---|---|---|
| pause | running | moves the folder to `_paused` — the runner `SIGSTOP`s its process group |
| resume | paused | moves it back to `_running` — `SIGCONT` |
| stop | running or paused | moves it to `_failed` — `SIGTERM`, then `SIGKILL` after ten seconds |
| hold | queued | moves it to `_paused`, so it is skipped until you move it back |

Every one of those is a single `rename`. That is why they work from here at all:
this app has no runner in it and no way to signal anything, but the runner on
the other machine is watching its own folders and does the rest.

The running row carries one line more — the last thing the job printed:

```
▷    my night collection                    45%
1:38 ▓▓▓▓▓▓▓▓▓▓▓▓░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░
     45% · frame 64512 · 18.2 fps · eta 1:52:10
```

The bar is real when the log carries a percentage — `topaz-encode` prints one
every 5% — and striped when it doesn't, so it reads as motion without claiming
a position it doesn't know.

`cargo run --example render_rows -- /tmp/rows.png dark` in `utils/job-core`
draws a sample menu to a PNG, which is the quick way to look at a change to the
row design without waiting for a real job.

## What it can and can't tell you

Everything comes from the folder itself, through `job-core`'s observer — the
same code it renders the local queue with, so a folder watched across the LAN reads
exactly the way it does on the machine running it.

- **Not mounted** is a distinct state from **Idle**. An unreachable share
  rendered as an empty queue reads as "nothing to do", which is the one
  actively harmful answer; the icon dims and the menu says so instead.
- **no output 41m** in red reports what was observed rather than inferring —
  but only when nothing better is available. On the machine running the job its
  process is asked directly, and silence is ignored; elsewhere the bar is 45
  minutes, because a slow encoder logging every 5% can legitimately go half an
  hour between lines.
- **Runner not responding** needs two independent signals to agree: nothing has
  heartbeated `.lock` in five minutes *and* the job has printed nothing in ten.
  Either alone lies — a job can outlive a heartbeat for dull reasons, and
  plenty of jobs are legitimately silent.
- Elapsed time comes from the run folder's birth time, i.e. the *server's*
  clock, and is clamped at zero if that clock runs ahead of yours.

## Freshness

macOS caches SMB directory listings — `dir_cache_min` 30s and `dir_cache_max`
60s by default (`man nsmb.conf`; the file doesn't exist until you create it,
and there is a per-user `~/Library/Preferences/nsmb.conf` too). The client does
request change notifications from the server, which in practice invalidates
sooner, but that ceiling is why polling is paced at 2s while something is
running, 8s idle and 15s when the folder is unreachable — anything faster is
answered from the same cache.

If it feels sluggish, tighten the cache for that share alone:

```
[SERVER:JOBS]
dir_cache_min=2
dir_cache_max=5
```

Every poll runs on a background thread, always: a `read_dir` on a dead mount
blocks until the mount times out, and doing that on the main thread is how a
menu bar app comes to beachball.

## Icon

A terminal-frame icon with a doubled chevron. Dim frame with an empty cursor
slot means the folder is unreachable, which is deliberately not how idle
looks.
