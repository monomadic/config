//! Per-core CPU utilization from the kernel's cumulative tick counters.
//!
//! One `host_processor_info` call per sample — no `top`, no `ps`, no process
//! enumeration, so a tick costs microseconds. Utilization is a rate, not a
//! reading, so it only exists between two samples: the first [`Sampler::sample`]
//! establishes a baseline and reports all zeros.

use std::ptr;

type NaturalT = u32;
type IntegerT = i32;
type KernReturnT = i32;
type MachPortT = u32;
type ProcessorFlavorT = i32;
type ProcessorInfoArrayT = *mut IntegerT;
type MachMsgTypeNumberT = u32;

const PROCESSOR_CPU_LOAD_INFO: ProcessorFlavorT = 2;

/// `processor_cpu_load_info` is four `natural_t` counters per core, in this
/// order. They are 32-bit and do wrap, which is why deltas use `wrapping_sub`.
const CPU_STATE_MAX: usize = 4;
const CPU_STATE_USER: usize = 0;
const CPU_STATE_SYSTEM: usize = 1;
const CPU_STATE_IDLE: usize = 2;
const CPU_STATE_NICE: usize = 3;

unsafe extern "C" {
    fn mach_host_self() -> MachPortT;
    fn mach_task_self() -> MachPortT;
    fn host_processor_info(
        host: MachPortT,
        flavor: ProcessorFlavorT,
        out_processor_count: *mut NaturalT,
        out_processor_info: *mut ProcessorInfoArrayT,
        out_processor_info_count: *mut MachMsgTypeNumberT,
    ) -> KernReturnT;
    fn vm_deallocate(target_task: MachPortT, address: usize, size: usize) -> KernReturnT;
}

type Ticks = [u32; CPU_STATE_MAX];

fn read_ticks() -> Result<Vec<Ticks>, String> {
    let mut count: NaturalT = 0;
    let mut info: ProcessorInfoArrayT = ptr::null_mut();
    let mut info_count: MachMsgTypeNumberT = 0;

    // SAFETY: the three out-parameters are valid for writes, and the kernel
    // either fills all of them or returns non-KERN_SUCCESS.
    let result = unsafe {
        host_processor_info(
            mach_host_self(),
            PROCESSOR_CPU_LOAD_INFO,
            &mut count,
            &mut info,
            &mut info_count,
        )
    };
    if result != 0 || info.is_null() || count == 0 {
        return Err(format!("host_processor_info failed: {result}"));
    }

    let mut cores = Vec::with_capacity(count as usize);
    for core in 0..count as usize {
        let mut ticks = Ticks::default();
        for (state, slot) in ticks.iter_mut().enumerate() {
            // SAFETY: the kernel returned `count` cores of CPU_STATE_MAX
            // counters each, so every index below is in bounds.
            *slot = unsafe { *info.add(core * CPU_STATE_MAX + state) as u32 };
        }
        cores.push(ticks);
    }

    // The array is kernel-allocated into our address space; the caller owns it.
    // SAFETY: `info` and `info_count` came from the call above and the data has
    // been copied out.
    unsafe {
        vm_deallocate(
            mach_task_self(),
            info as usize,
            info_count as usize * size_of::<IntegerT>(),
        )
    };

    Ok(cores)
}

/// Holds the previous tick counters so successive samples can be differenced.
#[derive(Default)]
pub struct Sampler {
    previous: Vec<Ticks>,
}

impl Sampler {
    /// Busy ratio in `0.0..=1.0` for each core, measured over the interval
    /// since the previous call. All zeros on the first call, and whenever the
    /// core count changes underneath us.
    pub fn sample(&mut self) -> Result<Vec<f64>, String> {
        let current = read_ticks()?;

        let mut usage = vec![0.0; current.len()];
        if self.previous.len() == current.len() {
            for (index, (now, before)) in current.iter().zip(&self.previous).enumerate() {
                let delta = |state: usize| now[state].wrapping_sub(before[state]) as u64;
                let busy = delta(CPU_STATE_USER) + delta(CPU_STATE_SYSTEM) + delta(CPU_STATE_NICE);
                let total = busy + delta(CPU_STATE_IDLE);
                if total > 0 {
                    usage[index] = busy as f64 / total as f64;
                }
            }
        }

        self.previous = current;
        Ok(usage)
    }
}
