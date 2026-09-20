//! Conservative payload budget. This is an observation, not an APFS reservation.
use crate::run::Operation;
use anyhow::{Context, Result, ensure};
use serde::Serialize;
use std::{fs::File, os::fd::AsRawFd};

pub const DEFAULT_RESERVE_BYTES: u64 = 1024 * 1024 * 1024;
#[derive(Clone, Copy, Debug, Serialize)]
pub struct Space {
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub allocation_unit_bytes: u64,
    pub read_only: bool,
}
#[derive(Debug, Serialize)]
pub struct Budget {
    pub incoming_logical_bytes: u64,
    pub incoming_rounded_bytes: u64,
    pub largest_staged_file_bytes: u64,
    pub retained_predecessor_logical_bytes: u64,
    pub verification_read_bytes: u64,
    pub reserve_bytes: u64,
    pub required_additional_bytes: u64,
    pub available_bytes: u64,
    pub shortfall_bytes: u64,
    pub headroom_after_reserve_bytes: u64,
    pub sufficient: bool,
    pub allocation_reserved: bool,
}
fn checked_bytes(blocks: u64, unit: u64) -> Result<u64> {
    ensure!(unit > 0, "Filesystem reported zero allocation unit");
    blocks
        .checked_mul(unit)
        .context("Filesystem capacity overflow")
}
pub(crate) fn space(directory: &File) -> Result<Space> {
    let mut stats = std::mem::MaybeUninit::<libc::statfs>::uninit();
    // SAFETY: the descriptor is live and the output points to statfs-sized storage.
    ensure!(
        unsafe { libc::fstatfs(directory.as_raw_fd(), stats.as_mut_ptr()) } == 0,
        "Cannot read destination capacity: {}",
        std::io::Error::last_os_error()
    );
    let stats = unsafe { stats.assume_init() };
    let unit = u64::from(stats.f_bsize);
    Ok(Space {
        total_bytes: checked_bytes(stats.f_blocks, unit)?,
        available_bytes: checked_bytes(stats.f_bavail, unit)?,
        allocation_unit_bytes: unit,
        read_only: stats.f_flags & libc::MNT_RDONLY as u32 != 0,
    })
}
fn rounded(size: u64, unit: u64) -> Result<u64> {
    ensure!(unit > 0, "Zero allocation unit");
    let remainder = size % unit;
    if remainder == 0 {
        Ok(size)
    } else {
        size.checked_add(unit - remainder)
            .context("Rounded file size overflow")
    }
}
pub fn budget(operations: &[Operation], space: Space, reserve_bytes: u64) -> Result<Budget> {
    ensure!(space.allocation_unit_bytes > 0, "Zero allocation unit");
    let mut incoming_logical_bytes = 0u64;
    let mut incoming_rounded_bytes = 0u64;
    let mut largest_staged_file_bytes = 0u64;
    let mut retained_predecessor_logical_bytes = 0u64;
    for op in operations {
        let size = op.source.stamp.size;
        let allocated = rounded(size, space.allocation_unit_bytes)?;
        incoming_logical_bytes = incoming_logical_bytes
            .checked_add(size)
            .context("Incoming payload overflow")?;
        incoming_rounded_bytes = incoming_rounded_bytes
            .checked_add(allocated)
            .context("Incoming allocation overflow")?;
        largest_staged_file_bytes = largest_staged_file_bytes.max(allocated);
        if let Some(old) = &op.predecessor {
            retained_predecessor_logical_bytes = retained_predecessor_logical_bytes
                .checked_add(old.stamp.size)
                .context("Predecessor size overflow")?;
        }
    }
    // Old data is already allocated and remains allocated after moving to history.
    // All incoming bytes must fit; never subtract predecessors or unpruned history.
    // The current staged file is part of that total, not an extra second copy.
    let required_additional_bytes = incoming_rounded_bytes
        .checked_add(reserve_bytes)
        .context("Required capacity overflow")?;
    Ok(Budget {
        incoming_logical_bytes,
        incoming_rounded_bytes,
        largest_staged_file_bytes,
        retained_predecessor_logical_bytes,
        verification_read_bytes: incoming_logical_bytes,
        reserve_bytes,
        required_additional_bytes,
        available_bytes: space.available_bytes,
        shortfall_bytes: required_additional_bytes.saturating_sub(space.available_bytes),
        headroom_after_reserve_bytes: space
            .available_bytes
            .saturating_sub(required_additional_bytes),
        sufficient: !space.read_only && space.available_bytes >= required_additional_bytes,
        allocation_reserved: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{filesystem::Stamp, manifest::Entry, run::Kind};
    fn operation(size: u64, old: Option<u64>) -> Operation {
        let entry = |size| Entry {
            path_base64: "YQ==".into(),
            stamp: Stamp {
                device: 1,
                file_id: 1,
                size,
                mtime_seconds: 1,
                mtime_nanos: 0,
                ctime_seconds: 1,
                ctime_nanos: 0,
            },
            sha256: None,
        };
        Operation {
            id: "op".into(),
            kind: if old.is_some() {
                Kind::ReplaceWithHistory
            } else {
                Kind::Copy
            },
            source: entry(size),
            predecessor: old.map(entry),
        }
    }
    fn space(available_bytes: u64) -> Space {
        Space {
            total_bytes: 1_000_000,
            available_bytes,
            allocation_unit_bytes: 4096,
            read_only: false,
        }
    }
    #[test]
    fn budgets_all_payloads_and_keeps_history_without_double_counting_staging() {
        let budget = budget(
            &[operation(1, None), operation(4097, Some(100_000))],
            space(20_000),
            1000,
        )
        .unwrap();
        assert_eq!(budget.incoming_logical_bytes, 4098);
        assert_eq!(budget.incoming_rounded_bytes, 12288);
        assert_eq!(budget.largest_staged_file_bytes, 8192);
        assert_eq!(budget.retained_predecessor_logical_bytes, 100_000);
        assert_eq!(budget.required_additional_bytes, 13288);
        assert_eq!(budget.verification_read_bytes, 4098);
        assert!(budget.sufficient && !budget.allocation_reserved);
    }
    #[test]
    fn boundary_shortfall_and_read_only_volume_are_explicit() {
        let ops = [operation(4096, Some(500_000))];
        assert!(budget(&ops, space(4196), 100).unwrap().sufficient);
        let short = budget(&ops, space(4195), 100).unwrap();
        assert!(!short.sufficient);
        assert_eq!(short.shortfall_bytes, 1);
        let mut read_only = space(100_000);
        read_only.read_only = true;
        assert!(!budget(&ops, read_only, 0).unwrap().sufficient);
    }
    #[test]
    fn empty_files_still_keep_the_requested_reserve() {
        let b = budget(&[operation(0, None)], space(10), 11).unwrap();
        assert_eq!(b.incoming_rounded_bytes, 0);
        assert_eq!(b.required_additional_bytes, 11);
        assert_eq!(b.shortfall_bytes, 1);
    }
    #[test]
    fn invalid_units_and_all_overflow_paths_fail() {
        assert!(checked_bytes(u64::MAX, 4096).is_err());
        assert!(checked_bytes(1, 0).is_err());
        assert!(budget(&[operation(u64::MAX, None)], space(u64::MAX), 0).is_err());
        let mut byte_units = space(u64::MAX);
        byte_units.allocation_unit_bytes = 1;
        assert!(
            budget(
                &[operation(u64::MAX, None), operation(1, None)],
                byte_units,
                0
            )
            .is_err()
        );
        assert!(budget(&[operation(u64::MAX, None)], byte_units, 1).is_err());
        assert!(
            budget(
                &[operation(0, Some(u64::MAX)), operation(0, Some(1))],
                byte_units,
                0
            )
            .is_err()
        );
    }
}
