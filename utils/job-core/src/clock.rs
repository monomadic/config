//! Local-time formatting via Foundation, so the log trail matches the shell
//! job-server-cli's timestamps without pulling in a date crate.

use objc2_foundation::{NSDate, NSDateFormatter, NSString};

fn format(pattern: &str) -> String {
    let formatter = NSDateFormatter::new();
    formatter.setDateFormat(Some(&NSString::from_str(pattern)));
    formatter.stringFromDate(&NSDate::now()).to_string()
}

/// `2026-07-30T09:14:02+1000` — one line per job in the status trail.
pub fn timestamp() -> String {
    format("yyyy-MM-dd'T'HH:mm:ssZ")
}

/// `20260730-091402` — inserted into artifact names to avoid collisions.
pub fn file_stamp() -> String {
    format("yyyyMMdd-HHmmss")
}
