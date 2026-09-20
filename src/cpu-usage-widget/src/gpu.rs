//! GPU utilization from the IOKit registry.
//!
//! Every accelerator driver publishes a `PerformanceStatistics` dictionary on
//! its `IOAccelerator` service, and `Device Utilization %` in it is the same
//! number Activity Monitor's GPU History graph plots. It is readable without
//! elevated privileges, which `powermetrics` — the other way to get this — is
//! not. Unlike the CPU counters this is already a rate, so a single read is a
//! complete answer; there is no baseline to prime.

use std::ffi::{CString, c_void};

use objc2_core_foundation::{CFDictionary, CFNumber, CFNumberType, CFRetained, CFString, CFType};
use objc2_io_kit::{
    IOIteratorNext, IOObjectRelease, IORegistryEntryCreateCFProperty, IOServiceGetMatchingServices,
    IOServiceMatching, kIOMainPortDefault,
};

/// A single accelerator's load, as ratios in `0.0..=1.0`.
#[derive(Clone, Copy, Default)]
pub struct Gpu {
    /// Share of the sampling window the GPU was doing any work at all. This is
    /// the headline number the widget draws.
    pub device: f64,
    /// The two pipeline halves behind it, useful for telling a shading-bound
    /// load from a geometry-bound one.
    pub renderer: f64,
    pub tiler: f64,
}

/// The busiest accelerator on the machine, or `None` when nothing publishes
/// utilization — a Mac with a discrete GPU has more than one service, and the
/// idle one is not the interesting one.
pub fn read() -> Option<Gpu> {
    let matching = unsafe { IOServiceMatching(CString::new("IOAccelerator").ok()?.as_ptr()) }?;
    // SAFETY: IOServiceMatching returns a dictionary; IOServiceGetMatchingServices
    // consumes the reference it is handed.
    let matching = unsafe { CFRetained::cast_unchecked::<CFDictionary>(matching) };

    let mut iterator = 0;
    // SAFETY: `iterator` is valid for writes and only read back on success.
    let result =
        unsafe { IOServiceGetMatchingServices(kIOMainPortDefault, Some(matching), &mut iterator) };
    if result != 0 {
        return None;
    }

    let statistics = CFString::from_str("PerformanceStatistics");
    let mut busiest: Option<Gpu> = None;
    loop {
        let entry = IOIteratorNext(iterator);
        if entry == 0 {
            break;
        }

        // SAFETY: `entry` is a live registry entry until we release it below.
        if let Some(properties) =
            unsafe { IORegistryEntryCreateCFProperty(entry, Some(&statistics), None, 0) }
            && let Ok(dictionary) = properties.downcast::<CFDictionary>()
        {
            // SAFETY: IOKit property dictionaries are keyed by CFString.
            let dictionary = unsafe { dictionary.cast_unchecked::<CFString, CFType>() };
            let percent = |key: &str| {
                dictionary
                    .get(&CFString::from_str(key))
                    .and_then(|value| number(&value))
                    .unwrap_or(0.0)
                    / 100.0
            };
            let gpu = Gpu {
                device: percent("Device Utilization %"),
                renderer: percent("Renderer Utilization %"),
                tiler: percent("Tiler Utilization %"),
            };
            if busiest.is_none_or(|best| gpu.device > best.device) {
                busiest = Some(gpu);
            }
        }

        IOObjectRelease(entry);
    }
    IOObjectRelease(iterator);

    busiest
}

fn number(value: &CFType) -> Option<f64> {
    let number = value.downcast_ref::<CFNumber>()?;
    let mut out = 0.0f64;
    // SAFETY: DoubleType matches the f64 we hand it; CFNumber converts to it.
    let ok = unsafe { number.value(CFNumberType::DoubleType, (&raw mut out).cast::<c_void>()) };
    ok.then_some(out)
}
