# safesync

An independent Rust tool for portable drive inventories and honest offline file
lookup. Inspired by spill's feedback and copying design; **not a version of spill**.

Version 0.1 implements the proposal's inventory milestone: scan, content hashes,
flat-file manifests, local offline exports, lookup, and historical comparison. It has no media copy,
rename, delete, repair, or sync execution commands yet. The full-screen TUI,
executor-integrated recovery journal and executable synchronization planner are future
milestones in [the proposal](../../docs/SAFESYNC-RUST-PROPOSAL.md).

## Build and install

From the repository root:

```sh
setup/install/install-safesync.sh
```

Installs `~/.local/bin/safesync`. macOS only; no SQLite, daemon, or application
bundle. The shell installer resolves the source relative to itself.

## Create an inventory

```sh
# Fast metadata inventory: filename lookup, but no proof of content equality.
safesync scan /Volumes/Tower --save-local \
  --exclude .Trashes --exclude .TemporaryItems --exclude .Spotlight-V100 \
  --exclude .DocumentRevisions-V100 --exclude .fseventsd --exclude .rclone

# Read every regular file's complete contents to enable offline content lookup.
safesync scan /path/to/media --hash --save-local

# Explicit output location; must be a NEW file in an existing directory.
safesync scan /path/to/media --hash --output /path/to/new-manifest.jsonl
```

Full-content scanning reads all selected bytes, so the first hashed inventory of
a multi-terabyte library takes time. Version 0.1 deliberately does not reuse hashes
from metadata alone. Filesystem checks/repairs are not run for these read-only
media scans; the only writes are the requested manifest files/directories.

The default drive-resident output is
`ROOT/.safesync/manifest-GENERATION.jsonl`. Each successful run publishes a new
immutable file. It does not replace previous generations. `--save-local` also saves
an **offline snapshot** under:

```text
~/Library/Application Support/safesync/manifests/
```

The local snapshot is historical search evidence, not an authoritative sync index.
It can be copied between Macs without the original disk. The manifest stores the
volume UUID/name, original scan root, scan start/end, exclusions, file identities,
sizes, nanosecond timestamps and optional SHA-256 hashes. Schema 1 explicitly tags
the hash algorithm; it uses SHA-256, not the proposal's later BLAKE3 transfer hash.

`--exclude` takes a literal path relative to the scan root and excludes that file
or subtree. Repeat it as needed. It is not a glob or ignore-file expression.
`.safesync` entries, symlinks, special files and other mounted filesystems are always
excluded, with the policy/counts recorded. Hidden regular files are included.
Unreadable included files or directories abort the scan rather than disappearing
from a supposedly complete inventory.

## Save an existing manifest on this Mac

```sh
safesync export /Volumes/Tower/.safesync/manifest-GENERATION.jsonl
safesync manifests
safesync info /path/to/manifest.jsonl
```

Export validates the entire manifest and marks the result as an offline snapshot.
It does not need the original media or open the original drive. Existing output
files are never overwritten. Old snapshots remain searchable until you remove
them intentionally; there is no automatic retention or “latest wins” policy yet.

## Search while drives are disconnected

```sh
# Exact case-sensitive basename presence; makes no content claim.
safesync lookup --name 'Holiday.mov'

# Full-content match regardless of filename. Only the query file is read now.
safesync lookup --file /path/to/downloaded-video.mov

# Restrict to a particular snapshot; otherwise all local snapshots are searched.
safesync lookup --manifest /path/to/manifest.jsonl --file /path/to/video.mov --json
```

Results show the recorded drive, relative path, scan timestamp, generation and
evidence type. They mean **“recorded at scan time,” not “confirmed present now.”**
A name match is not proof of content equality. A content match compares SHA-256
of complete file contents and size. Same-size candidates without stored hashes
remain unknown; safesync never guesses from filename, mtime or size.

If multiple snapshots contain the same file, multiple historical matches are
returned. Use `--manifest` to select the generation you intend. A negative result
is only about the selected manifests and their declared scopes, never every file
on every disk. Corrupt or unreadable selected manifests fail the entire query.

