//! Which processes are using the CPU, from libproc.
//!
//! Every tick reads the cumulative user+system time of each process with
//! `proc_pidinfo` / `PROC_PIDTASKINFO` and differences it against the previous
//! tick, the same way `top` does. One pass over a few hundred processes costs
//! a millisecond or two, which is why only the busiest few are named: resolving
//! a path is an extra call per process, so it only happens for the winners.
//!
//! Without root only this user's processes are readable — `top` and `ps` can
//! see everyone's because they are setuid root, and the `p_pctcpu` field the
//! sysctl interface exposes is never filled in. So `kernel_task`, `WindowServer`,
//! `coreaudiod` and the rest are invisible here, and the list is of this
//! user's processes only. Rolling the unreadable remainder into a row of its
//! own was tried and dropped: it was the top entry nearly always, which told
//! the user nothing they could act on.

use std::collections::HashMap;
use std::ffi::{CStr, c_char, c_int, c_void};
use std::time::Instant;

/// How many processes the dropdown names.
pub const TOP: usize = 3;

const PROC_ALL_PIDS: u32 = 1;
const PROC_PIDTASKINFO: c_int = 4;
const PROC_PIDPATHINFO_MAXSIZE: usize = 4 * 1024;

/// `struct proc_taskinfo` from `<sys/proc_info.h>`. Only the two time
/// counters matter here, but the kernel fills the whole struct and checks the
/// size it is handed, so the layout has to be complete.
#[repr(C)]
#[derive(Default, Clone, Copy)]
struct ProcTaskInfo {
    virtual_size: u64,
    resident_size: u64,
    total_user: u64,
    total_system: u64,
    threads_user: u64,
    threads_system: u64,
    policy: i32,
    faults: i32,
    pageins: i32,
    cow_faults: i32,
    messages_sent: i32,
    messages_received: i32,
    syscalls_mach: i32,
    syscalls_unix: i32,
    csw: i32,
    threadnum: i32,
    numrunning: i32,
    priority: i32,
}

#[repr(C)]
#[derive(Default)]
struct MachTimebaseInfo {
    numer: u32,
    denom: u32,
}

unsafe extern "C" {
    fn proc_listpids(kind: u32, typeinfo: u32, buffer: *mut c_void, buffersize: c_int) -> c_int;
    fn proc_pidinfo(
        pid: c_int,
        flavor: c_int,
        arg: u64,
        buffer: *mut c_void,
        buffersize: c_int,
    ) -> c_int;
    fn proc_pidpath(pid: c_int, buffer: *mut c_void, buffersize: u32) -> c_int;
    fn proc_name(pid: c_int, buffer: *mut c_void, buffersize: u32) -> c_int;
    fn mach_timebase_info(info: *mut MachTimebaseInfo) -> c_int;
}

/// One row of the dropdown's process list.
pub struct Process {
    pub name: String,
    /// The executable's full path, when libproc will say — what the app icon
    /// is looked up from.
    pub path: Option<String>,
    /// CPU time per wall-clock time, in cores: `1.0` is one core fully busy,
    /// the same convention as Activity Monitor and `top`, so a multithreaded
    /// process can exceed it.
    pub share: f64,
}

/// Cumulative CPU time of every process this user may read, in Mach time
/// units, keyed by pid.
fn read_cpu_times() -> HashMap<i32, u64> {
    // SAFETY: a null buffer asks only for the size the full list would need.
    let bytes = unsafe { proc_listpids(PROC_ALL_PIDS, 0, std::ptr::null_mut(), 0) };
    if bytes <= 0 {
        return HashMap::new();
    }
    // Processes can start between the two calls, so leave headroom.
    let mut pids = vec![0 as c_int; bytes as usize / size_of::<c_int>() + 64];
    // SAFETY: the buffer is valid for `buffersize` bytes.
    let bytes = unsafe {
        proc_listpids(
            PROC_ALL_PIDS,
            0,
            pids.as_mut_ptr().cast(),
            (pids.len() * size_of::<c_int>()) as c_int,
        )
    };
    if bytes <= 0 {
        return HashMap::new();
    }
    pids.truncate(bytes as usize / size_of::<c_int>());

    let mut times = HashMap::with_capacity(pids.len());
    for pid in pids {
        if pid <= 0 {
            continue;
        }
        let mut info = ProcTaskInfo::default();
        // SAFETY: `info` is a correctly laid-out proc_taskinfo of the size we
        // pass. The call fails rather than writing anything for processes we
        // may not inspect.
        let written = unsafe {
            proc_pidinfo(
                pid,
                PROC_PIDTASKINFO,
                0,
                (&raw mut info).cast(),
                size_of::<ProcTaskInfo>() as c_int,
            )
        };
        if written as usize == size_of::<ProcTaskInfo>() {
            times.insert(pid, info.total_user.saturating_add(info.total_system));
        }
    }
    times
}

