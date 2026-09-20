//! Read-only live preconditions for a prepared, unstarted run. No write capability.
use crate::{
    capacity::{self, Budget, Space},
    enrollment::{DriveLease, PairLease},
    filesystem::{self, Stamp},
    manifest::Entry,
    run,
};
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::{
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

#[derive(Debug, Serialize)]
pub struct Check {
    pub operation_id: String,
    pub side: String,
    pub path_base64: String,
    pub valid: bool,
    pub detail: String,
}
#[derive(Debug, Serialize)]
pub struct Report {
    pub run_id: String,
    pub plan_sha256: String,
    pub live_preconditions_valid: bool,
    pub file_preconditions_valid: bool,
    pub space_before: Space,
    pub space_after: Space,
    pub capacity: Budget,
    pub execution_enabled: bool,
    pub checks: Vec<Check>,
    pub limitations: Vec<String>,
}
impl Report {
    pub fn exit_code(&self) -> i32 {
        if self.live_preconditions_valid { 0 } else { 3 }
    }
}
fn planned_version(actual: &Stamp, expected: &Stamp) -> bool {
    // Device numbers can change after remount. The enclosing lease validates UUID.
    actual.file_id == expected.file_id
        && actual.size == expected.size
        && actual.mtime_seconds == expected.mtime_seconds
        && actual.mtime_nanos == expected.mtime_nanos
        && actual.ctime_seconds == expected.ctime_seconds
        && actual.ctime_nanos == expected.ctime_nanos
}
fn check_file(lease: &DriveLease, path: &Path, expected: Option<&Entry>) -> Result<Option<Stamp>> {
    let actual = lease.media_file(path)?;
    match (actual, expected) {
        (None, None) => Ok(None),
        (Some(_), None) => anyhow::bail!("Copy target is occupied"),
        (None, Some(_)) => anyhow::bail!("Expected file is missing"),
        (Some(mut file), Some(expected)) => {
            let metadata = file.metadata()?;
            ensure!(metadata.is_file(), "Expected a regular file");
            let stamp = Stamp::of(&metadata);
            ensure!(
                planned_version(&stamp, &expected.stamp),
                "File identity or metadata changed since preparation"
            );
            let hash = filesystem::hash_file(&mut file, &stamp)?;
            ensure!(
                Some(&hash) == expected.sha256.as_ref(),
                "File contents differ from the prepared fingerprint"
            );
            Ok(Some(stamp))
        }
    }
}

pub(crate) fn check(pair: &PairLease, run_id: &str) -> Result<Report> {
    check_with_reserve(pair, run_id, capacity::DEFAULT_RESERVE_BYTES)
}
pub(crate) fn check_with_reserve(
    pair: &PairLease,
    run_id: &str,
    reserve_bytes: u64,
) -> Result<Report> {
    check_with_budget_hook(pair, run_id, reserve_bytes, || {})
}
#[cfg(test)]
fn check_with_hook(pair: &PairLease, run_id: &str, after_hashing: impl FnOnce()) -> Result<Report> {
    check_with_budget_hook(pair, run_id, 0, after_hashing)
}
fn check_with_budget_hook(
    pair: &PairLease,
    run_id: &str,
    reserve_bytes: u64,
    after_hashing: impl FnOnce(),
) -> Result<Report> {
    ensure!(
        !run_id.is_empty()
            && run_id.len() <= 100
            && run_id.bytes().all(|b| b.is_ascii_digit() || b == b'-'),
        "Invalid run ID"
    );
    pair.revalidate()?;
    let namespace = pair.destination.catalog_directory()?;
    let name = format!("run-{run_id}");
    let directory =
        filesystem::open_relative(&namespace, Path::new(&name), namespace.metadata()?.dev())?;
    let inspected = run::inspect_directory(&directory)?;
    let plan = &inspected.plan;
    let space_before = capacity::space(&namespace)?;
    capacity::budget(&plan.operations, space_before, reserve_bytes)?;
    ensure!(plan.run_id == run_id, "Run directory identity mismatch");
    ensure!(
        pair.source.enrollment() == &plan.source_enrollment
            && pair.destination.enrollment() == &plan.destination_enrollment,
        "Prepared enrollment/direction no longer matches attached pair"
    );
    ensure!(
        pair.source.root_file_id()? == plan.source.header.root_file_id
            && pair.destination.root_file_id()? == plan.destination.header.root_file_id,
        "Prepared scan root identity changed"
    );
    ensure!(
        !inspected.journal.torn_tail
            && inspected.journal.records == 1
            && !inspected.journal.state.committed,
        "Run has started or journal is incomplete; recovery is required before live preflight"
    );
    let mut checks = Vec::new();
    let mut observed: Vec<(usize, &DriveLease, PathBuf, Option<Stamp>)> = Vec::new();
    for op in &plan.operations {
        let path = op.source.path()?;
        for (side, lease, expected) in [
            ("source", &pair.source, Some(&op.source)),
            ("destination", &pair.destination, op.predecessor.as_ref()),
        ] {
            eprintln!("Checking {side} file {:?}", path);
            let result = check_file(lease, &path, expected);
            let valid = result.is_ok();
            let detail = match &result {
                Ok(Some(_)) => "Identity, metadata and full-content fingerprint match".into(),
                Ok(None) => "Copy target is absent".into(),
                Err(error) => format!("{error:#}"),
            };
            if let Ok(stamp) = result {
                observed.push((checks.len(), lease, path.clone(), stamp));
            }
            checks.push(Check {
                operation_id: op.id.clone(),
                side: side.into(),
                path_base64: op.source.path_base64.clone(),
                valid,
                detail,
            });
        }
    }
    after_hashing();
    // Reopen paths to catch replacements after their content checks. This is still
    // an observation interval, not a filesystem snapshot or protection from writers.
    for (index, lease, path, before) in observed {
        let result = (|| -> Result<()> {
            let after = lease
                .media_file(&path)?
                .map(|f| f.metadata().map(|m| Stamp::of(&m)))
                .transpose()?;
            ensure!(after == before, "Path changed during preflight");
            Ok(())
        })();
        if let Err(error) = result {
            checks[index].valid = false;
            checks[index].detail = format!("{error:#}");
        }
    }
    pair.revalidate()?;
    let current_namespace = pair.destination.catalog_directory()?;
    let current_directory = filesystem::open_relative(
        &current_namespace,
        Path::new(&name),
        current_namespace.metadata()?.dev(),
    )?;
    ensure!(
        current_directory.metadata()?.ino() == directory.metadata()?.ino()
            && current_directory.metadata()?.dev() == directory.metadata()?.dev(),
        "Run directory changed during preflight"
    );
    let current = run::inspect_directory(&current_directory)
        .context("Run metadata changed during preflight")?;
    ensure!(
        current.plan_sha256 == inspected.plan_sha256
            && current.journal.records == 1
            && !current.journal.torn_tail,
        "Plan or journal changed during preflight"
    );
    let space_after = capacity::space(&current_namespace)?;
    ensure!(
        space_before.allocation_unit_bytes == space_after.allocation_unit_bytes,
        "Filesystem allocation unit changed during preflight"
    );
    let conservative_space = Space {
        available_bytes: space_before
            .available_bytes
            .min(space_after.available_bytes),
        read_only: space_before.read_only || space_after.read_only,
        ..space_after
    };
    let capacity = capacity::budget(&plan.operations, conservative_space, reserve_bytes)?;
    let file_preconditions_valid = checks.iter().all(|c| c.valid);
    Ok(Report { run_id: run_id.into(), plan_sha256: inspected.plan_sha256,
        live_preconditions_valid: file_preconditions_valid && capacity.sufficient, file_preconditions_valid,
        space_before, space_after, capacity, execution_enabled: false, checks,
        limitations: vec![
            "Read-only observations of planned files, not a filesystem snapshot or an execution authorization.".into(),
            "Unchanged/unplanned files, metadata-preservation policy and free-space reservation are not checked.".into(),
            "The executor must revalidate immediately before each mutation; concurrent writers remain possible.".into(),
            "Capacity uses the lower before/after available-space observation. APFS shared-container usage, quotas and snapshots can change; purgeable space is not added.".into(),
            "Incoming data is rounded per file; the reserve covers unmeasured metadata/journal overhead. This does not reserve blocks or promise allocation success.".into(),
        ],
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        catalog,
        enrollment::DriveRole,
        filesystem::Volume,
        journal::{Event, Journal, Stage},
        manifest, scan,
    };
    struct Fixture {
        root: PathBuf,
        pair: PairLease,
        run: run::Prepared,
    }
    impl Fixture {
        fn new() -> Self {
            let root =
                std::env::temp_dir().join(format!("safesync-preflight-{}", manifest::generation()));
            std::fs::create_dir(&root).unwrap();
            let root = root.canonicalize().unwrap();
            for name in ["source", "destination"] {
                std::fs::create_dir(root.join(name)).unwrap();
            }
            std::fs::create_dir(root.join("source/nested")).unwrap();
            std::fs::write(root.join("source/nested/new"), b"new").unwrap();
            std::fs::write(root.join("source/replace"), b"new version").unwrap();
            std::fs::write(root.join("destination/replace"), b"old version").unwrap();
            let pair = PairLease {
                source: DriveLease::test_role_lease(
                    &root.join("source"),
                    "source",
                    DriveRole::ProtectedSource,
                )
                .unwrap(),
                destination: DriveLease::test_role_lease(
                    &root.join("destination"),
                    "destination",
                    DriveRole::Destination,
                )
                .unwrap(),
            };
            for lease in [&pair.source, &pair.destination] {
                let inventory = scan::scan(
                    lease.scan_root(),
                    Volume {
                        uuid: lease.enrollment().volume_uuid.clone(),
                        name: "fixture".into(),
                        filesystem: "apfs".into(),
                    },
                    true,
                    |_| {},
                )
                .unwrap();
                catalog::publish(lease, &inventory).unwrap();
            }
            let run = run::prepare(&pair).unwrap();
            Self { root, pair, run }
        }
        fn check(&self) -> Report {
            check_with_reserve(&self.pair, &self.run.run_id, 0).unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
    #[test]
    fn validates_prepared_files_and_missing_parent_without_writes() {
        let f = Fixture::new();
        let journal_before = std::fs::read(f.run.directory.join("journal")).unwrap();
        let report = f.check();
        assert_eq!(report.exit_code(), 0);
        assert_eq!(report.checks.len(), 4);
        assert!(!report.execution_enabled);
        assert!(!f.root.join("destination/nested").exists());
        assert_eq!(
            std::fs::read(f.root.join("destination/replace")).unwrap(),
            b"old version"
        );
        assert_eq!(
            std::fs::read(f.run.directory.join("journal")).unwrap(),
            journal_before
        );
    }

    #[test]
    fn insufficient_capacity_blocks_otherwise_valid_file_preconditions() {
        let f = Fixture::new();
        let namespace = f.pair.destination.catalog_directory().unwrap();
        let reserve = capacity::space(&namespace).unwrap().total_bytes;
        let report = check_with_reserve(&f.pair, &f.run.run_id, reserve).unwrap();
        assert!(report.file_preconditions_valid);
        assert!(!report.live_preconditions_valid);
        assert_eq!(report.exit_code(), 3);
        assert!(!report.capacity.sufficient);
        assert!(report.capacity.shortfall_bytes > 0);
        assert!(!report.capacity.allocation_reserved);
        assert_eq!(report.capacity.reserve_bytes, reserve);
        assert!(report.capacity.available_bytes <= report.space_before.available_bytes);
        assert!(report.capacity.available_bytes <= report.space_after.available_bytes);
    }
    #[test]
    fn rejects_changed_source_predecessor_and_occupied_copy_target() {
        for side in [
            "source/replace",
            "destination/replace",
            "destination/nested/new",
        ] {
            let f = Fixture::new();
            let target = f.root.join(side);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::write(target, b"changed").unwrap();
            let report = f.check();
            assert_eq!(report.exit_code(), 3, "{side}");
            assert!(report.checks.iter().any(|c| !c.valid));
        }
    }
    #[test]
    fn unsafe_or_non_directory_parent_never_counts_as_absence() {
        for symlink in [true, false] {
            let f = Fixture::new();
            let parent = f.root.join("destination/nested");
            if symlink {
                std::os::unix::fs::symlink(f.root.join("does-not-exist"), parent).unwrap();
            } else {
                std::fs::write(parent, b"not a directory").unwrap();
            }
            assert_eq!(f.check().exit_code(), 3);
        }
    }
    #[test]
    fn final_pass_detects_changes_after_hashing() {
        let f = Fixture::new();
        let report = check_with_hook(&f.pair, &f.run.run_id, || {
            std::fs::write(f.root.join("source/replace"), b"changed after hash").unwrap();
            std::fs::create_dir(f.root.join("destination/nested")).unwrap();
            std::fs::write(f.root.join("destination/nested/new"), b"arrived").unwrap();
        })
        .unwrap();
        assert_eq!(report.exit_code(), 3);
        assert_eq!(report.checks.iter().filter(|c| !c.valid).count(), 2);
    }
    #[test]
    fn checks_hash_even_when_identity_and_metadata_match() {
        let f = Fixture::new();
        let inspected = run::inspect(&f.run.directory).unwrap();
        let mut expected = inspected.plan.operations[0].source.clone();
        expected.sha256 = Some("0".repeat(64));
        let error =
            check_file(&f.pair.source, &expected.path().unwrap(), Some(&expected)).unwrap_err();
        assert!(error.to_string().contains("contents differ"));
    }
    #[test]
    fn rejects_started_torn_and_unowned_runs() {
        let f = Fixture::new();
        assert!(check(&f.pair, "../escape").is_err());
        let inspected = run::inspect(&f.run.directory).unwrap();
        let path = f.run.directory.join("journal");
        std::fs::remove_file(&path).unwrap();
        let mut journal = Journal::create(
            &path,
            Event::Start {
                run_id: f.run.run_id.clone(),
                plan_sha256: f.run.plan_sha256.clone(),
                operation_ids: inspected
                    .plan
                    .operations
                    .iter()
                    .map(|op| op.id.clone())
                    .collect(),
            },
        )
        .unwrap();
        journal
            .append(Event::Intent {
                operation_id: inspected.plan.operations[0].id.clone(),
                stage: Stage::Copying,
            })
            .unwrap();
        drop(journal);
        assert!(check(&f.pair, &f.run.run_id).is_err());
        use std::io::Write;
        std::fs::OpenOptions::new()
            .append(true)
            .open(path)
            .unwrap()
            .write_all(b"partial")
            .unwrap();
        assert!(check(&f.pair, &f.run.run_id).is_err());
    }
}
