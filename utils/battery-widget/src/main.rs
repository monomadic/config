mod battery;

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use battery::{BatteryInfo, BatteryState, read_battery};
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{
    AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel,
};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSBezierPath, NSCellImagePosition, NSColor,
    NSControlStateValueOff, NSControlStateValueOn, NSFont, NSFontAttributeName,
    NSFontWeightRegular, NSForegroundColorAttributeName, NSImage, NSMenu, NSMenuItem, NSStatusBar,
    NSStatusItem, NSStringDrawing, NSVariableStatusItemLength,
};
use objc2_foundation::{
    NSMutableAttributedString, NSMutableDictionary, NSObject, NSPoint, NSRect, NSSize, NSString,
    NSTimer, ns_string,
};

const BATTERY_ICON: &str = "\u{1006E8}"; // SF Symbols battery.100
const LOW_BATTERY_ICON: &str = "\u{1006EA}"; // battery.0
const BOLT_ICON: &str = "\u{1002E6}"; // bolt.fill

const TITLE_FONT_SIZE: f64 = 14.0;
const LOW_BATTERY_THRESHOLD: i32 = 10;
const UPDATE_INTERVAL_SECONDS: f64 = 10.0;

// Bar image geometry, in points.
const BAR_IMAGE_WIDTH: f64 = 40.0;
const BAR_IMAGE_HEIGHT: f64 = 16.0;
const BAR_HEIGHT: f64 = 6.0;
const BAR_RADIUS: f64 = 3.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum LayoutStyle {
    Text,
    IconText,
    BarText,
    IconBar,
    PercentBar,
    Bar,
    BarPower,
    SmartBar,
    SmartBarTimer,
}

const ALL_STYLES: [LayoutStyle; 9] = [
    LayoutStyle::Text,
    LayoutStyle::IconText,
    LayoutStyle::BarText,
    LayoutStyle::IconBar,
    LayoutStyle::PercentBar,
    LayoutStyle::Bar,
    LayoutStyle::BarPower,
    LayoutStyle::SmartBar,
    LayoutStyle::SmartBarTimer,
];

impl LayoutStyle {
    fn label(self) -> &'static str {
        match self {
            LayoutStyle::Text => "Text",
            LayoutStyle::IconText => "Icon and Text",
            LayoutStyle::BarText => "Bar and Text",
            LayoutStyle::IconBar => "Icon and Bar",
            LayoutStyle::PercentBar => "Percentage and Bar",
            LayoutStyle::Bar => "Bar",
            LayoutStyle::BarPower => "Bar and Power",
            LayoutStyle::SmartBar => "Smart Bar",
            LayoutStyle::SmartBarTimer => "Smart Bar and Timer",
        }
    }

    fn key(self) -> &'static str {
        match self {
            LayoutStyle::Text => "text",
            LayoutStyle::IconText => "icon_text",
            LayoutStyle::BarText => "bar_text",
            LayoutStyle::IconBar => "icon_bar",
            LayoutStyle::PercentBar => "percent_bar",
            LayoutStyle::Bar => "bar",
            LayoutStyle::BarPower => "bar_power",
            LayoutStyle::SmartBar => "smart_bar",
            LayoutStyle::SmartBarTimer => "smart_bar_timer",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        ALL_STYLES.iter().copied().find(|s| s.key() == key)
    }
}

fn style_config_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".config/battery-widget/style"))
}

fn load_style() -> LayoutStyle {
    style_config_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|key| LayoutStyle::from_key(key.trim()))
        .unwrap_or(LayoutStyle::SmartBar)
}

fn save_style(style: LayoutStyle) {
    let Some(path) = style_config_path() else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Err(err) = fs::write(&path, style.key()) {
        eprintln!("error saving style: {err}");
    }
}

/// One styled segment of the menu bar title.
struct Run {
    text: String,
    color: Option<Retained<NSColor>>,
    monospaced: bool,
}

