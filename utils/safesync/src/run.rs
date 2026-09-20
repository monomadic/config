//! Durable preparation only: immutable plan + bound journal, no media executor.
use crate::{
    catalog,
    enrollment::{DriveRole, Enrollment, PairLease},
    filesystem,
    journal::{self, Event, Journal},
    manifest::{self, Entry, Manifest, Role},
    plan::{self, Action},
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    ffi::CString,
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
};
const MAX_PLAN: u64 = 256 * 1024 * 1024;
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Copy,
    ReplaceWithHistory,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Operation {
    pub id: String,
    pub kind: Kind,
    pub source: Entry,
    pub predecessor: Option<Entry>,
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenPlan {
    pub schema: u32,
    pub run_id: String,
    pub execution_enabled: bool,
    pub source_enrollment: Enrollment,
    pub destination_enrollment: Enrollment,
    pub source: Manifest,
    pub destination: Manifest,
    pub operations: Vec<Operation>,
}
fn operations(source: &Manifest, destination: &Manifest) -> Result<Vec<Operation>> {
    let preview = plan::preview(source, destination)?;
    ensure!(
        preview.exit_code() == 0,
        "Plan has blockers or unresolved content-review items"
    );
    let mut result = Vec::new();
    for item in preview.items {
        let kind = match item.proposed_action {
            Action::Copy => Kind::Copy,
            Action::ReplaceWithHistory => Kind::ReplaceWithHistory,
            Action::KeepContent | Action::PreserveDestination => continue,
            _ => anyhow::bail!("Unresolved review item"),
        };
        result.push(Operation {
            id: format!("op-{:08}", result.len() + 1),
            kind,
            source: item.source.context("Missing source precondition")?,
            predecessor: item.destination,
        });
    }
    Ok(result)
}
impl FrozenPlan {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.schema == 1 && !self.execution_enabled,
            "Unsupported or executable run plan"
        );
        ensure!(
            !self.run_id.is_empty()
                && self.run_id.len() <= 100
                && self.run_id.bytes().all(|b| b.is_ascii_digit() || b == b'-'),
            "Unsafe run ID"
        );
        self.source_enrollment
            .validate(&self.source.header.volume.uuid)?;
        self.destination_enrollment
            .validate(&self.destination.header.volume.uuid)?;
        ensure!(
            self.source_enrollment.role == DriveRole::ProtectedSource
                && self.destination_enrollment.role == DriveRole::Destination
                && self.source_enrollment.enrollment_id
                    != self.destination_enrollment.enrollment_id,
            "Invalid run direction/enrollment"
        );
        ensure!(
            matches!(self.source.header.role, Role::Inventory)
                && matches!(self.destination.header.role, Role::Inventory)
                && self.source.header.content_hashed
                && self.destination.header.content_hashed,
            "Run preparation requires hashed drive inventories"
        );
        ensure!(
            self.operations == operations(&self.source, &self.destination)?,
            "Run operations do not match recorded inventory preconditions"
        );
        ensure!(
            !self.operations.is_empty(),
            "No copy/replacement operations to prepare"
        );
        Ok(())
    }
}
#[derive(Debug, Serialize)]
pub struct Prepared {
    pub directory: PathBuf,
    pub run_id: String,
    pub plan_sha256: String,
    pub operations: usize,
    pub media_changed: bool,
    pub execution_enabled: bool,
}
#[derive(Debug, Serialize)]
pub struct Inspection {
    pub plan: FrozenPlan,
    pub plan_sha256: String,
    pub journal: journal::Inspection,
    pub execution_enabled: bool,
}
fn open_regular(directory: &File, name: &str) -> Result<File> {
    let file = filesystem::open_relative(directory, Path::new(name), directory.metadata()?.dev())?;
    ensure!(
        file.metadata()?.is_file() && file.metadata()?.nlink() == 1,
        "Unsafe run metadata file"
    );
    Ok(file)
}
pub fn inspect(path: &Path) -> Result<Inspection> {
    let directory = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    let mut bytes = Vec::new();
    open_regular(&directory, "plan.json")?
        .take(MAX_PLAN + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= MAX_PLAN,
        "Run plan exceeds prototype size limit"
    );
    let plan: FrozenPlan =
        serde_json::from_slice(&bytes).context("Incomplete or invalid run plan")?;
    plan.validate()?;
    let plan_sha256 = filesystem::hex(&Sha256::digest(&bytes));
    let journal = journal::inspect_file(open_regular(&directory, "journal")?)?;
    ensure!(
        journal.state.run_id.as_ref() == Some(&plan.run_id)
            && journal.state.plan_sha256.as_ref() == Some(&plan_sha256),
        "Journal is not bound to this exact plan"
    );
    ensure!(
        journal
            .state
            .operations
            .keys()
            .eq(plan.operations.iter().map(|op| &op.id)),
        "Journal operation set differs from the immutable plan"
    );
    Ok(Inspection {
        plan,
        plan_sha256,
        journal,
        execution_enabled: false,
    })
}

