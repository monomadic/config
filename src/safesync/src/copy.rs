//! One file, copied the way big video wants to be copied: uncached, preallocated,
//! read and written on separate threads, hashed on the way through, published
//! under its real name only once it is complete. Never overwrites.
use crate::{
    filesystem::{Stamp, hex},
    manifest::Entry,
};
use anyhow::{Context, Result, bail, ensure};
use sha2::{Digest, Sha256};
use std::{
    ffi::CString,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::{
        fd::AsRawFd,
        unix::{ffi::OsStrExt, fs::OpenOptionsExt},
    },
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
};

const BUFFER: usize = 8 << 20;
const BUFFERS: usize = 3; // one being filled, one in flight, one being written
const ALIGN: usize = 16 << 10;
pub const PARTIAL_PREFIX: &str = ".safesync-part-";

pub struct Copied {
    pub sha256: String,
    /// The published destination file, for the destination's index.
    pub stamp: Stamp,
}

// With F_NOCACHE the kernel talks to the device directly; an unaligned buffer
// makes it bounce the data through an intermediate copy first.
struct Aligned(Vec<u8>, usize);
impl Aligned {
    fn new() -> Self {
        let raw = vec![0; BUFFER + ALIGN];
        let offset = (ALIGN - raw.as_ptr() as usize % ALIGN) % ALIGN;
        Self(raw, offset)
    }
    fn bytes(&mut self) -> &mut [u8] {
        &mut self.0[self.1..self.1 + BUFFER]
    }
}

fn no_cache(file: &File) {
    // Best effort: gigabytes of video would otherwise evict everything else
    // from RAM and make the reported write speed a fiction.
    unsafe { libc::fcntl(file.as_raw_fd(), libc::F_NOCACHE, 1) };
}

// Reserve the blocks up front: less fragmentation, and "the disk is full"
// arrives before the first byte rather than six gigabytes in.
fn preallocate(file: &File, size: u64) -> Result<()> {
    if size == 0 {
        return Ok(());
    }
    let mut store = libc::fstore_t {
        fst_flags: libc::F_ALLOCATECONTIG,
        fst_posmode: libc::F_PEOFPOSMODE,
        fst_offset: 0,
        fst_length: size as libc::off_t,
        fst_bytesalloc: 0,
    };
    let mut result = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_PREALLOCATE, &mut store) };
    if result == -1 {
        store.fst_flags = libc::F_ALLOCATEALL;
        result = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_PREALLOCATE, &mut store) };
    }
    let error = std::io::Error::last_os_error();
    // A filesystem that cannot preallocate is not a problem; a full one is.
    ensure!(
        result != -1 || error.raw_os_error() != Some(libc::ENOSPC),
        "Not enough space on the destination"
    );
    Ok(())
}

fn c_path(path: &Path) -> Result<CString> {
    Ok(CString::new(path.as_os_str().as_bytes())?)
}

/// Rename that fails instead of replacing whatever is at `to`.
pub fn rename_exclusive(from: &Path, to: &Path) -> Result<()> {
    let result =
        unsafe { libc::renamex_np(c_path(from)?.as_ptr(), c_path(to)?.as_ptr(), libc::RENAME_EXCL) };
    ensure!(
        result == 0,
        "Cannot move {:?} to {:?}: {}",
        from,
        to,
        std::io::Error::last_os_error()
    );
    Ok(())
}

