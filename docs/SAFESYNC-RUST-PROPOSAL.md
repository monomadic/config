# Safesync: Rust media sync and transfer TUI

Status: **inventory and offline-lookup milestone implemented in `utils/safesync/`;
remaining sections describe the planned sync engine and TUI**. Safesync is an
independent application, not a replacement or new version of spill.
Scope: macOS, locally attached APFS volumes first. Designed for Tower → Tower
Backup, with a later mode that fills a third disk from either verified replica.

## 1. Recommendation

Build a Rust application with spill's visual language and sequential media-copy
optimizations, backed by persistent, drive-resident flat-file inventories and a
recoverable one-way synchronization engine. Borrow FreeFileSync's documented
idea of tracking file identities across runs, rather than its database format.
This is an independent design, not a claim to reproduce its implementation.

Keep the authoritative indexes **on the drives**. This is appropriate when the
same physical disks travel between Macs: the next computer sees the same state.
Do not maintain writable, authoritative computer-local replicas of those indexes.
Local thumbnail caches and transient in-memory lookup tables are disposable.
User-requested **offline manifest exports** may also live on the system drive.
These immutable historical snapshots are for lookup only; they never become
writable synchronization authorities or participate in index reconciliation.

Use filesystem identity to recognize moves. Use `(size, modification time)` as a
**non-unique secondary lookup**, never as the primary identity or proof that two
files contain the same video. Avoid reading entire existing videos merely to
match renames. Verify newly copied data separately from detecting changes.

The main performance gain should come from avoiding unnecessary copies and,
later, avoiding unnecessary scans. Rust alone does not establish a speedup over
rclone or the existing Go spill implementation.

### Implemented first milestone and offline manifests

Safesync 0.1 provides `scan`, `export`, `lookup`, `compare`, `info`, and `manifests`. Scans
publish immutable flat-file snapshots with integrity footers. Exports saved under
`~/Library/Application Support/safesync/manifests/` let the user search inventories
of external drives without connecting those drives. Keep them separate from the
future mutable drive-owned catalog/journal protocol below.

`scan --hash` records complete-content SHA-256 fingerprints; a metadata-only scan
supports filename presence but cannot establish content equality. The initial
schema explicitly names SHA-256; BLAKE3 remains a candidate for later transfer
verification, not an interchangeable interpretation of existing digests.

`lookup --name` reports exact basename presence. `lookup --file` hashes the query
file and compares saved full-content fingerprints independently of filename.
Matches report volume identity, path, scan time, and evidence. Missing hashes
produce an unknown result rather than a guessed match or a false negative.
All results describe historical observations, never unverified live presence.
The default search includes all locally saved snapshots; explicit manifest
selection distinguishes particular generations. Local snapshots are never
silently merged into the authoritative drive state.

`compare` adds a pure, read-only comparison of two historical snapshots, with
content matches/differences, unknowns, one-sided paths and alternate hash matches.
It preserves ambiguous candidates and records both scopes, but does not authorize
renames/deletions or produce an executable sync plan. Relationship enrollment,
live preconditions and case/Unicode collision checks remain future gates.

`enroll` and `enrollment` create/inspect fixed drive-role records at local APFS
volume roots, checking volume identity and refusing role replacement. This is
metadata-only groundwork, not an executor capability or relationship baseline.

`check-pair` validates an enrolled direction under ordered exclusive leases and
rechecks attachments, namespace identity and markers. The current lease locks the
existing `.safesync` directory, matching enrollment, rather than creating a lock
file. It does not run filesystem verification or authorize writes; leases end on
command exit. Integration with catalog and executor sessions remains pending.

`plan` now produces pure initial-adoption previews from historical manifests:
conditional copies, history-preserving replacements, content review and retained
destination-only files. It includes original observations and logical byte totals,
blocks known namespace/scope problems, and conservatively blocks non-ASCII paths
until destination collation is supported. It neither uses a relationship baseline
nor authorizes execution; identity-based rename planning remains future work.

The implementation does not yet copy, rename or delete media, perform filesystem
repairs, enforce roles in a write executor, or provide the full-screen TUI.
These remain later delivery gates. Spill remains untouched.

## 2. User experience and visual direction

Retain the concrete design choices in `utils/spill/ui.go` and `gradient.go`:

