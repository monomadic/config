//! Immutable destination-owned adoption evidence, never a whole-sync success marker.
use crate::{
    catalog,
    enrollment::{DriveRole, Enrollment, PairLease},
    filesystem, history,
    manifest::{self, Manifest, Role},
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::OpenOptions,
    io::Read,
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

// A bounded prototype: reject rather than allocate an unbounded relationship record.
const MAX_BYTES: u64 = 256 * 1024 * 1024;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Baseline {
    pub schema: u32,
    pub generation: String,
    pub profile_revision: u64,
    pub policy: String,
    pub source_enrollment: Enrollment,
    pub destination_enrollment: Enrollment,
    pub source: Manifest,
    pub destination: Manifest,
    pub matched_paths_base64: Vec<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    sha256: String,
    baseline: Baseline,
}
#[derive(Serialize)]
pub struct Adoption {
    pub path: PathBuf,
    pub generation: String,
    pub matched_files: usize,
    pub unmatched_source_files: usize,
    pub unmatched_destination_files: usize,
    pub whole_sync_committed: bool,
}
fn matches(source: &Manifest, destination: &Manifest) -> Vec<String> {
    let paths: BTreeMap<_, _> = destination
        .entries
        .iter()
        .map(|e| (&e.path_base64, e))
        .collect();
    let mut result: Vec<_> = source
        .entries
        .iter()
        .filter(|s| {
            paths.get(&s.path_base64).is_some_and(|d| {
                s.sha256.is_some() && s.sha256 == d.sha256 && s.stamp.size == d.stamp.size
            })
        })
        .map(|e| e.path_base64.clone())
        .collect();
    result.sort();
    result
}
impl Baseline {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.schema == 1
                && self.profile_revision == 1
                && self.policy == "same_path_full_sha256",
            "Unsupported relationship schema/profile policy"
        );
        ensure!(
            !self.generation.is_empty()
                && self.generation.len() <= 100
                && self
                    .generation
                    .bytes()
                    .all(|b| b.is_ascii_digit() || b == b'-'),
            "Invalid relationship generation"
        );
        self.source.validate()?;
        self.destination.validate()?;
        self.source_enrollment
            .validate(&self.source.header.volume.uuid)?;
        self.destination_enrollment
            .validate(&self.destination.header.volume.uuid)?;
        ensure!(
            self.source_enrollment.role == DriveRole::ProtectedSource
                && self.destination_enrollment.role == DriveRole::Destination,
            "Invalid relationship direction"
        );
        ensure!(
            self.source_enrollment.enrollment_id != self.destination_enrollment.enrollment_id,
            "Duplicated relationship enrollment"
        );
        ensure!(
            matches!(self.source.header.role, Role::Inventory)
                && matches!(self.destination.header.role, Role::Inventory),
            "Offline exports are not adoption authority"
        );
        ensure!(
            self.source.header.content_hashed && self.destination.header.content_hashed,
            "Adoption requires complete-content inventories"
        );
        let plan = crate::plan::preview(&self.source, &self.destination)?;
        ensure!(
            plan.blockers.is_empty(),
            "Relationship snapshots have namespace or scope blockers"
        );
        ensure!(
            self.matched_paths_base64 == matches(&self.source, &self.destination),
            "Relationship match set is inconsistent"
        );
        ensure!(
            !self.matched_paths_base64.is_empty(),
            "No same-path full-content matches to adopt"
        );
        Ok(())
    }
    pub fn load(path: &Path) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(path)?;
        ensure!(
            file.metadata()?.is_file() && file.metadata()?.nlink() == 1,
            "Unsafe relationship file"
        );
        let mut bytes = Vec::new();
        file.take(MAX_BYTES + 1).read_to_end(&mut bytes)?;
        ensure!(
            bytes.len() as u64 <= MAX_BYTES,
            "Relationship exceeds prototype size limit"
        );
        let envelope: Envelope =
            serde_json::from_slice(&bytes).context("Invalid relationship record")?;
        let encoded = serde_json::to_vec(&envelope.baseline)?;
        ensure!(
            filesystem::hex(&Sha256::digest(&encoded)) == envelope.sha256,
            "Relationship checksum mismatch"
        );
        envelope.baseline.validate()?;
        Ok(envelope.baseline)
    }
    pub fn preview(
        &self,
        source: &Manifest,
        destination: &Manifest,
    ) -> Result<history::HistoryPreview> {
        self.validate()?;
        let mut report = history::preview(&self.source, &self.destination, source, destination)?;
        report.committed_relationship = true;
        report.warnings[0] = format!(
            "Using recorded adoption baseline {} (profile revision 1). Current enrollment and presence remain unverified; no media operation is authorized.",
            self.generation
        );
        Ok(report)
    }
}

