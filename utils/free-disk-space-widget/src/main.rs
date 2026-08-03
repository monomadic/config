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
    NSControlStateValueOff, NSControlStateValueOn, NSImage, NSImageSymbolConfiguration,
    NSImageView, NSMenu, NSMenuDelegate, NSMenuItem, NSStatusBar, NSStatusItem,
    NSVariableStatusItemLength,
};
use objc2_foundation::{
    NSObject, NSObjectProtocol, NSPoint, NSSize, NSString, NSTimer, ns_string,
};

use bar::Fill;
use volumes::{Volume, format_bytes, format_compact_bytes};

const DISK_ICON: &str = "\u{100902}"; // SF Symbols internaldrive.fill

/// Below this much free space a pulsing red badge appears over the item.
const LOW_SPACE_RATIO: f64 = 0.10;
const UPDATE_INTERVAL_SECONDS: f64 = 10.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum LayoutStyle {
    IconTextBar,
    Text,
    IconText,
    BarText,
    IconBar,
    Bar,
}

const ALL_STYLES: [LayoutStyle; 6] = [
    LayoutStyle::IconTextBar,
    LayoutStyle::Text,
    LayoutStyle::IconText,
    LayoutStyle::BarText,
    LayoutStyle::IconBar,
    LayoutStyle::Bar,
];

impl LayoutStyle {
    fn label(self) -> &'static str {
        match self {
            LayoutStyle::IconTextBar => "Icon, Text and Bar",
            LayoutStyle::Text => "Text",
            LayoutStyle::IconText => "Icon and Text",
            LayoutStyle::BarText => "Bar and Text",
            LayoutStyle::IconBar => "Icon and Bar",
            LayoutStyle::Bar => "Bar",
        }
    }

    fn key(self) -> &'static str {
        match self {
            LayoutStyle::IconTextBar => "icon_text_bar",
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

/// Which quantity the menu bar reports.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Metric {
    Free,
    Used,
}

const ALL_METRICS: [Metric; 2] = [Metric::Free, Metric::Used];

impl Metric {
    fn label(self) -> &'static str {
        match self {
            Metric::Free => "Free Space",
            Metric::Used => "Used Space",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Metric::Free => "free",
            Metric::Used => "used",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        ALL_METRICS.iter().copied().find(|metric| metric.key() == key)
    }
}

/// How that quantity is written: as a percentage or in bytes units.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Unit {
    Percent,
    Bytes,
}

const ALL_UNITS: [Unit; 2] = [Unit::Percent, Unit::Bytes];

impl Unit {
    fn label(self) -> &'static str {
        match self {
            Unit::Percent => "Percentage",
            Unit::Bytes => "Unit",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Unit::Percent => "percent",
            Unit::Bytes => "bytes",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        ALL_UNITS.iter().copied().find(|unit| unit.key() == key)
    }
}

#[derive(Clone, Copy)]
struct Settings {
    style: LayoutStyle,
    metric: Metric,
    unit: Unit,
    /// Count purgeable space as free (Finder's convention) or as used.
    include_purgeable: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            style: LayoutStyle::BarText,
            metric: Metric::Free,
            unit: Unit::Bytes,
            include_purgeable: true,
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
                "metric" => {
                    if let Some(metric) = Metric::from_key(value.trim()) {
                        settings.metric = metric;
                    }
                }
                "unit" => {
                    if let Some(unit) = Unit::from_key(value.trim()) {
                        settings.unit = unit;
                    }
                }
                "purgeable" => settings.include_purgeable = value.trim() != "off",
                // Pre-Format-menu spelling: `display` carried both choices.
                "display" => match value.trim() {
                    "gb" => settings.unit = Unit::Bytes,
                    "percent" => settings.unit = Unit::Percent,
                    _ => {}
                },
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
            "style={}\nmetric={}\nunit={}\npurgeable={}\n",
            self.style.key(),
            self.metric.key(),
            self.unit.key(),
            if self.include_purgeable { "on" } else { "off" }
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
    bar: Option<Fill>,
    bar_glyph: Option<&'static str>,
    compact_image_text: Option<String>,
}