impl Run {
    fn plain(text: impl Into<String>) -> Self {
        Run {
            text: text.into(),
            color: None,
            monospaced: false,
        }
    }

    fn colored(text: impl Into<String>, color: Retained<NSColor>) -> Self {
        Run {
            color: Some(color),
            ..Run::plain(text)
        }
    }
}

/// The drawn progress-bar image, and how it sits relative to the text.
struct BarSpec {
    percent: i32,
    fill: Option<Retained<NSColor>>, // None = template image, adapts to menu bar
    bolt: bool,
}

struct TitleSpec {
    runs: Vec<Run>,
    bar: Option<BarSpec>,
    bar_on_left: bool,
}

fn attributed_title(runs: &[Run]) -> Retained<NSMutableAttributedString> {
    let result = NSMutableAttributedString::new();
    for run in runs {
        let weight = unsafe { NSFontWeightRegular };
        let font = if run.monospaced {
            NSFont::monospacedSystemFontOfSize_weight(TITLE_FONT_SIZE, weight)
        } else {
            NSFont::monospacedDigitSystemFontOfSize_weight(TITLE_FONT_SIZE, weight)
        };

        let attrs = NSMutableDictionary::<NSString, AnyObject>::new();
        unsafe {
            attrs.setObject_forKey(&font, ProtocolObject::from_ref(NSFontAttributeName));
            if let Some(color) = &run.color {
                attrs.setObject_forKey(
                    &**color,
                    ProtocolObject::from_ref(NSForegroundColorAttributeName),
                );
            }
        }

        let piece = unsafe {
            NSMutableAttributedString::initWithString_attributes(
                NSMutableAttributedString::alloc(),
                &NSString::from_str(&run.text),
                Some(&attrs),
            )
        };
        result.appendAttributedString(&piece);
    }
    result
}

/// Draw the bar the mockup way: a rounded track with a rounded fill, and the
/// charging bolt overlaid with a dark halo. Neutral bars are template images
/// so macOS tints them to match the menu bar in any appearance. Block-based
/// so AppKit re-renders at the backing scale and current appearance.
fn bar_image(spec: &BarSpec) -> Retained<NSImage> {
    let size = NSSize {
        width: BAR_IMAGE_WIDTH,
        height: BAR_IMAGE_HEIGHT,
    };
    let is_template = spec.fill.is_none();
    let fill = spec.fill.clone().unwrap_or_else(NSColor::blackColor);
    let percent = spec.percent.clamp(0, 100);
    let bolt = spec.bolt;

    let handler = block2::RcBlock::new(move |_bounds: NSRect| -> objc2::runtime::Bool {
        let bar_y = (BAR_IMAGE_HEIGHT - BAR_HEIGHT) / 2.0;
        let track_rect = NSRect {
            origin: NSPoint { x: 0.0, y: bar_y },
            size: NSSize {
                width: BAR_IMAGE_WIDTH,
                height: BAR_HEIGHT,
            },
        };
        fill.colorWithAlphaComponent(0.3).set();
        NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(track_rect, BAR_RADIUS, BAR_RADIUS)
            .fill();

        if percent > 0 {
            let filled_width = (BAR_IMAGE_WIDTH * percent as f64 / 100.0).max(BAR_RADIUS * 2.0);
            let fill_rect = NSRect {
                origin: NSPoint { x: 0.0, y: bar_y },
                size: NSSize {
                    width: filled_width,
                    height: BAR_HEIGHT,
                },
            };
            fill.set();
            NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                fill_rect, BAR_RADIUS, BAR_RADIUS,
            )
            .fill();
        }

        if bolt {
            // White bolt over a dark halo so it reads against the fill in
            // both menu bar appearances.
            let halo = NSColor::blackColor().colorWithAlphaComponent(0.7);
            draw_glyph_centered(BOLT_ICON, 11.0, &halo, size);
            draw_glyph_centered(BOLT_ICON, 9.0, &NSColor::whiteColor(), size);
        }

        objc2::runtime::Bool::YES
    });

    let image = NSImage::imageWithSize_flipped_drawingHandler(size, false, &handler);
    image.setTemplate(is_template);
    image
}

