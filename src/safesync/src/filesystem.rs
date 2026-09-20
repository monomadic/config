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
    path::{Component, Path},
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
