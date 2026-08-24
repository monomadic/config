//! CoreAudio, called directly.
//!
//! The handful of HAL calls this widget needs are declared by hand rather
//! than through a bindings crate, in the family style: no wrapper library.
//! Reads are cheap property fetches; the only writes are `mute` on devices
//! the user has marked Always Mute, and the default-output device when a row
//! is clicked. Devices carrying a Never Mute rule are read-only by contract —
//! nothing in this module will write to them.
//!
//! Enforcement is event-driven: property listeners re-assert mute the moment
//! anything unmutes a protected device, from the HAL's own listener thread,
//! so a fallback reroute can never produce an audible burst while the UI
//! catches up on its own cadence.

use std::collections::HashSet;
use std::ffi::c_void;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use objc2::rc::Retained;
use objc2_foundation::NSString;

type OSStatus = i32;
pub type AudioObjectID = u32;

#[repr(C)]
#[derive(Clone, Copy)]
struct AudioObjectPropertyAddress {
    selector: u32,
    scope: u32,
    element: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct AudioStreamBasicDescription {
    sample_rate: f64,
    format_id: u32,
    format_flags: u32,
    bytes_per_packet: u32,
    frames_per_packet: u32,
    bytes_per_frame: u32,
    channels_per_frame: u32,
    bits_per_channel: u32,
    reserved: u32,
}

type ListenerProc =
    extern "C" fn(AudioObjectID, u32, *const AudioObjectPropertyAddress, *mut c_void) -> OSStatus;

#[link(name = "CoreAudio", kind = "framework")]
unsafe extern "C" {
    fn AudioObjectGetPropertyData(
        object: AudioObjectID,
        address: *const AudioObjectPropertyAddress,
        qualifier_size: u32,
        qualifier: *const c_void,
        size: *mut u32,
        data: *mut c_void,
    ) -> OSStatus;
    fn AudioObjectGetPropertyDataSize(
        object: AudioObjectID,
        address: *const AudioObjectPropertyAddress,
        qualifier_size: u32,
        qualifier: *const c_void,
        size: *mut u32,
    ) -> OSStatus;
    fn AudioObjectSetPropertyData(
        object: AudioObjectID,
        address: *const AudioObjectPropertyAddress,
        qualifier_size: u32,
        qualifier: *const c_void,
        size: u32,
        data: *const c_void,
    ) -> OSStatus;
    fn AudioObjectHasProperty(object: AudioObjectID, address: *const AudioObjectPropertyAddress)
    -> u8;
    fn AudioObjectAddPropertyListener(
        object: AudioObjectID,
        address: *const AudioObjectPropertyAddress,
        listener: ListenerProc,
        client_data: *mut c_void,
    ) -> OSStatus;
}

const fn fourcc(code: &[u8; 4]) -> u32 {
    u32::from_be_bytes(*code)
}

const SYSTEM_OBJECT: AudioObjectID = 1;

const PROP_DEVICES: u32 = fourcc(b"dev#");
const PROP_DEFAULT_OUTPUT: u32 = fourcc(b"dOut");
const PROP_DEVICE_UID: u32 = fourcc(b"uid ");
const PROP_NAME: u32 = fourcc(b"lnam");
const PROP_TRANSPORT: u32 = fourcc(b"tran");
const PROP_STREAM_CONFIG: u32 = fourcc(b"slay");
const PROP_VOLUME_SCALAR: u32 = fourcc(b"volm");
const PROP_MUTE: u32 = fourcc(b"mute");
const PROP_SAMPLE_RATE: u32 = fourcc(b"nsrt");
const PROP_STREAMS: u32 = fourcc(b"stm#");
const PROP_PHYSICAL_FORMAT: u32 = fourcc(b"pft ");

const SCOPE_GLOBAL: u32 = fourcc(b"glob");
const SCOPE_OUTPUT: u32 = fourcc(b"outp");
const SCOPE_INPUT: u32 = fourcc(b"inpt");
const ELEMENT_MAIN: u32 = 0;

const TRANSPORT_BUILTIN: u32 = fourcc(b"bltn");
const TRANSPORT_USB: u32 = fourcc(b"usb ");
const TRANSPORT_BLUETOOTH: u32 = fourcc(b"blue");
const TRANSPORT_BLUETOOTH_LE: u32 = fourcc(b"blea");
const TRANSPORT_HDMI: u32 = fourcc(b"hdmi");
const TRANSPORT_DISPLAYPORT: u32 = fourcc(b"dprt");
const TRANSPORT_THUNDERBOLT: u32 = fourcc(b"thun");
const TRANSPORT_AIRPLAY: u32 = fourcc(b"airp");
const TRANSPORT_VIRTUAL: u32 = fourcc(b"virt");
const TRANSPORT_AGGREGATE: u32 = fourcc(b"grup");
const TRANSPORT_CONTINUITY: u32 = fourcc(b"cont");

const fn address(selector: u32, scope: u32, element: u32) -> AudioObjectPropertyAddress {
    AudioObjectPropertyAddress {
        selector,
        scope,
        element,
    }
}

fn get<T: Copy>(object: AudioObjectID, addr: AudioObjectPropertyAddress) -> Option<T> {
    let mut value = std::mem::MaybeUninit::<T>::uninit();
    let mut size = size_of::<T>() as u32;
    let status = unsafe {
        AudioObjectGetPropertyData(
            object,
            &addr,
            0,
            std::ptr::null(),
            &mut size,
            value.as_mut_ptr() as *mut c_void,
        )
    };
    (status == 0 && size as usize == size_of::<T>()).then(|| unsafe { value.assume_init() })
}

fn set<T: Copy>(object: AudioObjectID, addr: AudioObjectPropertyAddress, value: T) -> bool {
    let status = unsafe {
        AudioObjectSetPropertyData(
            object,
            &addr,
            0,
            std::ptr::null(),
            size_of::<T>() as u32,
            &value as *const T as *const c_void,
        )
    };
    status == 0
}

/// A CFString-valued property. `AudioObjectGetPropertyData` hands back a +1
/// reference; `Retained::from_raw` takes that ownership through the toll-free
/// NSString bridge.
fn get_string(object: AudioObjectID, selector: u32) -> Option<String> {
    let addr = address(selector, SCOPE_GLOBAL, ELEMENT_MAIN);
    let ptr: *mut NSString = get(object, addr)?;
    let string: Retained<NSString> = unsafe { Retained::from_raw(ptr) }?;
    Some(string.to_string())
}

/// The bus a device hangs off, as far as the row's pill needs to know.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    BuiltIn,
    Usb,
    Bluetooth,
    Hdmi,
    DisplayPort,
    Thunderbolt,
    AirPlay,
    Virtual,
    Aggregate,
    Other,
}