pub(crate) fn prepare(pair: &PairLease) -> Result<Prepared> {
    prepare_with_hook(pair, || Ok(()))
}
fn prepare_with_hook(
    pair: &PairLease,
    after_plan: impl FnOnce() -> Result<()>,
) -> Result<Prepared> {
    pair.revalidate()?;
    let (_, source) = catalog::load(&pair.source)?.context("Missing source catalog")?;
    let (_, destination) =
        catalog::load(&pair.destination)?.context("Missing destination catalog")?;
    let plan = FrozenPlan {
        schema: 1,
        run_id: manifest::generation(),
        execution_enabled: false,
        source_enrollment: pair.source.enrollment().clone(),
        destination_enrollment: pair.destination.enrollment().clone(),
        operations: operations(&source, &destination)?,
        source,
        destination,
    };
    plan.validate()?;
    let bytes = serde_json::to_vec(&plan)?;
    ensure!(
        bytes.len() as u64 <= MAX_PLAN,
        "Run plan exceeds prototype size limit"
    );
    let plan_sha256 = filesystem::hex(&Sha256::digest(&bytes));
    let namespace = pair.destination.catalog_directory()?;
    let name = format!("run-{}", plan.run_id);
    let cname = CString::new(name.as_str())?;
    ensure!(
        unsafe { libc::mkdirat(namespace.as_raw_fd(), cname.as_ptr(), 0o700) } == 0,
        "Cannot create unique run directory: {}",
        std::io::Error::last_os_error()
    );
    namespace.sync_all()?;
    let directory =
        filesystem::open_relative(&namespace, Path::new(&name), namespace.metadata()?.dev())?;
    ensure!(directory.metadata()?.is_dir(), "Unsafe run directory");
    pair.revalidate()?;
    let fd = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            c"plan.json".as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0o600,
        )
    };
    ensure!(
        fd >= 0,
        "Cannot create immutable plan: {}",
        std::io::Error::last_os_error()
    );
    let mut file = unsafe { File::from_raw_fd(fd) };
    file.write_all(&bytes)?;
    manifest::full_sync(&file)?;
    directory.sync_all()?;
    manifest::full_sync(&file)?;
    after_plan()?;
    pair.revalidate()?;
    let _journal = Journal::create_in(
        &directory,
        Event::Start {
            run_id: plan.run_id.clone(),
            plan_sha256: plan_sha256.clone(),
            operation_ids: plan.operations.iter().map(|op| op.id.clone()).collect(),
        },
    )?;
    pair.revalidate()?;
    Ok(Prepared {
        directory: pair.destination.scan_root().join(".safesync").join(name),
        run_id: plan.run_id,
        plan_sha256,
        operations: plan.operations.len(),
        media_changed: false,
        execution_enabled: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{enrollment::DriveLease, filesystem::Volume, scan};
    struct Fixture {
        root: PathBuf,
        pair: PairLease,
    }
    impl Fixture {
        fn new() -> Self {
            let root =
                std::env::temp_dir().join(format!("safesync-run-{}", manifest::generation()));
            std::fs::create_dir(&root).unwrap();
            let root = root.canonicalize().unwrap();
            for name in ["source", "destination"] {
                std::fs::create_dir(root.join(name)).unwrap();
            }
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
            std::fs::write(root.join("source/new"), b"new").unwrap();
            std::fs::write(root.join("source/replace"), b"current").unwrap();
            std::fs::write(root.join("destination/replace"), b"old").unwrap();
            std::fs::write(root.join("destination/retain"), b"retain").unwrap();
            Self { root, pair }
        }
        fn refresh(&self, hash: bool) {
            for lease in [&self.pair.source, &self.pair.destination] {
                let inventory = scan::scan(
                    lease.scan_root(),
                    Volume {
                        uuid: lease.enrollment().volume_uuid.clone(),
                        name: "fixture".into(),
                        filesystem: "apfs".into(),
                    },
                    hash,
                    |_| {},
                )
                .unwrap();
                catalog::publish(lease, &inventory).unwrap();
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
    #[test]
    fn prepares_exact_plan_and_start_only_journal_without_media_changes() {
        let f = Fixture::new();
        f.refresh(true);
        let prepared = prepare(&f.pair).unwrap();
        assert_eq!(prepared.operations, 2);
        assert!(!prepared.execution_enabled && !prepared.media_changed);
        let report = inspect(&prepared.directory).unwrap();
        assert_eq!(report.plan_sha256, prepared.plan_sha256);
        assert_eq!(report.journal.records, 1);
        assert!(report.journal.recovery_required);
        assert_eq!(report.plan.operations[0].kind, Kind::Copy);
        assert_eq!(report.plan.operations[1].kind, Kind::ReplaceWithHistory);
        assert!(report.plan.operations[1].predecessor.is_some());
        assert!(
            report
                .journal
                .state
                .operations
                .values()
                .all(|op| op.pending.is_none() && op.completed.is_none())
        );
        assert_eq!(
            std::fs::read(f.root.join("destination/replace")).unwrap(),
            b"old"
        );
        assert!(!f.root.join("destination/new").exists());
        assert_eq!(
            std::fs::read(f.root.join("destination/retain")).unwrap(),
            b"retain"
        );
    }
    #[test]
    fn exact_byte_binding_rejects_edits_and_swapped_journals() {
        let f = Fixture::new();
        f.refresh(true);
        let first = prepare(&f.pair).unwrap();
        let second = prepare(&f.pair).unwrap();
        let plan_path = first.directory.join("plan.json");
        let bytes = std::fs::read(&plan_path).unwrap();
        OpenOptions::new()
            .append(true)
            .open(&plan_path)
            .unwrap()
            .write_all(b"\n")
            .unwrap();
        assert!(inspect(&first.directory).is_err()); // same JSON, different exact bytes
        std::fs::write(&plan_path, bytes).unwrap();
        std::fs::copy(
            second.directory.join("journal"),
            first.directory.join("journal"),
        )
        .unwrap();
        assert!(inspect(&first.directory).is_err());
        assert!(inspect(&second.directory).is_ok());
    }
    #[test]
    fn interrupted_preparation_preserves_incomplete_evidence_and_new_run_is_separate() {
        let f = Fixture::new();
        f.refresh(true);
        assert!(
            prepare_with_hook(&f.pair, || anyhow::bail!("simulated stop before journal")).is_err()
        );
        let namespace = f.root.join("destination/.safesync");
        let interrupted = std::fs::read_dir(&namespace)
            .unwrap()
            .map(|e| e.unwrap().path())
            .find(|p| p.file_name().unwrap().to_string_lossy().starts_with("run-"))
            .unwrap();
        assert!(interrupted.join("plan.json").is_file());
        assert!(!interrupted.join("journal").exists());
        assert!(inspect(&interrupted).is_err());
        let new = prepare(&f.pair).unwrap();
        assert_ne!(new.directory, interrupted);
        assert!(interrupted.join("plan.json").exists());
        inspect(&new.directory).unwrap();
    }
    #[test]
    fn rejects_weak_inputs_and_unsafe_metadata() {
        let f = Fixture::new();
        f.refresh(false);
        assert!(prepare(&f.pair).is_err());
        f.refresh(true);
        let prepared = prepare(&f.pair).unwrap();
        let journal = prepared.directory.join("journal");
        std::fs::remove_file(&journal).unwrap();
        std::os::unix::fs::symlink(prepared.directory.join("plan.json"), &journal).unwrap();
        assert!(inspect(&prepared.directory).is_err());
        assert!(
            Journal::create_in(
                &File::open(&prepared.directory).unwrap(),
                Event::Start {
                    run_id: "wrong".into(),
                    plan_sha256: "0".repeat(64),
                    operation_ids: vec![]
                }
            )
            .is_err()
        );
    }
    #[test]
    fn validates_operations_against_snapshots_even_with_rewritten_journal() {
        let f = Fixture::new();
        f.refresh(true);
        let prepared = prepare(&f.pair).unwrap();
        let mut report = inspect(&prepared.directory).unwrap();
        report.plan.operations[0].source.stamp.size += 1;
        let bytes = serde_json::to_vec(&report.plan).unwrap();
        std::fs::write(prepared.directory.join("plan.json"), &bytes).unwrap();
        std::fs::remove_file(prepared.directory.join("journal")).unwrap();
        drop(
            Journal::create(
                &prepared.directory.join("journal"),
                Event::Start {
                    run_id: prepared.run_id,
                    plan_sha256: filesystem::hex(&Sha256::digest(&bytes)),
                    operation_ids: report
                        .plan
                        .operations
                        .iter()
                        .map(|op| op.id.clone())
                        .collect(),
                },
            )
            .unwrap(),
        );
        assert!(inspect(&prepared.directory).is_err());
    }
}
