//! Per-process stats, cache-free and permission-free.
//!
//! RAM and CPU time come from one `proc_pidinfo` syscall per pid
//! (microseconds). Window counts come from one `CGWindowListCopyWindowInfo`
//! call per refresh — public CoreGraphics metadata, no Accessibility or
//! Screen Recording permission needed for pid/layer.

use std::collections::HashMap;
use std::ffi::{c_int, c_void};

use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_foundation::{NSArray, NSObject, NSString};

const PROC_PIDTASKINFO: c_int = 4;

#[repr(C)]
#[derive(Default)]
struct ProcTaskInfo {
    pti_virtual_size: u64,
    pti_resident_size: u64,
    pti_total_user: u64,
    pti_total_system: u64,
    pti_threads_user: u64,
    pti_threads_system: u64,
    pti_policy: i32,
    pti_faults: i32,
    pti_pageins: i32,
    pti_cow_faults: i32,
    pti_messages_sent: i32,
    pti_messages_received: i32,
    pti_syscalls_mach: i32,
    pti_syscalls_unix: i32,
    pti_csw: i32,
    pti_threadnum: i32,
    pti_numrunning: i32,
    pti_priority: i32,
}

#[repr(C)]
struct MachTimebaseInfo {
    numer: u32,
    denom: u32,
}

extern "C" {
    fn proc_pidinfo(
        pid: c_int,
        flavor: c_int,
        arg: u64,
        buffer: *mut c_void,
        buffersize: c_int,
    ) -> c_int;
    fn mach_timebase_info(info: *mut MachTimebaseInfo) -> c_int;
    fn sysctlbyname(
        name: *const std::ffi::c_char,
        oldp: *mut c_void,
        oldlenp: *mut usize,
        newp: *mut c_void,
        newlen: usize,
    ) -> c_int;
}

/// Physical RAM in bytes (a hardware constant, read once at startup).
pub fn total_memory() -> u64 {
    let mut size: u64 = 0;
    let mut len = std::mem::size_of::<u64>();
    let r = unsafe {
        sysctlbyname(
            c"hw.memsize".as_ptr(),
            &mut size as *mut _ as *mut c_void,
            &mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    if r == 0 {
        size
    } else {
        0
    }
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGWindowListCopyWindowInfo(option: u32, relative_to_window: u32) -> *mut c_void;
}

const ON_SCREEN_ONLY: u32 = 1 << 0;
const EXCLUDE_DESKTOP_ELEMENTS: u32 = 1 << 4;

/// (resident bytes, cumulative CPU seconds) for a pid.
pub fn proc_stats(pid: i32) -> Option<(u64, f64)> {
    let mut info = ProcTaskInfo::default();
    let size = std::mem::size_of::<ProcTaskInfo>() as c_int;
    let got = unsafe {
        proc_pidinfo(pid, PROC_PIDTASKINFO, 0, &mut info as *mut _ as *mut c_void, size)
    };
    if got < size {
        return None;
    }
    let mut tb = MachTimebaseInfo { numer: 1, denom: 1 };
    unsafe { mach_timebase_info(&mut tb) };
    let cpu_ns =
        (info.pti_total_user + info.pti_total_system) as f64 * tb.numer as f64 / tb.denom as f64;
    Some((info.pti_resident_size, cpu_ns / 1e9))
}

/// On-screen normal-layer window count per owning pid.
pub fn window_counts() -> HashMap<i32, u32> {
    let mut map = HashMap::new();
    unsafe {
        let ptr = CGWindowListCopyWindowInfo(ON_SCREEN_ONLY | EXCLUDE_DESKTOP_ELEMENTS, 0);
        if ptr.is_null() {
            return map;
        }
        // CFArray of CFDictionary is toll-free bridged to NSArray/NSDictionary.
        let Some(arr) = Retained::<NSArray<NSObject>>::from_raw(ptr.cast()) else {
            return map;
        };
        let pid_key = NSString::from_str("kCGWindowOwnerPID");
        let layer_key = NSString::from_str("kCGWindowLayer");
        for i in 0..arr.len() {
            let dict = arr.objectAtIndex(i);
            let layer_obj: *mut AnyObject = msg_send![&*dict, objectForKey: &*layer_key];
            if layer_obj.is_null() {
                continue;
            }
            let layer: i64 = msg_send![layer_obj, longLongValue];
            if layer != 0 {
                continue; // menu bar items, overlays, etc.
            }
            let pid_obj: *mut AnyObject = msg_send![&*dict, objectForKey: &*pid_key];
            if pid_obj.is_null() {
                continue;
            }
            let pid: i64 = msg_send![pid_obj, longLongValue];
            *map.entry(pid as i32).or_insert(0) += 1;
        }
    }
    map
}

/// Lowercase-compact RAM value per the design spec: "1.2 gb", "840 mb".
pub fn fmt_ram(bytes: u64) -> String {
    const MIB: f64 = 1024.0 * 1024.0;
    const GIB: f64 = MIB * 1024.0;
    let b = bytes as f64;
    if b >= GIB {
        format!("{:.1} gb", b / GIB)
    } else {
        format!("{:.0} mb", b / MIB)
    }
}

#[allow(dead_code)] // kept: cumulative-time formatting, may return as an option
pub fn fmt_cpu(secs: f64) -> String {
    if secs < 60.0 {
        format!("{secs:.0}s")
    } else if secs < 3600.0 {
        format!("{:.0}m", secs / 60.0)
    } else {
        format!("{:.1}h", secs / 3600.0)
    }
}
