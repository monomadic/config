//! volume-control-widget — output devices, mute, and a gig guard, in the
//! menu bar.
//!
//! Rust talking to AppKit via [objc2](https://github.com/madsmtm/objc2) and
//! to the CoreAudio HAL directly, in the same family as `battery-widget` and
//! `free-disk-space-widget`: no wrapper library, no vendored fork, no `.app`
//! bundle. The dropdown is a real `NSMenu`, rebuilt from the device list
//! each time it opens.
//!
//! The design premise inverts the usual volume widget: for a DJ, muted is
//! the safe state. Devices carry rules — Always Mute holds a device muted
//! through anything (listener-enforced, no polling window), Never Mute marks
//! the performance output as read-only and worth a warning when it
//! disappears. The menu bar names the route (`DJM`, `INT`) because *where
//! audio goes* is the glanceable fact.

mod audio;
mod bar;
mod row;
mod settings;

use std::cell::RefCell;
use std::collections::HashSet;
use std::process::Command;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSCellImagePosition, NSColor,
    NSControlStateValueOff, NSControlStateValueOn, NSFont, NSFontAttributeName,
    NSForegroundColorAttributeName, NSMenu, NSMenuDelegate, NSMenuItem, NSStatusBar, NSStatusItem,
    NSVariableStatusItemLength,
};
use objc2_foundation::{
    NSMutableAttributedString, NSMutableDictionary, NSObject, NSObjectProtocol, NSString, NSTimer,
    ns_string,
};

use audio::{Device, Transport};
use bar::{BarFill, Chip, Tint};
use row::{RowSpec, RowState};
use settings::{ALL_STYLES, LayoutStyle, Policy, Settings};

const TICK_SECONDS: f64 = 1.0;
/// Belt and braces alongside the listeners: a full refresh at least this often.
const FORCED_REFRESH_TICKS: u32 = 30;

/// The route's health, as the header and the chip report it.
enum Guard {
    /// The Never Mute device is connected and is the default output.
    Active { tag: String },
    /// Connected, but something else holds the route.
    Bypassed { tag: String },
    /// The Never Mute device has disappeared.
    Missing { tag: String, name: String },
    /// Missing, and macOS fell back to an Always Mute device — which was
    /// already held muted, so the reroute could not burst.
    Fallback { tag: String },
    /// No Never Mute rule anywhere: plain volume-widget behaviour.
    None,
}

struct Snapshot {
    devices: Vec<Device>,
    default_id: Option<audio::AudioObjectID>,
    settings: Settings,
    guard: Guard,
}

fn snapshot() -> Snapshot {
    let mut settings = Settings::load();
    let devices = audio::output_devices();

    audio::enforce(&devices, &settings.always_mute);
    settings.remember(&devices);
    settings.save();

    let always_ids: HashSet<audio::AudioObjectID> = devices
        .iter()
        .filter(|device| settings.always_mute.contains(&device.uid))
        .map(|device| device.id)
        .collect();
    audio::watch(&devices, always_ids);

    let default_id = audio::default_output();
    let default_device = devices.iter().find(|device| Some(device.id) == default_id);

    let guard = {
        let expected = settings
            .known
            .iter()
            .filter(|known| settings.never_mute.contains(&known.uid))
            .collect::<Vec<_>>();
        let connected = expected
            .iter()
            .find_map(|known| devices.iter().find(|device| device.uid == known.uid));
        let missing = expected
            .iter()
            .find(|known| !devices.iter().any(|device| device.uid == known.uid));
        match (connected, missing) {
            (Some(device), _) if Some(device.id) == default_id => Guard::Active {
                tag: device.tag(),
            },
            (Some(device), _) => Guard::Bypassed { tag: device.tag() },
            (None, Some(known)) => {
                let fallback = default_device
                    .is_some_and(|device| settings.always_mute.contains(&device.uid));
                if fallback {
                    Guard::Fallback {
                        tag: default_device.map(Device::tag).unwrap_or_default(),
                    }
                } else {
                    Guard::Missing {
                        tag: tag_for(&known.name, known.transport),
                        name: known.name.clone(),
                    }
                }
            }
            (None, None) => Guard::None,
        }
    };

    Snapshot {
        devices,
        default_id,
        settings,
        guard,
    }
}

/// `Device::tag`, for devices we only remember.
fn tag_for(name: &str, transport: Transport) -> String {
    if transport == Transport::BuiltIn {
        return "INT".into();
    }
    let tag: String = name
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .take(4)
        .collect::<String>()
        .to_uppercase();
    if tag.is_empty() { "EXT".into() } else { tag }
}