fn draw_glyph_centered(glyph: &str, font_size: f64, color: &NSColor, canvas: NSSize) {
    let attrs = NSMutableDictionary::<NSString, AnyObject>::new();
    let font = NSFont::systemFontOfSize(font_size);
    unsafe {
        attrs.setObject_forKey(&font, ProtocolObject::from_ref(NSFontAttributeName));
        attrs.setObject_forKey(
            color,
            ProtocolObject::from_ref(NSForegroundColorAttributeName),
        );
    }
    let text = NSString::from_str(glyph);
    let glyph_size = unsafe { text.sizeWithAttributes(Some(&attrs)) };
    let origin = NSPoint {
        x: (canvas.width - glyph_size.width) / 2.0,
        y: (canvas.height - glyph_size.height) / 2.0,
    };
    unsafe { text.drawAtPoint_withAttributes(origin, Some(&attrs)) };
}

fn state_color(info: &BatteryInfo) -> Option<Retained<NSColor>> {
    match info.state {
        BatteryState::Charging => Some(NSColor::systemGreenColor()),
        BatteryState::Discharging if info.percent < LOW_BATTERY_THRESHOLD => {
            Some(NSColor::systemRedColor())
        }
        _ if info.low_power_mode => Some(NSColor::systemYellowColor()),
        _ => None,
    }
}

// power_text renders battery power flow compactly: "8.4w" discharging,
// "+42w" charging.
fn power_text(info: &BatteryInfo) -> String {
    let prefix = if info.state == BatteryState::Charging {
        "+"
    } else {
        ""
    };
    if info.watts >= 10.0 {
        format!("{prefix}{:.0}w", info.watts)
    } else {
        format!("{prefix}{:.1}w", info.watts)
    }
}

fn status_icon(info: &BatteryInfo) -> &'static str {
    match info.state {
        BatteryState::Charging => BOLT_ICON,
        _ if info.percent < LOW_BATTERY_THRESHOLD => LOW_BATTERY_ICON,
        _ => BATTERY_ICON,
    }
}

fn neutral_bar(percent: i32) -> Option<BarSpec> {
    Some(BarSpec {
        percent,
        fill: None,
        bolt: false,
    })
}

fn smart_title(info: &BatteryInfo, with_timer: bool) -> TitleSpec {
    let accent = state_color(info);
    let accented = |text: String| match &accent {
        Some(color) => Run::colored(text, color.clone()),
        None => Run::plain(text),
    };

    let mut runs = vec![accented(power_text(info))];
    if with_timer {
        if let Some(time) = &info.time_remaining {
            runs.push(Run::colored(
                " · ".to_string(),
                NSColor::tertiaryLabelColor(),
            ));
            runs.push(Run {
                monospaced: true,
                ..Run::colored(time.clone(), NSColor::secondaryLabelColor())
            });
        }
    }

    TitleSpec {
        runs,
        bar: Some(BarSpec {
            percent: info.percent,
            fill: accent,
            bolt: info.state == BatteryState::Charging,
        }),
        bar_on_left: true,
    }
}

