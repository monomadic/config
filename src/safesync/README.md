# safesync

One-way media sync between drives that carry their own index, with offline
lookup and a full-screen progress view. Built for Tower → Tower Backup, and for
pulling a selection off both onto an SSD at the speed of two drives.

```sh
scripts/install/install-safesync.sh     # → ~/.local/bin/safesync
```

## The sentinel

Every participating drive carries `ROOT/.safesync/drive.toml`. It is both the
drive's config and the thing that stops a copy in the wrong direction:

```toml
role = "backup"          # source | backup | scratch
name = "Tower Backup"
volume_uuid = "215CF628-…"   # pinned to this disk; a copied sentinel is refused
source_uuid = "695E64CC-…"   # backup only: the one source it mirrors
exclude = [".Trashes", ".Spotlight-V100", ".fseventsd", ".rclone", …]
extras = "keep"          # keep | history — backup files the source no longer has
```

- **source** — never written. `sync` refuses to run towards it, `fill` refuses to
  write onto it.
- **backup** — written only by `sync`, and only from the source whose UUID it names.
- **scratch** — a working disk `fill` may copy onto.

```sh
safesync init /Volumes/Tower --role source
safesync init "/Volumes/Tower Backup" --role backup --source /Volumes/Tower
safesync show /Volumes/Tower
```

## The index

`ROOT/.safesync/index-GENERATION.jsonl` is the drive's source of truth: every
regular file with size, nanosecond mtime, file ID and — once known — a SHA-256
fingerprint. The last three generations are kept on the drive, and a copy of
each lands in `~/Library/Application Support/safesync/manifests/` so you can
search a drive that is in a drawer.

```sh
safesync scan /Volumes/Tower           # metadata; carries over known fingerprints
safesync scan /Volumes/Tower --hash    # also reads files that have no fingerprint yet
safesync scan /Volumes/Tower --hash --rehash   # audit: read everything again
```

A fingerprint is reused when the file ID, size and mtime are unchanged, so a
renamed video is not read again. `sync` fingerprints everything it copies (it
read the bytes anyway), so after one full sync both indexes are fully hashed
without a separate `--hash` pass. What reuse cannot see is an in-place edit
that keeps size and mtime, or rot at rest — `--rehash` occasionally is the check.

## sync

```sh
safesync sync /Volumes/Tower "/Volumes/Tower Backup" [--verify] [--hash] [--yes]
```

Scans both drives in parallel (only new or changed files are read), shows what
it would do, and waits for Enter:

- **copy** — on the source only.
- **rename** — the backup already holds the content under another name (same
  size and mtime, or same fingerprint); moved, not copied.
- **replace** — same path, different content. The backup's version goes to
  `.safesync/history/GENERATION/` first, never deleted.
- **retire** — only with `extras = "history"`: backup-only files move to history.

The source is never written except for its own `.safesync/index`. Each file is
copied uncached (`F_NOCACHE`), preallocated, read and written on separate
threads, hashed on the way through, given the source's mtime and xattrs, and
published under its real name with `renamex_np(RENAME_EXCL)` — a name that is
already taken is an error, never an overwrite. `--verify` reads each file back
after the copy. Esc stops after the current file. Both indexes are updated in
memory as files land and republished at the end.

## fill

```sh
safesync fill --from /Volumes/Tower --from "/Volumes/Tower Backup" --to /Volumes/SSD/dump clips/2026
fd -0 . /Volumes/Tower/clips | safesync fill --from /Volumes/Tower --from "/Volumes/Tower Backup" --to /Volumes/SSD/dump --stdin
```

Reads the drives' indexes (no scan), works out which drives hold each selected
file, and runs one copy worker per drive: files only one drive has go first,
files both have go to whichever drive frees up. Two drives, double rate. The
destination must be a scratch drive or an unmarked disk; a source or backup is
refused. Files already at the destination with the same size and mtime are skipped.

## Offline lookup

```sh
safesync lookup --name 'Holiday.mov'
safesync lookup --file ~/Downloads/clip.mov       # same content under any name
safesync compare tower.jsonl backup.jsonl
safesync manifests
```

Results describe what the index recorded at scan time, not what is on the disk
right now. Exit 0 match, 1 no match, 3 same-size candidates without fingerprints.

## Non-interactive use

Without a TTY every command prints plain lines and `sync`/`fill` need `--yes`.
Exit 0 done, 1 cancelled or some files failed, 2 refused or errored.

## Tests

```sh
cargo test --manifest-path src/safesync/Cargo.toml
```

`tests/sync.rs` creates APFS ram disks with `hdiutil`/`diskutil` so sentinel
UUIDs, preallocation and exclusive renames are the real thing (~25 s).
