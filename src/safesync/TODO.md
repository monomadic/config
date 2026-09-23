# Safesync TODO

Scope: indexing, offline lookup, sentinel-guarded one-way sync, parallel fill,
TUI. Not a journaled transaction engine — see git history before 2026-09-23
for the version that tried to be.

- [ ] First real run: `init` on Tower and Tower Backup, `scan --hash` on both
      (the one full read), then `sync`. Compare the plan with `rclone-tower-safe --dry-run`.
- [ ] Replace `rclone-tower-safe` / `rclone-tower-ui` once a few syncs have gone well.
- [ ] `--pause` (SIGSTOP-style) in the TUI; currently only stop-after-current-file.
- [ ] Rename detection by fingerprint when mtime was touched (needs `--hash` on both sides today).
- [ ] History pruning: `safesync history --prune 30d` for `.safesync/history/`.
- [ ] `fill` capacity policy: stop-at-first-non-fitting vs skip-and-continue.
- [ ] Disk-image power-loss test of index publication (fsync + hard-link ordering).