| Element | Proposed treatment |
|---|---|
| Transfer bar | Hot pink `#FF2EC0` → violet `#8A5CFF` → cyan `#1EE6FF` |
| Capacity bar | Teal `#22F5C8` → blue `#4F9CFF` → pink `#FF3C8A` |
| Main text / subdued text | `#E8ECF4` / `#6B7280`; dark unfilled cells `#2A2E3A` |
| Success / warning / error | `#3BE38B` / `#FFC24B` / `#FF5C7A`, plus words and symbols |
| Media preview | Optional cached thumbnail, generated outside the copy path |
| Activity | Compact recent-event list, with an expandable full history |

The persistent header shows **Tower — protected source → Tower Backup — writable
destination**, attachment status, and short identity suffixes. Never abbreviate
both volumes to an indistinguishable label. Direction is visible during preview,
execution, recovery, and the final report.

Workflow: **Identify → Check filesystem → Recover → Scan → Review → Transfer →
Verify → Commit → Summary**. Each phase explains what it is doing. For example,
“Checking the backup filesystem; no media is being copied” is better feedback
than an apparently frozen transfer bar.

Display per-drive read/write rates, current file, total planned bytes, copied
bytes, verified bytes, rename count, bytes avoided by renames, remaining free
space, and space retained in history. Distinguish “copied” from “verified.” While
scanning, show files discovered and directories checked; do not invent a total
or display 100% while the scan is still finding work. ETA should include the
verification phase when known and say “estimating” when it is not.

Provide pause-after-current-file and cancel. Cancellation waits for workers and
journal persistence before restoring the terminal. Physical removal transitions
to “disconnected; recovery required,” never “done.” Pending repairs and skipped
checks remain visible in the final summary.

Support small terminals, resizing, 256-color and monochrome output, reduced
animation, keyboard-only operation, and escaped control characters in filenames.
Render at a bounded rate, independently of I/O events; a slow terminal must not
slow the copying engine. Non-TTY mode emits structured JSON events and a readable
stderr summary with documented nonzero outcomes for failures and incomplete work.

## 3. What belongs on each drive

Proposed reserved namespace:

```text
/.safesync/
    volume.json                  durable role marker, independent of inventory
    CURRENT                      committed generation manifest
    generations/<id>/            immutable inventory and relationship snapshots
    runs/<run-id>/                immutable plan and append-only operation journal
    lock                         OS advisory lock; not a PID-based ownership claim
    history/<run-id>/...          recoverable displaced files, on destinations
    staging/<run-id>/...          owned temporary files, on destinations
    reports/<run-id>.json         bounded portable audit reports
```

The `.safesync` namespace belongs only to this tool. It is not spill metadata.

The namespace is always excluded from media selection and mirroring. Refuse to
use it if it is a symlink, belongs to an unrelated application, or lacks a valid
enrollment record. Its contents must not be silently deleted by another backup
tool; document the exclusion during migration.

**Each catalog has one purpose and one owner:** Tower's inventory describes Tower;
Tower Backup's inventory describes Tower Backup. They are not two competing
copies of the same index. The destination also owns the pair's last committed
source-to-destination mapping, transfer journal, and retention records. A stored
source observation in that mapping is a historical baseline, not a second live
source catalog.

One source may have multiple destination relationships. Each destination owns
its own baseline, so a newer backup to disk B does not imply disk C is current.
A fill-to-third-disk job is owned by the third disk and records both input catalogs'
generations. Another Mac must open that same job record to continue it.

Drive catalogs are not committed atomically together. Bind plans to explicit
catalog generations and commit the destination's mapping only after successful
operations. If Tower's latest scan was committed but its backup was interrupted,
the destination baseline remains older: this is expected and recoverable.

No background index replication or timestamp-based “newest database wins.” If
metadata is missing or corrupt, rebuild observations from the filesystem, retain
history, and disable destructive reconciliation until the relationship has been
re-established. Retain previous immutable catalog generations and copy only fully
published generations for metadata backup; never treat a partly written snapshot
as committed state.

### Multiple computers

V1 supports the disks being attached to **one Mac at a time**. Use kernel locks on
the actual drive-resident lock files, acquired in stable volume order, to exclude
other cooperating app instances. Hold source read leases and destination write
leases for media operations; take exclusive catalog leases for inventory changes.
A conservative initial implementation can hold exclusive leases on all involved
volumes for the whole job.

