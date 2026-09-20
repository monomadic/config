//! Filesystem-verified sessions for catalog metadata; no media writes or repairs.
use crate::{
    enrollment::{DriveRole, EnrolledRoot, Enrollment, PairLease},
    filesystem, manifest,
};
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::{
    fs::{File, OpenOptions},
    os::{
        fd::AsRawFd,
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Target {
    mount: PathBuf,
    device: String,
    enrollment: Enrollment,
}
#[derive(Debug, Serialize)]
pub struct Verification {
    pub volume_uuid: String,
    pub device: String,
    pub started_unix: u64,
    pub finished_unix: u64,
}

trait Backend {
    type Lease;
    fn inspect(&mut self, path: &Path) -> Result<Target>;
    fn verify(&mut self, target: &Target) -> Result<()>;
    fn acquire(&mut self, source: &Path, destination: &Path) -> Result<Self::Lease>;
    fn validate(
        &mut self,
        lease: &Self::Lease,
        source: &Target,
        destination: &Target,
    ) -> Result<()>;
}

fn run<B: Backend>(
    backend: &mut B,
    source: &Path,
    destination: &Path,
) -> Result<(B::Lease, Vec<Verification>)> {
    let src = backend.inspect(source)?;
    let dst = backend.inspect(destination)?;
    ensure!(
        src.enrollment.role == DriveRole::ProtectedSource
            && dst.enrollment.role == DriveRole::Destination,
        "Enrolled drive roles do not match source → destination"
    );
    ensure!(
        src.enrollment.volume_uuid != dst.enrollment.volume_uuid
            && src.enrollment.enrollment_id != dst.enrollment.enrollment_id,
        "Source and destination identities must be distinct"
    );
    let mut checks = Vec::new();
    for target in [&src, &dst] {
        ensure!(
            backend.inspect(&target.mount)? == *target,
            "Drive changed before filesystem verification"
        );
        let started_unix = manifest::now();
        backend.verify(target)?;
        ensure!(
            backend.inspect(&target.mount)? == *target,
            "Drive changed during filesystem verification"
        );
        checks.push(Verification {
            volume_uuid: target.enrollment.volume_uuid.clone(),
            device: target.device.clone(),
            started_unix,
            finished_unix: manifest::now(),
        });
    }
    // No drive handles from inspect survive to here. Verification can unmount/remount.
    let lease = backend.acquire(&src.mount, &dst.mount)?;
    backend.validate(&lease, &src, &dst)?;
    Ok((lease, checks))
}

struct System;
impl Backend for System {
    type Lease = PairLease;
    fn inspect(&mut self, path: &Path) -> Result<Target> {
        let root = EnrolledRoot::open(path)?;
        let enrollment = root.inspect()?;
        let mount = root.mount_path().to_owned();
        let info = filesystem::disk_info(&mount)?;
        ensure!(
            info["VolumeUUID"].as_str() == Some(enrollment.volume_uuid.as_str()),
            "Device identity changed"
        );
        let device = info["DeviceIdentifier"]
            .as_str()
            .context("Missing device identifier")?;
        ensure!(
            device.starts_with("disk")
                && device.len() > 4
                && device[4..].bytes().all(|b| b.is_ascii_digit() || b == b's'),
            "Invalid device identifier"
        );
        let reopened = EnrolledRoot::open(&mount)?;
        ensure!(
            reopened.inspect()? == enrollment,
            "Enrollment changed during inspection"
        );
        Ok(Target {
            mount,
            device: device.into(),
            enrollment,
        })
    }
    fn verify(&mut self, target: &Target) -> Result<()> {
        eprintln!(
            "Checking filesystem on {} [{}]; no repair requested.",
            target.device, target.enrollment.volume_uuid
        );
        // Keep stdout reserved for the final structured result. Stream diskutil progress to stderr.
        let status = Command::new("/usr/sbin/diskutil")
            .arg("verifyVolume")
            .arg(format!("/dev/{}", target.device))
            .stdin(Stdio::null())
            .stdout(Stdio::from(std::io::stderr()))
            .stderr(Stdio::inherit())
            .status()
            .context("Cannot start filesystem verification")?;
        ensure!(
            status.success(),
            "Filesystem verification failed for {} ({status}); no session was authorized",
            target.device
        );
        Ok(())
    }
    fn acquire(&mut self, source: &Path, destination: &Path) -> Result<PairLease> {
        PairLease::acquire(
            EnrolledRoot::open(source)?,
            EnrolledRoot::open(destination)?,
        )
    }
    fn validate(&mut self, pair: &PairLease, src: &Target, dst: &Target) -> Result<()> {
        pair.revalidate()?;
        ensure!(
            pair.source.enrollment() == &src.enrollment
                && pair.destination.enrollment() == &dst.enrollment,
            "Enrollment changed before session acquisition"
        );
        ensure!(
            self.inspect(&src.mount)? == *src && self.inspect(&dst.mount)? == *dst,
            "Device changed before session acquisition"
        );
        Ok(())
    }
}

// A host-local advisory mutex serializes safesync verification preflights. Never unlink
// this file: doing so could split concurrent callers across two lock inodes.
fn host_lock(path: &Path) -> Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)?;
    let meta = file.metadata()?;
    ensure!(
        meta.is_file()
            && meta.nlink() == 1
            && meta.uid() == unsafe { libc::geteuid() }
            && meta.mode() & 0o077 == 0,
        "Unsafe host maintenance lock"
    );
    ensure!(
        unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0,
        "Another safesync filesystem preflight is active"
    );
    Ok(file)
}

