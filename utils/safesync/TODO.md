# Safesync TODO

Implementation checklist for the [design proposal](../../docs/SAFESYNC-RUST-PROPOSAL.md).
See [README.md](README.md) for the currently available commands.
Work through the milestones in order; media writes must wait for the safety gates.

## Completed foundation

- [x] macOS Rust package and canonical installer.
- [x] Regular-file inventory with volume identity, literal exclusions and optional full-content SHA-256 hashes.
- [x] Immutable JSONL manifests with integrity footers and no-overwrite publication.
- [x] Local offline exports, manifest inspection and historical filename/content lookup.
- [x] Read-only historical comparison with content matches, differences, unknowns and alternate matching paths.
- [x] Reject unsafe/duplicate manifest paths and preserve ambiguous content candidates.
- [x] Tests for inventory integrity, source changes, offline lookup and comparison.

## 1. Enrollment and read-only planning

- [x] Add portable drive enrollment records and fixed protected-source/writable-destination roles.
- [x] Add enrollment inspection, volume-root/local-APFS checks, no-overwrite publication and competing-enrollment exclusion.
- [x] Add read-only `check-pair`, ordered exclusive namespace leases, role/disk checks, attachment/namespace/marker revalidation and partial-lock cleanup.
- [x] Add pure initial-adoption `plan` previews with conditional copy/history proposals, content-review items, destination preservation, logical byte totals and JSON output.
- [x] Test initial-adoption proposals against hand-checked fixtures, including offline CLI outcomes, ambiguous matches, overflow and deterministic ordering.
- [x] Block initial previews on scope mismatch, skipped entry types, reserved paths, ASCII case/file-directory conflicts and non-ASCII paths requiring further validation.
- [ ] Validate volume UUID, enrollment identity and marker revision; reject copied, missing or mismatched markers for writes.
- [ ] Anchor catalog and future media operations to validated open roots; handle disconnects and mount substitution.
- [ ] Acquire drive-resident advisory locks in stable volume order; reject network storage for writable operations.
  The read-only lease foundation exists; integrate it with future catalog/executor sessions and filesystem-check ordering before completing this gate.
- [ ] Add immutable drive-owned catalog generations and durable `CURRENT` publication, separate from offline exports.
- [ ] Store destination-owned relationship baselines bound to source/destination generations and profile revisions.
- [ ] Build a pure planner using prior observations and volume-scoped file identities; handle hard links and ambiguous identity reuse.
- [ ] Distinguish metadata-based observations from verified content equality; never infer equivalence from size/mtime alone.
- [ ] Require complete scans with compatible scopes before proposing disappearance/archive operations.
- [ ] Detect case-folding, Unicode-normalization and file/directory path collisions.
  ASCII prefix and file/directory checks exist; Unicode and actual destination filesystem rules remain pending.
- [ ] Define support and explicit outcomes for empty directories, symlinks, hard links, sparse files, timestamps, permissions, ACLs, extended attributes, Finder tags and resource forks.
- [ ] Estimate staging, history, verification and journal space, including APFS shared-container capacity and a reserve.
- [ ] Freeze reviewable plans with entry preconditions; revalidate before application.
- [ ] Compare generated plans with hand-checked fixtures before implementing media mutations.

## 2. Preview and terminal interface

- [ ] Add identify → check → recover → scan → review → transfer → verify → commit → summary phases.
- [ ] Keep drive identities, roles and direction visible throughout the workflow.
- [ ] Add full-screen plan review using spill's visual language while keeping spill independent.
- [ ] Show discovered files during scans and separate copied, verified and committed progress.
- [ ] Show per-drive rates, current file, rename savings, history usage, remaining space and verification-aware ETA.
- [ ] Support small terminals, resize, keyboard navigation, monochrome/256-color output and reduced animation.
- [ ] Escape filename control characters and bound rendering independently of I/O.
- [ ] Add structured events and documented headless outcomes.
- [ ] Add pause-after-current-file and cancellation that wait for workers and durable journal state.

## 3. Single-source safe executor