A persistent run record identifies interrupted work. A PID, hostname, or expired
heartbeat is diagnostic information, not sufficient authority to steal a lock.
Physical removal releases the old machine's access; the next attachment still
requires journal reconciliation. Reject writable operation over SMB/NFS and
multi-host storage in v1. Local advisory locking is not a distributed lock and
cannot prevent unrelated tools from changing files.

## 4. Database and comparison model

Use **flat files for v1**, with one state-owning worker per volume. This workload
has a single writer, sequential scans, and lookups that can be built in memory;
it does not specifically require SQL or a database server/library.

Separate a rebuildable inventory from the safety-critical operation journal:

- **Inventory snapshots:** versioned, immutable files grouped into generations.
  JSON Lines is a reasonable initial encoding for inspection and streaming; encode
  raw path bytes explicitly rather than assuming every filename is valid Unicode.
  Include format version, record count, generation, and content checksums in a
  manifest. Build in-memory maps by path, file ID, and size/mtime after loading.
- **Operation journal:** immutable plan plus append-only, framed records containing
  sequence numbers, lengths and checksums. Persist operation intent before its
  filesystem mutation and completion afterward. Distinguish a torn final record
  from corruption in the middle; uncertain state blocks normal execution and is
  reconciled with actual files. A JSON line alone is not a durability protocol.
- **Publication:** write a new generation to temporary paths on the same volume,
  flush files and required directory changes, then atomically replace the small
  `CURRENT` manifest and persist that directory update. Keep the previous valid
  generation. Validate ordering and full-sync behavior on macOS/APFS with fault
  injection; an atomic rename alone does not guarantee power-loss durability.
- **Compaction:** create a fresh snapshot from committed observations and completed
  operations. Publish it before reclaiming old state. Never discard journals or
  generations still needed by an unfinished operation. Batch scan observations;
  do not rewrite the complete inventory after every copied file.

This removes SQLite, but not the need for transactions, recovery, checksums,
locking, version migrations and tests. It is a deliberately small storage protocol,
not an attempt to build a general-purpose database. Initially expect O(N) catalog
loading and O(N) snapshot publication, with O(N) memory for lookup maps. Set a
measured memory budget; if actual inventories exceed it, use sorted/partitioned
snapshots and merge comparisons, or reconsider SQLite before inventing a complex
on-disk index.

