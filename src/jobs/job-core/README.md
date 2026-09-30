# job-core

The drawing half of [`job-folder`](../job-folder): the menu rows, the menu bar
icon, and the small helpers they share. Not a tool — nothing here builds or
installs on its own, and nothing here runs a job.

| module | |
|---|---|
| `row` | the menu row — state symbol, name, value, progress bar, the running job's last log line, and its buttons — plus the shared `Layout` that lines rows up down a menu |
| `icon` | the menu bar icon, drawn rather than glyph-based, in several selectable styles |
| `progress` | `parse_progress`: the last percentage in a line of output, as a fraction |
| `clock` | local-time formatting via Foundation |

A row's buttons are `Act::Call` tokens handed to the handler the app registers
with `row::on_call`, or `Act::Open` for the log. `JobRow::update` hands a row a
new spec while the menu is open, so a percentage climbing redraws in place
rather than the menu being rebuilt under the pointer.

```bash
cd src/jobs && cargo test -p job-core
```