impl Transport {
    fn from_code(code: u32) -> Self {
        match code {
            TRANSPORT_BUILTIN => Transport::BuiltIn,
            TRANSPORT_USB => Transport::Usb,
            TRANSPORT_BLUETOOTH | TRANSPORT_BLUETOOTH_LE | TRANSPORT_CONTINUITY => {
                Transport::Bluetooth
            }
            TRANSPORT_HDMI => Transport::Hdmi,
            TRANSPORT_DISPLAYPORT => Transport::DisplayPort,
            TRANSPORT_THUNDERBOLT => Transport::Thunderbolt,
            TRANSPORT_AIRPLAY => Transport::AirPlay,
            TRANSPORT_VIRTUAL => Transport::Virtual,
            TRANSPORT_AGGREGATE => Transport::Aggregate,
            _ => Transport::Other,
        }
    }

    /// The pill after the device name. Built-in devices wear no pill — the
    /// machine itself is not a bus worth naming.
    pub fn pill(self) -> Option<&'static str> {
        match self {
            Transport::BuiltIn => None,
            Transport::Usb => Some("USB"),
            Transport::Bluetooth => Some("BT"),
            Transport::Hdmi => Some("HDMI"),
            Transport::DisplayPort => Some("DP"),
            Transport::Thunderbolt => Some("TB"),
            Transport::AirPlay => Some("AIRPLAY"),
            Transport::Virtual => Some("VIRTUAL"),
            Transport::Aggregate => Some("AGG"),
            Transport::Other => None,
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Transport::BuiltIn => "builtin",
            Transport::Usb => "usb",
            Transport::Bluetooth => "bluetooth",
            Transport::Hdmi => "hdmi",
            Transport::DisplayPort => "displayport",
            Transport::Thunderbolt => "thunderbolt",
            Transport::AirPlay => "airplay",
            Transport::Virtual => "virtual",
            Transport::Aggregate => "aggregate",
            Transport::Other => "other",
        }
    }

    pub fn from_key(key: &str) -> Self {
        match key {
            "builtin" => Transport::BuiltIn,
            "usb" => Transport::Usb,
            "bluetooth" => Transport::Bluetooth,
            "hdmi" => Transport::Hdmi,
            "displayport" => Transport::DisplayPort,
            "thunderbolt" => Transport::Thunderbolt,
            "airplay" => Transport::AirPlay,
            "virtual" => Transport::Virtual,
            "aggregate" => Transport::Aggregate,
            _ => Transport::Other,
        }
    }
}

