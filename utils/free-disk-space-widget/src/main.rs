//! free-disk-space-widget — free space on the startup disk, in the menu bar.
//!
//! Rust talking to AppKit directly via [objc2](https://github.com/madsmtm/objc2),
//! in the same family as `battery-widget`, `job-folder` and `menu-tidy`: no
//! wrapper library, no vendored fork, no `.app` bundle. The dropdown is a real
//! `NSMenu` assigned to the status item, rebuilt from the mount table each time
//! it opens, so the volume list is never stale.
//!
//! Sizing is macOS's business, not ours: text uses the menu bar font, menu rows
//! use the menu font, and the drawn bar is scaled from those fonts' metrics.

mod bar;
mod row;
mod volumes;

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSCellImagePosition, NSColor,
    NSControlStateValueOff, NSControlStateValueOn, NSMenu, NSMenuDelegate, NSMenuItem, NSStatusBar,
    NSStatusItem, NSVariableStatusItemLength,
};
use objc2_foundation::{NSObject, NSObjectProtocol, NSString, NSTimer, ns_string};

use volumes::{Volume, format_bytes};

const DISK_ICON: &str = "\u{100902}"; // SF Symbols internaldrive.fill
const LOW_DISK_ICON: &str = "\u{101625}"; // internaldrive.badge.xmark

/// Below this much free space the title turns red and swaps its glyph.
const LOW_SPACE_RATIO: f64 = 0.10;
const UPDATE_INTERVAL_SECONDS: f64 = 10.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum LayoutStyle {
    Text,
    IconText,
    BarText,
    IconBar,
    Bar,
}

const ALL_STYLES: [LayoutStyle; 5] = [
    LayoutStyle::Text,
    LayoutStyle::IconText,
    LayoutStyle::BarText,
    LayoutStyle::IconBar,
    LayoutStyle::Bar,
];

impl LayoutStyle {
    fn label(self) -> &'static str {
        match self {
            LayoutStyle::Text => "Text",
            LayoutStyle::IconText => "Icon and Text",
            LayoutStyle::BarText => "Bar and Text",
            LayoutStyle::IconBar => "Icon and Bar",
            LayoutStyle::Bar => "Bar",
        }
    }

    fn key(self) -> &'static str {
        match self {
            LayoutStyle::Text => "text",
            LayoutStyle::IconText => "icon_text",
            LayoutStyle::BarText => "bar_text",
            LayoutStyle::IconBar => "icon_bar",
            LayoutStyle::Bar => "bar",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        // `bar_icon` is the pre-rename spelling of the same layout.
        if key == "bar_icon" {
            return Some(LayoutStyle::IconBar);
        }
        ALL_STYLES.iter().copied().find(|style| style.key() == key)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DisplayMode {
    Gigabytes,
    Percent,
}

const ALL_MODES: [DisplayMode; 2] = [DisplayMode::Gigabytes, DisplayMode::Percent];

impl DisplayMode {
    fn label(self) -> &'static str {
        match self {
            DisplayMode::Gigabytes => "Free Space",
            DisplayMode::Percent => "Percentage",
        }
    }

    fn key(self) -> &'static str {
        match self {
            DisplayMode::Gigabytes => "gb",
            DisplayMode::Percent => "percent",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        ALL_MODES.iter().copied().find(|mode| mode.key() == key)
    }
}

#[derive(Clone, Copy)]
struct Settings {
    style: LayoutStyle,
    display: DisplayMode,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            style: LayoutStyle::BarText,
            display: DisplayMode::Gigabytes,
        }
    }
}

