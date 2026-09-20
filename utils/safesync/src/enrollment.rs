//! Drive role records only: enrollment does not grant a media-write capability.
use crate::{filesystem, manifest};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "snake_case")]
pub enum DriveRole {
    ProtectedSource,
    Destination,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Enrollment {
    pub schema: u32,
    pub application: String,
    pub enrollment_id: String,
    pub revision: u64,
    pub volume_uuid: String,
    pub role: DriveRole,
    pub created_unix: u64,
}
impl Enrollment {
    pub fn validate(&self, volume_uuid: &str) -> Result<()> {
        ensure!(
            self.schema == 1 && self.application == "safesync",
            "Unsupported enrollment record"
        );
        ensure!(self.revision == 1, "Unsupported enrollment revision");
        ensure!(
            !self.enrollment_id.is_empty(),
            "Missing enrollment identity"
        );
        ensure!(
            !volume_uuid.is_empty() && self.volume_uuid == volume_uuid,
            "Enrollment belongs to a different volume"
        );
        Ok(())
    }
}

pub struct EnrolledRoot {
    root: File,
    volume_uuid: String,
    mount_path: PathBuf,
    #[cfg(test)]
    synthetic: bool,
}
impl EnrolledRoot {
    pub(crate) fn mount_path(&self) -> &Path {
        &self.mount_path
    }
    /// Require the actual local APFS volume root, not a selected media subtree.
    pub fn open(path: &Path) -> Result<Self> {
        let root = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path)?;
        let mut stat = std::mem::MaybeUninit::<libc::statfs>::uninit();
        // SAFETY: root is live and stat points to sufficient writable storage.
        ensure!(
            unsafe { libc::fstatfs(root.as_raw_fd(), stat.as_mut_ptr()) } == 0,
            "Cannot inspect volume mount: {}",
            std::io::Error::last_os_error()
        );
        let stat = unsafe { stat.assume_init() };
        ensure!(
            stat.f_flags & libc::MNT_LOCAL as u32 != 0,
            "Enrollment requires local storage"
        );
        let mount = unsafe { std::ffi::CStr::from_ptr(stat.f_mntonname.as_ptr()) };
        use std::os::unix::ffi::OsStrExt;
        let mount_path = Path::new(std::ffi::OsStr::from_bytes(mount.to_bytes()));
        let mounted = std::fs::metadata(mount_path)?;
        let opened = root.metadata()?;
        ensure!(
            mounted.dev() == opened.dev() && mounted.ino() == opened.ino(),
            "Enrollment requires the volume root, not a subdirectory"
        );
        let volume = filesystem::volume_for(mount_path)?;
        ensure!(
            volume.filesystem.eq_ignore_ascii_case("apfs"),
            "Enrollment currently requires APFS"
        );
        let current = std::fs::metadata(mount_path)?;
        ensure!(
            current.dev() == opened.dev() && current.ino() == opened.ino(),
            "Volume changed during identification"
        );
        Ok(Self {
            root,
            volume_uuid: volume.uuid,
            mount_path: mount_path.to_owned(),
            #[cfg(test)]
            synthetic: false,
        })
    }