Exit statuses:

| Code | Meaning |
|---|---|
| 0 | Command succeeded / at least one recorded match |
| 1 | No recorded match in selected manifests, with no relevant missing hashes |
| 2 | Invalid arguments, scan/read/integrity error, or no saved manifests |
| 3 | No confirmed content match, but same-size entries lack hashes |

Even with a match, JSON includes the count of unhashed candidates on other
snapshots, so consumers can see that the result list may be incomplete.

## Refresh drive-owned catalogs

```sh
safesync catalog-refresh /Volumes/Source /Volumes/Backup --hash \
  --exclude .Trashes --exclude .Spotlight-V100 --exclude .fseventsd
safesync catalog /Volumes/Source
```

Both drives must already be enrolled with the correct roles. Refresh always runs
filesystem verification before acquiring the pair leases; there is no skip option
for catalog publication yet. It scans both complete volume roots with the same
literal exclusions. Both scans must succeed before publication begins. Omit
`--hash` for metadata-only inventories. This writes safesync metadata on both
drives, including the protected source, and never writes source or destination media.

Each drive owns immutable `.safesync/catalog-GENERATION.jsonl` files and a small
`.safesync/CURRENT` record bound to its enrollment. Publication flushes a staged
inventory, links it into place without overwriting, persists the directory, then
flushes and atomically replaces `CURRENT`. Older generations remain intact.
The current layout keeps generation files directly inside `.safesync`; the
proposal's richer generation directories are not needed for inventory-only state.

`catalog` validates the pointer, enrollment, generation and manifest integrity
under a drive lease. Corrupt or unsafe current metadata blocks refresh instead
of being silently replaced. Offline exports cannot become authoritative catalogs.
Catalog JSONL files still work with `export`, `compare` and `plan`; `CURRENT`
itself is not a manifest.

The two drives commit independently. If source publication succeeds and destination
publication fails, the error identifies that partial result; this is not a committed
sync relationship. An interrupted publication can leave an unreferenced complete
generation or a partial staging file. Neither is selected automatically, and no
history is pruned. An error after replacing `CURRENT` can leave a valid new current
catalog: inspect it before retrying. Tests exercise interruption between generation
publication and pointer replacement; physical power-loss durability remains untested.

## Compare two inventories

```sh
safesync compare source.jsonl destination.jsonl
safesync compare source.jsonl destination.jsonl --json
```

This reads only the two manifests, so both drives can be disconnected. Relative
paths are compared within the selected scan roots. Results distinguish matching
full-content hashes, differing contents, unknown contents, source-only paths and
destination-only paths. Equal sizes, timestamps or inode numbers never prove
content equality. A different size establishes a difference even without hashes.

Each source row also lists alternate destination paths with the same size and
full-content hash (lossless base64 paths in JSON). Multiple candidates stay
ambiguous; these are not rename instructions. Destination-only files are not
deletion candidates without a relationship baseline. Exclusion differences and
same-volume comparisons produce warnings. The JSON includes both complete scan
headers, entry preconditions, and explicit `historical_only` / `executable` flags.

This is **not an executable sync plan**: it does not validate current attachments,
roles, case/Unicode collisions, directory conflicts, metadata preservation, or
space requirements. It compares regular-file contents only. Successful comparison
returns 0 even when differences or unknowns exist; invalid input returns 2.

## Preview initial synchronization proposals

```sh
safesync plan source.jsonl destination.jsonl
safesync plan source.jsonl destination.jsonl --json
```

The pure planner reads historical manifests and proposes keeping matching content,
copying source-only files, replacing differing files with predecessor history,
reviewing unknown/alternate content, and preserving destination-only files. It
never proposes deletion or infers a rename from an alternate hash match. There is
no relationship baseline yet, so this is an initial-adoption preview.

The entire preview is blocked for differing exclusion scopes, same-volume inputs,
omitted symlinks/special files/mounted subtrees, reserved metadata paths, ASCII
case collisions, or file/directory conflicts inferred from file paths. Case checks
include parent directories across both manifests. Non-ASCII paths (including raw
non-UTF-8 names) block planning until destination Unicode/collation checks exist.
This conservative policy also applies to case-sensitive targets for now.