fn title_spec(info: &BatteryInfo, style: LayoutStyle) -> TitleSpec {
    let percent = format!("{}%", info.percent);
    let text_only = |runs: Vec<Run>| TitleSpec {
        runs,
        bar: None,
        bar_on_left: true,
    };

    match style {
        LayoutStyle::Text => text_only(vec![Run::plain(percent)]),
        LayoutStyle::IconText => {
            text_only(vec![Run::plain(format!("{} {percent}", status_icon(info)))])
        }
        LayoutStyle::BarText => TitleSpec {
            runs: vec![Run::plain(percent)],
            bar: neutral_bar(info.percent),
            bar_on_left: true,
        },
        LayoutStyle::IconBar => TitleSpec {
            runs: vec![Run::colored(
                BOLT_ICON.to_string(),
                NSColor::secondaryLabelColor(),
            )],
            bar: neutral_bar(info.percent),
            bar_on_left: false,
        },
        LayoutStyle::PercentBar => TitleSpec {
            runs: vec![Run::plain(percent)],
            bar: neutral_bar(info.percent),
            bar_on_left: false,
        },
        LayoutStyle::Bar => TitleSpec {
            runs: Vec::new(),
            bar: neutral_bar(info.percent),
            bar_on_left: true,
        },
        LayoutStyle::BarPower => TitleSpec {
            runs: vec![Run::plain(power_text(info))],
            bar: neutral_bar(info.percent),
            bar_on_left: true,
        },
        LayoutStyle::SmartBar => smart_title(info, false),
        LayoutStyle::SmartBarTimer => smart_title(info, true),
    }
}

struct Ui {
    status_item: Retained<NSStatusItem>,
    info_status: Retained<NSMenuItem>,
    info_power: Retained<NSMenuItem>,
    info_health: Retained<NSMenuItem>,
    lpm_item: Retained<NSMenuItem>,
    style_items: Vec<Retained<NSMenuItem>>,
    style: LayoutStyle,
    last_info: Option<BatteryInfo>,
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
            let style = ALL_STYLES[sender.tag() as usize];
            if let Some(ui) = self.ivars().borrow_mut().as_mut() {
                ui.style = style;
            }
            save_style(style);
            self.refresh_style_checks();
            self.update();
        }

        #[unsafe(method(lpmAction:))]
        fn lpm_action(&self, _sender: &NSMenuItem) {
            let enabled = self
                .ivars()
                .borrow()
                .as_ref()
                .and_then(|ui| ui.last_info.as_ref().map(|i| i.low_power_mode))
                .unwrap_or(false);
            std::thread::spawn(move || toggle_low_power_mode(enabled));
        }
    }
);