/// Copies `source` to `destination`, which must not exist. `expected` is the
/// source's index entry: a file whose size or mtime has moved on is not copied,
/// and one that no longer reads back as its recorded fingerprint is not published. `progress` receives cumulative bytes written.
pub fn copy_file(
    source: &Path,
    destination: &Path,
    expected: Option<&Entry>,
    verify: bool,
    cancel: &AtomicBool,
    mut progress: impl FnMut(u64),
) -> Result<Copied> {
    let mut input = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(source)
        .with_context(|| format!("Cannot open {source:?}"))?;
    let before = Stamp::of(&input.metadata()?);
    ensure!(input.metadata()?.is_file(), "Not a regular file");
    if let Some(Entry { stamp: expected, .. }) = expected {
        ensure!(
            (before.size, before.mtime_seconds, before.mtime_nanos)
                == (expected.size, expected.mtime_seconds, expected.mtime_nanos),
            "Changed since it was indexed; scan again"
        );
    }
    ensure!(
        fs::symlink_metadata(destination).is_err(),
        "Destination already exists"
    );
    let parent = destination.parent().context("Destination has no parent")?;
    fs::create_dir_all(parent)?;
    let partial = parent.join(format!("{PARTIAL_PREFIX}{}", crate::manifest::generation()));
    let mut output = OpenOptions::new()
        .write(true)
        .read(true)
        .create_new(true)
        .mode(0o600)
        .open(&partial)?;

    let result = (|| -> Result<Copied> {
        no_cache(&input);
        no_cache(&output);
        preallocate(&output, before.size)?;

        // Buffers cycle between the threads so neither drive idles while the
        // other works; a serial loop lands at the harmonic mean of the two.
        let (free_tx, free_rx) = mpsc::sync_channel::<Aligned>(BUFFERS);
        let (filled_tx, filled_rx) = mpsc::sync_channel::<(Aligned, usize)>(BUFFERS);
        for _ in 0..BUFFERS {
            free_tx.send(Aligned::new()).expect("receiver is alive");
        }
        let unchanged = before.clone();
        let digest = std::thread::scope(|scope| -> Result<String> {
            let reader = scope.spawn(move || -> Result<String> {
                let mut hash = Sha256::new();
                while let Ok(mut buffer) = free_rx.recv() {
                    let mut filled = 0;
                    while filled < BUFFER {
                        let n = input.read(&mut buffer.bytes()[filled..])?;
                        if n == 0 {
                            break;
                        }
                        filled += n;
                    }
                    if filled == 0 {
                        break;
                    }
                    hash.update(&buffer.bytes()[..filled]);
                    if filled_tx.send((buffer, filled)).is_err() || filled < BUFFER {
                        break;
                    }
                }
                ensure!(
                    Stamp::of(&input.metadata()?) == unchanged,
                    "Source changed while it was being copied"
                );
                Ok(hex(&hash.finalize()))
            });
            let mut written = 0_u64;
            let mut failure = None;
            for (mut buffer, filled) in &filled_rx {
                if cancel.load(Ordering::Relaxed) {
                    failure = Some(anyhow::anyhow!("Cancelled"));
                    break;
                }
                if let Err(error) = output.write_all(&buffer.bytes()[..filled]) {
                    failure = Some(error.into());
                    break;
                }
                written += filled as u64;
                progress(written);
                let _ = free_tx.send(buffer);
            }
            // Dropping both ends releases a reader blocked on either channel.
            drop(filled_rx);
            drop(free_tx);
            let digest = reader.join().expect("reader thread panicked");
            if let Some(failure) = failure {
                return Err(failure);
            }
            let digest = digest?;
            ensure!(written == before.size, "Short copy");
            Ok(digest)
        })?;

        if let Some(known) = expected.and_then(|entry| entry.sha256.as_ref()) {
            ensure!(
                *known == digest,
                "Read back differently from its indexed fingerprint — the source copy may be damaged"
            );
        }
        // Preallocation can leave the file longer than the data.
        output.set_len(before.size)?;
        // Finder tags and other xattrs, ACLs, flags, mode and timestamps. The
        // mtime matters most: it is how later scans recognise this file.
        let source_fd = File::open(source)?;
        let copied = unsafe {
            libc::fcopyfile(
                source_fd.as_raw_fd(),
                output.as_raw_fd(),
                std::ptr::null_mut(),
                libc::COPYFILE_ACL | libc::COPYFILE_STAT | libc::COPYFILE_XATTR,
            )
        };
        ensure!(
            copied == 0,
            "Cannot copy metadata: {}",
            std::io::Error::last_os_error()
        );
        output.sync_all()?;
        if verify {
            use std::io::Seek;
            output.rewind()?;
            let mut hash = Sha256::new();
            let mut buffer = Aligned::new();
            loop {
                if cancel.load(Ordering::Relaxed) {
                    bail!("Cancelled");
                }
                let n = output.read(buffer.bytes())?;
                if n == 0 {
                    break;
                }
                hash.update(&buffer.bytes()[..n]);
            }
            ensure!(
                hex(&hash.finalize()) == digest,
                "Verification failed: the destination does not read back as written"
            );
        }
        rename_exclusive(&partial, destination)?;
        Ok(Copied {
            sha256: digest,
            stamp: Stamp::of(&fs::symlink_metadata(destination)?),
        })
    })();
    if result.is_err() {
        let _ = fs::remove_file(&partial);
    }
    result
}

/// (available, total) bytes of the volume holding `path`.
pub fn space(path: &Path) -> Result<(u64, u64)> {
    let mut stat = std::mem::MaybeUninit::<libc::statfs>::uninit();
    let result = unsafe { libc::statfs(c_path(path)?.as_ptr(), stat.as_mut_ptr()) };
    ensure!(
        result == 0,
        "Cannot read free space: {}",
        std::io::Error::last_os_error()
    );
    let stat = unsafe { stat.assume_init() };
    let block = stat.f_bsize as u64;
    Ok((stat.f_bavail * block, stat.f_blocks * block))
}