JSON contains the original scan headers, source/destination entries, proposed
actions, blockers and logical transfer/predecessor byte totals. These totals are
not an estimate of required free space. Proposals remain conditional when blockers
exist. Empty directories and current attachments, metadata, filesystem health,
recovery and destination filename rules have not been validated.

Exit 0 means the historical preview has no blockers or content-review items;
exit 3 means it needs review; exit 2 means invalid input or an error. **No exit
status authorizes execution.** Every preview has `historical_only: true` and
`executable: false`; there is no apply command. Drives may remain disconnected.

## Adopt an existing backup relationship

```sh
safesync relationship-adopt /Volumes/Source /Volumes/Backup \
  --exclude .Trashes --exclude .Spotlight-V100 --exclude .fseventsd
safesync plan-relationship /path/to/relationship-GENERATION.json \
  current-source.jsonl current-backup.jsonl
```

Adoption runs filesystem verification, refreshes both catalogs with complete-content
hashes, and records same-path files whose size and SHA-256 match. This reads all
selected bytes on both drives. It writes catalogs on both drives and an immutable
`.safesync/relationship-GENERATION.json` baseline on the destination. It does not
copy, rename or delete media. Unmatched files are counted separately; adoption is
not a claim that the entire backup is synchronized. No matches or namespace/scope
blockers refuse adoption, although the refreshed catalogs can already exist.

The baseline binds both enrollment identities, catalog generations and a fixed
profile revision (1: same-path full-SHA-256 adoption). It embeds both inventories
so later review works offline and preserves hard-link/identity ambiguity. Old
baselines remain intact. There is no automatic latest-baseline selection or profile
migration; pass the intended file explicitly. The current in-memory format has a
256 MiB record limit and duplicates catalog data deliberately for portability.

`plan-relationship` validates the baseline checksum, policy and recorded match set,
then produces a JSON rename review against two selected current manifests. Its
`committed_relationship: true` means an adoption record was supplied, not that a
sync run completed or the current attached drives were validated. The result
remains `historical_only: true` and `executable: false`, with the same review/error
exit statuses as `plan-renames`. The checksum detects damage, not malicious
rewriting. Symlinked or hard-linked baseline files are refused.

If adoption reports a publication error, a valid immutable baseline may already
exist; inspect destination metadata before retrying. Relationship updates after
copy/rename operations and recovery-aware baseline selection remain unfinished.

## Review possible renames across scans

```sh
safesync plan-renames previous-source.jsonl previous-backup.jsonl \
  current-source.jsonl current-backup.jsonl --json
```

This compares caller-selected historical observations, not a committed relationship
baseline. Prior and current snapshots must have matching volume UUIDs, scan-root
file IDs and exclusion policies on each side. Mount paths may differ between Macs.
Only a unique source identity moving to another relative path is considered a
rename candidate. The prior same-path backup must have a matching full-content
hash, and its current identity, size and modification time must remain consistent.

Current matching hashes produce `full_content_at_scan_time` evidence. Missing
current hashes produce `identity_and_metadata_only`, never confirmed content
equality. Contradictory hashes or changed size/mtime prevent a rename candidate.
Device numbers are ignored across mounts; source and destination inode numbers
are never matched to each other. ctime changes alone do not disprove a rename.

Hard-link ambiguity, reused source paths, occupied targets, namespace blockers,
changed counterparts and missing files remain review items. A disappearance never
authorizes deletion or archiving. Metadata continuity cannot exclude inode reuse
or same-size edits with preserved timestamps.

JSON includes the four scan headers, entry evidence, prior namespace blockers and
the unchanged initial-adoption preview. Rename observations are alternatives for
review, not operations to add to that preview. Exit 3 means observations or review
items exist; 2 means invalid input. `executable` and `committed_relationship` are
always false. This command neither stores a destination-owned relationship nor
applies renames.

## Enroll a drive role

```sh
safesync enroll /Volumes/Source --role protected-source
safesync enroll /Volumes/Backup --role destination
safesync enrollment /Volumes/Source
```

