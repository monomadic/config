//! Watching a job's own folder, so that moving it is how you command it.
//!
//! The runner holds an open descriptor on the run folder for as long as the
//! job lasts. A descriptor follows the directory through renames, so:
//!
//!   * `kqueue` with `NOTE_RENAME` on that descriptor fires the moment someone
//!     moves the folder — from Finder, from a script, or from a menu bar app
//!     on another machine over SMB. Nothing polls.
//!   * `F_GETPATH` on the same descriptor answers "where is this *now*", so
//!     the runner learns the destination without scanning anything.
//!
//! The destination directory is the verb. `_paused` means stop the process
//! group, `_running` means resume it, anything else means the job is over.

use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::path::{Path, PathBuf};

/// An open handle on a run folder that reports where it has been moved to.
pub struct FolderWatch {
    dir: OwnedFd,
    queue: OwnedFd,
}

impl FolderWatch {
    pub fn open(path: &Path) -> Option<FolderWatch> {
        let c_path = std::ffi::CString::new(path.as_os_str().as_encoded_bytes()).ok()?;
        // O_EVTONLY is the macOS flag for "I only want to watch this" — it
        // doesn't count as a reference that would keep a volume busy.
        let dir = unsafe { libc::open(c_path.as_ptr(), libc::O_EVTONLY) };
        if dir < 0 {
            return None;
        }
        let dir = unsafe { OwnedFd::from_raw_fd(dir) };

        let queue = unsafe { libc::kqueue() };
        if queue < 0 {
            return None;
        }
        let queue = unsafe { OwnedFd::from_raw_fd(queue) };

        let event = libc::kevent {
            ident: dir.as_raw_fd() as usize,
            filter: libc::EVFILT_VNODE,
            flags: libc::EV_ADD | libc::EV_CLEAR,
            // DELETE too: a folder can be moved to another filesystem, which
            // arrives as a delete rather than a rename.
            fflags: libc::NOTE_RENAME | libc::NOTE_DELETE,
            data: 0,
            udata: std::ptr::null_mut(),
        };
        let registered = unsafe {
            libc::kevent(
                queue.as_raw_fd(),
                &event,
                1,
                std::ptr::null_mut(),
                0,
                std::ptr::null(),
            )
        };
        (registered >= 0).then_some(FolderWatch { dir, queue })
    }

    /// Wait up to `timeout` for the folder to be moved. Returns its new path
    /// if it was, `None` if the wait simply timed out.
    ///
    /// The timeout is what lets the caller interleave this with waiting on the
    /// child process, without a second thread to coordinate.
    pub fn moved(&self, timeout: std::time::Duration) -> Option<PathBuf> {
        let mut event = libc::kevent {
            ident: 0,
            filter: 0,
            flags: 0,
            fflags: 0,
            data: 0,
            udata: std::ptr::null_mut(),
        };
        let spec = libc::timespec {
            tv_sec: timeout.as_secs() as i64,
            tv_nsec: timeout.subsec_nanos() as i64,
        };
        let fired = unsafe {
            libc::kevent(
                self.queue.as_raw_fd(),
                std::ptr::null(),
                0,
                &mut event,
                1,
                &spec,
            )
        };
        (fired > 0).then(|| self.path()).flatten()
    }

    /// Where the folder lives now, following every rename since it was opened.
    pub fn path(&self) -> Option<PathBuf> {
        let mut buffer = [0i8; libc::PATH_MAX as usize];
        let ok = unsafe { libc::fcntl(self.dir.as_raw_fd(), libc::F_GETPATH, buffer.as_mut_ptr()) };
        if ok < 0 {
            return None;
        }
        let bytes: Vec<u8> = buffer
            .iter()
            .take_while(|byte| **byte != 0)
            .map(|byte| *byte as u8)
            .collect();
        Some(PathBuf::from(String::from_utf8_lossy(&bytes).to_string()))
    }
}

/// Rename that refuses to overwrite. Plain `rename(2)` clobbers its
/// destination silently, which makes it useless as a claim: two runners
/// racing the same job would both "succeed". `RENAME_EXCL` fails with EEXIST
/// instead, giving the same all-or-nothing guarantee `mkdir` has.
pub fn rename_exclusive(from: &Path, to: &Path) -> bool {
    let Ok(from) = std::ffi::CString::new(from.as_os_str().as_encoded_bytes()) else {
        return false;
    };
    let Ok(to) = std::ffi::CString::new(to.as_os_str().as_encoded_bytes()) else {
        return false;
    };
    unsafe {
        libc::renamex_np(
            from.as_ptr(),
            to.as_ptr(),
            libc::RENAME_EXCL,
        ) == 0
    }
}