fn device_symbol(name: &str, transport: Transport) -> &'static str {
    let lowered = name.to_lowercase();
    if lowered.contains("airpods") {
        return "airpods";
    }
    if lowered.contains("headphone") || lowered.contains("wh-") {
        return "headphones";
    }
    match transport {
        Transport::BuiltIn => "laptopcomputer",
        Transport::Bluetooth => "headphones",
        Transport::Usb | Transport::Thunderbolt => "hifispeaker",
        Transport::Hdmi | Transport::DisplayPort => "display",
        Transport::AirPlay => "airplayaudio",
        Transport::Virtual | Transport::Aggregate => "waveform",
        Transport::Other => "speaker.wave.2",
    }
}

fn format_rate(rate: f64) -> Option<String> {
    if rate <= 0.0 {
        return None;
    }
    let khz = rate / 1000.0;
    Some(if (khz - khz.round()).abs() < 0.005 {
        format!("{}khz", khz.round() as u64)
    } else {
        format!("{khz:.1}khz")
    })
}

/// `24bit 48khz – 12 in, 10 out`, dropping whatever the device won't say.
fn spec_line(device: &Device) -> String {
    let mut format = Vec::new();
    if device.bits > 0 {
        format.push(format!("{}bit", device.bits));
    }
    if let Some(rate) = format_rate(device.sample_rate) {
        format.push(rate);
    }
    let channels = if device.inputs > 0 {
        format!("{} in, {} out", device.inputs, device.outputs)
    } else {
        format!("{} out", device.outputs)
    };
    if format.is_empty() {
        channels
    } else {
        format!("{} – {}", format.join(" "), channels)
    }
}

struct Ui {
    status_item: Retained<NSStatusItem>,
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements and Widget does not
    // implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = RefCell<Option<Ui>>]
    struct Widget;

    impl Widget {
        #[unsafe(method(tick:))]
        fn tick(&self, _timer: &NSTimer) {
            // The listeners flag changes; ticks between changes cost a load.
            static FORCED: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let forced =
                FORCED.fetch_add(1, std::sync::atomic::Ordering::Relaxed) % FORCED_REFRESH_TICKS
                    == 0;
            if audio::take_dirty() || forced {
                self.refresh();
            }
        }

        #[unsafe(method(styleAction:))]
        fn style_action(&self, sender: &NSMenuItem) {
            let mut settings = Settings::load();
            settings.style = ALL_STYLES[sender.tag() as usize];
            settings.save();
            self.refresh();
        }

        #[unsafe(method(openSoundSettings:))]
        fn open_sound_settings(&self, _sender: &NSMenuItem) {
            let _ = Command::new("open")
                .arg("x-apple.systempreferences:com.apple.Sound-Settings.extension")
                .spawn();
        }

        #[unsafe(method(quitAction:))]
        fn quit_action(&self, _sender: &NSMenuItem) {
            let mtm = MainThreadMarker::new().unwrap();
            NSApplication::sharedApplication(mtm).terminate(None);
        }
    }

    unsafe impl NSObjectProtocol for Widget {}

    // Rebuilding immediately before display keeps the device list honest.
    unsafe impl NSMenuDelegate for Widget {
        #[unsafe(method(menuNeedsUpdate:))]
        fn menu_needs_update(&self, menu: &NSMenu) {
            self.rebuild_menu(menu);
        }
    }
);

impl Widget {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(RefCell::new(None));
        let this: Retained<Self> = unsafe { msg_send![super(this), init] };