/// One output device, read in full — the properties are all cheap fetches.
#[derive(Clone)]
pub struct Device {
    pub id: AudioObjectID,
    pub uid: String,
    pub name: String,
    pub transport: Transport,
    pub outputs: u32,
    pub inputs: u32,
    pub sample_rate: f64,
    /// Zero when the output stream keeps its format to itself.
    pub bits: u32,
    /// None when the device exposes no settable volume (fixed-level DACs).
    pub volume: Option<f64>,
    pub muted: bool,
    pub can_mute: bool,
}

impl Device {
    /// The route tag the menu bar wears: INT for the machine's own output,
    /// otherwise the leading run of letters and digits, up to four.
    pub fn tag(&self) -> String {
        if self.transport == Transport::BuiltIn {
            return "INT".into();
        }
        let tag: String = self
            .name
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .take(4)
            .collect::<String>()
            .to_uppercase();
        if tag.is_empty() { "EXT".into() } else { tag }
    }
}

fn channel_count(id: AudioObjectID, scope: u32) -> u32 {
    let addr = address(PROP_STREAM_CONFIG, scope, ELEMENT_MAIN);
    let mut size = 0u32;
    let status =
        unsafe { AudioObjectGetPropertyDataSize(id, &addr, 0, std::ptr::null(), &mut size) };
    if status != 0 || size == 0 {
        return 0;
    }
    let mut buffer = vec![0u8; size as usize];
    let status = unsafe {
        AudioObjectGetPropertyData(
            id,
            &addr,
            0,
            std::ptr::null(),
            &mut size,
            buffer.as_mut_ptr() as *mut c_void,
        )
    };
    if status != 0 || (size as usize) < 4 {
        return 0;
    }
    // AudioBufferList: a u32 count, then AudioBuffer { u32 channels,
    // u32 byte_size, ptr data } entries. Only the channel counts matter.
    let count = u32::from_ne_bytes(buffer[0..4].try_into().unwrap()) as usize;
    let entry = size_of::<u32>() * 2 + size_of::<*const c_void>();
    // The list head is padded to pointer alignment before the first entry.
    let first = size_of::<*const c_void>();
    (0..count)
        .filter_map(|i| {
            let at = first + i * entry;
            buffer
                .get(at..at + 4)
                .map(|b| u32::from_ne_bytes(b.try_into().unwrap()))
        })
        .sum()
}

/// Volume lives on the main element for some devices and on per-channel
/// elements for others; read whichever answers.
fn output_volume(id: AudioObjectID) -> Option<f64> {
    let main = address(PROP_VOLUME_SCALAR, SCOPE_OUTPUT, ELEMENT_MAIN);
    if unsafe { AudioObjectHasProperty(id, &main) } != 0
        && let Some(volume) = get::<f32>(id, main)
    {
        return Some(volume as f64);
    }
    let channels: Vec<f64> = [1u32, 2u32]
        .iter()
        .filter_map(|&element| {
            get::<f32>(id, address(PROP_VOLUME_SCALAR, SCOPE_OUTPUT, element)).map(f64::from)
        })
        .collect();
    (!channels.is_empty()).then(|| channels.iter().sum::<f64>() / channels.len() as f64)
}

