//! Per-process stats, cache-free and permission-free.
//!
//! RAM and CPU time come from one `proc_pidinfo` syscall per pid
//! (microseconds). Window counts come from one `CGWindowListCopyWindowInfo`
//! call per refresh — public CoreGraphics metadata, no Accessibility or
//! Screen Recording permission needed for pid/layer.

use std::collections::{HashMap, HashSet};
use std::ffi::{c_int, c_void};

use objc2::msg_send;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2_foundation::{NSArray, NSObject, NSString};

const PROC_PIDTASKINFO: c_int = 4;
const PROC_PIDT_SHORTBSDINFO: c_int = 13;
const PROC_ALL_PIDS: u32 = 1;

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

#[repr(C)]
#[derive(Default)]
struct ProcBsdShortInfo {
    pbsi_pid: u32,
    pbsi_ppid: u32,
    pbsi_pgid: u32,
    pbsi_status: u32,
    pbsi_comm: [u8; 16],
    pbsi_flags: u32,
    pbsi_uid: u32,
    pbsi_gid: u32,
    pbsi_ruid: u32,
    pbsi_rgid: u32,
    pbsi_svuid: u32,
    pbsi_svgid: u32,
    pbsi_rfu: u32,
}

extern "C" {
    fn proc_pidinfo(
        pid: c_int,
        flavor: c_int,
        arg: u64,
        buffer: *mut c_void,
        buffersize: c_int,
    ) -> c_int;
    fn proc_listpids(kind: u32, typeinfo: u32, buffer: *mut c_void, buffersize: c_int) -> c_int;
    fn mach_timebase_info(info: *mut MachTimebaseInfo) -> c_int;
}

#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGWindowListCopyWindowInfo(option: u32, relative_to_window: u32) -> *mut c_void;
}

const ON_SCREEN_ONLY: u32 = 1 << 0;
const EXCLUDE_DESKTOP_ELEMENTS: u32 = 1 << 4;

/// One snapshot of the whole process table, for walking an app's process
/// tree. Helper processes (browser renderers, GPU processes, XPC services)
/// are children of the app process, not part of it — per-pid stats alone
/// miss nearly all of a browser's CPU and RAM.
pub struct ProcSnapshot {
    children: HashMap<i32, Vec<i32>>,
    alive: HashSet<i32>,
}

impl ProcSnapshot {
    /// proc_listpids plus one short-bsdinfo call per pid — the same
    /// permission-free libproc surface as `proc_stats`.
    pub fn new() -> Self {
        let mut children: HashMap<i32, Vec<i32>> = HashMap::new();
        let mut alive = HashSet::new();
        unsafe {
            let bytes = proc_listpids(PROC_ALL_PIDS, 0, std::ptr::null_mut(), 0);
            if bytes > 0 {
                // Headroom for processes spawned between the two calls.
                let cap = bytes as usize / 4 + 16;
                let mut pids = vec![0i32; cap];
                let got = proc_listpids(
                    PROC_ALL_PIDS,
                    0,
                    pids.as_mut_ptr() as *mut c_void,
                    (cap * 4) as c_int,
                );
                if got > 0 {
                    pids.truncate(got as usize / 4);
                    let size = std::mem::size_of::<ProcBsdShortInfo>() as c_int;
                    for &pid in pids.iter().filter(|&&p| p > 0) {
                        alive.insert(pid);
                        let mut info = ProcBsdShortInfo::default();
                        let r = proc_pidinfo(
                            pid,
                            PROC_PIDT_SHORTBSDINFO,
                            0,
                            &mut info as *mut _ as *mut c_void,
                            size,
                        );
                        let ppid = info.pbsi_ppid as i32;
                        if r >= size && ppid != pid {
                            children.entry(ppid).or_default().push(pid);
                        }
                    }
                }
            }
        }
        Self { children, alive }
    }

    /// `root` plus every live descendant, breadth-first.
    pub fn tree_pids(&self, root: i32) -> Vec<i32> {
        let mut out = vec![root];
        let mut i = 0;
        while i < out.len() {
            if let Some(kids) = self.children.get(&out[i]) {
                out.extend_from_slice(kids);
            }
            i += 1;
        }
        out
    }

    pub fn is_alive(&self, pid: i32) -> bool {
        self.alive.contains(&pid)
    }
}

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

/// Lowercase-compact RAM value: "1.2 gb", "840 mb".
#[allow(dead_code)] // kept: absolute-size formatting, may return as an option
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_walks_process_trees() {
        let s = ProcSnapshot::new();
        let tree = s.tree_pids(1);
        assert!(tree.len() > 10, "launchd should have many descendants, got {}", tree.len());
        assert!(s.is_alive(1));
    }
}