// Only called by the filesystem-verified session after a fresh hashed catalog refresh.
pub(crate) fn adopt(pair: &PairLease) -> Result<Adoption> {
    pair.revalidate()?;
    let (_, source) = catalog::load(&pair.source)?.context("Missing source catalog")?;
    let (_, destination) =
        catalog::load(&pair.destination)?.context("Missing destination catalog")?;
    let baseline = Baseline {
        schema: 1,
        generation: manifest::generation(),
        profile_revision: 1,
        policy: "same_path_full_sha256".into(),
        source_enrollment: pair.source.enrollment().clone(),
        destination_enrollment: pair.destination.enrollment().clone(),
        matched_paths_base64: matches(&source, &destination),
        source,
        destination,
    };
    baseline.validate()?;
    let matched_files = baseline.matched_paths_base64.len();
    let record = Envelope {
        sha256: filesystem::hex(&Sha256::digest(serde_json::to_vec(&baseline)?)),
        baseline,
    };
    let bytes = serde_json::to_vec(&record)?;
    ensure!(
        bytes.len() as u64 <= MAX_BYTES,
        "Relationship exceeds prototype size limit"
    );
    pair.revalidate()?;
    let name = format!("relationship-{}.json", record.baseline.generation);
    catalog::publish_metadata(&pair.destination, &name, &bytes)?;
    pair.revalidate()?;
    Ok(Adoption {
        path: pair.destination.scan_root().join(".safesync").join(name),
        generation: record.baseline.generation,
        matched_files,
        unmatched_source_files: record.baseline.source.entries.len() - matched_files,
        unmatched_destination_files: record.baseline.destination.entries.len() - matched_files,
        whole_sync_committed: false,
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
            let root = std::env::temp_dir()
                .join(format!("safesync-relationship-{}", manifest::generation()));
            std::fs::create_dir(&root).unwrap();
            let root = root.canonicalize().unwrap();
            for dir in ["source", "destination"] {
                std::fs::create_dir(root.join(dir)).unwrap();
            }
            let pair = PairLease {
                source: DriveLease::test_role_lease(
                    &root.join("source"),
                    "source-volume",
                    DriveRole::ProtectedSource,
                )
                .unwrap(),
                destination: DriveLease::test_role_lease(
                    &root.join("destination"),
                    "destination-volume",
                    DriveRole::Destination,
                )
                .unwrap(),
            };
            for dir in ["source", "destination"] {
                std::fs::write(root.join(dir).join("shared"), b"same").unwrap();
            }
            std::fs::write(root.join("source/only-source"), b"new").unwrap();
            std::fs::write(root.join("destination/only-destination"), b"old").unwrap();
            Self { root, pair }
        }
        fn scan(lease: &DriveLease, hash: bool) -> Manifest {
            scan::scan(
                lease.scan_root(),
                Volume {
                    uuid: lease.enrollment().volume_uuid.clone(),
                    name: "fixture".into(),
                    filesystem: "apfs".into(),
                },
                hash,
                |_| {},
            )
            .unwrap()
        }
        fn refresh(&self, hash: bool) {
            for lease in [&self.pair.source, &self.pair.destination] {
                catalog::publish(lease, &Self::scan(lease, hash)).unwrap();
            }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
    #[test]
    fn adopts_only_proven_matches_and_preserves_prior_baselines() {
        let f = Fixture::new();
        f.refresh(true);
        let adoption = adopt(&f.pair).unwrap();
        assert_eq!(adoption.matched_files, 1);
        assert_eq!(adoption.unmatched_source_files, 1);
        assert_eq!(adoption.unmatched_destination_files, 1);
        assert!(!adoption.whole_sync_committed);
        assert!(
            adoption
                .path
                .starts_with(f.root.join("destination/.safesync"))
        );
        let baseline = Baseline::load(&adoption.path).unwrap();
        assert_eq!(baseline.source.entries.len(), 2);
        assert_eq!(baseline.destination.entries.len(), 2);
        assert_eq!(
            baseline.matched_paths_base64,
            vec![manifest::encode_path(Path::new("shared"))]
        );
        let next = adopt(&f.pair).unwrap();
        assert_ne!(adoption.generation, next.generation);
        assert!(Baseline::load(&adoption.path).is_ok());
        assert_eq!(
            std::fs::read(f.root.join("source/shared")).unwrap(),
            b"same"
        );
        assert!(
            catalog::publish_metadata(
                &f.pair.destination,
                adoption.path.file_name().unwrap().to_str().unwrap(),
                b"overwrite"
            )
            .is_err()
        );
        Baseline::load(&adoption.path).unwrap();
    }
    #[test]
    fn rejects_unhashed_and_zero_match_adoption() {
        let f = Fixture::new();
        f.refresh(false);
        assert!(adopt(&f.pair).is_err());
        std::fs::write(f.root.join("destination/shared"), b"different").unwrap();
        f.refresh(true);
        assert!(adopt(&f.pair).is_err());
    }
    #[test]
    fn detects_corruption_and_semantic_tampering() {
        let f = Fixture::new();
        f.refresh(true);
        let adoption = adopt(&f.pair).unwrap();
        let original = std::fs::read(&adoption.path).unwrap();
        let mut envelope: Envelope = serde_json::from_slice(&original).unwrap();
        envelope.baseline.matched_paths_base64.clear();
        std::fs::write(&adoption.path, serde_json::to_vec(&envelope).unwrap()).unwrap();
        assert!(Baseline::load(&adoption.path).is_err());
        envelope.sha256 = filesystem::hex(&Sha256::digest(
            serde_json::to_vec(&envelope.baseline).unwrap(),
        ));
        std::fs::write(&adoption.path, serde_json::to_vec(&envelope).unwrap()).unwrap();
        assert!(Baseline::load(&adoption.path).is_err()); // valid checksum, invalid claims
        std::fs::write(&adoption.path, &original[..original.len() / 2]).unwrap();
        assert!(Baseline::load(&adoption.path).is_err());
    }
    #[test]
    fn baseline_supports_offline_rename_review_and_rejects_wrong_scope() {
        let f = Fixture::new();
        f.refresh(true);
        let adoption = adopt(&f.pair).unwrap();
        let baseline = Baseline::load(&adoption.path).unwrap();
        std::fs::rename(f.root.join("source/shared"), f.root.join("source/renamed")).unwrap();
        let source = Fixture::scan(&f.pair.source, true);
        let mut destination = Fixture::scan(&f.pair.destination, true);
        let report = baseline.preview(&source, &destination).unwrap();
        assert!(report.committed_relationship);
        assert!(!report.executable);
        assert!(
            report
                .observations
                .iter()
                .any(|o| o.evidence == history::Evidence::FullContentAtScanTime)
        );
        destination.header.volume.uuid = "wrong-drive".into();
        assert!(baseline.preview(&source, &destination).is_err());
    }
    #[test]
    fn refuses_symlinked_and_hardlinked_baselines() {
        let f = Fixture::new();
        f.refresh(true);
        let adoption = adopt(&f.pair).unwrap();
        let link = f.root.join("link");
        std::os::unix::fs::symlink(&adoption.path, &link).unwrap();
        assert!(Baseline::load(&link).is_err());
        std::fs::remove_file(&link).unwrap();
        std::fs::hard_link(&adoption.path, &link).unwrap();
        assert!(Baseline::load(&adoption.path).is_err());
    }
}