fn set_output_volume(id: AudioObjectID, volume: f64) {
    let value = volume.clamp(0.0, 1.0) as f32;
    let main = address(PROP_VOLUME_SCALAR, SCOPE_OUTPUT, ELEMENT_MAIN);
    if unsafe { AudioObjectHasProperty(id, &main) } != 0 && set(id, main, value) {
        return;
    }
    for element in [1u32, 2u32] {
        set(id, address(PROP_VOLUME_SCALAR, SCOPE_OUTPUT, element), value);
    }
}

fn mute_address(id: AudioObjectID) -> Option<AudioObjectPropertyAddress> {
    let main = address(PROP_MUTE, SCOPE_OUTPUT, ELEMENT_MAIN);
    if unsafe { AudioObjectHasProperty(id, &main) } != 0 {
        return Some(main);
    }
    let channel = address(PROP_MUTE, SCOPE_OUTPUT, 1);
    (unsafe { AudioObjectHasProperty(id, &channel) } != 0).then_some(channel)
}

fn is_muted(id: AudioObjectID) -> bool {
    mute_address(id)
        .and_then(|addr| get::<u32>(id, addr))
        .is_some_and(|muted| muted != 0)
}

pub fn set_muted(id: AudioObjectID, muted: bool) {
    if let Some(addr) = mute_address(id) {
        set(id, addr, u32::from(muted));
    }
}

/// Bit depth of the first output stream's physical format, zero if unknown.
fn output_bits(id: AudioObjectID) -> u32 {
    let addr = address(PROP_STREAMS, SCOPE_OUTPUT, ELEMENT_MAIN);
    let mut size = 0u32;
    if unsafe { AudioObjectGetPropertyDataSize(id, &addr, 0, std::ptr::null(), &mut size) } != 0
        || size < 4
    {
        return 0;
    }
    let mut streams = vec![0 as AudioObjectID; size as usize / 4];
    if unsafe {
        AudioObjectGetPropertyData(
            id,
            &addr,
            0,
            std::ptr::null(),
            &mut size,
            streams.as_mut_ptr() as *mut c_void,
        )
    } != 0
    {
        return 0;
    }
    streams
        .first()
        .and_then(|&stream| {
            get::<AudioStreamBasicDescription>(
                stream,
                address(PROP_PHYSICAL_FORMAT, SCOPE_GLOBAL, ELEMENT_MAIN),
            )
        })
        .map(|format| format.bits_per_channel)
        .unwrap_or(0)
}

fn read_device(id: AudioObjectID) -> Option<Device> {
    let uid = get_string(id, PROP_DEVICE_UID)?;
    let outputs = channel_count(id, SCOPE_OUTPUT);
    if outputs == 0 {
        return None;
    }
    Some(Device {
        id,
        uid,
        name: get_string(id, PROP_NAME).unwrap_or_else(|| "Audio Device".into()),
        transport: Transport::from_code(
            get(id, address(PROP_TRANSPORT, SCOPE_GLOBAL, ELEMENT_MAIN)).unwrap_or(0),
        ),
        outputs,
        inputs: channel_count(id, SCOPE_INPUT),
        sample_rate: get(id, address(PROP_SAMPLE_RATE, SCOPE_GLOBAL, ELEMENT_MAIN)).unwrap_or(0.0),
        bits: output_bits(id),
        volume: output_volume(id),
        muted: is_muted(id),
        can_mute: mute_address(id).is_some(),
    })
}

/// Every device with output channels, in HAL order.
pub fn output_devices() -> Vec<Device> {
    let addr = address(PROP_DEVICES, SCOPE_GLOBAL, ELEMENT_MAIN);
    let mut size = 0u32;
    if unsafe {
        AudioObjectGetPropertyDataSize(SYSTEM_OBJECT, &addr, 0, std::ptr::null(), &mut size)
    } != 0
    {
        return Vec::new();
    }
    let mut ids = vec![0 as AudioObjectID; size as usize / 4];
    if unsafe {
        AudioObjectGetPropertyData(
            SYSTEM_OBJECT,
            &addr,
            0,
            std::ptr::null(),
            &mut size,
            ids.as_mut_ptr() as *mut c_void,
        )
    } != 0
    {
        return Vec::new();
    }
    ids.into_iter().filter_map(read_device).collect()
}