Enrollment requires the actual root of a local APFS volume. It creates a private
`.safesync/volume.json` record containing the volume UUID, an enrollment ID, a
revision and a fixed role. Repeating the same role returns the existing record;
changing roles is refused. Inspection validates the record against the mounted
volume. Both commands print JSON. Enrollment only writes tool metadata; it does
not provide OS write protection or enable copying/synchronization.

An unenrolled `.safesync` directory must be empty. In particular, existing 0.1
inventory folders are not automatically adopted: preserve and inspect their
manifests before moving them aside and enrolling. Existing valid enrollment
records can coexist with later inventories. Symlinked namespaces/markers,
hard-linked markers, unknown formats, wrong-volume records and corrupt records
are refused. Enrollment uses an advisory namespace lock and publishes without
overwriting. A publication error may leave a valid marker; inspect it before
retrying. Interrupted temporary files are preserved if the process is killed and
block automatic adoption of an unrecognized namespace.

This is the role-record foundation. Relationship enrollment, attachment
revalidation for each future mutation, and executor capability enforcement remain
unfinished. Identically cloned volume and enrollment
IDs cannot be distinguished by this record alone.

## Check an enrolled drive pair

```sh
safesync check-pair /Volumes/Source /Volumes/Backup
```

This read-only check validates source/destination roles, distinct volume and
enrollment identities, and acquires exclusive advisory leases in volume-UUID
order. It rechecks attachment identity, the opened namespace and the enrollment
record while holding the leases. A busy drive fails immediately; partial leases
are released on failure. JSON output explicitly reports that filesystem checking
has not run and the result is not executable. Locks last only until command exit.

Leases lock the existing `.safesync` directory descriptor, sharing the enrollment
lock, rather than creating the proposal's future lock file. They exclude other
cooperating enrollment/session operations; existing inventory scan commands and
unrelated programs do not participate. The lease API exposes no write capability.
Tests simulate marker, namespace and root replacement; real disconnect testing
on expendable media remains outstanding.

### Optional filesystem verification

```sh
safesync check-pair /Volumes/Source /Volumes/Backup --verify-filesystem
```

This first inspects both enrollments and resolves their device identifiers, then
closes drive inspection handles. It runs macOS `diskutil verifyVolume` on the
source and destination sequentially, before acquiring drive leases. It checks
identity before and after each verification and again after acquiring the pair.
A nonzero verification result or changed identity stops the preflight with exit 2.
There is no automatic repair or force-unmount command.

Verification progress streams to stderr. Successful stdout remains JSON, with
`filesystem_checked: true` and per-volume device IDs and check timestamps. The
result still has `executable: false`: filesystem verification is not content
verification, journal recovery or permission to apply a plan. Without the flag,
the command continues to report `filesystem_checked: false`.

A private per-user lock in `/private/tmp` serializes verification preflights on
this Mac and is retained until the resulting drive leases are released. The lock
file persists after exit; kernel lock ownership, not file existence, determines
whether it is busy. Inventory commands and other applications do not participate.
The workflow is tested with simulated verification failures and identity changes;
real DiskManagement verification and controlled cancellation still need testing
on disposable APFS volumes.

## Prepare an immutable run

```sh
safesync prepare-run /Volumes/Source /Volumes/Backup \
  --exclude .Trashes --exclude .Spotlight-V100 --exclude .fseventsd
safesync run-info /Volumes/Backup/.safesync/run-RUN_ID
```

Preparation verifies both filesystems and refreshes both catalogs with full-content
hashes. It refuses unresolved review items, namespace/scope blockers and an empty
copy/replacement plan. It preserves destination-only files and does not schedule
renames. The destination receives a private, unique `run-RUN_ID` directory with
`plan.json` and `journal`; source writes are limited to the refreshed catalog.

The immutable plan contains both enrollment records, catalog observations and
explicit source/predecessor versions for each copy or history-preserving replacement.
It is flushed before creating the journal, whose first record binds the exact plan
bytes by SHA-256 and fixes the operation-ID set. No operation intent is started.
The current prototype embeds inventories and limits the plan to 256 MiB.