SQLite was originally proposed because it already implements transactional
updates, recovery, indexed queries and concurrency control. Those are useful
engineering conveniences, not a functional requirement of file-ID matching or
drive portability. Reconsider it only if measured catalog scale, frequent small
updates or query requirements justify the dependency. Neither SQLite nor flat
files can atomically commit a filesystem copy together with its catalog record;
the application recovery journal remains necessary.
[SQLite atomic commit design](https://www.sqlite.org/atomiccommit.html).

Proposed logical records:

| Record | Important fields |
|---|---|
| Volume | APFS volume UUID, app enrollment UUID, filesystem capabilities, schema version |
| Scan | generation, start/end, coverage, excluded scopes, errors, completeness |
| Entry | entry ID, relative path bytes, kind, filesystem file ID, optional birth/generation discriminator, size, mtime in native precision, ctime, scan generation |
| Replica | source entry/version, destination entry/version, baseline generation, verification level and time, optional content digest |
| Pair | fixed source/destination UUIDs, roots, exclusion policy, history policy, profile revision |
| Run / operation | run ID, immutable plan, expected preconditions, temporary/archive paths, durable state, outcome |
| Media facts | version identity, probe version, dimensions, frame rate, URL/audit facts |

An entry's primary identity is an internal ID with a unique `(volume, relative
path)` constraint. Index filesystem file IDs separately: hard links mean one file
ID may legitimately have multiple paths. Combine a file ID with its volume UUID;
never compare raw inode numbers across drives or persist a transient device number
as the disk identity. Optional creation/generation information can reduce ambiguity
from ID reuse, but is not a content guarantee.

Additional indexes: `(volume, file_id)`, `(size, mtime)`, `(scan_generation)`, and
fields needed for strategies. Size/mtime returns **candidates**, potentially many;
those values change when a file is edited and are not unique even for large videos.
Use integer sizes and timestamps, preserving precision and recording filesystem
timestamp resolution. `mtime` is modification time; `ctime` is metadata-change
time, not creation time. Keep `ctime` for conservative invalidation, not rename
identity, since metadata operations can change it.

### Comparison sequence

1. Validate volume identity and recover unfinished operations.
2. Scan both selected roots. Accumulate observations into a new inventory
   generation and bulk-load lookup tables; avoid durable metadata writes per file.
3. Match current source entries to prior source observations by identity and path.
4. Resolve the recorded destination counterpart and validate its present state.
5. Same identity and content-related metadata, new path: propose a destination
   rename. If source or destination changed, copy or report a conflict instead.
6. New source entry: propose a copy. Changed entry: preserve the previous backup
   and propose a replacement. Confirmed source disappearance: propose archiving.
7. Freeze a plan with preconditions; revalidate them immediately before execution.
8. Commit successfully established replica relationships. Do not mark an entire
   run complete merely because some files finished.

A missing entry is a deletion only after a **complete, error-free scan of its
scope under the same filter policy**. An unreadable directory, disconnected disk,
new exclusion, or incomplete stdin list cannot authorize deletions. V1 blocks the
whole destructive phase when scan coverage is incomplete.

Use file-ID matching to avoid full-content hashing for renames. Metadata-only
change detection cannot detect every same-size edit with a deliberately preserved
mtime. Document this limitation and offer periodic content audits. Ambiguous
identity or an unknown destination must not silently become a confirmed replica.
An optional size/mtime heuristic can produce a reviewable suggestion; keep it off
by default for this safety-focused app.

FreeFileSync documents database-backed move detection using stable file IDs,
available after the initial synchronization. This proposal uses that principle,
not a compatible `sync.ffs_db` format.
[FreeFileSync synchronization settings](https://freefilesync.org/manual.php?topic=synchronization-settings).

### Initial adoption of existing backups

A first scan can inventory Tower and Tower Backup without copying their media.
Matching paths with matching metadata may be enrolled as **metadata-matched,
not content-verified**. Show the distinction and permit a staged audit. Do not
infer verified replica equivalence from a matching filename, size, and timestamp.
Keep existing `.rclone` history excluded and intact; importing its retention
records is a separate migration action.

## 5. Fast scans and realistic performance expectations

Start with complete scans so recovery and comparison can be validated. Complexity
is approximately O(source entries + destination entries), plus actual transfers.
A database makes matching cheap; it does not reveal external filesystem changes
without observation. The earlier 12-second fd/rclone listings were exploratory,
not matched cold-cache benchmarks or evidence of a guaranteed Rust speedup.

Use shallow, bounded parallelism across directories, independently tuned per
physical disk. Investigate macOS bulk attribute enumeration (`getattrlistbulk`)
after establishing a correct portable Rust traversal baseline. Do not turn each
HDD into dozens of competing metadata and thumbnail readers.

A later FSEvents accelerator can rescan changed subtrees. Persist a per-volume
stream identity/cursor, validate continuity after moving between Macs, and fall
back to a full scan on missing history, dropped events, root changes, or uncertain
coverage. A computer-local event number alone is not a portable cursor. Record
scan-start boundaries and reconcile changes occurring during the scan before
advancing the cursor. Events are invalidation hints, never proof of file equality.
[Apple FSEvents guide](https://developer.apple.com/library/archive/documentation/Darwin/Conceptual/FSEvents_ProgGuide/UsingtheFSEventsFramework/UsingtheFSEventsFramework.html).

## 6. Direction and filesystem safety

Enroll a named relationship once: **Tower is the authority; Tower Backup receives
changes**. Resolve roles using volume UUIDs and app enrollment records, not drive
names, mount order, positional CLI arguments, or whichever disk has newer files.
Check source and destination are distinct actual volumes and disallow overlapping
roots. Detect duplicate UUID/enrollment identities caused by clones and stop for
re-enrollment rather than guessing.

Make this structural in Rust: the planner receives a `SourceReader`; only a
validated destination produces a `DestinationWriter`. Tower cannot obtain a media
writer in ordinary sync or fill mode, even via an explicit output path. Restore
is a separately named workflow targeting an explicitly enrolled restore disk;
it does not bypass master protection. There is no generic `--force` that reverses
the relationship.

### Drive-resident roles: master protection follows the disk

Use a small `/.safesync/volume.json` role marker separate from the rebuildable
inventory. Its fields identify the format version, volume UUID, random enrollment
ID, role (`master`, `backup`, or `fill_target`), and, for backups, the enrolled
master they accept updates from. Both ends record the relationship and must agree.
The master role applies to the whole volume, even if the requested target is a
subdirectory. It follows the disk between Macs without a computer-local index.

An enrollment ID is an identifier, not a secret password or cryptographic key.
No secret stored next to writable data is needed to prevent accidental reversal.
If malicious tampering becomes part of the threat model, signatures would require
an independently trusted key and a separate trust design; a self-contained marker
cannot authenticate itself against an attacker able to replace it.

Every write-capable workflow must require positive authorization from a valid
target role marker **and** the matching relationship. A master marker is an
unconditional denial of media writes. Never implement the weaker rule “no master
marker means writable.” Missing, malformed, unknown-version, mismatched or
conflicting role records all mean read-only/stop, not automatic enrollment.
Deleting the marker, rebuilding an index, or supplying a local exported manifest
must not grant write permission. Read-only inventory and lookup can still operate
without role enrollment.

The marker is created only during an explicit enrollment workflow. Normal scans,
index compaction, exports and sync cannot alter it. A catalog writer is authorized
only for its specific metadata paths, not arbitrary files under `/.safesync/`;
in particular it cannot rewrite `volume.json`. The separate lifecycle operation
for retiring or changing a master is not exposed as a sync override and requires
new relationship enrollment. The normal restore command cannot perform it.

Proposed authorization behavior:

| Observed state | Media writes allowed? |
|---|---|
| Valid master | Never, including restore and fill destinations |
| Valid backup linked to the connected enrolled master | Only the authorized one-way backup plan |
| Valid fill target selected by the user | Only the authorized fill plan |
| Missing/corrupt marker or identity mismatch | No; inspect or explicitly enroll |
| Offline manifest alone | No; historical lookup evidence only |

Check roles before staging, replacement, renaming, archiving and pruning, not
only in the TUI. Bind the writer capability to the validated open volume and
marker revision, and invalidate it on remount, role changes or identity loss.
Test swapped arguments, copied markers, deleted markers, master subdirectories,
and disconnected disks. A byte-for-byte clone with duplicated volume and app IDs
requires explicit re-enrollment; the marker is not hardware identity.

This is an application safety boundary. It cannot stop Finder, other programs or
an administrator from modifying the disk. OS read-only mounting or hardware write
protection is stronger, but also prevents on-drive index updates.

Implementation status: this is a required gate for the future write executor.
Version 0.1 implements fixed role records and inspection, but not executor marker
enforcement; it writes only manifests and enrollment metadata and has no media
mutation commands.

On-drive indexes require **metadata writes to Tower**. Be explicit about this
exception: a narrowly scoped catalog writer may create/update inventory files in
`/.safesync/` (but never the role marker), while the
sync executor cannot overwrite, rename, or delete source media. Filesystem repair
is another separate maintenance exception. A physically read-only source cannot
refresh an on-drive catalog; permit a temporary in-memory scan with reduced
capabilities, not a silently divergent persistent local index.

Use directory-relative operations anchored to validated open roots, reject path
traversal, and do not follow destination symlinks outside those roots. Revalidate
attachment identity and operation preconditions after unmount/remount events.
Changing the device at an old mount path must not redirect writes to a different
disk or the startup filesystem.

### First Aid and startup recovery

Treat every start as potentially following an interrupted run, even when a clean
marker exists. First inspect attachment identity and the small enrollment record;
then perform filesystem verification **before opening writable catalogs or media
handles**. Checks that need to unmount a volume must run before drive-held locks
and open catalog handles; take a host-level maintenance lock, then revalidate and acquire
drive locks afterward.

Default: run `diskutil verifyVolume` against both resolved devices in sequence,
including underlying storage checks provided by macOS. Show progress/output and
allow an explicit `--skip-fs-check` or TUI skip before checking. Record the skip.
Verification checks structures; it does not repair them. If it finds errors,
block ordinary writes and offer a separate repair workflow using `repairVolume`,
followed by verification and identity checks. Never force-unmount another app's
open work or continue past a known failure because a skip flag was supplied.

This is a verification-first First Aid workflow, not a promise that an unchecked
or failed repair succeeded. Full device maintenance may also involve containers
and the partition map. Source repair can modify filesystem structures and needs
a distinct user-visible decision. No routine sync process should run as root;
use a narrowly scoped maintenance action when macOS requires authorization.
[Apple's First Aid guidance](https://support.apple.com/en-us/102611); command
semantics also checked against this Mac's `diskutil(8)` manual.

Skipping a filesystem check **does not skip** identity validation, database
recovery, interrupted-operation reconciliation, or free-space checks. First Aid
does not validate video contents, prove drive hardware healthy, or complete a
half-written application transaction. Always inspect journals and reconcile the
actual filesystem against unfinished work. Corrupt catalog: preserve evidence,
rebuild observations, and prohibit destructive actions until safe to proceed.

## 7. Recoverable writes, verification, and history

Catalog publication and filesystem renames are not one atomic transaction. Use a
write-ahead operation journal with idempotent reconciliation of each boundary:

```text
planned → copying → staged-and-verified → old-version-archived
        → installed → catalog-committed