impl Widget {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(RefCell::new(None));
        unsafe { msg_send![super(this), init] }
    }

    fn build_ui(&self, mtm: MainThreadMarker) {
        let status_bar = NSStatusBar::systemStatusBar();
        let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);

        let menu = NSMenu::new(mtm);
        menu.setAutoenablesItems(false);

        let info_item = |title: &str| {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(title));
            item.setEnabled(false);
            menu.addItem(&item);
            item
        };
        let info_status = info_item("Battery: —");
        let info_power = info_item("Power draw: —");
        let info_health = info_item("Health: —");
        menu.addItem(&NSMenuItem::separatorItem(mtm));

        let style_menu = NSMenu::new(mtm);
        style_menu.setAutoenablesItems(false);
        let mut style_items = Vec::new();
        for (index, style) in ALL_STYLES.iter().enumerate() {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(style.label()));
            item.setTag(index as isize);
            item.setEnabled(true);
            unsafe {
                item.setTarget(Some(self.as_ref()));
                item.setAction(Some(sel!(styleAction:)));
            }
            style_menu.addItem(&item);
            style_items.push(item);
        }
        let style_root = NSMenuItem::new(mtm);
        style_root.setTitle(ns_string!("Style"));
        style_root.setEnabled(true);
        style_root.setSubmenu(Some(&style_menu));
        menu.addItem(&style_root);

        menu.addItem(&NSMenuItem::separatorItem(mtm));

        let lpm_item = NSMenuItem::new(mtm);
        lpm_item.setTitle(ns_string!("Low Power Mode: Off"));
        lpm_item.setEnabled(true);
        unsafe {
            lpm_item.setTarget(Some(self.as_ref()));
            lpm_item.setAction(Some(sel!(lpmAction:)));
        }
        menu.addItem(&lpm_item);

        menu.addItem(&NSMenuItem::separatorItem(mtm));

        let quit_item = NSMenuItem::new(mtm);
        quit_item.setTitle(ns_string!("Quit"));
        quit_item.setEnabled(true);
        unsafe { quit_item.setAction(Some(sel!(terminate:))) };
        menu.addItem(&quit_item);

        // The native menu path: macOS anchors and presents it instantly.
        status_item.setMenu(Some(&menu));

        *self.ivars().borrow_mut() = Some(Ui {
            status_item,
            info_status,
            info_power,
            info_health,
            lpm_item,
            style_items,
            style: load_style(),
            last_info: None,
        });
        self.refresh_style_checks();
    }

    fn update(&self) {
        let info = match read_battery() {
            Ok(info) => info,
            Err(err) => {
                eprintln!("error reading battery: {err}");
                return;
            }
        };

        let style = {
            let mut ivars = self.ivars().borrow_mut();
            let Some(ui) = ivars.as_mut() else { return };
            ui.last_info = Some(info.clone());
            ui.style
        };

        let spec = title_spec(&info, style);
        let title = attributed_title(&spec.runs);

        let ivars = self.ivars().borrow();
        let Some(ui) = ivars.as_ref() else { return };
        if let Some(button) = ui.status_item.button(MainThreadMarker::new().unwrap()) {
            button.setAttributedTitle(&title);
            match &spec.bar {
                Some(bar) => {
                    button.setImage(Some(&bar_image(bar)));
                    button.setImagePosition(if spec.runs.is_empty() {
                        NSCellImagePosition::ImageOnly
                    } else if spec.bar_on_left {
                        NSCellImagePosition::ImageLeft
                    } else {
                        NSCellImagePosition::ImageRight
                    });
                }
                None => {
                    button.setImage(None);
                    button.setImagePosition(NSCellImagePosition::NoImage);
                }
            }
        }

        let mut status_line = format!("Battery: {}%", info.percent);
        match (info.state, &info.time_remaining) {
            (BatteryState::Charging, Some(time)) => {
                status_line += &format!(" — {time} until full");
            }
            (BatteryState::Discharging, Some(time)) => {
                status_line += &format!(" — {time} remaining");
            }
            (BatteryState::Idle, _) if info.on_ac => status_line += " — charged",
            _ => {}
        }
        let source = if info.on_ac {
            "power adapter"
        } else {
            "battery"
        };
        ui.info_status.setTitle(&NSString::from_str(&status_line));
        ui.info_power.setTitle(&NSString::from_str(&format!(
            "Power draw: {} ({source})",
            power_text(&info)
        )));
        ui.info_health.setTitle(&NSString::from_str(&format!(
            "Health: {}% · {} cycles",
            info.health_percent, info.cycle_count
        )));
        ui.lpm_item
            .setTitle(&NSString::from_str(if info.low_power_mode {
                "Low Power Mode: On"
            } else {
                "Low Power Mode: Off"
            }));
    }

    fn refresh_style_checks(&self) {
        let ivars = self.ivars().borrow();
        let Some(ui) = ivars.as_ref() else { return };
        for (index, item) in ui.style_items.iter().enumerate() {
            item.setState(if ALL_STYLES[index] == ui.style {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            });
        }
    }
}

fn toggle_low_power_mode(currently_enabled: bool) {
    let target = if currently_enabled { "0" } else { "1" };
    let script =
        format!("do shell script \"pmset -a lowpowermode {target}\" with administrator privileges");
    if let Err(err) = Command::new("osascript").args(["-e", &script]).status() {
        eprintln!("error toggling low power mode: {err}");
    }
}

fn main() {
    let mtm = MainThreadMarker::new().expect("must run on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let widget = Widget::new(mtm);
    widget.build_ui(mtm);
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