        let status_item =
            NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength);
        let menu = NSMenu::new(mtm);
        menu.setAutoenablesItems(false);
        menu.setDelegate(Some(ProtocolObject::from_ref(&*this)));
        status_item.setMenu(Some(&menu));

        *this.ivars().borrow_mut() = Some(Ui { status_item });
        this
    }

    fn refresh(&self) {
        let mtm = MainThreadMarker::new().unwrap();
        let snapshot = snapshot();

        let ivars = self.ivars().borrow();
        let Some(ui) = ivars.as_ref() else { return };
        let Some(button) = ui.status_item.button(mtm) else {
            return;
        };

        let default_device = snapshot
            .devices
            .iter()
            .find(|device| Some(device.id) == snapshot.default_id);

        let tint = match snapshot.guard {
            Guard::Missing { .. } => Tint::Red,
            Guard::Fallback { .. } => Tint::Orange,
            _ => Tint::Normal,
        };
        let muted = default_device.is_some_and(|device| device.muted);
        let symbol = match snapshot.guard {
            Guard::Missing { .. } => "exclamationmark.triangle.fill",
            _ if muted => "speaker.slash.fill",
            _ => "speaker.wave.2.fill",
        };
        let level = default_device.and_then(|device| device.volume).unwrap_or(0.0);
        let route = match &snapshot.guard {
            Guard::Missing { tag, .. } => tag.clone(),
            Guard::Fallback { tag } => tag.clone(),
            _ => default_device.map(Device::tag).unwrap_or_else(|| "—".into()),
        };
        let percent = match snapshot.guard {
            Guard::Missing { .. } => "—".into(),
            _ => format!("{:.0}%", level * 100.0),
        };

        let style = snapshot.settings.style;
        let chip = Chip {
            symbol,
            bar: matches!(style, LayoutStyle::IconBar | LayoutStyle::IconBarRoute).then_some(
                BarFill {
                    level,
                    dim: muted,
                },
            ),
            text: match style {
                LayoutStyle::IconRoute | LayoutStyle::IconBarRoute => Some(route),
                LayoutStyle::IconPercent => Some(percent),
                LayoutStyle::Icon | LayoutStyle::IconBar => None,
            },
            tint,
        };
        button.setImage(Some(&bar::chip_image(chip)));
        button.setImagePosition(NSCellImagePosition::ImageOnly);

        let tooltip = match default_device {
            Some(device) => format!(
                "{} — {}{}",
                device.name,
                if device.muted { "muted" } else { "unmuted" },
                device
                    .volume
                    .map(|volume| format!(" · {:.0}%", volume * 100.0))
                    .unwrap_or_default()
            ),
            None => "No output device".into(),
        };
        button.setToolTip(Some(&NSString::from_str(&tooltip)));
    }

    fn rebuild_menu(&self, menu: &NSMenu) {
        let mtm = MainThreadMarker::new().unwrap();
        menu.removeAllItems();

        let snapshot = snapshot();

        self.add_header(menu, &snapshot, mtm);
        menu.addItem(&NSMenuItem::separatorItem(mtm));

        let rows = build_rows(&snapshot);
        if rows.is_empty() {
            let item = NSMenuItem::new(mtm);
            item.setTitle(ns_string!("No output devices"));
            item.setEnabled(false);
            menu.addItem(&item);
        } else {
            let layout = row::layout(&rows);
            for spec in rows {
                let item = NSMenuItem::new(mtm);
                item.setEnabled(true);
                item.setView(Some(&row::DeviceRow::new(spec, &layout, mtm)));
                menu.addItem(&item);
            }
        }

        menu.addItem(&NSMenuItem::separatorItem(mtm));
        self.add_style_submenu(menu, snapshot.settings.style, mtm);

        let action = |title: &str, selector| {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(title));
            item.setEnabled(true);
            unsafe {
                item.setTarget(Some(self.as_ref()));
                item.setAction(Some(selector));
            }
            menu.addItem(&item);
            item
        };
        action("Open Sound Settings…", sel!(openSoundSettings:));
        // Our own selector rather than `terminate:` so macOS doesn't decorate
        // the item with its standard-action symbol.
        let quit = action("Quit", sel!(quitAction:));
        quit.setImage(None);
    }

    /// Two disabled lines: the menu's name, and the guard status in its own
    /// colour — green is reserved for "the performance route is confirmed".
    fn add_header(&self, menu: &NSMenu, snapshot: &Snapshot, mtm: MainThreadMarker) {
        let title = NSMenuItem::new(mtm);
        title.setAttributedTitle(Some(&attributed(
            "Audio Outputs",
            &NSFont::menuFontOfSize(0.0),
            &NSColor::labelColor(),
        )));
        title.setEnabled(false);
        menu.addItem(&title);

        let default_device = snapshot
            .devices
            .iter()
            .find(|device| Some(device.id) == snapshot.default_id);
        let (text, color) = match &snapshot.guard {
            Guard::Active { tag } => (format!("{tag} is active"), NSColor::systemGreenColor()),
            Guard::Bypassed { tag } => (
                format!("{tag} connected · not the output"),
                NSColor::secondaryLabelColor(),
            ),
            Guard::Missing { name, .. } => (
                format!("{name} missing · check USB and power"),
                NSColor::systemRedColor(),
            ),
            Guard::Fallback { .. } => (
                "Fallback caught · internal output muted".into(),
                NSColor::systemOrangeColor(),
            ),
            Guard::None => match default_device {
                Some(device) => (
                    format!(
                        "{} · {}",
                        device.name,
                        if device.muted {
                            "muted".to_string()
                        } else {
                            device
                                .volume
                                .map(|volume| format!("{:.0}%", volume * 100.0))
                                .unwrap_or_else(|| "unmuted".into())
                        }
                    ),
                    NSColor::secondaryLabelColor(),
                ),
                None => ("No output device".into(), NSColor::secondaryLabelColor()),
            },
        };

        let size = NSFont::menuFontOfSize(0.0).pointSize() * 0.82;
        let status = NSMenuItem::new(mtm);
        status.setAttributedTitle(Some(&attributed(
            &text,
            &NSFont::systemFontOfSize(size.round()),
            &color,
        )));
        status.setEnabled(false);
        menu.addItem(&status);
    }

    fn add_style_submenu(&self, menu: &NSMenu, current: LayoutStyle, mtm: MainThreadMarker) {
        let submenu = NSMenu::new(mtm);
        submenu.setAutoenablesItems(false);
        for (index, style) in ALL_STYLES.iter().enumerate() {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(style.label()));
            item.setTag(index as isize);
            item.setEnabled(true);
            item.setState(if *style == current {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            });
            unsafe {
                item.setTarget(Some(self.as_ref()));
                item.setAction(Some(sel!(styleAction:)));
            }
            submenu.addItem(&item);
        }
        let root = NSMenuItem::new(mtm);
        root.setTitle(ns_string!("Style"));
        root.setEnabled(true);
        root.setSubmenu(Some(&submenu));
        menu.addItem(&root);
    }
}