pub struct VerifiedPair {
    pair: PairLease,
    pub checks: Vec<Verification>,
    _host_lock: File,
}

impl VerifiedPair {
    pub fn check_run_with_reserve(
        &self,
        run_id: &str,
        reserve_bytes: u64,
    ) -> Result<crate::preflight::Report> {
        crate::preflight::check_with_reserve(&self.pair, run_id, reserve_bytes)
    }
    pub fn check_run(&self, run_id: &str) -> Result<crate::preflight::Report> {
        crate::preflight::check(&self.pair, run_id)
    }
    pub fn prepare_run(&self, exclusions: &[PathBuf]) -> Result<crate::run::Prepared> {
        self.refresh_catalogs(true, true, exclusions)?;
        crate::run::prepare(&self.pair).context("Run preparation failed after catalog refresh; incomplete run metadata may remain and must be inspected")
    }
    pub fn adopt_relationship(
        &self,
        exclusions: &[PathBuf],
    ) -> Result<crate::relationship::Adoption> {
        self.refresh_catalogs(true, true, exclusions)?;
        crate::relationship::adopt(&self.pair).context(
            "Catalogs refreshed but relationship adoption did not report success; a publication error may leave a valid baseline, so inspect destination metadata before retrying",
        )
    }
    pub fn pair(&self) -> &PairLease {
        &self.pair
    }

    /// Refresh drive-owned inventories only; no media or relationship mutations.
    /// `reuse` carries fingerprints over from the drive's earlier catalogs and
    /// manifests when a file's ID, size and mtime are unchanged.
    pub fn refresh_catalogs(
        &self,
        hash: bool,
        reuse: bool,
        exclusions: &[PathBuf],
    ) -> Result<(crate::catalog::Current, crate::catalog::Current)> {
        self.pair.revalidate()?;
        // Validate both current catalogs before scanning or publishing either side.
        crate::catalog::load(&self.pair.source)?;
        crate::catalog::load(&self.pair.destination)?;
        let scan = |lease: &crate::enrollment::DriveLease| -> Result<crate::manifest::Manifest> {
            lease.revalidate()?;
            let volume = filesystem::volume_for(lease.scan_root())?;
            ensure!(
                volume.uuid == lease.enrollment().volume_uuid,
                "Volume changed before catalog scan"
            );
            let mut cache = crate::scan::HashCache::new(&volume);
            if hash && reuse {
                cache.add_directory(&lease.scan_root().join(".safesync"));
            }
            eprintln!(
                "Scanning catalog on {:?} ({} fingerprints known)",
                lease.scan_root(),
                cache.len()
            );
            let mut last = std::time::Instant::now();
            let inventory = crate::scan::scan_with_reuse(
                lease.scan_root(),
                volume,
                hash,
                exclusions,
                Some(&cache),
                |p| {
                    if last.elapsed() >= std::time::Duration::from_secs(1) {
                        eprintln!(
                            "{} files · {} bytes catalogued · {} fingerprints reused",
                            p.files, p.bytes, p.reused
                        );
                        last = std::time::Instant::now();
                    }
                },
            )?;
            lease.revalidate()?;
            Ok(inventory)
        };
        let source = scan(&self.pair.source)?;
        let destination = scan(&self.pair.destination)?;
        self.pair.revalidate()?;
        let source = crate::catalog::publish(&self.pair.source, &source)?;
        eprintln!("Source catalog committed: {}", source.generation);
        let destination = crate::catalog::publish(&self.pair.destination, &destination)
            .context("Source catalog committed, but destination publication failed; catalogs are independent")?;
        Ok((source, destination))
    }
}

