use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    ffi::CString,
    fs::{self, File, Metadata},
    io::{Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::{
            ffi::OsStrExt,
            fs::{MetadataExt, OpenOptionsExt},
        },
    },
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Stamp {
    pub device: u64,
    pub file_id: u64,
    pub size: u64,
    pub mtime_seconds: i64,
    pub mtime_nanos: i64,
    pub ctime_seconds: i64,
    pub ctime_nanos: i64,
}
impl Stamp {
    pub fn of(m: &Metadata) -> Self {
        Self {
            device: m.dev(),
            file_id: m.ino(),
            size: m.len(),
            mtime_seconds: m.mtime(),
            mtime_nanos: m.mtime_nsec(),
            ctime_seconds: m.ctime(),
            ctime_nanos: m.ctime_nsec(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Volume {
    pub uuid: String,
    pub name: String,
    pub filesystem: String,
}

pub fn volume_for(root: &Path) -> Result<Volume> {
    // diskutil expects a device or mount point, not an arbitrary subdirectory.
    // Resolve the actual containing mount through the opened directory's fd.
    let directory = File::open(root).context("Cannot open scan root")?;
    let mut mount = std::mem::MaybeUninit::<libc::statfs>::uninit();
    let result = unsafe { libc::fstatfs(directory.as_raw_fd(), mount.as_mut_ptr()) };
    ensure!(
        result == 0,
        "Cannot identify containing filesystem: {}",
        std::io::Error::last_os_error()
    );
    let mount = unsafe { mount.assume_init() };
    let mount_path = unsafe { std::ffi::CStr::from_ptr(mount.f_mntonname.as_ptr()) };
    let info = disk_info(Path::new(std::ffi::OsStr::from_bytes(
        mount_path.to_bytes(),
    )))?;
    let uuid = info["VolumeUUID"]
        .as_str()
        .context("No stable volume UUID; refusing to invent an identity")?;
    ensure!(!uuid.is_empty(), "Empty volume UUID");
    Ok(Volume {
        uuid: uuid.into(),
        name: info["VolumeName"]
            .as_str()
            .unwrap_or("Unnamed volume")
            .into(),
        filesystem: info["FilesystemType"].as_str().unwrap_or("unknown").into(),
    })
}

/// A mounted filesystem as `drives` lists it: every child of `/Volumes` that is
/// a real mount point, plus the boot volume. Volumes without a stable UUID
/// (network mounts, some disk images) come back with no `volume`.
#[derive(Clone, Debug)]
pub struct Mount {
    pub path: PathBuf,
    pub volume: Option<Volume>,
    pub filesystem: String,
    pub total: u64,
    pub free: u64,
    pub writable: bool,
    pub boot: bool,
}

pub fn mounts() -> Vec<Mount> {
    let mut paths = vec![PathBuf::from("/")];
    let root_device = fs::metadata("/Volumes").map(|m| m.dev()).ok();
    if let Ok(entries) = fs::read_dir("/Volumes") {
        let mut children: Vec<PathBuf> = entries
            .flatten()
            .filter(|e| e.file_type().is_ok_and(|t| t.is_dir() && !t.is_symlink()))
            .map(|e| e.path())
            // A plain directory under /Volumes shares its device; a mount does not.
            .filter(|p| {
                fs::metadata(p)
                    .map(|m| Some(m.dev()) != root_device)
                    .unwrap_or(false)
            })
            .collect();
        children.sort();
        paths.extend(children);
    }
    // `diskutil info` takes about 150 ms a volume; ask about them all at once.
    let infos: Vec<_> = std::thread::scope(|scope| {
        let handles: Vec<_> = paths
            .iter()
            .map(|path| scope.spawn(move || disk_info(path).ok()))
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().unwrap_or_default())
            .collect()
    });
    paths
        .into_iter()
        .zip(infos)
        .filter_map(|(path, info)| {
            let (free, total) = crate::copy::space(&path).ok()?;
            let str_key = |key: &str| {
                info.as_ref()
                    .and_then(|i| i[key].as_str())
                    .map(str::to_owned)
            };
            let name = str_key("VolumeName")
                .or_else(|| path.file_name().map(|n| n.to_string_lossy().into_owned()))
                .unwrap_or_else(|| "/".into());
            let filesystem = str_key("FilesystemType")
                .or_else(|| str_key("FilesystemUserVisibleName"))
                .unwrap_or_else(|| "unknown".into());
            let volume = str_key("VolumeUUID")
                .filter(|uuid| !uuid.is_empty())
                .map(|uuid| Volume {
                    uuid,
                    name: name.clone(),
                    filesystem: filesystem.clone(),
                });
            let writable = info
                .as_ref()
                .and_then(|i| i["WritableVolume"].as_bool())
                .unwrap_or(true);
            Some(Mount {
                boot: path == Path::new("/"),
                path,
                volume,
                filesystem,
                total,
                free,
                writable,
            })
        })
        .collect()
}

pub(crate) fn disk_info(path: &Path) -> Result<serde_json::Value> {
    let result = Command::new("/usr/sbin/diskutil")
        .args(["info", "-plist"])
        .arg(path)
        .output()
        .context("Cannot inspect volume with diskutil")?;
    ensure!(
        result.status.success(),
        "diskutil failed: {} {}",
        String::from_utf8_lossy(&result.stderr),
        String::from_utf8_lossy(&result.stdout)
    );
    let mut child = Command::new("/usr/bin/plutil")
        .args(["-convert", "json", "-o", "-", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .context("plutil stdin unavailable")?
        .write_all(&result.stdout)?;
    let result = child.wait_with_output()?;
    ensure!(result.status.success(), "Cannot decode volume information");
    let info: serde_json::Value = serde_json::from_slice(&result.stdout)?;
    Ok(info)
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn hash_file(file: &mut File, expected: &Stamp) -> Result<String> {
    hash_file_with(file, expected, |_| {})
}

/// `hash_file`, calling `on_read` with each chunk's length so a caller can
/// show progress through a multi-gigabyte file.
pub fn hash_file_with(
    file: &mut File,
    expected: &Stamp,
    mut on_read: impl FnMut(u64),
) -> Result<String> {
    ensure!(file.metadata()?.is_file(), "Not a regular file");
    ensure!(
        Stamp::of(&file.metadata()?) == *expected,
        "File changed before hashing"
    );
    let mut hash = Sha256::new();
    let mut buffer = vec![0; 1024 * 1024];
    let mut read = 0_u64;
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        read += n as u64;
        hash.update(&buffer[..n]);
        on_read(n as u64);
    }
    ensure!(
        read == expected.size && Stamp::of(&file.metadata()?) == *expected,
        "File changed while hashing; no trustworthy fingerprint was produced"
    );
    Ok(hex(&hash.finalize()))
}

pub fn hash_path(path: &Path) -> Result<(Stamp, String)> {
    let mut file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .with_context(|| format!("Cannot open file {:?}", path))?;
    let stamp = Stamp::of(&file.metadata()?);
    let digest = hash_file(&mut file, &stamp)?;
    Ok((stamp, digest))
}

// Walk through directory descriptors. A symlink or replacement mount must not
// redirect a scan to unrelated files. No writes are exposed by this interface.
pub fn open_relative(root: &File, relative: &Path, device: u64) -> Result<File> {
    open_relative_optional(root, relative, device)?.context("Relative path does not exist")
}

/// An opened media root. Every descendant is resolved without following links
/// or crossing mounts, including directories created for a copy or history.
pub struct Root {
    directory: File,
    device: u64,
}

pub struct FilePath {
    parent: File,
    name: CString,
    device: u64,
}

impl Root {
    pub fn open(path: &Path) -> Result<Self> {
        let directory = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path)?;
        let device = directory.metadata()?.dev();
        Ok(Self { directory, device })
    }

    pub fn file(&self, relative: &Path, create_parents: bool) -> Result<FilePath> {
        let parts: Vec<_> = relative.components().collect();
        ensure!(
            !parts.is_empty() && parts.iter().all(|p| matches!(p, Component::Normal(_))),
            "Unsafe relative file path"
        );
        let mut parent = self.directory.try_clone()?;
        for component in &parts[..parts.len() - 1] {
            let name = CString::new(component.as_os_str().as_bytes())?;
            let flags = libc::O_RDONLY | libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC;
            let mut fd = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags) };
            if fd < 0
                && create_parents
                && std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT)
            {
                let result = unsafe { libc::mkdirat(parent.as_raw_fd(), name.as_ptr(), 0o755) };
                ensure!(
                    result == 0
                        || std::io::Error::last_os_error().raw_os_error() == Some(libc::EEXIST),
                    "Cannot create directory: {}",
                    std::io::Error::last_os_error()
                );
                fd = unsafe { libc::openat(parent.as_raw_fd(), name.as_ptr(), flags) };
            }
            ensure!(
                fd >= 0,
                "Cannot safely open parent of {relative:?}: {}",
                std::io::Error::last_os_error()
            );
            let next = unsafe { File::from_raw_fd(fd) };
            ensure!(
                next.metadata()?.dev() == self.device,
                "Path crossed into another mounted filesystem"
            );
            parent = next;
        }
        Ok(FilePath {
            parent,
            name: CString::new(parts.last().unwrap().as_os_str().as_bytes())?,
            device: self.device,
        })
    }

    pub fn remove_empty_parents(&self, relative: &Path) {
        let mut current = relative.parent();
        while let Some(path) = current.filter(|p| !p.as_os_str().is_empty()) {
            let Ok(directory) = self.file(path, false) else {
                break;
            };
            if directory.unlink(libc::AT_REMOVEDIR).is_err() {
                break;
            }
            current = path.parent();
        }
    }
}

impl FilePath {
    pub fn open(&self) -> Result<File> {
        let fd = unsafe {
            libc::openat(
                self.parent.as_raw_fd(),
                self.name.as_ptr(),
                libc::O_RDONLY | libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC,
            )
        };
        ensure!(
            fd >= 0,
            "Cannot safely open file: {}",
            std::io::Error::last_os_error()
        );
        let file = unsafe { File::from_raw_fd(fd) };
        let metadata = file.metadata()?;
        ensure!(metadata.is_file(), "Not a regular file");
        ensure!(
            metadata.dev() == self.device,
            "File crossed into another mounted filesystem"
        );
        Ok(file)
    }