impl Settings {
    /// `key=value` lines, in the same place `battery-widget` keeps its style.
    fn path() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        Some(PathBuf::from(home).join(".config/free-disk-space-widget/settings"))
    }

    fn load() -> Self {
        let mut settings = Settings::default();
        let Some(text) = Settings::path().and_then(|path| fs::read_to_string(path).ok()) else {
            return settings;
        };

        for line in text.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key.trim() {
                "style" => {
                    if let Some(style) = LayoutStyle::from_key(value.trim()) {
                        settings.style = style;
                    }
                }
                "display" => {
                    if let Some(display) = DisplayMode::from_key(value.trim()) {
                        settings.display = display;
                    }
                }
                _ => {}
            }
        }
        settings
    }

    fn save(&self) {
        let Some(path) = Settings::path() else { return };
        if let Some(dir) = path.parent() {
            let _ = fs::create_dir_all(dir);
        }
        let body = format!(
            "style={}\ndisplay={}\n",
            self.style.key(),
            self.display.key()
        );
        if let Err(err) = fs::write(&path, body) {
            eprintln!("error saving settings: {err}");
        }
    }
}

/// What the status item shows: a title, and optionally a bar beside it. A
/// glyph paired with the bar rides *inside* the bar image rather than in the
/// title, which is what keeps the item from padding out around two pieces of
/// content.
struct TitleSpec {
    text: String,
    bar: Option<f64>,
    bar_glyph: Option<&'static str>,
    color: Option<Retained<NSColor>>,
}

fn title_spec(volume: &Volume, settings: Settings) -> TitleSpec {
    let ratio = volume.free_ratio();
    let low = ratio < LOW_SPACE_RATIO;
    let icon = if low { LOW_DISK_ICON } else { DISK_ICON };
    let value = match settings.display {
        DisplayMode::Gigabytes => format_bytes(volume.free),
        DisplayMode::Percent => format!("{:.0}%", ratio * 100.0),
    };
    let color = low.then(NSColor::systemRedColor);

    // The bar fills with free space, matching the number beside it.
    let spec = |text: String, bar: Option<f64>, bar_glyph| TitleSpec {
        text,
        bar,
        bar_glyph,
        color,
    };

    match settings.style {
        LayoutStyle::Text => spec(value, None, None),
        LayoutStyle::IconText => spec(format!("{icon} {value}"), None, None),
        LayoutStyle::BarText => spec(value, Some(ratio), None),
        LayoutStyle::IconBar => spec(String::new(), Some(ratio), Some(icon)),
        LayoutStyle::Bar => spec(String::new(), Some(ratio), None),
    }
}