pub fn verify_pair(source: &Path, destination: &Path) -> Result<VerifiedPair> {
    let uid = unsafe { libc::geteuid() };
    let lock = host_lock(&PathBuf::from(format!(
        "/private/tmp/safesync-maintenance-{uid}.lock"
    )))?;
    let (pair, checks) = run(&mut System, source, destination)?;
    Ok(VerifiedPair {
        pair,
        checks,
        _host_lock: lock,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::bail;
    struct Fake {
        source: Target,
        destination: Target,
        events: Vec<String>,
        fail_verify: Option<String>,
        mutate_after_verify: bool,
        fail_acquire: bool,
        fail_validate: bool,
    }
    impl Fake {
        fn new() -> Self {
            let target = |name: &str, role| Target {
                mount: PathBuf::from(format!("/{name}")),
                device: format!("device-{name}"),
                enrollment: Enrollment {
                    schema: 1,
                    application: "safesync".into(),
                    enrollment_id: name.into(),
                    revision: 1,
                    volume_uuid: name.into(),
                    role,
                    created_unix: 1,
                },
            };
            Self {
                source: target("source", DriveRole::ProtectedSource),
                destination: target("destination", DriveRole::Destination),
                events: vec![],
                fail_verify: None,
                mutate_after_verify: false,
                fail_acquire: false,
                fail_validate: false,
            }
        }
        fn run(&mut self) -> Result<((), Vec<Verification>)> {
            run(self, Path::new("/source"), Path::new("/destination"))
        }
    }
    impl Backend for Fake {
        type Lease = ();
        fn inspect(&mut self, path: &Path) -> Result<Target> {
            self.events.push(format!("inspect:{}", path.display()));
            Ok(if path == Path::new("/source") {
                self.source.clone()
            } else {
                self.destination.clone()
            })
        }
        fn verify(&mut self, target: &Target) -> Result<()> {
            self.events.push(format!("verify:{}", target.device));
            if self.fail_verify.as_ref() == Some(&target.device) {
                bail!("simulated verification failure");
            }
            if self.mutate_after_verify {
                self.source.enrollment.revision += 1;
            }
            Ok(())
        }
        fn acquire(&mut self, _: &Path, _: &Path) -> Result<()> {
            self.events.push("acquire".into());
            ensure!(!self.fail_acquire, "simulated busy drive");
            Ok(())
        }
        fn validate(&mut self, _: &(), _: &Target, _: &Target) -> Result<()> {
            self.events.push("validate".into());
            ensure!(!self.fail_validate, "simulated final identity mismatch");
            Ok(())
        }
    }
    #[test]
    fn verifies_sequentially_before_acquiring_leases() {
        let mut fake = Fake::new();
        let (_, checks) = fake.run().unwrap();
        assert_eq!(checks.len(), 2);
        assert_eq!(checks[0].volume_uuid, "source");
        assert_eq!(
            fake.events,
            [
                "inspect:/source",
                "inspect:/destination",
                "inspect:/source",
                "verify:device-source",
                "inspect:/source",
                "inspect:/destination",
                "verify:device-destination",
                "inspect:/destination",
                "acquire",
                "validate"
            ]
        );
    }
    #[test]
    fn failures_and_identity_changes_never_acquire_leases() {
        for device in ["device-source", "device-destination"] {
            let mut fake = Fake::new();
            fake.fail_verify = Some(device.into());
            assert!(fake.run().is_err());
            assert!(!fake.events.iter().any(|e| e == "acquire"));
            if device == "device-source" {
                assert!(!fake.events.iter().any(|e| e == "verify:device-destination"));
            }
        }
        let mut fake = Fake::new();
        fake.mutate_after_verify = true;
        assert!(fake.run().is_err());
        assert!(!fake.events.iter().any(|e| e == "acquire"));
    }
    #[test]
    fn rejects_roles_and_duplicate_identity_before_verification() {
        for mutation in 0..3 {
            let mut fake = Fake::new();
            match mutation {
                0 => fake.source.enrollment.role = DriveRole::Destination,
                1 => {
                    fake.destination.enrollment.volume_uuid =
                        fake.source.enrollment.volume_uuid.clone()
                }
                _ => {
                    fake.destination.enrollment.enrollment_id =
                        fake.source.enrollment.enrollment_id.clone()
                }
            }
            assert!(fake.run().is_err());
            assert_eq!(fake.events.len(), 2);
        }
    }
    #[test]
    fn final_acquisition_and_validation_failures_are_not_success() {
        let mut fake = Fake::new();
        fake.fail_acquire = true;
        assert!(fake.run().is_err());
        assert_eq!(fake.events.last().unwrap(), "acquire");
        let mut fake = Fake::new();
        fake.fail_validate = true;
        assert!(fake.run().is_err());
        assert_eq!(fake.events.last().unwrap(), "validate");
    }
    #[test]
    fn maintenance_lock_excludes_competitors_and_refuses_links() {
        let dir =
            std::env::temp_dir().join(format!("safesync-lock-test-{}", manifest::generation()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("lock");
        let lock = host_lock(&path).unwrap();
        assert!(host_lock(&path).is_err());
        drop(lock);
        assert!(host_lock(&path).is_ok());
        let linked = dir.join("linked");
        std::os::unix::fs::symlink(&path, &linked).unwrap();
        assert!(host_lock(&linked).is_err());
        std::fs::remove_file(&linked).unwrap();
        std::fs::hard_link(&path, &linked).unwrap();
        assert!(host_lock(&path).is_err());
        std::fs::remove_dir_all(dir).unwrap();
    }
}