    pub fn ensure_absent(&self) -> Result<()> {
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        let result = unsafe {
            libc::fstatat(
                self.parent.as_raw_fd(),
                self.name.as_ptr(),
                stat.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        ensure!(result != 0, "Destination already exists");
        ensure!(
            std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT),
            "Cannot inspect destination: {}",
            std::io::Error::last_os_error()
        );
        Ok(())
    }

    pub fn temporary(&self) -> Result<(Self, File)> {
        let temp = Self {
            parent: self.parent.try_clone()?,
            name: CString::new(format!(
                "{}{}",
                crate::copy::PARTIAL_PREFIX,
                crate::manifest::generation()
            ))?,
            device: self.device,
        };
        let fd = unsafe {
            libc::openat(
                temp.parent.as_raw_fd(),
                temp.name.as_ptr(),
                libc::O_RDWR | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                0o600,
            )
        };
        ensure!(
            fd >= 0,
            "Cannot create partial file: {}",
            std::io::Error::last_os_error()
        );
        Ok((temp, unsafe { File::from_raw_fd(fd) }))
    }

    pub fn rename_to(&self, destination: &Self) -> Result<()> {
        ensure!(
            self.device == destination.device,
            "Cannot move between filesystems"
        );
        let result = unsafe {
            libc::renameatx_np(
                self.parent.as_raw_fd(),
                self.name.as_ptr(),
                destination.parent.as_raw_fd(),
                destination.name.as_ptr(),
                libc::RENAME_EXCL,
            )
        };
        ensure!(
            result == 0,
            "Cannot move file (existing destinations are never overwritten): {}",
            std::io::Error::last_os_error()
        );
        Ok(())
    }

    fn unlink(&self, flags: i32) -> Result<()> {
        let result = unsafe { libc::unlinkat(self.parent.as_raw_fd(), self.name.as_ptr(), flags) };
        ensure!(
            result == 0,
            "Cannot remove entry: {}",
            std::io::Error::last_os_error()
        );
        Ok(())
    }

    pub fn remove(&self) -> Result<()> {
        self.unlink(0)
    }
}

// Only ENOENT means absent. Symlinks, permission errors and non-directory parents
// must never be mistaken for a free destination path.
pub(crate) fn open_relative_optional(
    root: &File,
    relative: &Path,
    device: u64,
) -> Result<Option<File>> {
    let parts: Vec<_> = relative.components().collect();
    ensure!(!parts.is_empty(), "Empty relative file path");
    let mut current = root.try_clone()?;
    for (index, component) in parts.iter().enumerate() {
        let Component::Normal(name) = component else {
            bail!("Unsafe relative path");
        };
        let name = CString::new(name.as_bytes())?;
        let directory = index + 1 < parts.len();
        let flags = libc::O_RDONLY
            | libc::O_NOFOLLOW
            | libc::O_CLOEXEC
            | libc::O_NONBLOCK
            | if directory { libc::O_DIRECTORY } else { 0 };
        // SAFETY: name is NUL-terminated, fd is live, and successful fd ownership
        // is transferred to File exactly once.
        let fd = unsafe { libc::openat(current.as_raw_fd(), name.as_ptr(), flags) };
        if fd < 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::ENOENT) {
            return Ok(None);
        }
        ensure!(
            fd >= 0,
            "Cannot safely open {:?}: {}",
            relative,
            std::io::Error::last_os_error()
        );
        let next = unsafe { File::from_raw_fd(fd) };
        ensure!(
            next.metadata()?.dev() == device,
            "File crossed into another mounted filesystem"
        );
        current = next;
    }
    Ok(Some(current))
}

pub fn names(directory: &File) -> Result<Vec<std::ffi::OsString>> {
    // fdopendir takes ownership of the duplicate, keeping traversal anchored to
    // the opened directory even if a parent pathname changes during the scan.
    let fd = unsafe { libc::dup(directory.as_raw_fd()) };
    ensure!(fd >= 0, "Cannot duplicate directory handle");
    let stream = unsafe { libc::fdopendir(fd) };
    if stream.is_null() {
        unsafe {
            libc::close(fd);
        }
        bail!("Cannot enumerate directory");
    }
    let result = (|| -> Result<Vec<std::ffi::OsString>> {
        use std::os::unix::ffi::OsStringExt;
        let mut names = Vec::new();
        loop {
            unsafe {
                *libc::__error() = 0;
            }
            let entry = unsafe { libc::readdir(stream) };
            if entry.is_null() {
                let error = std::io::Error::last_os_error();
                ensure!(
                    error.raw_os_error() == Some(0),
                    "Directory read failed: {error}"
                );
                break;
            }
            let name = unsafe { std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()) }.to_bytes();
            if name != b"." && name != b".." {
                names.push(std::ffi::OsString::from_vec(name.to_vec()));
            }
        }
        names.sort();
        Ok(names)
    })();
    unsafe {
        libc::closedir(stream);
    }
    result
}

pub fn child_mode(directory: &File, name: &std::ffi::OsStr) -> Result<(u32, u64)> {
    let name = CString::new(name.as_bytes())?;
    let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
    let result = unsafe {
        libc::fstatat(
            directory.as_raw_fd(),
            name.as_ptr(),
            stat.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    };
    ensure!(
        result == 0,
        "Cannot inspect directory entry: {}",
        std::io::Error::last_os_error()
    );
    let stat = unsafe { stat.assume_init() };
    Ok(((stat.st_mode & libc::S_IFMT) as u32, stat.st_dev as u64))
}
