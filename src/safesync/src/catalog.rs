//! Drive-owned snapshots. Publication changes tool metadata only, never media.
use crate::{
    enrollment::DriveLease,
    filesystem,
    manifest::{self, Manifest, Role},
};
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    ffi::CString,
    fs::File,
    io::{BufWriter, Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::MetadataExt,
    },
    path::Path,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Current {
    pub schema: u32,
    pub enrollment_id: String,
    pub volume_uuid: String,
    pub generation: String,
}
fn generation_name(generation: &str) -> Result<CString> {
    ensure!(
        !generation.is_empty()
            && generation.len() <= 100
            && generation.bytes().all(|b| b.is_ascii_digit() || b == b'-'),
        "Unsafe catalog generation"
    );
    Ok(CString::new(format!("catalog-{generation}.jsonl"))?)
}
fn open_file(directory: &File, name: &Path) -> Result<File> {
    let file = filesystem::open_relative(directory, name, directory.metadata()?.dev())?;
    ensure!(
        file.metadata()?.is_file() && file.metadata()?.nlink() == 1,
        "Unsafe catalog file"
    );
    Ok(file)
}
fn pointer(directory: &File) -> Result<Option<Current>> {
    // Distinguish a missing pointer from corrupt, unreadable or symlinked metadata.
    let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
    let result = unsafe {
        libc::fstatat(
            directory.as_raw_fd(),
            c"CURRENT".as_ptr(),
            stat.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    if result != 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::ENOENT) {
            return Ok(None);
        }
        return Err(error.into());
    }
    let mut bytes = Vec::new();
    open_file(directory, Path::new("CURRENT"))?
        .take(65537)
        .read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= 65536, "Oversized CURRENT record");
    Ok(Some(
        serde_json::from_slice(&bytes).context("Invalid CURRENT record")?,
    ))
}
fn validate_inventory(lease: &DriveLease, inventory: &Manifest) -> Result<()> {
    inventory.validate()?;
    ensure!(
        matches!(inventory.header.role, Role::Inventory),
        "Offline snapshots cannot become authoritative catalogs"
    );
    ensure!(
        inventory.header.volume.uuid == lease.enrollment().volume_uuid,
        "Catalog volume mismatch"
    );
    ensure!(
        manifest::decode_path(&inventory.header.root_base64)? == lease.scan_root()
            && inventory.header.root_file_id == lease.root_file_id()?,
        "Catalog must describe the enrolled volume root"
    );
    generation_name(&inventory.header.generation)?;
    Ok(())
}

pub fn load(lease: &DriveLease) -> Result<Option<(Current, Manifest)>> {
    let directory = lease.catalog_directory()?;
    let Some(current) = pointer(&directory)? else {
        return Ok(None);
    };
    ensure!(
        current.schema == 1
            && current.enrollment_id == lease.enrollment().enrollment_id
            && current.volume_uuid == lease.enrollment().volume_uuid,
        "CURRENT does not match this enrollment or schema"
    );
    let name = generation_name(&current.generation)?;
    let inventory = Manifest::read_from(open_file(&directory, Path::new(name.to_str()?))?)?;
    validate_inventory(lease, &inventory)?;
    ensure!(
        inventory.header.generation == current.generation,
        "Catalog generation mismatch"
    );
    lease.revalidate()?;
    Ok(Some((current, inventory)))
}

fn temporary(directory: &File) -> Result<(CString, File)> {
    let name = CString::new(format!(".catalog-{}.partial", manifest::generation()))?;
    let fd = unsafe {
        libc::openat(
            directory.as_raw_fd(),
            name.as_ptr(),
            libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
            0o600,
        )
    };
    ensure!(
        fd >= 0,
        "Cannot stage catalog: {}",
        std::io::Error::last_os_error()
    );
    Ok((name, unsafe { File::from_raw_fd(fd) }))
}
fn unlink(directory: &File, name: &CString) {
    unsafe {
        libc::unlinkat(directory.as_raw_fd(), name.as_ptr(), 0);
    }
}

