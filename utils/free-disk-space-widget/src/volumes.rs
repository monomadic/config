//! The mount table, read through Foundation rather than `df`.
//!
//! `NSFileManager` enumerates exactly the volumes Finder shows, and each
//! volume URL carries the resource values we need: capacity, name, and enough
//! about the device to know whether offering an unmount would make sense.
//! "Available capacity for important usage" is the number Finder and System
//! Settings quote — it counts purgeable space, so it matches what the user
//! sees elsewhere. Only local APFS/HFS+ volumes report it, so exFAT drives and
//! network shares fall back to the plain available capacity.

use std::path::{Path, PathBuf};

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_foundation::{
    NSArray, NSError, NSFileManager, NSFileManagerUnmountOptions, NSNumber, NSString, NSURL,
    NSURLResourceKey, NSURLVolumeAvailableCapacityForImportantUsageKey,
    NSURLVolumeAvailableCapacityKey, NSURLVolumeIsBrowsableKey, NSURLVolumeIsInternalKey,
    NSURLVolumeIsLocalKey, NSURLVolumeIsRootFileSystemKey, NSURLVolumeNameKey,
    NSURLVolumeTotalCapacityKey, NSVolumeEnumerationOptions, ns_string,
};

/// What physically backs a volume, as far as the menu needs to know.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VolumeKind {
    Internal,
    External,
    Network,
}

#[derive(Clone)]
pub struct Volume {
    pub name: String,
    pub path: PathBuf,
    pub free: u64,
    pub total: u64,
    /// The startup disk, and any volume macOS keeps its own files on.
    pub is_system: bool,
    pub kind: VolumeKind,
}

impl Volume {
    /// Free space as a 0..1 fraction of capacity.
    pub fn free_ratio(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        (self.free as f64 / self.total as f64).clamp(0.0, 1.0)
    }

    /// Everything except the system volumes gets an eject button.
    pub fn unmountable(&self) -> bool {
        !self.is_system
    }
}

/// Every user-visible mounted volume, startup disk first and the rest
/// alphabetical — a stable order, so the menu doesn't reshuffle between opens.
pub fn mounted() -> Vec<Volume> {
    let keys = unsafe {
        NSArray::from_slice(&[
            NSURLVolumeNameKey,
            NSURLVolumeTotalCapacityKey,
            NSURLVolumeAvailableCapacityForImportantUsageKey,
            NSURLVolumeAvailableCapacityKey,
            NSURLVolumeIsBrowsableKey,
            NSURLVolumeIsRootFileSystemKey,
            NSURLVolumeIsInternalKey,
            NSURLVolumeIsLocalKey,
        ])
    };

    let Some(urls) = NSFileManager::defaultManager()
        .mountedVolumeURLsIncludingResourceValuesForKeys_options(
            Some(&keys),
            NSVolumeEnumerationOptions::SkipHiddenVolumes,
        )
    else {
        return Vec::new();
    };

    let mut volumes: Vec<Volume> = urls.iter().filter_map(|url| volume(&url)).collect();
    volumes.sort_by(|left, right| {
        let root = right
            .path
            .as_path()
            .eq(Path::new("/"))
            .cmp(&left.path.as_path().eq(Path::new("/")));
        root.then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
    });
    volumes
}

/// The startup disk — what the menu bar itself reports.
pub fn startup() -> Option<Volume> {
    volume(&NSURL::fileURLWithPath_isDirectory(ns_string!("/"), true))
}

/// Ask macOS to unmount a volume. Options are left empty on purpose: the
/// system then puts up its own "disk in use" dialog and handles the retry,
/// which is the behaviour people expect from Finder's eject.
pub fn unmount(path: &Path) {
    let path_string = path.to_string_lossy().into_owned();
    let url = NSURL::fileURLWithPath_isDirectory(&NSString::from_str(&path_string), true);
    let handler = block2::RcBlock::new(move |error: *mut NSError| {
        if let Some(error) = unsafe { error.as_ref() } {
            eprintln!(
                "error unmounting {path_string}: {}",
                error.localizedDescription()
            );
        }
    });

    unsafe {
        NSFileManager::defaultManager().unmountVolumeAtURL_options_completionHandler(
            &url,
            NSFileManagerUnmountOptions::empty(),
            &handler,
        );
    }
}

/// `86 MB`, `3.4 GB`, `123 GB`, `1.9 TB` — one decimal only where it carries
/// meaning. Disk sizes are decimal here, as they are everywhere else in macOS.
pub fn format_bytes(bytes: u64) -> String {
    let gigabytes = bytes as f64 / 1e9;
    if gigabytes >= 1000.0 {
        format!("{:.1} TB", gigabytes / 1000.0)
    } else if gigabytes >= 10.0 {
        format!("{gigabytes:.0} GB")
    } else if gigabytes >= 1.0 {
        format!("{gigabytes:.1} GB")
    } else {
        format!("{:.0} MB", bytes as f64 / 1e6)
    }
}