/// Rows in the order the design settled on: the performance device first,
/// then the current output, then other connected devices, then missing
/// expected devices, then remembered-but-away devices dimmed at the bottom.
fn build_rows(snapshot: &Snapshot) -> Vec<RowSpec> {
    let settings = &snapshot.settings;
    let mut rows: Vec<(u8, RowSpec)> = Vec::new();

    for device in &snapshot.devices {
        let policy = settings.policy(&device.uid);
        let is_default = Some(device.id) == snapshot.default_id;
        let state = if is_default && policy == Policy::AlwaysMute {
            RowState::Fallback
        } else if is_default {
            RowState::Active
        } else {
            RowState::Normal
        };
        let rank = match (policy, is_default) {
            (Policy::NeverMute, _) => 0,
            (_, true) => 1,
            _ => 2,
        };
        rows.push((
            rank,
            RowSpec {
                device_id: Some(device.id),
                uid: device.uid.clone(),
                name: device.name.clone(),
                transport: device.transport,
                symbol: device_symbol(&device.name, device.transport),
                spec: spec_line(device),
                volume: device.volume,
                muted: device.muted,
                can_mute: device.can_mute,
                state,
                policy,
            },
        ));
    }

    for known in &settings.known {
        if snapshot
            .devices
            .iter()
            .any(|device| device.uid == known.uid)
        {
            continue;
        }
        let policy = settings.policy(&known.uid);
        let (state, spec, rank) = match policy {
            Policy::NeverMute => (RowState::Missing, "No connection".to_string(), 3),
            _ => (RowState::Away, "Not connected".to_string(), 4),
        };
        rows.push((
            rank,
            RowSpec {
                device_id: None,
                uid: known.uid.clone(),
                name: known.name.clone(),
                transport: known.transport,
                symbol: device_symbol(&known.name, known.transport),
                spec,
                volume: None,
                muted: false,
                can_mute: false,
                state,
                policy,
            },
        ));
    }

    rows.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.name.cmp(&b.1.name)));
    rows.into_iter().map(|(_, spec)| spec).collect()
}

fn attributed(
    text: &str,
    font: &NSFont,
    color: &NSColor,
) -> Retained<NSMutableAttributedString> {
    let attrs = NSMutableDictionary::<NSString, AnyObject>::new();
    unsafe {
        attrs.setObject_forKey(font, ProtocolObject::from_ref(NSFontAttributeName));
        attrs.setObject_forKey(
            color,
            ProtocolObject::from_ref(NSForegroundColorAttributeName),
        );
        NSMutableAttributedString::initWithString_attributes(
            objc2::AnyThread::alloc(),
            &NSString::from_str(text),
            Some(&attrs),
        )
    }
}

fn main() {
    // `--dump` prints the device snapshot and exits — for eyeballing what
    // the HAL reports without launching the status item.
    if std::env::args().any(|arg| arg == "--dump") {
        let default_id = audio::default_output();
        for device in audio::output_devices() {
            println!(
                "{}{} [{}] tag={} spec=\"{}\" volume={:?} muted={} can_mute={} uid={}",
                if Some(device.id) == default_id { "* " } else { "  " },
                device.name,
                device.transport.pill().unwrap_or("BUILTIN"),
                device.tag(),
                spec_line(&device),
                device.volume,
                device.muted,
                device.can_mute,
                device.uid,
            );
        }
        return;
    }

    let mtm = MainThreadMarker::new().expect("must run on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let widget = Widget::new(mtm);
    widget.refresh();

    unsafe {
        NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
            TICK_SECONDS,
            &widget,
            sel!(tick:),
            None,
            true,
        );
    }

    app.run();
}
