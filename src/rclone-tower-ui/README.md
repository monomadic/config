# Tower backup dashboard

Run `scripts/install/install-rclone-tower-ui.sh`, then `rclone-tower-safe` in a
terminal. The shell command opens this Bubble Tea dashboard automatically;
piped/non-terminal runs retain text output. Uses spill's pink/cyan palette.

The shell script remains responsible for volume UUID checks, locking, filesystem
verification, sync limits, history and exit status. Before a normal sync it runs
`diskutil verifyVolume` on both volumes, in sequence. This verifies filesystem
structures (including underlying APFS storage), not file contents or hardware
health. It does not repair anything. Verification can take time and can temporarily
unmount a volume; close apps using the disks first. Any verification failure
prevents sync. Identities are checked again afterward.

- `--check-only`: identity, filesystem and free-space checks, without syncing.
- `--dry-run`: preview sync; skips filesystem verification and performs no repairs.
- `--prune`: explicit history cleanup in the existing text interface.
- `q`, Escape or Ctrl-C: ask the shell to stop, wait for its active child, retain
  the result on screen. Press again after completion to close.

The dashboard shows rclone's listed/checked counts, current transfer, speed,
ETA, deletion count, destination capacity and warnings. Totals grow while scanning;
100% of currently known bytes does not mean the entire run is finished.
Success means the shell exited successfully; skipped-file warnings remain visible.
Rename matching still uses size + modification time, without added hashing.

Dashboard runs leave history pruning to a separate `--prune` invocation, so no
confirmation prompt is hidden behind the full-screen display. Logs remain in
`~/.rclone-tower-safe/sync.log`; the preceding log becomes `previous.log`, with
rclone rotating the active log at 10 MiB. Log parsing is exclusively for display.

Validation: `go test ./...` and `python3 workflow_test.py`. The workflow tests use
fake diskutil/rclone commands and temporary directories, never the actual disks.