    fn revalidate_attachment(&self) -> Result<()> {
        let current = OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&self.mount_path)?;
        ensure!(
            same_object(&current, &self.root)?,
            "Volume root was replaced or disconnected"
        );
        #[cfg(test)]
        if self.synthetic {
            return Ok(());
        }
        let reopened = Self::open(&self.mount_path)?;
        ensure!(
            reopened.volume_uuid == self.volume_uuid && same_object(&reopened.root, &self.root)?,
            "Volume attachment identity changed"
        );
        Ok(())
    }

    fn namespace(&self, create: bool) -> Result<File> {
        if create {
            // SAFETY: fixed NUL-terminated component and a live directory fd.
            let result =
                unsafe { libc::mkdirat(self.root.as_raw_fd(), c".safesync".as_ptr(), 0o700) };
            if result != 0 {
                ensure!(
                    std::io::Error::last_os_error().raw_os_error() == Some(libc::EEXIST),
                    "Cannot create .safesync: {}",
                    std::io::Error::last_os_error()
                );
            }
            self.root.sync_all()?;
        }
        let directory = filesystem::open_relative(
            &self.root,
            Path::new(".safesync"),
            self.root.metadata()?.dev(),
        )?;
        ensure!(
            directory.metadata()?.is_dir(),
            ".safesync is not a directory"
        );
        Ok(directory)
    }

    pub fn inspect(&self) -> Result<Enrollment> {
        self.read_marker(&self.namespace(false)?)
    }

    fn read_marker(&self, directory: &File) -> Result<Enrollment> {
        let mut marker = filesystem::open_relative(
            directory,
            Path::new("volume.json"),
            self.root.metadata()?.dev(),
        )?;
        ensure!(
            marker.metadata()?.is_file() && marker.metadata()?.nlink() == 1,
            "Unsafe enrollment marker"
        );
        let before = filesystem::Stamp::of(&marker.metadata()?);
        let mut bytes = Vec::new();
        Read::by_ref(&mut marker)
            .take(65537)
            .read_to_end(&mut bytes)?;
        ensure!(bytes.len() <= 65536, "Oversized enrollment marker");
        ensure!(
            before == filesystem::Stamp::of(&marker.metadata()?),
            "Enrollment changed while reading"
        );
        let record: Enrollment =
            serde_json::from_slice(&bytes).context("Invalid enrollment marker")?;
        record.validate(&self.volume_uuid)?;
        Ok(record)
    }

    /// Create once. Existing roles are never changed, even when requested again.
    pub fn enroll(&self, role: DriveRole) -> Result<Enrollment> {
        self.revalidate_attachment()?;
        let directory = self.namespace(true)?;
        // Lock the opened namespace without creating a lock file in an unknown namespace.
        ensure!(
            unsafe { libc::flock(directory.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0,
            "Enrollment is busy: {}",
            std::io::Error::last_os_error()
        );
        let names = filesystem::names(&directory)?;
        if names.iter().any(|name| name == "volume.json") {
            let existing = self.read_marker(&directory)?;
            ensure!(
                existing.role == role,
                "Drive already enrolled with a different role; role changes are not supported"
            );
            return Ok(existing);
        }
        ensure!(
            names.is_empty(),
            "Refusing an unrecognized nonempty .safesync directory; preserve and inspect its contents before enrollment"
        );
        let record = Enrollment {
            schema: 1,
            application: "safesync".into(),
            enrollment_id: manifest::generation(),
            revision: 1,
            volume_uuid: self.volume_uuid.clone(),
            role,
            created_unix: manifest::now(),
        };
        let bytes = serde_json::to_vec_pretty(&record)?;
        let temporary =
            std::ffi::CString::new(format!(".enrollment-{}.partial", manifest::generation()))?;
        let fd = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                temporary.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                0o600,
            )
        };
        ensure!(
            fd >= 0,
            "Cannot create enrollment: {}",
            std::io::Error::last_os_error()
        );
        let mut file = unsafe { File::from_raw_fd(fd) };
        let result = (|| -> Result<()> {
            file.write_all(&bytes)?;
            file.write_all(b"\n")?;
            file.sync_all()?;
            ensure!(
                unsafe { libc::fcntl(file.as_raw_fd(), libc::F_FULLFSYNC) } == 0,
                "Enrollment durability flush failed: {}",
                std::io::Error::last_os_error()
            );
            ensure!(
                unsafe {
                    libc::linkat(
                        directory.as_raw_fd(),
                        temporary.as_ptr(),
                        directory.as_raw_fd(),
                        c"volume.json".as_ptr(),
                        0,
                    )
                } == 0,
                "Cannot publish enrollment without overwriting: {}",
                std::io::Error::last_os_error()
            );
            Ok(())
        })();
        let removed = unsafe { libc::unlinkat(directory.as_raw_fd(), temporary.as_ptr(), 0) };
        result?;
        ensure!(
            removed == 0,
            "Enrollment published but temporary cleanup failed"
        );
        directory.sync_all()?;
        self.read_marker(&directory)
    }
}

fn same_object(a: &File, b: &File) -> Result<bool> {
    let a = a.metadata()?;
    let b = b.metadata()?;
    Ok(a.dev() == b.dev() && a.ino() == b.ino())
}

/// Holds an exclusive advisory lease on the existing namespace directory.
/// This exposes no media or catalog write capability.
pub struct DriveLease {
    root: EnrolledRoot,
    directory: File,
    enrollment: Enrollment,
}
impl DriveLease {
    pub(crate) fn catalog_directory(&self) -> Result<File> {
        self.revalidate()?;
        self.directory.try_clone().map_err(Into::into)
    }

    pub(crate) fn scan_root(&self) -> &Path {
        &self.root.mount_path
    }

    pub(crate) fn root_file_id(&self) -> Result<u64> {
        Ok(self.root.root.metadata()?.ino())
    }

    #[cfg(test)]
    pub(crate) fn test_lease(path: &Path) -> Result<Self> {
        Self::test_role_lease(path, "test-catalog-volume", DriveRole::ProtectedSource)
    }

