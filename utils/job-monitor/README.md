# job-monitor

A read-only menu bar view of one or more jobs folders — normally a
[`job-server`](../job-server) queue on another machine, mounted over SMB at
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

A `--remote` flag on `job-server` would be one typo away from a second machine
*claiming and running* jobs off the share. The `.lock` mkdir would probably
save you; over SMB, "probably" is not the guarantee you want on an encode
queue. A binary with no runner compiled into it cannot claim a job however it
is launched.

The bundle earns its keep twice over: `UNUserNotificationCenter` refuses to
work without a bundle identifier, and a bundle is what makes it a normal app
with a Quit item rather than a service.

The only thing it ever writes to the share is the `.paused` marker, and only
when you ask — pausing a remote queue is one file, and the runner over there
already honours it. Acknowledged errors and the notification mute live in
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

## What it can and can't tell you

Everything comes from the folder itself, through `job-core`'s observer — the
same code `job-server` renders locally, so a queue watched across the LAN reads
exactly the way it does on the machine running it.

- **Not mounted** is a distinct state from **Idle**. An unreachable share
  rendered as an empty queue reads as "nothing to do", which is the one
  actively harmful answer; the icon dims and the menu says so instead.
- **Runner not responding** means a job is sitting in `_running/` with no live
  runner behind it — killed mid-job. Detected from `.lock`: gone entirely, or
  not heartbeated in an hour.
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

The same terminal-frame icon as `job-server`, with a doubled chevron — so two
menu bars' worth of job icons stay tellable apart. Dim frame with an empty
cursor slot means the folder is unreachable, which is deliberately not how idle
looks.