pub fn default_output() -> Option<AudioObjectID> {
    get::<AudioObjectID>(
        SYSTEM_OBJECT,
        address(PROP_DEFAULT_OUTPUT, SCOPE_GLOBAL, ELEMENT_MAIN),
    )
    .filter(|&id| id != 0)
}

pub fn set_default_output(id: AudioObjectID) {
    set(
        SYSTEM_OBJECT,
        address(PROP_DEFAULT_OUTPUT, SCOPE_GLOBAL, ELEMENT_MAIN),
        id,
    );
}

pub fn set_volume(id: AudioObjectID, volume: f64) {
    set_output_volume(id, volume);
}

// --- enforcement -----------------------------------------------------------
//
// The listener callbacks run on the HAL's thread. They touch only this
// module's statics: the set of device IDs currently under Always Mute, and a
// dirty flag the main thread polls. Re-muting happens right here in the
// callback — the whole point is not waiting for the UI.

static ENFORCED: Mutex<Option<HashSet<AudioObjectID>>> = Mutex::new(None);
static DIRTY: AtomicBool = AtomicBool::new(true);

/// True once since the last call — the UI's cue to re-read everything.
pub fn take_dirty() -> bool {
    DIRTY.swap(false, Ordering::Relaxed)
}

/// For UI-side changes (a rule cycled) that should reach the next tick.
pub fn mark_dirty() {
    DIRTY.store(true, Ordering::Relaxed);
}

extern "C" fn on_change(
    id: AudioObjectID,
    _count: u32,
    _addresses: *const AudioObjectPropertyAddress,
    _client: *mut c_void,
) -> OSStatus {
    let enforced = ENFORCED
        .lock()
        .ok()
        .and_then(|set| set.as_ref().map(|set| set.contains(&id)))
        .unwrap_or(false);
    if enforced && !is_muted(id) {
        set_muted(id, true);
    }
    DIRTY.store(true, Ordering::Relaxed);
    0
}

/// Idempotent per object/selector: listener registrations live for the life
/// of the process, so each pair is added once and remembered.
fn listen(registered: &mut HashSet<(AudioObjectID, u32)>, object: AudioObjectID, selector: u32) {
    if !registered.insert((object, selector)) {
        return;
    }
    let addr = address(selector, SCOPE_GLOBAL, ELEMENT_MAIN);
    let addr = if selector == PROP_MUTE || selector == PROP_VOLUME_SCALAR {
        AudioObjectPropertyAddress {
            scope: SCOPE_OUTPUT,
            ..addr
        }
    } else {
        addr
    };
    unsafe { AudioObjectAddPropertyListener(object, &addr, on_change, std::ptr::null_mut()) };
}

/// Point the listeners and the enforcement set at the current world: the
/// device list and default output on the system object, and mute on every
/// device (protected or not — any change should repaint the UI). Call after
/// every refresh; re-registration is a no-op.
pub fn watch(devices: &[Device], always_mute_ids: HashSet<AudioObjectID>) {
    static REGISTERED: Mutex<Option<HashSet<(AudioObjectID, u32)>>> = Mutex::new(None);
    let mut registered = REGISTERED.lock().unwrap();
    let registered = registered.get_or_insert_with(HashSet::new);

    listen(registered, SYSTEM_OBJECT, PROP_DEVICES);
    listen(registered, SYSTEM_OBJECT, PROP_DEFAULT_OUTPUT);
    for device in devices {
        listen(registered, device.id, PROP_MUTE);
        listen(registered, device.id, PROP_VOLUME_SCALAR);
    }

    *ENFORCED.lock().unwrap() = Some(always_mute_ids);
}

/// Re-assert mute on every protected, connected device — run at refresh so
/// a device that reconnected muted-off is caught even before its listener
/// fires again.
pub fn enforce(devices: &[Device], always_mute: &HashSet<String>) {
    for device in devices {
        if always_mute.contains(&device.uid) && device.can_mute && !device.muted {
            set_muted(device.id, true);
        }
    }
}
