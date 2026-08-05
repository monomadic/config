# job-core

The shared library behind [`job-server`](../job-server) and
[`job-monitor`](../job-monitor). Not a tool — nothing here builds or installs
on its own.

| module | |
|---|---|
| `observe` | the naming rules (`video.mp4.job` → `TARGET_FILE` / `JOB_NAME`), and the `Observer` that reads a jobs folder into a `Snapshot` |
| `icon` | the menu bar icon, drawn rather than glyph-based, with the remote and disconnected variants |
| `clock` | local-time formatting via Foundation, so run folders and log lines stamp the same way the shell runner does |

Nothing in here ever claims, moves or runs a job. Reading a jobs folder is
always safe, from any machine, however many readers there are — which is what
lets the monitor share this code with the runner instead of reimplementing it.

The `Snapshot` is derived entirely from the folder, never from a running
process: `*.job` at the top level is the queue, `_running/*` is what is
running, `_done`/`_err` are outcomes, `.paused` holds the queue, `.lock`'s
mtime is the runner's heartbeat. That is the property the whole design rests
on — a restarted runner and a monitor on another machine see the same thing,
because there is only one place either of them can look.

`_done` and `_err` grow without bound and every read of them over SMB is a
round trip, so both are cached against the directory's own mtime and re-read
only when an entry has actually been added or removed.

```bash
cd utils/job-core && cargo test
```