```

Persist intent before changing files. Copy into a uniquely owned temporary file
on the destination filesystem. Reserve space, capture source identity/metadata
from its open handle, and recheck them after copying. If the source changed,
reject the staged result and retain the old destination. Snapshot-backed reads
are a later option; pre/post metadata checks are not a perfect consistency proof
against every concurrent writer.

Default safety policy: calculate BLAKE3 during the source read, flush the staged
file, and read it back to verify before installing it. This hashes newly copied
bytes, **not every video during comparison**. It adds a destination read and must
be included in throughput/ETA estimates. Optional faster size-only verification
must remain clearly marked as weaker evidence and must not create a
content-verified replica record.

Preserve and validate the declared metadata policy before installation. Archive
an existing destination into this run's history, then install the verified file.
Use recoverable intermediate states for the gap between those two renames; do
not describe replacement of an existing file as a single globally atomic action.
Test filesystem durability ordering, directory updates, and macOS full-sync
behavior. Hardware may still lose writes despite successful flush requests.

On restart, inspect the expected temp, archive, and installed files plus their
identities. Finish a proven operation or restore the archived predecessor; stop
on conflicting evidence. Never blindly replay path-based journal commands.
Interrupted copies restart at file boundaries in v1; chunk resume needs persisted
chunk verification and belongs in a later phase. Stage rename cycles and swaps
through unique temporary names; handle case-only renames explicitly.

History is part of the storage budget. Prune only explicit committed history
records, never a directory selected solely by mtime. Pin everything needed by an
unfinished run. Present a separate pruning plan, retain a minimum number of
successful generations, and do not delete the last recoverable version merely
to make a new copy fit. Disk full must still leave room to record recovery state.

## 8. Reading both replicas to fill a smaller disk

Separate two operations:

- **Mirror:** Tower → Tower Backup; source authority is fixed.
- **Fill:** the selected current Tower versions → a third disk, with either
  verified physical replica supplying the bytes. Neither library is a target.

The planner selects one logical video once, regardless of replica count. Use
recorded replica relationships and verification evidence, not size/mtime alone,
to decide whether Tower Backup can supply Tower's current version. Revalidate the
chosen replica before and after reading. If stale or uncertain, use Tower or
verify the alternate; never silently substitute an older backup because it is
available. If Tower is absent, an explicit “fill from last backup generation”
mode can operate, with that older authority clearly labelled.

Start with **different whole files from each drive**, one sequential reader per
physical HDD, bounded buffers, and a scheduler that accounts for shared USB
controllers and target throughput. Target SSDs may benefit from multiple writers;
a target HDD may need a single writer to avoid seek contention. Two source
volumes on the same physical disk do not provide two independent read channels.

The upper bound is approximately:

```text
copy throughput ≤ min(source A + source B, destination write rate,
                      shared bus bandwidth, verification/CPU limits)