fn title_spec(volume: &Volume, settings: Settings) -> TitleSpec {
    let include = settings.include_purgeable;
    let amount = match settings.metric {
        Metric::Free => volume.available(include),
        Metric::Used => volume.used(include),
    };
    let ratio = match settings.metric {
        Metric::Free => volume.available_ratio(include),
        Metric::Used => volume.used_ratio(include),
    };
    let value = match settings.unit {
        Unit::Bytes => format_bytes(amount),
        Unit::Percent => format!("{:.0}%", ratio * 100.0),
    };
    let compact_value = match settings.unit {
        Unit::Bytes => format_compact_bytes(amount),
        Unit::Percent => format!("{:.0}%", ratio * 100.0),
    };

    // The bar reads left to right: used solid, purgeable translucent, free
    // dim. With purgeable counted as used it simply joins the solid fill.
    let fill = Fill {
        used: volume.used_ratio(include),
        purgeable: if include { volume.purgeable_ratio() } else { 0.0 },
    };

    let spec = |text: String, bar: Option<Fill>, bar_glyph, compact_image_text| TitleSpec {
        text,
        bar,
        bar_glyph,
        compact_image_text,
    };

    match settings.style {
        LayoutStyle::IconTextBar => spec(
            String::new(),
            Some(fill),
            Some(DISK_ICON),
            Some(compact_value),
        ),
        LayoutStyle::Text => spec(value, None, None, None),
        LayoutStyle::IconText => spec(String::new(), None, Some(DISK_ICON), Some(compact_value)),
        LayoutStyle::BarText => spec(value, Some(fill), None, None),
        LayoutStyle::IconBar => spec(String::new(), Some(fill), Some(DISK_ICON), None),
        LayoutStyle::Bar => spec(String::new(), Some(fill), None, None),
    }
}

struct Ui {
    status_item: Retained<NSStatusItem>,
    settings: Settings,
    /// The pulsing low-space badge, created the first time it is needed and
    /// hidden rather than torn down when space recovers.
    badge: Option<Retained<NSImageView>>,
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

        #[unsafe(method(metricAction:))]
        fn metric_action(&self, sender: &NSMenuItem) {
            self.apply(|settings| settings.metric = ALL_METRICS[sender.tag() as usize]);
        }

        #[unsafe(method(unitAction:))]
        fn unit_action(&self, sender: &NSMenuItem) {
            self.apply(|settings| settings.unit = ALL_UNITS[sender.tag() as usize]);
        }

        #[unsafe(method(purgeableAction:))]
        fn purgeable_action(&self, _sender: &NSMenuItem) {
            self.apply(|settings| settings.include_purgeable = !settings.include_purgeable);
        }

        #[unsafe(method(quitAction:))]
        fn quit_action(&self, _sender: &NSMenuItem) {
            let mtm = MainThreadMarker::new().unwrap();
            NSApplication::sharedApplication(mtm).terminate(None);
        }