- [ ] Add filesystem verification before writable handles/drive locks, with a recorded explicit skip option.
- [ ] Block writes after known filesystem-check failures; provide a separate authorized repair workflow and revalidate afterward.
- [ ] Add immutable run plans and framed, checksummed append-only journals with intent/completion records.
- [ ] Distinguish a torn final journal record from corruption; block uncertain execution pending reconciliation.
- [ ] Implement a capability-checked destination writer that cannot mutate source media.
- [ ] Copy into owned destination staging files with bounded buffers, allocation checks and source pre/post validation.
- [ ] Hash new transfers with BLAKE3 and reread flushed staged data before installation; keep SHA-256 manifest semantics explicit.
- [ ] Preserve and validate the declared metadata policy before installation.
- [ ] Journal archive-before-replace operations so failure retains a recoverable predecessor.
- [ ] Handle rename cycles, swaps and case-only renames through owned staging paths.
- [ ] Commit replica relationships only after proven completion; report incomplete operations honestly.
- [ ] Add explicit history-pruning plans that pin unfinished-run recovery data and retain minimum successful generations.
- [ ] Close handles, flush and release locks before offering safe eject.

## 4. Recovery and drive portability

- [ ] Reconcile unfinished runs on every startup using actual staged, archived and installed file identities.
- [ ] Make recovery idempotent; stop on conflicting evidence instead of blindly replaying paths.
- [ ] Restart interrupted copies at file boundaries.
- [ ] Preserve corrupt catalog evidence, rebuild observations and disable destructive reconciliation until the relationship is re-established.
- [ ] Reject unsupported schemas and implement recoverable migrations under exclusive access.
- [ ] Verify interrupted-job continuation on a second Mac, including permissions, missing peers and stale baselines.
- [ ] Use generations rather than wall-clock timestamps for ordering and conservative retention.

## 5. Selection and multi-source fill

- [ ] Add root-aware NUL-separated stdin selection without basename flattening.
- [ ] Implement filter → rank → capacity selection for supplied order, latest, quality, audit and missing-URL strategies.
- [ ] Preserve 29.97/59.94 fps tolerance and derive star ratings from current filenames.
- [ ] Cache media probes by file version and probe version; add optional low-priority thumbnail previews with timeouts.
- [ ] Offer explicit stop-at-first-non-fitting versus skip-and-continue capacity policies.
- [ ] Fill a third disk from verified replicas without modifying either source library.
- [ ] Revalidate alternate replicas; expose an explicit last-backup-generation mode when the authoritative source is absent.
- [ ] Schedule whole files by physical disk and shared-bus limits, including destination verification reads.

## 6. Failure testing and release gates

- [ ] Inject termination at every journal boundary and verify recovery/predecessor preservation.
- [ ] Cover unplug, mount substitution, short writes, ENOSPC, read failures and source mutation.
- [ ] Cover corrupted catalogs/journals, clock jumps, missing peers and competing processes.
- [ ] Cover swapped roles, copied/deleted markers, source subdirectories and disconnected disks.
- [ ] Cover rename cycles, hard links, path traversal and case/Unicode collisions.
- [ ] Test cancellation during checks, copying, verification and commit.
- [ ] Validate macOS/APFS flush and directory-publication ordering using disposable disk images.
- [ ] Use expendable media for real disconnect/durability tests before valuable media.
- [ ] Assert release invariants: source media remains untouched; failed scans never authorize deletion; failed replacements retain predecessors; only recovered/committed operations count as complete; plans never apply to another disk; recovery history cannot be pruned.

## 7. Measured acceleration — after correctness

- [ ] Benchmark cold/warm scans, renames, small updates, initial copies, strict verification and third-disk fill.
- [ ] Compare with rclone/spill under identical scope, integrity, topology and cache conditions.
- [ ] Report phase times, I/O bytes, metadata calls, CPU, memory and recovery time.
- [ ] Set and measure an inventory memory budget; consider partitioned snapshots only if needed.
- [ ] Benchmark scan concurrency, bulk metadata calls, buffer sizes, `F_NOCACHE` and preallocation.
- [ ] Add conservative hash-cache invalidation and later FSEvents acceleration with full-scan fallback.
- [ ] Defer chunk resume/striping, network writes and automatic eviction until the recovery model is proven.
