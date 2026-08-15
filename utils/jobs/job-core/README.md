# job-core

The shared library behind [`job-daemon`](../job-daemon) and
[`job-monitor`](../job-monitor). Not a tool — nothing here builds or installs
on its own.

| module | |
|---|---|
| `observe` | the naming rules (`video.mp4.job` → `TARGET_FILE` / `JOB_NAME`), and the `Observer` that reads a jobs folder into a `Snapshot` |
| `icon` | the menu bar icon, drawn rather than glyph-based, with the remote and disconnected variants |
| `row` | the menu row every app draws — icon, name, value, progress bar, and the running job's last log line — plus `sections`, which turns a `Snapshot` into them |
| `clock` | local-time formatting via Foundation, so run folders and log lines stamp the same way the shell runner does |

A row's buttons are the one place this crate acts. Normally that action is a
folder move, which is why it needs no callback and both folder-watching apps
behave identically. `job-folder` holds its queue in memory instead, so a button
there is an `Act::Call` carrying a token to the handler that app registered with
`row::on_call` — and `JobRow::update` lets it hand a row a new spec while the
menu is open, rather than the menu being rebuilt under the pointer.

Nothing in here ever claims, moves or runs a job. Reading a jobs folder is
always safe, from any machine, however many readers there are — which is what
lets the monitor share this code with the runner instead of reimplementing it.

The `Snapshot` is derived entirely from the folder, never from a running
process — and the folder a job sits in *is* its state: `_ready`, `_running`,
`_paused`, `_ok`, `_failed`. No lock file, no pause flag, nothing the tools
have to agree on beyond where things are. That is the property the whole design
rests on: a restarted runner and a monitor on another machine see the same
thing, because there is only one place either of them can look — and moving a
folder is therefore also how the queue is commanded.

`_ok` and `_failed` grow without bound and every read of them over SMB is a
round trip, so both are cached against the directory's own mtime and re-read
only when an entry has actually been added or removed.

```bash
cd utils/job-core && cargo test
cargo run --example render_rows -- /tmp/rows.png dark    # or light, and add
cargo run --example render_rows -- /tmp/rows.png dark quiet   # a silent job
```

`render_rows` draws a sample menu — running, queued and finished jobs — to a
PNG using the real views and layout. Row geometry is the kind of thing you can
only judge by looking at it, and waiting for a real encode to reach 45% is a
poor edit-test loop.