/// Integer, lowercase, and unspaced for the deliberately compact stacked
/// menu-bar style: `860mb`, `3gb`, `137gb`, `2tb`.
pub fn format_compact_bytes(bytes: u64) -> String {
    let gigabytes = bytes as f64 / 1e9;
    if gigabytes >= 1000.0 {
        format!("{:.0}tb", gigabytes / 1000.0)
    } else if gigabytes >= 1.0 {
        format!("{gigabytes:.0}gb")
    } else {
        format!("{:.0}mb", bytes as f64 / 1e6)
    }
}

fn volume(url: &NSURL) -> Option<Volume> {
    if !flag(url, unsafe { NSURLVolumeIsBrowsableKey }) {
        return None;
    }

    let path = PathBuf::from(url.path()?.to_string());
    let total = bytes(url, unsafe { NSURLVolumeTotalCapacityKey });
    // Pseudo filesystems report no capacity; they are not disks to us.
    if total == 0 {
        return None;
    }

    let is_root = flag(url, unsafe { NSURLVolumeIsRootFileSystemKey });
    let name = text(url, unsafe { NSURLVolumeNameKey }).unwrap_or_else(|| {
        path.file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.to_string_lossy().into_owned())
    });

    // A network mount is simply "not local"; among local disks, IsInternal
    // separates the built-in drive from anything plugged in. The system
    // volumes count as internal regardless — they are the startup disk's.
    let is_system = is_root || is_system_path(&path);
    let kind = if !flag(url, unsafe { NSURLVolumeIsLocalKey }) {
        VolumeKind::Network
    } else if is_system || flag(url, unsafe { NSURLVolumeIsInternalKey }) {
        VolumeKind::Internal
    } else {
        VolumeKind::External
    };

    Some(Volume {
        name,
        free: free_bytes(url),
        total,
        is_system,
        kind,
        path,
    })
}

/// Finder's number where macOS reports it, the filesystem's own otherwise.
fn free_bytes(url: &NSURL) -> u64 {
    match bytes(url, unsafe {
        NSURLVolumeAvailableCapacityForImportantUsageKey
    }) {
        0 => bytes(url, unsafe { NSURLVolumeAvailableCapacityKey }),
        important => important,
    }
}

/// macOS mounts its own volumes under these prefixes (Preboot, Recovery, VM,
/// the Data volume). They are normally hidden from enumeration, but nothing
/// guarantees that, so the eject button is withheld from them by path too.
fn is_system_path(path: &Path) -> bool {
    path == Path::new("/") || path.starts_with("/System") || path.starts_with("/private")
}

fn value(url: &NSURL, key: &NSURLResourceKey) -> Option<Retained<AnyObject>> {
    let mut value = None;
    unsafe { url.getResourceValue_forKey_error(&mut value, key) }.ok()?;
    value
}

fn bytes(url: &NSURL, key: &NSURLResourceKey) -> u64 {
    value(url, key)
        .and_then(|value| {
            value
                .downcast_ref::<NSNumber>()
                .map(NSNumber::unsignedLongLongValue)
        })
        .unwrap_or(0)
}

fn flag(url: &NSURL, key: &NSURLResourceKey) -> bool {
    value(url, key)
        .and_then(|value| value.downcast_ref::<NSNumber>().map(NSNumber::boolValue))
        .unwrap_or(false)
}

fn text(url: &NSURL, key: &NSURLResourceKey) -> Option<String> {
    value(url, key).and_then(|value| value.downcast_ref::<NSString>().map(NSString::to_string))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_volume_reports_capacity() {
        let volume = startup().expect("/ is always a mounted volume");
        assert!(volume.total > 0);
        assert!(volume.free <= volume.total);
        assert!(!volume.unmountable(), "the startup disk must never eject");
    }

    #[test]
    fn mounted_lists_the_startup_disk_first() {
        let volumes = mounted();
        assert_eq!(
            volumes.first().map(|volume| volume.path.as_path()),
            Some(Path::new("/"))
        );
    }

    #[test]
    fn capacities_read_as_they_are_shown() {
        assert_eq!(format_bytes(86_000_000), "86 MB");
        assert_eq!(format_bytes(3_400_000_000), "3.4 GB");
        assert_eq!(format_bytes(137_000_000_000), "137 GB");
        assert_eq!(format_bytes(1_900_000_000_000), "1.9 TB");
    }

    #[test]
    fn compact_capacities_are_integer_lowercase_and_unspaced() {
        assert_eq!(format_compact_bytes(860_000_000), "860mb");
        assert_eq!(format_compact_bytes(3_400_000_000), "3gb");
        assert_eq!(format_compact_bytes(137_000_000_000), "137gb");
        assert_eq!(format_compact_bytes(1_900_000_000_000), "2tb");
    }
}
