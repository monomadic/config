# Safesync TODO

Scope: source indexing, offline lookup, sentinel-guarded one-way sync, parallel fill,
TUI. Not a journaled transaction engine — see git history before 2026-09-23
for the version that tried to be.

## Index the source, walk the backup

Agreed model: **index the source; check the backup**. A source carries its own
index, with a copy saved on this Mac for offline search. Backups and scratch
drives keep their role sentinels and no file index. A local copy of a source
index is a catalog of the source, not evidence of a second copy of its files.

Why both sides are indexed today: the engine plans from two manifests, and the
backup manifest was the cheapest way to feed it. Nothing else needs it.

- [ ] Sync = refresh the source index (reuse unchanged fingerprints), then walk
      the backup live and answer, per file, "is this on the source?":
      same path, same size+mtime → unchanged; same path, different stamp →
      replace; no path but a source file with the same stamp or fingerprint →
      rename; nothing matching → extra, kept or moved to history per the
      sentinel. Then copy every source file the backup lacks. Prune history
      in the same pass. Keep the sentinel checks and the preview before copying.
      The backup walk is the existing scanner without publish; nothing about
      the backup is written except media and history.
- [ ] Rename candidates with no stamp match may be hashed live. Only the backup
      file is read (the source fingerprint is in its index), and only for files
      that match nothing by size+mtime, so it is normally zero files. Show
      progress when a check starts reading files.
- [ ] Exclusions: the source is king. Its sentinel's `exclude` list defines the
      catalog; backups have no exclude list, they take what the source gives them.
      The backup walk applies the source's list plus the built-in system folders
      (`.Spotlight-V100`, `.fseventsd`, `.Trashes`, ...), which every volume grows
      on its own and which must never look like extras. Drop `exclude` from
      backup and scratch sentinels and from the `sync` command. Today the
      merged list is applied to the source scan and that result is published,
      so a backup exclusion silently shrinks the source's saved index.
- [ ] Guided setup: choose the originals drive, choose its backup, show the
      direction, then **Save and check backup**. The index refresh and the
      backup walk happen inside that flow; no per-disk indexing chore.
- [ ] Volume rows by role: sources show index state (not indexed / indexing /
      indexed at); backups show check state (not checked / checking / last
      checked at). A backup's old index date is not a check timestamp. Scratch
      drives show neither.
- [ ] Offline volumes: matching saved indexes to mounted volumes by UUID, one
      row per UUID across generations, already exists (`drives.rs`, the
      "drawer" block). What is missing: once backups have no index, they have
      nothing to be listed from. Save a small local volume record (UUID, name,
      role, source UUID, last checked) at setup and after each check, next to
      the source catalogs, so a backup in a drawer still gets a row. Display
      only; live sentinels authorize. Keep the legacy backup index headers as
      those records when their inventories are retired.
- [x] Distinguish loading comparisons from missing indexes in the drives screen.
- [x] **y Check / sync** on backup rows, source rows and group headings, without
      requiring manual indexing first.
- [ ] First real run: works on today's engine. Set up Tower as source and Tower
      Backup as its backup, **Check / sync**, compare the plan with
      `rclone-tower-safe --dry-run` before confirming. Known caveat until the
      exclusion fix lands: the published source index is narrowed by the
      backup's exclusions.

## Other work

- [ ] Replace `rclone-tower-safe` / `rclone-tower-ui` once a few syncs have gone well.
- [ ] Drives screen: `--file` fingerprint search (today: name search only), and changing a role
      once there is a story for what happens to the old sentinel and any source or legacy index.
- [ ] `--pause` (SIGSTOP-style) in the TUI; currently only stop-after-current-file.
- [ ] History pruning: `safesync history --prune 30d` for `.safesync/history/`.
- [ ] Standing fill: a `[fill]` table on the scratch sentinel so `safesync fill ROOT`
      with no arguments tops the drive up. Fill needs only the **source** mounted:
      the source index says what exists, the sentinel's rules say what is wanted.
      `from` = source UUID, `select` = paths, `order` = newest | largest | path,
      `when_full` = stop | skip, `keep` = paths fill never writes or removes.
      Extra readers are simple and need no index: if the same relative path exists
      on a mounted backup or another fill drive with the same size and mtime, it
      is used as a second (or third) reader. Fill drives are loose caches, not a
      safe copy method; the source index stays the only authority. Files already
      present with matching size and mtime are skipped. Retirement of files that
      fell out of the selection comes later, and only after confirming a copy on
      the mounted source.