`run-info` checks that the operations agree with the saved inventories and that
the journal identifies this exact plan and operation set. Even whitespace edits to
`plan.json` invalidate the binding. An unfinished prepared run returns exit 3;
missing, corrupt or mismatched metadata returns 2. Inspection never changes files.

Preparation is not execution approval: `execution_enabled` is false, and there is
no apply command. Metadata preservation, free-space reservation, live preconditions
at execution, staged copying and recovery remain unimplemented. Interrupted
preparation leaves its unique directory for inspection; it is neither reused nor
deleted automatically. An incomplete plan without its journal cannot be treated as
a prepared run. A retry creates a separate run and may leave valid earlier metadata.

## Inspect an operation journal

```sh
safesync journal-info /path/to/run.journal
```

The journal foundation supports framed copy-lifecycle events and JSON inspection.
Each run binds an immutable plan digest and a fixed set of operation IDs. Each
stage requires a durable intent before its completion: copying, staged verification,
optional predecessor archival, installation and catalog commit. Run commit requires
all planned operations to finish. This is record validation, not proof that the
corresponding filesystem operations occurred.

Frames contain sequence numbers, bounded payload lengths, a previous-frame digest,
an independently checksummed header and a payload checksum. An incomplete final
frame reports `torn_tail` and the last valid byte offset. A complete frame with bad
checksums, broken ordering or invalid transitions is an error. The reader never
truncates or replays anything. Every unfinished run needs recovery, even with no
torn tail; a committed run with extra partial bytes also needs recovery.

The writer library only creates new journals, flushes each accepted append with
macOS full-sync and refuses further appends after an uncertain write. It holds an
exclusive advisory lock; inspection refuses a busy writer. It does not reopen
existing journals, reconcile files, or execute media operations yet. `prepare-run`
publishes a plan before creating its initial journal; there is no arbitrary
journal-creation CLI.

Inspection exits 0 for a clean recorded commit, 3 for recovery-required state, and
2 for corruption, unsupported data or access errors. These statuses describe the
journal alone. Tests cover every truncation point in a final frame, corrupted
headers/payloads, invalid transitions, lock contention and CLI outcomes. Real
power-loss and filesystem reconciliation tests remain outstanding.

## Safety and format

The format is newline-delimited JSON: one header, regular-file entries, then a
footer with the record count and SHA-256 of every preceding byte. Paths are encoded
losslessly as base64 bytes; display strings are not used as filesystem identities.
Corrupt/truncated files and unsupported schemas/algorithms are rejected. The
checksum detects damage, not a maliciously rewritten manifest.

Publication writes a private temporary file, flushes it, requests macOS
`F_FULLFSYNC`, then creates the final name atomically without replacing any existing
file. Failed/interrupted temporary files are not selected by local-library lookup.
An error after final publication may leave a valid final manifest; inspect it
before retrying with another name. This is snapshot publication, not yet a sync
operation journal or an atomic transaction across two drives.

Traversal uses directory descriptors and refuses symlink traversal and filesystem
crossings. Hashing checks the open file's identity, size, modification time and
change time before/after reading. A final metadata pass catches files/directories
changed during scanning. This costs metadata I/O, and still is not a filesystem
snapshot or a lock against arbitrary concurrent writers. Scan a quiet tree.

The prototype loads inventories into memory. Large-scale memory/scan performance,
FSEvents acceleration and hash-cache invalidation are future work; this release
makes no speed claim over rclone or fd. It does not read video contents unless
`--hash` or content lookup was requested.

## Verification

```sh
cargo test --manifest-path utils/safesync/Cargo.toml --locked
cargo clippy --manifest-path utils/safesync/Cargo.toml --locked --all-targets -- -D warnings
```

Tests use disposable temporary directories and synthetic volume identities. They
cover renamed-content lookup, same-size mismatches, unknown metadata-only results,
offline exports, corruption/truncation, no-overwrite publication, symlinks, raw path
encoding, concurrent source changes, and CLI exit statuses. They do not scan,
repair, synchronize or change the real Tower disks.