```

A near-2× gain is plausible only when one source is the original bottleneck and
the target and bus have spare capacity. Destination rereads for verification also
consume bandwidth. Do not promise twice the speed. Splitting one file into chunks
across replicas is deferred: it introduces seek behavior and requires exact
version equivalence plus an integrity scheme.

## 9. Features and device-specific tweaks to borrow from spill

| Existing behavior | Rust proposal |
|---|---|
| Three 8 MiB buffers, 16 KiB alignment | Start with the same bounded pipeline; benchmark buffer sizes rather than assuming optimality |
| `F_NOCACHE` on media descriptors | Keep configurable and benchmarked; do not bypass cache for tiny catalog reads |
| `F_PREALLOCATE`, contiguous then fragmented allocation | Retain fallback; fail on real allocation errors/ENOSPC, report unsupported behavior, and check actual free space |
| Reader-side hash | Retain for new transfers; hash computation uses CPU even when overlapped with I/O |
| Temp file + flush + rename | Retain, adding journaled replacement, verification before installation, and tested durability ordering |
| Source/destination verification in parallel | Retain for audits on separate physical disks, respecting the device scheduler |
| Smoothed speed, file and capacity bars | Preserve; separately report copying, verification, and durable completion |
| `--modest`, chafa, ffmpeg previews | Preserve optional previews; lazy low-priority generation with timeouts and a bounded cache |
| NUL-separated stdin | Preserve; root-aware relative paths and explicit absolute-path mapping, never silent basename flattening |
| Retry count and free-space reserve | Preserve with failure classification; no blind retry of corruption, permission failure, or wrong-disk errors |

The current Go implementation verifies hashes **after** renaming the temporary
file over the destination, and removes that installed file on verification
failure. Do not carry that ordering into the sync engine: verify the staged file
first and preserve the predecessor. Metadata errors also must not be silently
reported as a fully successful backup.

Preserve strategies as composable **filter → rank → capacity selection** steps:

- `none`: supplied order, streaming when the operation does not need a global plan.
- `latest`: descending mtime, with deterministic tie-breaking.
- `good-quality`: 3★+, 1080p+, 30 fps+.
- `high-quality`: 3★+, 1080p+, 60 fps+.
- `highest-quality`: 4★+, 2160p+, 60 fps+.
- `audit`: black intro or missing faststart; retain the atom inspection and media checks.
- `url-missing`: missing/empty embedded source URL using spill's existing tag policy.

Retain spill's one-fps tolerance so 29.97/59.94 qualify. Cache expensive media
facts by file version plus probe version; derive star ratings from the **current
filename**, since a rename can change the rating without changing video bytes.
Cached thumbnails should survive a rename where identity is known.

Expose two capacity policies: stop at the first non-fitting file, or skip it and
continue filling. Strict newest-first selection and maximum space utilization
are different policies; show which was used and why each file was omitted.
Existing matching files consume no new copy budget. Account for staging,
verification, history, and a safety reserve; never count not-yet-pruned history
as free space. Fill mode does not evict existing target files unless a separately
reviewed replacement policy requests it.

## 10. Rust architecture and command surface

Use one macOS-only package, initially `utils/safesync/`, with a canonical
`setup/install/install-safesync.sh` resolving the repository from its own path.
No new daemon, application bundle, or nested workspace is required. Keep the Go
spill independent and unchanged. The new app may supersede the rclone backup
workflow after validation, but it does not replace spill or take over its command.
The package and installer names are `safesync`.

Suggested modules: `volume`, `catalog`, `scan`, `compare`, `plan`, `journal`,
`copy`, `verify`, `strategy`, `scheduler`, `maintenance`, `events`, and `tui`.
The comparison planner should be pure and independently testable. Filesystem
mutations live behind capability-checked interfaces; the TUI observes structured
events and submits commands, and never infers success from display text.

Candidate libraries: Ratatui/Crossterm for the terminal, `clap` for arguments,
`serde`/`serde_json` for manifests, snapshots and events, BLAKE3 for verification, and
small reviewed macOS bindings for disk identity, bulk attributes, and I/O controls.
Prefer a bounded blocking-worker design for local disk I/O; an async runtime is
optional, not a throughput requirement. Pin versions during implementation and
review the maintenance and licensing of the selected dependencies.
[Ratatui project](https://ratatui.rs/).

Illustrative future commands (the current inventory CLI is documented in
`utils/safesync/README.md`):

```text
safesync enroll                 # choose identities and fixed roles once
safesync mirror tower           # preflight, recovery, scan, preview, apply
safesync mirror tower --plan-only
safesync mirror tower --skip-fs-check
safesync fill travel --from tower --strategy latest
safesync verify tower
safesync recover tower
safesync history tower
```

A saved plan includes pair/profile revision, volume identities, scan generations,
expected entry versions, and sufficient recovery-space requirements. Applying an
old plan always revalidates preconditions. Headless approval authorizes that plan,
not arbitrary changes discovered later. App versions must reject newer unsupported
schemas; migrations require exclusive access and a recoverable catalog snapshot.

## 11. Additional considerations

- **Case sensitivity:** the inspected Tower volume is case-sensitive APFS; Tower
  Backup is case-insensitive APFS. Detect case-folding and Unicode-normalization
  collisions before any writes. Reformatting either drive is outside this proposal.
- **Metadata and special files:** define support for Finder tags, extended
  attributes, resource forks, timestamps, permissions, ACLs, symlinks, hard links,
  sparse files, and empty directories. Never follow links outside selected roots
  by default. Unsupported items produce explicit outcomes, not silent completeness.
- **Protected history and malicious changes:** archival retention helps recover
  accidental deletions; an always-writable attached disk is not an immutable or
  offline backup. Offer protected generations and keep independent backups.
- **Database privacy:** filenames and media facts can be sensitive. Store them
  under appropriate permissions and use encrypted volumes when required; a database
  checksum is not authentication against someone able to alter the disk.
- **Permissions across Macs:** ownership and mount options vary. Diagnose access
  problems instead of recursively changing ownership or running all transfers as root.
- **Read failure vs media damage:** stop ordinary mutation on persistent I/O errors.
  A rescue-copy workflow may be useful later, but must not weaken normal success
  criteria or automatically prune files it could not read.
- **Changing files:** offer quiescence guidance and later source snapshots for
  active downloads or edits. A directory listing is not a point-in-time snapshot.
- **Durability reserve:** allow enough space for the next staged file and journal
  growth even when users request “fill.” Include APFS shared-container capacity
  and snapshot retention in capacity diagnostics.
- **Clock differences:** use journal generations for ordering, not the computers'
  wall clocks. Retention should be conservative after a clock jump and preserve
  generation-count minima.
- **Safe eject:** close catalogs, flush pending work, release handles, then offer
  OS eject. “Run committed” does not prove the next disconnect was clean.

## 12. Delivery and acceptance gates

1. **Specification and read-only prototype:** portable enrollment/catalogs,
   identity checks, complete scans, size/mtime and ID matching, TUI preview.
   Compare generated plans against hand-checked fixtures. No media writes.
2. **Single-source safe executor:** journaled copy/rename/archive, staged
   verification, startup filesystem checks, recovery, explicit pruning, JSON mode.
   Pass failure injection before running against valuable media.
3. **Drive portability:** resume an interrupted job on a second Mac; schema
   compatibility, permissions, stale baseline and missing-catalog handling.
4. **Selection and multi-source features:** stdin mapping, strategy parity, media cache,
   capacity selection, and dual-replica reads to a third disk.
5. **Measured acceleration:** tune scan concurrency, bulk metadata enumeration,
   FSEvents invalidation, and device scheduling after correctness is stable.

Required fault tests: terminate at every journal boundary; simulate unplug,
short write, ENOSPC, source mutation, mount substitution, read errors, corrupted
catalogs, clock jumps, missing peers, rename cycles, hard links, path traversal,
case collisions, and a competing process. Test cancellation while checking,
copying, verifying, and committing. Use disposable disk images for automated
filesystem tests and expendable media for real disconnect/durability tests.

Release invariants: source media is never written by normal sync/fill; a failed
scan never authorizes deletion; a failed replacement retains its predecessor;
only a recovered or fully committed operation is counted as complete; no plan
applies to a different disk; history needed for recovery cannot be pruned.

Benchmark cold and warm unchanged scans, file and folder renames, small updates,
initial copies, strict verification, and third-disk filling. Use identical scopes,
exclusions, integrity settings, disk topology, and cache conditions for rclone,
spill, and the prototype. Report phase times, bytes read/written, metadata calls,
CPU, memory, and interruption recovery time. Target no media-content reads for
unambiguous unchanged/renamed entries, bounded memory independent of total media
bytes, and measured improvements on the actual disks—not a promised multiplier.

## 13. Decisions proposed for the first version

Approve drive-resident flat-file catalogs with destination-owned relationship state;
identity-based rename detection; no automatic size/mtime-only equivalence;
verification-first First Aid on each write session with an explicit skip; strict
verification of newly copied data; recoverable archive-before-replace; and a
single-source executor before dual-replica reads. Defer FSEvents, chunk striping,
network filesystems, and automatic eviction until the recovery model is proven.