struct Ui {
    status_item: Retained<NSStatusItem>,
    settings: Settings,
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
            self.update();
        }

        #[unsafe(method(styleAction:))]
        fn style_action(&self, sender: &NSMenuItem) {
            self.apply(|settings| settings.style = ALL_STYLES[sender.tag() as usize]);
        }

        #[unsafe(method(displayAction:))]
        fn display_action(&self, sender: &NSMenuItem) {
            self.apply(|settings| settings.display = ALL_MODES[sender.tag() as usize]);
        }

        #[unsafe(method(openDiskUtility:))]
        fn open_disk_utility(&self, _sender: &NSMenuItem) {
            open_application("Disk Utility");
        }

        #[unsafe(method(openDaisyDisk:))]
        fn open_daisy_disk(&self, _sender: &NSMenuItem) {
            open_application("DaisyDisk");
        }
    }

    unsafe impl NSObjectProtocol for Widget {}

    // Rebuilding on demand is what keeps the volume list honest: macOS calls
    // this immediately before the menu is shown.
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

        *this.ivars().borrow_mut() = Some(Ui {
            status_item,
            settings: Settings::load(),
        });
        this
    }

    fn apply(&self, change: impl FnOnce(&mut Settings)) {
        let settings = {
            let mut ivars = self.ivars().borrow_mut();
            let Some(ui) = ivars.as_mut() else { return };
            change(&mut ui.settings);
            ui.settings
        };
        settings.save();
        self.update();
    }

    fn update(&self) {
        let mtm = MainThreadMarker::new().unwrap();
        let Some(volume) = volumes::startup() else {
            eprintln!("error reading disk space for /");
            return;
        };

        let ivars = self.ivars().borrow();
        let Some(ui) = ivars.as_ref() else { return };
        let Some(button) = ui.status_item.button(mtm) else {
            return;
        };

        let spec = title_spec(&volume, ui.settings);
        button.setAttributedTitle(&bar::attributed_title(&spec.text, spec.color.as_deref()));
        match spec.bar {
            Some(ratio) => {
                button.setImage(Some(&bar::bar_image(
                    ratio,
                    spec.color.as_deref(),
                    spec.bar_glyph,
                )));
                button.setImagePosition(if spec.text.is_empty() {
                    NSCellImagePosition::ImageOnly
                } else {
                    NSCellImagePosition::ImageLeft
                });
            }
            None => {
                button.setImage(None);
                button.setImagePosition(NSCellImagePosition::NoImage);
            }
        }

        button.setToolTip(Some(&NSString::from_str(&format!(
            "{} — {} free of {} ({:.0}%)",
            volume.name,
            format_bytes(volume.free),
            format_bytes(volume.total),
            volume.free_ratio() * 100.0
        ))));
    }

    fn rebuild_menu(&self, menu: &NSMenu) {
        let mtm = MainThreadMarker::new().unwrap();
        menu.removeAllItems();

        let settings = {
            let ivars = self.ivars().borrow();
            let Some(ui) = ivars.as_ref() else { return };
            ui.settings
        };

        let info = |title: &str| {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(title));
            item.setEnabled(false);
            menu.addItem(&item);
        };
        let action = |title: &str, selector| -> Retained<NSMenuItem> {
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

        let volumes = volumes::mounted();
        if volumes.is_empty() {
            info("No mounted volumes");
        } else {
            let layout = row::layout(&volumes);
            for volume in &volumes {
                let item = NSMenuItem::new(mtm);
                item.setEnabled(true);
                item.setView(Some(&row::VolumeRow::new(volume, &layout, mtm)));
                menu.addItem(&item);
            }
        }

        menu.addItem(&NSMenuItem::separatorItem(mtm));
        self.add_submenu(
            menu,
            "Style",
            ALL_STYLES.iter().map(|style| style.label()),
            ALL_STYLES.iter().position(|s| *s == settings.style),
            sel!(styleAction:),
            mtm,
        );
        self.add_submenu(
            menu,
            "Show In Menu Bar",
            ALL_MODES.iter().map(|mode| mode.label()),
            ALL_MODES.iter().position(|m| *m == settings.display),
            sel!(displayAction:),
            mtm,
        );

        menu.addItem(&NSMenuItem::separatorItem(mtm));
        action("Open Disk Utility", sel!(openDiskUtility:));
        action("Open DaisyDisk", sel!(openDaisyDisk:));

        menu.addItem(&NSMenuItem::separatorItem(mtm));
        let quit = NSMenuItem::new(mtm);
        quit.setTitle(ns_string!("Quit"));
        quit.setEnabled(true);
        unsafe { quit.setAction(Some(sel!(terminate:))) };
        menu.addItem(&quit);
    }

    fn add_submenu<'a>(
        &self,
        menu: &NSMenu,
        title: &str,
        labels: impl Iterator<Item = &'a str>,
        selected: Option<usize>,
        selector: objc2::runtime::Sel,
        mtm: MainThreadMarker,
    ) {
        let submenu = NSMenu::new(mtm);
        submenu.setAutoenablesItems(false);
        for (index, label) in labels.enumerate() {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(label));
            item.setTag(index as isize);
            item.setEnabled(true);
            item.setState(if selected == Some(index) {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            });
            unsafe {
                item.setTarget(Some(self.as_ref()));
                item.setAction(Some(selector));
            }
            submenu.addItem(&item);
        }

        let root = NSMenuItem::new(mtm);
        root.setTitle(&NSString::from_str(title));
        root.setEnabled(true);
        root.setSubmenu(Some(&submenu));
        menu.addItem(&root);
    }
}

fn open_application(name: &str) {
    if let Err(err) = Command::new("open").args(["-a", name]).spawn() {
        eprintln!("error opening {name}: {err}");
    }
}

fn main() {
    let mtm = MainThreadMarker::new().expect("must run on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let widget = Widget::new(mtm);
    widget.update();

    unsafe {
        NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
            UPDATE_INTERVAL_SECONDS,
            &widget,
            sel!(tick:),
            None,
            true,
        );
    }

    app.run();
}