    #[cfg(test)]
    pub(crate) fn test_role_lease(path: &Path, uuid: &str, role: DriveRole) -> Result<Self> {
        let root = EnrolledRoot {
            root: File::open(path)?,
            volume_uuid: uuid.into(),
            mount_path: path.to_owned(),
            synthetic: true,
        };
        root.enroll(role)?;
        Self::acquire(root, role)
    }
    pub fn acquire(root: EnrolledRoot, expected_role: DriveRole) -> Result<Self> {
        root.revalidate_attachment()?;
        let directory = root.namespace(false)?;
        // SAFETY: directory owns a live descriptor; closing it releases the lock.
        ensure!(
            unsafe { libc::flock(directory.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0,
            "Drive is busy in another safesync session: {}",
            std::io::Error::last_os_error()
        );
        let enrollment = root.read_marker(&directory)?;
        ensure!(
            enrollment.role == expected_role,
            "Drive role does not match the requested direction"
        );
        let lease = Self {
            root,
            directory,
            enrollment,
        };
        lease.revalidate()?;
        Ok(lease)
    }

    pub fn enrollment(&self) -> &Enrollment {
        &self.enrollment
    }

    pub fn revalidate(&self) -> Result<()> {
        self.root.revalidate_attachment()?;
        let current = self.root.namespace(false)?;
        ensure!(
            same_object(&current, &self.directory)?,
            "Locked .safesync namespace was replaced"
        );
        ensure!(
            self.root.read_marker(&current)? == self.enrollment,
            "Enrollment changed during the session"
        );
        Ok(())
    }
}

pub struct PairLease {
    pub source: DriveLease,
    pub destination: DriveLease,
}
impl PairLease {
    pub fn acquire(source: EnrolledRoot, destination: EnrolledRoot) -> Result<Self> {
        ensure!(
            source.volume_uuid != destination.volume_uuid,
            "Source and destination must be different volumes"
        );
        let (source, destination) = if source.volume_uuid < destination.volume_uuid {
            let source = DriveLease::acquire(source, DriveRole::ProtectedSource)?;
            let destination = DriveLease::acquire(destination, DriveRole::Destination)?;
            (source, destination)
        } else {
            let destination = DriveLease::acquire(destination, DriveRole::Destination)?;
            let source = DriveLease::acquire(source, DriveRole::ProtectedSource)?;
            (source, destination)
        };
        ensure!(
            source.enrollment.enrollment_id != destination.enrollment.enrollment_id,
            "Duplicated enrollment identity; explicit re-enrollment is required"
        );
        let pair = Self {
            source,
            destination,
        };
        pair.revalidate()?;
        Ok(pair)
    }

    pub fn revalidate(&self) -> Result<()> {
        self.source.revalidate()?;
        self.destination.revalidate()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fixture {
        path: std::path::PathBuf,
        root: EnrolledRoot,
    }
    impl Fixture {
        fn reopen(&self) -> EnrolledRoot {
            EnrolledRoot {
                root: File::open(&self.path).unwrap(),
                volume_uuid: self.root.volume_uuid.clone(),
                mount_path: self.path.clone(),
                synthetic: true,
            }
        }
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("safesync-enroll-{}", manifest::generation()));
            std::fs::create_dir(&path).unwrap();
            let root = EnrolledRoot {
                root: File::open(&path).unwrap(),
                volume_uuid: "test-volume".into(),
                mount_path: path.clone(),
                synthetic: true,
            };
            Self { path, root }
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
    #[test]
    fn persists_and_never_changes_roles() {
        let f = Fixture::new();
        let first = f.root.enroll(DriveRole::ProtectedSource).unwrap();
        assert_eq!(first, f.root.inspect().unwrap());
        assert_eq!(first, f.root.enroll(DriveRole::ProtectedSource).unwrap());
        assert!(f.root.enroll(DriveRole::Destination).is_err());
        assert_eq!(first, f.root.inspect().unwrap());
        assert!(first.validate("other-volume").is_err());
    }
    #[test]
    fn rejects_unknown_namespace_symlinks_and_corruption() {
        let f = Fixture::new();
        assert!(f.root.inspect().is_err());
        std::os::unix::fs::symlink(&f.path, f.path.join(".safesync")).unwrap();
        assert!(f.root.enroll(DriveRole::Destination).is_err());
        std::fs::remove_file(f.path.join(".safesync")).unwrap();
        std::fs::create_dir(f.path.join(".safesync")).unwrap();
        let marker = f.path.join(".safesync/volume.json");
        std::fs::write(&marker, b"corrupt").unwrap();
        assert!(f.root.enroll(DriveRole::Destination).is_err());
        assert_eq!(std::fs::read(&marker).unwrap(), b"corrupt");
        std::fs::remove_file(&marker).unwrap();
        std::fs::write(f.path.join(".safesync/unrelated"), b"keep").unwrap();
        assert!(f.root.enroll(DriveRole::Destination).is_err());
    }
    #[test]
    fn copied_deleted_and_future_markers_fail_closed() {
        let f = Fixture::new();
        let mut record = f.root.enroll(DriveRole::Destination).unwrap();
        let marker = f.path.join(".safesync/volume.json");
        record.volume_uuid = "other-volume".into();
        std::fs::write(&marker, serde_json::to_vec(&record).unwrap()).unwrap();
        assert!(f.root.inspect().is_err());
        record.volume_uuid = "test-volume".into();
        record.revision = 2;
        std::fs::write(&marker, serde_json::to_vec(&record).unwrap()).unwrap();
        assert!(f.root.inspect().is_err());
        std::fs::remove_file(marker).unwrap();
        assert!(f.root.inspect().is_err());
    }
    #[test]
    fn rejects_subdirectories_as_volume_roots() {
        let f = Fixture::new();
        assert!(EnrolledRoot::open(&f.path).is_err());
    }

    #[test]
    fn competing_enrollment_and_linked_markers_are_rejected() {
        let f = Fixture::new();
        let directory = f.root.namespace(true).unwrap();
        assert_eq!(
            unsafe { libc::flock(directory.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
            0
        );
        assert!(f.root.enroll(DriveRole::Destination).is_err());
        drop(directory);
        f.root.enroll(DriveRole::Destination).unwrap();
        let marker = f.path.join(".safesync/volume.json");
        let other = f.path.join("marker-copy");
        std::fs::hard_link(&marker, &other).unwrap();
        assert!(f.root.inspect().is_err());
        std::fs::remove_file(&marker).unwrap();
        std::os::unix::fs::symlink(&other, &marker).unwrap();
        assert!(f.root.inspect().is_err());
    }

    #[test]
    fn lease_excludes_competitors_and_releases_on_drop() {
        let f = Fixture::new();
        f.root.enroll(DriveRole::ProtectedSource).unwrap();
        let lease = DriveLease::acquire(f.reopen(), DriveRole::ProtectedSource).unwrap();
        assert!(DriveLease::acquire(f.reopen(), DriveRole::ProtectedSource).is_err());
        assert!(f.root.enroll(DriveRole::ProtectedSource).is_err());
        lease.revalidate().unwrap();
        drop(lease);
        assert!(DriveLease::acquire(f.reopen(), DriveRole::ProtectedSource).is_ok());
        assert!(DriveLease::acquire(f.reopen(), DriveRole::Destination).is_err());
    }

    #[test]
    fn lease_detects_marker_and_namespace_replacement() {
        let f = Fixture::new();
        let mut record = f.root.enroll(DriveRole::Destination).unwrap();
        let lease = DriveLease::acquire(f.reopen(), DriveRole::Destination).unwrap();
        record.role = DriveRole::ProtectedSource;
        std::fs::write(
            f.path.join(".safesync/volume.json"),
            serde_json::to_vec(&record).unwrap(),
        )
        .unwrap();
        assert!(lease.revalidate().is_err());
        record.role = DriveRole::Destination;
        std::fs::write(
            f.path.join(".safesync/volume.json"),
            serde_json::to_vec(&record).unwrap(),
        )
        .unwrap();
        lease.revalidate().unwrap();
        std::fs::rename(f.path.join(".safesync"), f.path.join("old-namespace")).unwrap();
        std::fs::create_dir(f.path.join(".safesync")).unwrap();
        std::fs::copy(
            f.path.join("old-namespace/volume.json"),
            f.path.join(".safesync/volume.json"),
        )
        .unwrap();
        assert!(lease.revalidate().is_err());
    }

    #[test]
    fn lease_detects_root_replacement() {
        let f = Fixture::new();
        f.root.enroll(DriveRole::Destination).unwrap();
        let lease = DriveLease::acquire(f.reopen(), DriveRole::Destination).unwrap();
        let moved = f.path.with_extension("moved");
        std::fs::rename(&f.path, &moved).unwrap();
        std::fs::create_dir(&f.path).unwrap();
        assert!(lease.revalidate().is_err());
        std::fs::remove_dir(&f.path).unwrap();
        std::fs::rename(moved, &f.path).unwrap();
    }

    #[test]
    fn pair_checks_direction_identity_and_releases_partial_locks() {
        let a = Fixture::new();
        let mut b = Fixture::new();
        b.root.volume_uuid = "z-other-volume".into();
        a.root.enroll(DriveRole::ProtectedSource).unwrap();
        b.root.enroll(DriveRole::Destination).unwrap();
        assert!(PairLease::acquire(a.reopen(), a.reopen()).is_err());
        assert!(PairLease::acquire(b.reopen(), a.reopen()).is_err());
        let busy = DriveLease::acquire(b.reopen(), DriveRole::Destination).unwrap();
        assert!(PairLease::acquire(a.reopen(), b.reopen()).is_err());
        assert!(DriveLease::acquire(a.reopen(), DriveRole::ProtectedSource).is_ok());
        drop(busy);
        let pair = PairLease::acquire(a.reopen(), b.reopen()).unwrap();
        pair.revalidate().unwrap();
    }
}