        #[unsafe(method(ejectAll:))]
        fn eject_all(&self, _sender: &NSMenuItem) {
            for volume in volumes::mounted().iter().filter(|v| v.unmountable()) {
                volumes::unmount(&volume.path);
            }
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
            badge: None,
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

        let mut ivars = self.ivars().borrow_mut();
        let Some(ui) = ivars.as_mut() else { return };
        let Some(button) = ui.status_item.button(mtm) else {
            return;
        };

        let spec = title_spec(&volume, ui.settings);
        button.setAttributedTitle(&bar::attributed_title(&spec.text));
        if let Some(text) = spec.compact_image_text.as_deref() {
            let glyph = spec
                .bar_glyph
                .expect("compact image layouts always have a glyph");
            let image = match spec.bar {
                Some(fill) => bar::stacked_image(fill, glyph, text),
                None => bar::icon_text_image(glyph, text),
            };
            button.setImage(Some(&image));
            button.setImagePosition(NSCellImagePosition::ImageOnly);
        } else {
            match spec.bar {
                Some(fill) => {
                    let image = bar::bar_image(fill, spec.bar_glyph);
                    button.setImage(Some(&image));
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
        }

        // The low-space warning lives in its own layer over the item, so the
        // icon and text keep their normal menu bar colour underneath it.
        let low = volume.available_ratio(ui.settings.include_purgeable) < LOW_SPACE_RATIO;
        if low && ui.badge.is_none() {
            let badge = warning_badge(mtm);
            button.addSubview(&badge);
            ui.badge = Some(badge);
        }
        if let Some(badge) = ui.badge.as_ref() {
            badge.setHidden(!low);
            if low {
                badge.setFrameOrigin(NSPoint { x: 0.0, y: 0.0 });
            }
        }

        let purgeable_note = if volume.purgeable > 0 {
            format!(", {} purgeable", format_bytes(volume.purgeable))
        } else {
            String::new()
        };
        button.setToolTip(Some(&NSString::from_str(&format!(
            "{} — {} free of {} ({:.0}%){}",
            volume.name,
            format_bytes(volume.available(ui.settings.include_purgeable)),
            format_bytes(volume.total),
            volume.available_ratio(ui.settings.include_purgeable) * 100.0,
            purgeable_note
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
            let layout = row::layout(&volumes, settings.include_purgeable);
            for volume in &volumes {
                let item = NSMenuItem::new(mtm);
                item.setEnabled(true);
                item.setView(Some(&row::VolumeRow::new(
                    volume,
                    &layout,
                    settings.include_purgeable,
                    mtm,
                )));
                menu.addItem(&item);
            }
        }

        // Greyed out rather than hidden when nothing can eject, so the menu
        // keeps the same shape whatever is plugged in.
        menu.addItem(&NSMenuItem::separatorItem(mtm));
        let eject_all = action("Eject All", sel!(ejectAll:));
        eject_all.setEnabled(volumes.iter().any(Volume::unmountable));

        menu.addItem(&NSMenuItem::separatorItem(mtm));
        self.add_submenu(
            menu,
            "Style",
            ALL_STYLES.iter().map(|style| style.label()),
            ALL_STYLES.iter().position(|s| *s == settings.style),
            sel!(styleAction:),
            mtm,
        );
        self.add_format_submenu(menu, settings, mtm);

        let purgeable = action("Include Purgeable Space", sel!(purgeableAction:));
        purgeable.setState(if settings.include_purgeable {
            NSControlStateValueOn
        } else {
            NSControlStateValueOff
        });

        menu.addItem(&NSMenuItem::separatorItem(mtm));
        action("Open Disk Utility", sel!(openDiskUtility:));
        action("Open DaisyDisk", sel!(openDaisyDisk:));

        // Our own selector rather than `terminate:`: recent macOS decorates
        // menu items it recognises as standard actions with an SF Symbol, and
        // an unfamiliar action is the way to opt out of that.
        menu.addItem(&NSMenuItem::separatorItem(mtm));
        let quit = action("Quit", sel!(quitAction:));
        quit.setImage(None);
    }

    /// Format holds two radio groups: which quantity (Free/Used Space) and
    /// how it is written (Percentage/Unit), separated within one submenu.
    fn add_format_submenu(&self, menu: &NSMenu, settings: Settings, mtm: MainThreadMarker) {
        let submenu = NSMenu::new(mtm);
        submenu.setAutoenablesItems(false);

        let add_choice = |label: &str, tag: usize, on: bool, selector| {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(label));
            item.setTag(tag as isize);
            item.setEnabled(true);
            item.setState(if on {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            });
            unsafe {
                item.setTarget(Some(self.as_ref()));
                item.setAction(Some(selector));
            }
            submenu.addItem(&item);
        };

        for (index, metric) in ALL_METRICS.iter().enumerate() {
            add_choice(
                metric.label(),
                index,
                *metric == settings.metric,
                sel!(metricAction:),
            );
        }
        submenu.addItem(&NSMenuItem::separatorItem(mtm));
        for (index, unit) in ALL_UNITS.iter().enumerate() {
            add_choice(unit.label(), index, *unit == settings.unit, sel!(unitAction:));
        }

        let root = NSMenuItem::new(mtm);
        root.setTitle(ns_string!("Format"));
        root.setEnabled(true);
        root.setSubmenu(Some(&submenu));
        menu.addItem(&root);
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

/// A small red exclamation badge for the top-left corner of the status item.
/// It sits in its own view over the button, so the template icon and text
/// underneath keep their normal menu bar colour; an NSImageView handles no
/// mouse events, so clicks fall through to the button.
fn warning_badge(mtm: MainThreadMarker) -> Retained<NSImageView> {
    let diameter = (NSStatusBar::systemStatusBar().thickness() * 0.62).round();
    let symbol = NSImage::imageWithSystemSymbolName_accessibilityDescription(
        ns_string!("exclamationmark.circle.fill"),
        None,
    )
    .expect("SF Symbols always has exclamationmark.circle.fill");
    let config = NSImageSymbolConfiguration::configurationWithPointSize_weight(
        diameter * 0.9,
        unsafe { objc2_app_kit::NSFontWeightBold },
    );
    let image = symbol.imageWithSymbolConfiguration(&config).unwrap_or(symbol);

    let badge = NSImageView::imageViewWithImage(&image, mtm);
    badge.setFrameSize(NSSize {
        width: diameter,
        height: diameter,
    });
    badge.setContentTintColor(Some(&NSColor::systemRedColor()));
    badge
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
