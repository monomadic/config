//! How long this Mac has been up, from the kernel's own boot timestamp.
//!
//! `kern.boottime` rather than `NSProcessInfo systemUptime`: the sysctl is a
//! wall-clock instant, so the elapsed time it yields includes any stretch the
//! machine spent asleep — which is what "uptime" means to someone reading a
//! menu bar.

use std::ffi::{c_char, c_int, c_void};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// `struct timeval` as macOS declares it: `time_t` is 64-bit, `suseconds_t` is
/// 32-bit. Only read by the sysctl below.
#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Timeval {
    sec: i64,
    usec: i32,
}

unsafe extern "C" {
    fn sysctlbyname(
        name: *const c_char,
        oldp: *mut c_void,
        oldlenp: *mut usize,
        newp: *mut c_void,
        newlen: usize,
    ) -> c_int;
}

fn boot_time() -> Result<SystemTime, String> {
    let mut value = Timeval::default();
    let mut size = size_of::<Timeval>();
    // SAFETY: the buffer and its length describe the same `Timeval`, and the
    // kernel writes at most that many bytes.
    let result = unsafe {
        sysctlbyname(
            c"kern.boottime".as_ptr(),
            (&raw mut value).cast(),
            &raw mut size,
            std::ptr::null_mut(),
            0,
        )
    };
    if result != 0 {
        return Err(format!(
            "kern.boottime: {}",
            std::io::Error::last_os_error()
        ));
    }
    if value.sec <= 0 {
        return Err("kern.boottime returned no boot time".to_string());
    }

    Ok(UNIX_EPOCH + Duration::new(value.sec as u64, value.usec.max(0) as u32 * 1000))
}

/// Time since boot. Errors if the clock has been wound back behind the boot
/// timestamp, which would otherwise render as a nonsense duration.
pub fn uptime() -> Result<Duration, String> {
    let booted = boot_time()?;
    SystemTime::now()
        .duration_since(booted)
        .map_err(|_| "system clock is behind the boot time".to_string())
}

fn parts(duration: Duration) -> (u64, u64, u64) {
    let total_minutes = duration.as_secs() / 60;
    (
        total_minutes / (24 * 60),
        (total_minutes / 60) % 24,
        total_minutes % 60,
    )
}

/// Compact menu bar form: `33.1D`, `3.5D`, `2D`, `5H`, `42M`. Past a day the
/// hours become a decimal rather than a second unit — one number is less to
/// read across in a menu bar than two. Truncated, not rounded, so the figure
/// never runs ahead of the machine.
///
/// The unit is capitalised: at menu bar size a lowercase `d` or `h` hangs off
/// the digits' x-height, where a cap sits flush with them.
pub fn format_uptime(duration: Duration) -> String {
    let (days, hours, minutes) = parts(duration);
    if days > 0 {
        let tenths = (hours * 10) / 24;
        return if tenths == 0 {
            format!("{days}D")
        } else {
            format!("{days}.{tenths}D")
        };
    }
    if hours > 0 {
        return format!("{hours}H");
    }
    format!("{minutes}M")
}

/// The coarsest form, for the styles that carry the detail somewhere other than
/// the digits: whole days once there is a day, otherwise hours, otherwise
/// minutes. Truncated, like [`format_uptime`].
pub fn format_uptime_coarse(duration: Duration) -> String {
    let (days, hours, minutes) = parts(duration);
    if days > 0 {
        return format!("{days}D");
    }
    if hours > 0 {
        return format!("{hours}H");
    }
    format!("{minutes}M")
}

/// How far through the current day of uptime the machine is, `0.0..1.0` — what
/// the progress style's bar fills to, so the bar carries the fraction the
/// whole-day figure drops.
pub fn day_fraction(duration: Duration) -> f64 {
    let (_, hours, minutes) = parts(duration);
    (hours * 60 + minutes) as f64 / (24.0 * 60.0)
}

/// Whole days of uptime — what the dot styles count out one mark at a time.
pub fn whole_days(duration: Duration) -> u64 {
    parts(duration).0
}

