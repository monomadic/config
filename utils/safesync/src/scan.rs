use crate::{
    filesystem::{self, Stamp, Volume},
    manifest::{Entry, Header, Manifest, Role, SCHEMA, encode_path, generation, now},
};
use anyhow::{Context, Result, ensure};
use std::{
    fs::{self, File},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::{Path, PathBuf},
};

pub struct Progress {
    pub files: usize,
    pub bytes: u64,
}

// No ignored errors, hidden-file rules, or incomplete manifest publication.
// The only implicit scope exclusions are our metadata and non-regular objects.
pub fn scan(
    root: &Path,
    volume: Volume,
    hash: bool,
    progress: impl FnMut(Progress),
) -> Result<Manifest> {
    scan_with_exclusions(root, volume, hash, &[], progress)
}

pub fn scan_with_exclusions(
    root: &Path,
    volume: Volume,
    hash: bool,
    exclusions: &[PathBuf],
    mut progress: impl FnMut(Progress),
) -> Result<Manifest> {
    for excluded in exclusions {
        ensure!(
            !excluded.as_os_str().is_empty()
                && excluded
                    .components()
                    .all(|c| matches!(c, std::path::Component::Normal(_))),
            "Exclusions must be relative paths without . or .."
        );
    }
    let root = root.canonicalize().context("Cannot resolve scan root")?;
    let directory = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW)
        .open(&root)?;
    let root_metadata = directory.metadata()?;
    let started = now();
    let device = root_metadata.dev();
    let mut directories = vec![(PathBuf::new(), Stamp::of(&root_metadata))];
    let mut queue = vec![PathBuf::new()];
    let mut entries = Vec::new();
    let mut skipped_symlinks = 0;
    let mut skipped_special = 0;
    let mut skipped_mounts = 0;
    let mut bytes = 0;
    while let Some(relative) = queue.pop() {
        let parent = if relative.as_os_str().is_empty() {
            directory.try_clone()?
        } else {
            filesystem::open_relative(&directory, &relative, device)?
        };
        for name in
            filesystem::names(&parent).with_context(|| format!("Cannot list {:?}", relative))?
        {
            if name == ".safesync" {
                continue;
            }
            let child = relative.join(&name);
            if exclusions
                .iter()
                .any(|excluded| child.starts_with(excluded))
            {
                continue;
            }
            let (mode, child_device) = filesystem::child_mode(&parent, &name)?;
            if child_device != device {
                skipped_mounts += 1;
                continue;
            }
            if mode == libc::S_IFLNK as u32 {
                skipped_symlinks += 1;
                continue;
            }
            if mode != libc::S_IFDIR as u32 && mode != libc::S_IFREG as u32 {
                skipped_special += 1;
                continue;
            }
            let mut opened = filesystem::open_relative(&directory, &child, device)
                .with_context(|| format!("Cannot read {:?}", child))?;
            let metadata = opened.metadata()?;
            let stamp = Stamp::of(&metadata);
            if mode == libc::S_IFDIR as u32 {
                ensure!(metadata.is_dir(), "Entry type changed while scanning");
                directories.push((child.clone(), stamp));
                queue.push(child);
            } else {
                ensure!(metadata.is_file(), "Entry type changed while scanning");
                let sha256 = if hash {
                    Some(
                        filesystem::hash_file(&mut opened, &stamp)
                            .with_context(|| format!("Cannot fingerprint {:?}", child))?,
                    )
                } else {
                    None
                };
                bytes += stamp.size;
                entries.push(Entry {
                    path_base64: encode_path(&child),
                    stamp,
                    sha256,
                });
                progress(Progress {
                    files: entries.len(),
                    bytes,
                });
            }
        }
    }
    // Catch changes during the scan rather than presenting a partial tree as a
    // complete observation. This is not a filesystem snapshot or a write lock.
    for (relative, expected) in &directories {
        let opened = if relative.as_os_str().is_empty() {
            directory.try_clone()?
        } else {
            filesystem::open_relative(&directory, relative, device)?
        };
        ensure!(
            Stamp::of(&opened.metadata()?) == *expected,
            "Directory changed during scan: {:?}; retry with a quiet source",
            relative
        );
    }
    for entry in &entries {
        let opened = filesystem::open_relative(&directory, &entry.path()?, device)?;
        ensure!(
            Stamp::of(&opened.metadata()?) == entry.stamp,
            "File changed during scan; no manifest was committed"
        );
    }
    let current_root = File::open(&root)?.metadata()?;
    ensure!(
        current_root.dev() == device && current_root.ino() == root_metadata.ino(),
        "Scan root was replaced"
    );
    entries.sort_by(|a, b| a.path_base64.cmp(&b.path_base64));
    Ok(Manifest {
        header: Header {
            schema: SCHEMA,
            generation: generation(),
            role: Role::Inventory,
            volume,
            root_base64: encode_path(&root),
            root_file_id: root_metadata.ino(),
            started_unix: started,
            finished_unix: now(),
            hash_algorithm: "sha256".into(),
            content_hashed: hash,
            exclusions: vec![
                "Any directory or file named .safesync".into(),
                "Symlinks and special files".into(),
                "Other mounted filesystems".into(),
            ]
            .into_iter()
            .chain(
                exclusions
                    .iter()
                    .map(|p| format!("Relative subtree: {:?}", p)),
            )
            .collect(),
            skipped_symlinks,
            skipped_special,
            skipped_mounts,
        },
        entries,
    })
}