/// The executable's full path, when the process is ours to inspect.
fn path(pid: i32) -> Option<String> {
    let mut buffer = [0 as c_char; PROC_PIDPATHINFO_MAXSIZE];
    // SAFETY: the buffer is valid for the size passed; libproc NUL-terminates.
    let length = unsafe { proc_pidpath(pid, buffer.as_mut_ptr().cast(), buffer.len() as u32) };
    if length <= 0 {
        return None;
    }
    let path = unsafe { CStr::from_ptr(buffer.as_ptr()) }.to_str().ok()?;
    (!path.is_empty()).then(|| path.to_string())
}

/// The executable's file name — `proc_name` truncates, and the path is what
/// `top` shows too. Falls back to the short name, then to the pid.
fn name(pid: i32, path: Option<&str>) -> String {
    if let Some(base) = path.and_then(|p| p.rsplit('/').next())
        && !base.is_empty()
    {
        return base.to_string();
    }

    let mut buffer = [0 as c_char; 256];
    // SAFETY: as above.
    let length = unsafe { proc_name(pid, buffer.as_mut_ptr().cast(), buffer.len() as u32) };
    if length > 0
        && let Ok(short) = unsafe { CStr::from_ptr(buffer.as_ptr()) }.to_str()
        && !short.is_empty()
    {
        return short.to_string();
    }
    format!("pid {pid}")
}

/// Holds the previous per-process totals so successive samples can be
/// differenced, like [`crate::cpu::Sampler`] does for the cores.
pub struct Sampler {
    previous: HashMap<i32, u64>,
    taken: Option<Instant>,
    /// Nanoseconds per Mach time unit; the task counters are in the latter.
    ns_per_unit: f64,
}

impl Default for Sampler {
    fn default() -> Self {
        let mut timebase = MachTimebaseInfo::default();
        // SAFETY: a plain out-parameter; the call cannot fail for a live task.
        unsafe { mach_timebase_info(&raw mut timebase) };
        let ns_per_unit = if timebase.denom == 0 {
            1.0
        } else {
            f64::from(timebase.numer) / f64::from(timebase.denom)
        };
        Sampler {
            previous: HashMap::new(),
            taken: None,
            ns_per_unit,
        }
    }
}

impl Sampler {
    /// The [`TOP`] busiest processes since the previous call, busiest first.
    /// Empty on the first call, which only primes the baseline.
    pub fn sample(&mut self) -> Vec<Process> {
        let now = Instant::now();
        let current = read_cpu_times();

        let mut top = Vec::new();
        if let Some(then) = self.taken {
            let seconds = now.duration_since(then).as_secs_f64();
            if seconds > 0.0 {
                let mut shares: Vec<(i32, f64)> = current
                    .iter()
                    .filter_map(|(&pid, &total)| {
                        let before = *self.previous.get(&pid)?;
                        // A pid recycled within one interval would go
                        // backwards; it has no meaningful delta.
                        let delta = total.checked_sub(before)?;
                        Some((pid, delta as f64 * self.ns_per_unit / 1e9 / seconds))
                    })
                    .collect();
                shares.sort_by(|a, b| b.1.total_cmp(&a.1));
                shares.truncate(TOP);
                top.extend(shares.into_iter().map(|(pid, share)| {
                    let path = path(pid);
                    Process {
                        name: name(pid, path.as_deref()),
                        path,
                        share,
                    }
                }));
            }
        }

        self.previous = current;
        self.taken = Some(now);
        top
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    /// Our own process, spinning one core, has to rank near the top.
    #[test]
    fn ranks_a_busy_process() {
        let mut sampler = Sampler::default();
        assert!(sampler.sample().is_empty(), "first sample only primes");

        let stop = Arc::new(AtomicBool::new(false));
        let spinner = {
            let stop = Arc::clone(&stop);
            std::thread::spawn(move || while !stop.load(Ordering::Relaxed) {})
        };
        std::thread::sleep(Duration::from_millis(600));
        let top = sampler.sample();
        stop.store(true, Ordering::Relaxed);
        spinner.join().unwrap();

        let me = name(
            std::process::id() as i32,
            path(std::process::id() as i32).as_deref(),
        );
        let mine = top
            .iter()
            .find(|p| p.name == me)
            .unwrap_or_else(|| panic!("{me} missing from {:?}", names(&top)));
        assert!(
            mine.share > 0.5,
            "spinning a core should read near 1.0, got {}",
            mine.share
        );
        assert!(top.len() <= TOP);
        assert!(
            top.windows(2).all(|w| w[0].share >= w[1].share),
            "not sorted: {:?}",
            names(&top)
        );
    }

    fn names(top: &[Process]) -> Vec<String> {
        top.iter()
            .map(|p| format!("{} {:.2}", p.name, p.share))
            .collect()
    }
}