/// Menu form: `1 day, 2 hours, 43 mins`. Every unit that is non-zero, plus the
/// minutes always — this is the one place the figure is spelled out in full.
pub fn expanded_uptime(duration: Duration) -> String {
    let (days, hours, minutes) = parts(duration);
    let plural = |n: u64, unit: &str| {
        if n == 1 {
            format!("{n} {unit}")
        } else {
            format!("{n} {unit}s")
        }
    };

    let mut out = Vec::new();
    if days > 0 {
        out.push(plural(days, "day"));
    }
    if hours > 0 {
        out.push(plural(hours, "hour"));
    }
    out.push(plural(minutes, "min"));
    out.join(", ")
}

/// Tooltip form: `2 days, 1 hour`.
pub fn human_uptime(duration: Duration) -> String {
    let (days, hours, minutes) = parts(duration);
    let plural = |n: u64, unit: &str| {
        if n == 1 {
            format!("{n} {unit}")
        } else {
            format!("{n} {unit}s")
        }
    };

    match (days, hours) {
        (0, 0) => plural(minutes, "minute"),
        (0, _) => plural(hours, "hour"),
        (_, 0) => plural(days, "day"),
        _ => format!("{}, {}", plural(days, "day"), plural(hours, "hour")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn duration(days: u64, hours: u64, minutes: u64) -> Duration {
        Duration::from_secs(((days * 24 + hours) * 60 + minutes) * 60)
    }

    #[test]
    fn compact_uses_one_capitalised_unit() {
        assert_eq!(format_uptime(duration(3, 12, 0)), "3.5D");
        assert_eq!(format_uptime(duration(33, 3, 12)), "33.1D");
        assert_eq!(format_uptime(duration(2, 0, 30)), "2D");
        assert_eq!(format_uptime(duration(0, 5, 30)), "5H");
        assert_eq!(format_uptime(duration(0, 0, 42)), "42M");
        assert_eq!(format_uptime(duration(0, 0, 0)), "0M");
    }

    /// A tenth of a day is 2.4 hours, so the decimal must not tick over until
    /// the hours actually reach it — and must never round a day up early.
    #[test]
    fn compact_truncates_the_decimal() {
        assert_eq!(format_uptime(duration(1, 2, 0)), "1D");
        assert_eq!(format_uptime(duration(1, 3, 0)), "1.1D");
        assert_eq!(format_uptime(duration(1, 23, 59)), "1.9D");
    }

    #[test]
    fn coarse_form_drops_the_decimal() {
        assert_eq!(format_uptime_coarse(duration(3, 12, 0)), "3D");
        assert_eq!(format_uptime_coarse(duration(1, 23, 59)), "1D");
        assert_eq!(format_uptime_coarse(duration(0, 5, 30)), "5H");
        assert_eq!(format_uptime_coarse(duration(0, 0, 42)), "42M");
    }

    #[test]
    fn day_fraction_tracks_the_part_the_digits_drop() {
        assert_eq!(day_fraction(duration(3, 0, 0)), 0.0);
        assert_eq!(day_fraction(duration(3, 12, 0)), 0.5);
        assert_eq!(day_fraction(duration(0, 6, 0)), 0.25);
    }

    #[test]
    fn whole_days_truncates() {
        assert_eq!(whole_days(duration(0, 23, 59)), 0);
        assert_eq!(whole_days(duration(1, 0, 0)), 1);
        assert_eq!(whole_days(duration(33, 3, 12)), 33);
    }

    #[test]
    fn expanded_form_keeps_the_minutes() {
        assert_eq!(expanded_uptime(duration(1, 2, 43)), "1 day, 2 hours, 43 mins");
        assert_eq!(expanded_uptime(duration(0, 2, 1)), "2 hours, 1 min");
        assert_eq!(expanded_uptime(duration(0, 0, 42)), "42 mins");
        assert_eq!(expanded_uptime(duration(2, 0, 5)), "2 days, 5 mins");
    }

    #[test]
    fn human_form_pluralises() {
        assert_eq!(human_uptime(duration(2, 1, 0)), "2 days, 1 hour");
        assert_eq!(human_uptime(duration(1, 2, 0)), "1 day, 2 hours");
        assert_eq!(human_uptime(duration(3, 0, 0)), "3 days");
        assert_eq!(human_uptime(duration(0, 1, 0)), "1 hour");
        assert_eq!(human_uptime(duration(0, 0, 1)), "1 minute");
    }

    #[test]
    fn boot_time_is_in_the_past() {
        let up = uptime().expect("kern.boottime is always readable");
        assert!(up.as_secs() > 0);
    }
}
