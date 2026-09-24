# Safesync TODO

Scope: indexing, offline lookup, sentinel-guarded one-way sync, parallel fill,
TUI. Not a journaled transaction engine — see git history before 2026-09-23
for the version that tried to be.

- [ ] First real run: `init` on Tower and Tower Backup, `scan --hash` on both
      (the one full read), then `sync`. Compare the plan with `rclone-tower-safe --dry-run`.
- [ ] Replace `rclone-tower-safe` / `rclone-tower-ui` once a few syncs have gone well.
- [ ] Drives screen: `--file` fingerprint search (today: name search only), and changing a role
      once there is a story for what happens to the old sentinel and index.
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

## Binary index

Why: Tower's index is 1.5 M files. As JSONL that is 498 MB, 325 bytes a file,
and every consumer parses all of it: 3.2 s of CPU and the whole thing in RAM
(the drives screen's search catalog alone is ~1.5 M lowercased Strings).
The 2026-09-25 change let the drives screen open on the header and footer,
but search, `behind`, `compare` and the sync plan still pay the full parse.
Measured on the same index: a NUL-terminated path heap is 141 MB and streams
through a pipe in 10–30 ms; a sorted record table is ~145 B/file unhashed,
~177 B hashed, so ~220–270 MB for Tower.

Chosen over SQLite on purpose: the three readers we have (summary, merge
comparison, path export) are all sequential or O(1) on a sorted table, so a
query engine buys nothing, and a mmap'd immutable file keeps the
hard-link-publish / footer-digest model unchanged. Revisit only if a reader
appears that needs an ad-hoc query.

- [ ] **Format** — `ROOT/.safesync/index-GENERATION.ssi`, one immutable file
      per generation, little-endian, every region 8-byte aligned, read via mmap.
      Layout, in file order:
      1. *Header*, fixed size: magic `SSIX`, format version u32, header length
         u32, then the fields the JSONL header has today (generation, role,
         volume uuid/name/filesystem, recorded drive role + source uuid,
         root path, root file id, device, started/finished unix, content_hashed,
         reused/skipped counts, exclusions), followed by `files` u64, `bytes`
         u64 and the offset + length of every region below. Variable-length
         strings in the header live in a small header string heap so the fixed
         part stays fixed. `device` moves here: a scan never crosses mounts,
         so it is one value per index, not per file.
      2. *Records*, `files` × 56 bytes, **sorted by path bytes**:
         file_id u64, size u64, mtime_seconds i64, mtime_nanos u32,
         ctime_seconds i64, ctime_nanos u32, path_offset u64 (into the path
         heap), path_len u32, flags u32 (bit 0 = has fingerprint).
      3. *Path heap*: every path, NUL-terminated, in record order. Raw bytes,
         no base64, no escaping — names may hold newlines and non-UTF-8, which
         is exactly why the terminator is NUL. This region is what `paths`
         writes to stdout in one `write`. No front-coding: it would cut the
         heap by ~2/3 but turns the one-write export into 1.5 M copies and
         binary search into a decode; not worth it at 141 MB.
      4. *Fingerprints*, `files` × 32 raw bytes in record order, all-zero
         where flags say none. Present only if any file is hashed.
      5. *Fingerprint index*: `hashed` × u32 record numbers, sorted by the
         fingerprint they point at. Binary search here is how rename
         detection and `lookup --file` find content under another name.
      6. *Trailer*: SHA-256 over regions 2–5, then magic `SSIX` again.
         No trailer = uncommitted, as the missing JSONL footer means today.
      Path validation (absolute? components? duplicates? NUL inside a name?)
      happens at write time and again on read, as `validate` does now;
      sortedness is checked on read with one linear pass the first time the
      records are touched, not on `summary`.
- [ ] **Readers** — `Manifest::summary` reads the header only (`files`,
      `bytes`, dates: what the drives screen shows). `behind`, `compare` and
      `plan` become a merge-join over two sorted record tables: one pass, no
      hash maps, memory flat. Rename detection is a binary search in region 5.
      `lookup --name` is a binary search on the heap for an exact basename
      only if we sort a second key; otherwise it is a substring scan of the
      heap, which is ~30 ms for Tower and fine.
- [ ] **`safesync paths [DRIVE|INDEX ...]`** — writes `DRIVE\tPATH\0` for every
      entry of the named indexes, or of every saved index when none is named;
      `--sizes` appends `\tSIZE`. It is the search interface: the TUI runs no
      search of its own any more. In the drives screen `/` restores the
      terminal, runs `fzf --read0 --print0 --exact --delimiter '\t' --with-nth 2..`
      with a preview showing drive, size, scan date and whether the drive is
      mounted, then re-enters the screen on the chosen line (focus the drive
      row; a second key reveals the file in Finder if mounted). Same hand-off
      shape as scan → drives. From the shell: `safesync paths Tower | fzf --read0`,
      `| rg -z`, `| xargs -0`. Start `--exact` because fuzzy over 1.5 M paths is
      noise; a quoted term still toggles per fzf's rules. Prefer naming drives:
      the whole library is several million lines and ~1 GB inside fzf.
      Drop the `Catalog` type and the in-memory lowercase copy with it.
- [ ] **Migration** — the writer switches to `.ssi` at once; the JSONL reader
      stays until every drive and every library copy has been rescanned, then
      goes. `info`, `compare`, `lookup` read both meanwhile. `Manifest` keeps
      its in-memory shape (`header` + `entries`) for the engine; only the on-disk
      codec changes, so tests that build manifests in memory are untouched.
- [ ] **Library pruning** — keep the newest N (3, like the drive) generations
      per volume in `~/Library/Application Support/safesync/manifests/`; today
      nothing prunes it and Tower adds half a gigabyte per scan. Do this before
      the migration rescans double it.