pub(crate) fn publish_metadata(lease: &DriveLease, name: &str, bytes: &[u8]) -> Result<()> {
    ensure!(
        !name.is_empty() && !name.contains('/') && name != "." && name != "..",
        "Unsafe metadata filename"
    );
    let final_name = CString::new(name)?;
    let directory = lease.catalog_directory()?;
    let (temp_name, mut file) = temporary(&directory)?;
    let result = (|| -> Result<()> {
        file.write_all(bytes)?;
        manifest::full_sync(&file)?;
        lease.revalidate()?;
        // Fixed directory handles and one-component names; never replace prior evidence.
        ensure!(
            unsafe {
                libc::linkat(
                    directory.as_raw_fd(),
                    temp_name.as_ptr(),
                    directory.as_raw_fd(),
                    final_name.as_ptr(),
                    0,
                )
            } == 0,
            "Cannot publish immutable metadata: {}",
            std::io::Error::last_os_error()
        );
        Ok(())
    })();
    unlink(&directory, &temp_name);
    result?;
    directory.sync_all()?;
    manifest::full_sync(&file)?;
    lease.revalidate()
}

// Only the verified-session API calls this outside tests. Failure can leave a
// complete unreferenced generation; never remove history or guess the latest file.
pub(crate) fn publish(lease: &DriveLease, inventory: &Manifest) -> Result<Current> {
    publish_with_hook(lease, inventory, || Ok(()))
}
fn publish_with_hook(
    lease: &DriveLease,
    inventory: &Manifest,
    after_generation: impl FnOnce() -> Result<()>,
) -> Result<Current> {
    validate_inventory(lease, inventory)?;
    load(lease)?; // Corrupt existing authority blocks replacement; preserve evidence.
    let directory = lease.catalog_directory()?;
    let final_name = generation_name(&inventory.header.generation)?;
    let (temp_name, file) = temporary(&directory)?;
    let result = (|| -> Result<()> {
        let mut writer = BufWriter::new(file);
        inventory.write_to(&mut writer)?;
        manifest::full_sync(writer.get_ref())?;
        lease.revalidate()?;
        ensure!(
            unsafe {
                libc::linkat(
                    directory.as_raw_fd(),
                    temp_name.as_ptr(),
                    directory.as_raw_fd(),
                    final_name.as_ptr(),
                    0,
                )
            } == 0,
            "Cannot publish immutable catalog: {}",
            std::io::Error::last_os_error()
        );
        Ok(())
    })();
    unlink(&directory, &temp_name);
    result?;
    directory.sync_all()?;
    // Sync file again after directory persistence, requesting macOS full flush.
    manifest::full_sync(&open_file(&directory, Path::new(final_name.to_str()?))?)?;
    after_generation()?;
    let current = Current {
        schema: 1,
        enrollment_id: lease.enrollment().enrollment_id.clone(),
        volume_uuid: lease.enrollment().volume_uuid.clone(),
        generation: inventory.header.generation.clone(),
    };
    let (temp_name, mut file) = temporary(&directory)?;
    let result = (|| -> Result<()> {
        serde_json::to_writer(&mut file, &current)?;
        file.write_all(b"\n")?;
        manifest::full_sync(&file)?;
        lease.revalidate()?;
        // Revalidate existing pointer immediately before replacement as well.
        load(lease)?;
        ensure!(
            unsafe {
                libc::renameat(
                    directory.as_raw_fd(),
                    temp_name.as_ptr(),
                    directory.as_raw_fd(),
                    c"CURRENT".as_ptr(),
                )
            } == 0,
            "Cannot commit CURRENT: {}",
            std::io::Error::last_os_error()
        );
        directory.sync_all()?;
        manifest::full_sync(&file)?;
        Ok(())
    })();
    unlink(&directory, &temp_name);
    result?;
    lease.revalidate()?;
    Ok(current)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{filesystem::Volume, scan};
    struct Fixture {
        path: std::path::PathBuf,
        lease: DriveLease,
    }
    impl Fixture {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("safesync-catalog-{}", manifest::generation()));
            std::fs::create_dir(&path).unwrap();
            let path = path.canonicalize().unwrap();
            let lease = DriveLease::test_lease(&path).unwrap();
            std::fs::write(path.join("media"), b"original").unwrap();
            Self { path, lease }
        }
        fn scan(&self) -> Manifest {
            scan::scan(
                &self.path,
                Volume {
                    uuid: self.lease.enrollment().volume_uuid.clone(),
                    name: "Test".into(),
                    filesystem: "apfs".into(),
                },
                true,
                |_| {},
            )
            .unwrap()
        }
        fn metadata(&self, name: &str) -> std::path::PathBuf {
            self.path.join(".safesync").join(name)
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
    #[test]
    fn publishes_and_retains_generations_without_changing_media() {
        let f = Fixture::new();
        assert!(load(&f.lease).unwrap().is_none());
        let first = f.scan();
        let pointer = publish(&f.lease, &first).unwrap();
        assert_eq!(load(&f.lease).unwrap().unwrap().0, pointer);
        let second = f.scan();
        publish(&f.lease, &second).unwrap();
        assert_eq!(
            load(&f.lease).unwrap().unwrap().0.generation,
            second.header.generation
        );
        assert!(
            f.metadata(&format!("catalog-{}.jsonl", first.header.generation))
                .is_file()
        );
        assert_eq!(std::fs::read(f.path.join("media")).unwrap(), b"original");
        assert!(publish(&f.lease, &first).is_err()); // immutable generations never overwritten
        assert_eq!(
            load(&f.lease).unwrap().unwrap().0.generation,
            second.header.generation
        );
    }
    #[test]
    fn interrupted_publication_keeps_previous_authority() {
        let f = Fixture::new();
        let first = f.scan();
        publish(&f.lease, &first).unwrap();
        let second = f.scan();
        assert!(
            publish_with_hook(&f.lease, &second, || anyhow::bail!(
                "simulated interruption"
            ))
            .is_err()
        );
        assert_eq!(
            load(&f.lease).unwrap().unwrap().0.generation,
            first.header.generation
        );
        let orphan = f.metadata(&format!("catalog-{}.jsonl", second.header.generation));
        Manifest::load(&orphan).unwrap();
        // Newer unreferenced generations are never selected by timestamp.
        let third = f.scan();
        publish(&f.lease, &third).unwrap();
        assert_eq!(
            load(&f.lease).unwrap().unwrap().0.generation,
            third.header.generation
        );
        assert!(orphan.exists());
    }
    #[test]
    fn rejects_offline_wrong_volume_scope_and_generation() {
        let f = Fixture::new();
        for change in 0..5 {
            let mut inventory = f.scan();
            match change {
                0 => inventory.header.role = Role::OfflineSnapshot,
                1 => inventory.header.volume.uuid = "other".into(),
                2 => inventory.header.root_base64 = manifest::encode_path(Path::new("/other")),
                3 => inventory.header.root_file_id += 1,
                _ => inventory.header.generation = "../escape".into(),
            }
            assert!(publish(&f.lease, &inventory).is_err());
            assert!(load(&f.lease).unwrap().is_none());
        }
    }
    #[test]
    fn corrupt_or_symlinked_authority_is_preserved_not_replaced() {
        let f = Fixture::new();
        publish(&f.lease, &f.scan()).unwrap();
        let current = f.metadata("CURRENT");
        let original = std::fs::read(&current).unwrap();
        std::fs::write(&current, b"broken").unwrap();
        assert!(load(&f.lease).is_err());
        assert!(publish(&f.lease, &f.scan()).is_err());
        assert_eq!(std::fs::read(&current).unwrap(), b"broken");
        std::fs::remove_file(&current).unwrap();
        let outside = f.path.join("outside");
        std::fs::write(&outside, &original).unwrap();
        std::os::unix::fs::symlink(&outside, &current).unwrap();
        assert!(publish(&f.lease, &f.scan()).is_err());
        assert_eq!(std::fs::read(&outside).unwrap(), original);
    }
    #[test]
    fn detects_generation_corruption_and_pointer_identity_tampering() {
        let f = Fixture::new();
        let inventory = f.scan();
        let mut current = publish(&f.lease, &inventory).unwrap();
        current.enrollment_id = "other".into();
        std::fs::write(f.metadata("CURRENT"), serde_json::to_vec(&current).unwrap()).unwrap();
        assert!(load(&f.lease).is_err());
        current.enrollment_id = f.lease.enrollment().enrollment_id.clone();
        std::fs::write(f.metadata("CURRENT"), serde_json::to_vec(&current).unwrap()).unwrap();
        std::fs::write(
            f.metadata(&format!("catalog-{}.jsonl", current.generation)),
            b"truncated\n",
        )
        .unwrap();
        assert!(load(&f.lease).is_err());
        assert!(publish(&f.lease, &f.scan()).is_err());
    }
}
