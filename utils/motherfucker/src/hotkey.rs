//! Global hotkey via Carbon's RegisterEventHotKey.
//!
//! Chosen deliberately over CGEventTap: it needs no Accessibility or Input
//! Monitoring permission, and it cannot stall system-wide key delivery if
//! this process hangs. The handler is dispatched on the main run loop.

use std::ffi::c_void;
use std::ptr;

pub type OSStatus = i32;

#[repr(C)]
pub struct EventTypeSpec {
    pub event_class: u32,
    pub event_kind: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EventHotKeyID {
    pub signature: u32,
    pub id: u32,
}

pub const CLASS_KEYBOARD: u32 = fourcc(b"keyb");
pub const HOT_KEY_PRESSED: u32 = 5;

// Kept even when unused: these are the values to reach for when changing
// SUMMON_MODS in main.rs.
#[allow(dead_code)]
pub const MOD_CMD: u32 = 0x0100;
#[allow(dead_code)]
pub const MOD_SHIFT: u32 = 0x0200;
pub const MOD_OPTION: u32 = 0x0800;
#[allow(dead_code)]
pub const MOD_CONTROL: u32 = 0x1000;
pub const VK_SPACE: u32 = 0x31;

const fn fourcc(b: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*b)
}

pub type HotKeyHandler =
    extern "C" fn(next: *mut c_void, event: *mut c_void, user: *mut c_void) -> OSStatus;

#[link(name = "Carbon", kind = "framework")]
extern "C" {
    fn GetApplicationEventTarget() -> *mut c_void;
    fn InstallEventHandler(
        target: *mut c_void,
        handler: HotKeyHandler,
        num_types: u32,
        type_list: *const EventTypeSpec,
        user_data: *mut c_void,
        out_handler_ref: *mut *mut c_void,
    ) -> OSStatus;
    fn RegisterEventHotKey(
        key_code: u32,
        modifiers: u32,
        hot_key_id: EventHotKeyID,
        target: *mut c_void,
        options: u32,
        out_hot_key_ref: *mut *mut c_void,
    ) -> OSStatus;
}

/// Register `handler` for a single global hotkey. `user` is passed through to
/// the handler and must stay valid for the life of the process.
pub unsafe fn register(key_code: u32, modifiers: u32, handler: HotKeyHandler, user: *mut c_void) -> Result<(), OSStatus> {
    let target = GetApplicationEventTarget();

    let spec = EventTypeSpec {
        event_class: CLASS_KEYBOARD,
        event_kind: HOT_KEY_PRESSED,
    };
    let mut handler_ref: *mut c_void = ptr::null_mut();
    let status = InstallEventHandler(target, handler, 1, &spec, user, &mut handler_ref);
    if status != 0 {
        return Err(status);
    }

    let id = EventHotKeyID {
        signature: fourcc(b"mfkr"),
        id: 1,
    };
    let mut hot_key_ref: *mut c_void = ptr::null_mut();
    let status = RegisterEventHotKey(key_code, modifiers, id, target, 0, &mut hot_key_ref);
    if status != 0 {
        return Err(status);
    }
    Ok(())
}
