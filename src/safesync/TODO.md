# Safesync TODO

Scope: indexing, offline lookup, sentinel-guarded one-way sync, parallel fill,
TUI. Not a journaled transaction engine — see git history before 2026-09-23
for the version that tried to be.

- [ ] First real run: `init` on Tower and Tower Backup, `scan --hash` on both
      (the one full read), then `sync`. Compare the plan with `rclone-tower-safe --dry-run`.
- [ ] Replace `rclone-tower-safe` / `rclone-tower-ui` once a few syncs have gone well.
- [ ] Drives screen: `--file` fingerprint search (today: name search only), and changing a role
      once there is a story for what happens to the old sentinel and index.
- [ ] Scan progress: split the walk from fingerprinting so the slow half has a known total.
      Walk = bar against the previous index's file count (held at 99% until done; spinner
      with a running count when there is no prior index). Hash = bytes bar with a time
      estimate from the copy screen's Speed/eta helpers, since the file list and byte total
      are exact once the walk finishes. Same for `--rehash`.
- [ ] `--pause` (SIGSTOP-style) in the TUI; currently only stop-after-current-file.
- [ ] Rename detection by fingerprint when mtime was touched (needs `--hash` on both sides today).
- [ ] History pruning: `safesync history --prune 30d` for `.safesync/history/`.
- [ ] Standing fill: a `[fill]` table on the scratch sentinel so `safesync fill ROOT` with no
      arguments tops the drive up from whichever library drives are mounted.
      `from` = library UUIDs in priority order: the first one mounted is the authority whose
      index builds the wanted set, the rest are extra readers for the parallel copy (so a
      drive can follow a backup instead of the source by listing it first). `select` = paths, `order` = newest | largest | path (decides what
      makes the cut; transfer order stays largest-first for the two readers),
      `when_full` = stop | skip, `retire` = bool, `keep` = paths fill never writes or removes.
      Retire works without indexing the scratch drive: walk it with stat, and a file is
      *disposable* only if a library index records the same path, size and mtime — fill never
      deletes the only copy. Copy list = wanted − present. Deletion is lazy: on ENOSPC at
      preallocation, remove disposables (inverse of `order`, oldest first for `newest`) until
      the file fits, then retry. Review screen shows copy / would-remove / not-on-any-library
      before Enter; drives screen shows "holds 812 of 1 204 wanted · 3 not in any library".
- [ ] Single-copy report: list every file that exists on exactly one known drive, across all
      saved indexes (mounted or not), matched by fingerprint where available and by size+mtime
      otherwise. Surface it in the drives screen's search view (a filter or a `!single` mode)
      and as `safesync lookup --single`. The same "is this recorded elsewhere?" check is what
      standing fill uses to decide what is disposable, so build it once.
- [ ] Disk-image power-loss test of index publication (fsync + hard-link ordering).