- [ ] Rework the single-copy report for source-only indexes: offline lookup can say
      "recorded on one source", not "exists on exactly one drive", since backups
      have no index and a local catalog copy is not a second copy. A live copy
      report checks mounted readers directly.
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
      per source generation, plus its local offline-search copy. No backup or scratch
      index writer. Little-endian, every region 8-byte aligned, read via mmap.
      Layout, in file order:
      1. *Header*, fixed size: magic `SSIX`, format version u32, header length
         u32, then source metadata (generation, source role,
         volume uuid/name/filesystem,
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
      3. *Path heap*: every source-relative path, NUL-terminated, in record order.
         Raw bytes, no base64, no escaping — names may hold tabs, newlines and
         non-UTF-8, which is exactly why the terminator is NUL. Keep paths relative
         in the index so a changed mount point does not require rewriting it.
         `paths` prefixes the source root and streams full paths through a bounded
         output buffer; it does not materialize the complete export in memory.
         No front-coding: keep direct path access and binary search without decoding.
      4. *Fingerprints*, `files` × 32 raw bytes in record order, all-zero
         where flags say none. Present only if any file is hashed.
      5. *Fingerprint index*: `hashed` × u32 record numbers, sorted by the
         fingerprint they point at. Binary search here is how rename
         detection and `lookup --file` find content under another name.
      6. *Trailer*: SHA-256 over every preceding byte of the file, including the
         finalized header, its string heap, regions 2–5 and alignment padding,
         then magic `SSIX` again. The trailer itself is outside the digest.
         No trailer = uncommitted, as the missing JSONL footer means today.
      Path validation (absolute? components? duplicates? NUL inside a name?)
      happens at write time and again on read, as `validate` does now;
      sortedness is checked on read with one linear pass the first time the
      records are touched, not on `summary`.
- [ ] **Readers** — `Manifest::summary` reads the header and committed trailer
      (`files`, `bytes`, dates: what the source row shows). Validate the format version,
      header length, region bounds/alignment/non-overlap and expected lengths using
      checked arithmetic, and require the complete trailer at the expected file end.
      A truncated file must never appear indexed. This remains O(1) in entry count;
      defer the full digest, record, path and sortedness checks until entries are read.
      Sync planning compares the source table with live destination metadata;
      a sorted transient destination table may
      support a merge-join, but must not become a persistent backup index requirement.
      Remove saved source/backup `behind` comparisons from the default drives screen;
      show check results only after checking the backup. Explicit `compare` between
      saved source catalogs remains historical, not a live backup check.
      Rename detection can search source fingerprints in region 5 and hash live
      destination candidates as needed.
      `lookup --name` is a binary search on the heap for an exact basename
      only if we sort a second key; otherwise it is a substring scan of the
      heap, which is ~30 ms for Tower and fine.
- [ ] **`safesync paths [DRIVE|INDEX ...]`** — writes one full absolute path followed
      by NUL for every entry of the named source indexes, or the newest saved index per source UUID
      when none is named; do not duplicate results for generations or local copies.
      Use the current source mount point identified by UUID when mounted, otherwise
      the recorded source root. An offline path describes the recorded location;
      its existence or current ownership is not established by this output.
      Preserve raw path bytes, including tabs and newlines. No separate drive field,
      sizes, escaping, headings or other stdout content: just `FULL_PATH\0` repeated.
      It is the search interface: the TUI runs no search of its own any more.
      In the drives screen `/` restores the
      terminal, runs `fzf --read0 --print0 --exact`, then re-enters the screen
      on the chosen path (focus the source row; a second key reveals the file
      in Finder after checking that the matching source UUID is mounted there).
      Same hand-off shape as scan → drives.
      From the shell: `safesync paths Tower | fzf --read0`,
      `| rg -z`, `| xargs -0`. Start `--exact` because fuzzy over 1.5 M paths is
      noise; a quoted term still toggles per fzf's rules. Prefer naming drives:
      the whole library is several million lines and ~1 GB inside fzf.
      Drop the `Catalog` type and the in-memory lowercase copy with it.
- [ ] **Migration** — implement source-only ownership before switching the source
      writer to `.ssi`. Keep the JSONL reader until source indexes and their local
      search copies have been migrated; do not require rescanning backups or scratch
      drives. `info`, `compare`, `lookup` read both formats meanwhile, with legacy
      backup/scratch catalogs identified as historical and excluded from normal
      source search and sync decisions. Retire those catalogs through an explicit
      migration step, retaining their offline volume metadata as described above,
      without touching sentinels, media or history. Temporary manifests
      may remain an engine implementation detail; tests must cover source-only
      publication and destinations with no saved index, separately from codec tests.
      Cover backups with different exclusions without shrinking the source catalog;
      source-excluded backup files staying untouched even with `extras = history`;
      offline backup rows surviving legacy-index retirement and generation pruning;
      same-name volumes being distinguished by UUID; fill with only the source
      mounted, and with a backup reader that lacks any index. Codec/export tests cover truncated
      trailers, header/string-heap corruption, invalid region bounds,
      raw tab/newline/non-UTF-8 paths, changed mount
      points, offline roots and exactly one NUL after each exported full path.
- [ ] **Library pruning** — already keeps the newest 3 generations per volume UUID
      (`KEPT_GENERATIONS` in `drive.rs`). Once backups stop publishing, their old
      copies in `~/Library/Application Support/safesync/manifests/` are retired by
      the migration step above, not by pruning.
