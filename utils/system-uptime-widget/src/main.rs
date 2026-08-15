//! system-uptime-widget — how long this Mac has been up, in the menu bar.
//!
//! Rust talking to AppKit directly via [objc2](https://github.com/madsmtm/objc2),
//! in the same family as `battery-widget` and `free-disk-space-widget`: no
//! wrapper library, no vendored fork, no `.app` bundle. The dropdown is a real
//! `NSMenu` assigned to the status item, so macOS presents it natively.
//!
//! Sizing is macOS's business, not ours: every size is a ratio of the menu bar
//! font or of the status bar's own thickness.

mod bar;
mod uptime;

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use objc2::rc::Retained;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSCellImagePosition, NSControlStateValueOff,
    NSControlStateValueOn, NSImage, NSMenu, NSMenuItem, NSStatusBar, NSStatusItem,
    NSVariableStatusItemLength,
};
use objc2_foundation::{NSObject, NSObjectProtocol, NSString, NSTimer, ns_string};

const UPTIME_ICON: &str = "\u{102754}"; // SF Symbols clock.arrow.circlepath

/// The value only ever changes by the minute, and by the hour once the machine
/// has been up for one.
const UPDATE_INTERVAL_SECONDS: f64 = 30.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum LayoutStyle {
    IconAboveText,
    IconText,
    Text,
    Boxed,
    Progress,
}

const ALL_STYLES: [LayoutStyle; 5] = [
    LayoutStyle::IconAboveText,
    LayoutStyle::IconText,
    LayoutStyle::Text,
    LayoutStyle::Boxed,
    LayoutStyle::Progress,
];

impl LayoutStyle {
    fn label(self) -> &'static str {
        match self {
            LayoutStyle::IconAboveText => "Icon above Text",
            LayoutStyle::IconText => "Icon and Text",
            LayoutStyle::Text => "Text",
            LayoutStyle::Boxed => "Boxed Text",
            LayoutStyle::Progress => "Day Progress",
        }
    }

    fn key(self) -> &'static str {
        match self {
            LayoutStyle::IconAboveText => "icon_above_text",
            LayoutStyle::IconText => "icon_text",
            LayoutStyle::Text => "text",
            LayoutStyle::Boxed => "boxed",
            LayoutStyle::Progress => "progress",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        ALL_STYLES.iter().copied().find(|style| style.key() == key)
    }

    /// What the menu bar item looks like in this style — the item itself uses
    /// it, and so does the style menu, which previews each style by showing the
    /// very image it would install rather than describing it in words.
    ///
    /// [`LayoutStyle::Text`] is the one style the menu bar draws differently:
    /// there the value stays the button's own title, but a preview has to be an
    /// image like every other row.
    fn image(self, duration: Duration) -> Retained<NSImage> {
        let value = uptime::format_uptime(duration);
        match self {
            LayoutStyle::IconAboveText => bar::stacked_image(UPTIME_ICON, &value),
            LayoutStyle::IconText => bar::icon_text_image(UPTIME_ICON, &value),
            LayoutStyle::Text => bar::text_image(&value),
            LayoutStyle::Boxed => bar::boxed_image(&value),
            LayoutStyle::Progress => bar::progress_image(
                &uptime::format_uptime_coarse(duration),
                uptime::day_fraction(duration),
            ),
        }
    }
}

/// A single-value file, in the same place `battery-widget` keeps its style.
fn style_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".config/system-uptime-widget/style"))
}

fn load_style() -> LayoutStyle {
    style_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|key| LayoutStyle::from_key(key.trim()))
        .unwrap_or(LayoutStyle::IconAboveText)
}

fn save_style(style: LayoutStyle) {
    let Some(path) = style_path() else { return };
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Err(err) = fs::write(&path, style.key()) {
        eprintln!("error saving style: {err}");
    }
}

/// The menu, plus the two sets of items whose contents follow the uptime.
struct Menu {
    menu: Retained<NSMenu>,
    uptime_item: Retained<NSMenuItem>,
    style_items: Vec<Retained<NSMenuItem>>,
}

struct Ui {
    status_item: Retained<NSStatusItem>,
    uptime_item: Retained<NSMenuItem>,
    style_items: Vec<Retained<NSMenuItem>>,
    style: LayoutStyle,
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

        #[unsafe(method(rebootAction:))]
        fn reboot_action(&self, _sender: &NSMenuItem) {
            power_action("Reboot", "restart");
        }

        #[unsafe(method(shutdownAction:))]
        fn shutdown_action(&self, _sender: &NSMenuItem) {
            power_action("Shutdown", "shut down");
        }

        #[unsafe(method(quitAction:))]
        fn quit_action(&self, _sender: &NSMenuItem) {
            let mtm = MainThreadMarker::new().unwrap();
            NSApplication::sharedApplication(mtm).terminate(None);
        }
    }

    unsafe impl NSObjectProtocol for Widget {}
);

impl Widget {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(RefCell::new(None));
        let this: Retained<Self> = unsafe { msg_send![super(this), init] };

        let status_item =
            NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength);
        let menu = this.build_menu(mtm);
        status_item.setMenu(Some(&menu.menu));

        *this.ivars().borrow_mut() = Some(Ui {
            status_item,
            uptime_item: menu.uptime_item,
            style_items: menu.style_items,
            style: load_style(),
        });
        this.refresh_style_checks();
        this
    }

    fn build_menu(&self, mtm: MainThreadMarker) -> Menu {
        let menu = NSMenu::new(mtm);
        menu.setAutoenablesItems(false);

        // The full figure, spelled out, at the top. No action: it is the
        // menu's heading, not a command. Enabled all the same, so it draws in
        // the normal text colour rather than greyed out.
        let uptime_item = NSMenuItem::new(mtm);
        uptime_item.setEnabled(true);
        menu.addItem(&uptime_item);
        menu.addItem(&NSMenuItem::separatorItem(mtm));

        // Our own selectors rather than `terminate:`: recent macOS decorates
        // menu items it recognises as standard actions with an SF Symbol, and
        // an unfamiliar action is the way to opt out of that.
        let action = |menu: &NSMenu, title: &str, selector| -> Retained<NSMenuItem> {
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

        let style_menu = NSMenu::new(mtm);
        style_menu.setAutoenablesItems(false);
        let style_items = ALL_STYLES
            .iter()
            .enumerate()
            .map(|(index, style)| {
                let item = action(&style_menu, style.label(), sel!(styleAction:));
                item.setTag(index as isize);
                item
            })
            .collect();
        let style_root = NSMenuItem::new(mtm);
        style_root.setTitle(ns_string!("Style"));
        style_root.setEnabled(true);
        style_root.setSubmenu(Some(&style_menu));
        menu.addItem(&style_root);

        menu.addItem(&NSMenuItem::separatorItem(mtm));
        action(&menu, "Reboot", sel!(rebootAction:));
        action(&menu, "Shutdown", sel!(shutdownAction:));
        action(&menu, "Quit", sel!(quitAction:));

        Menu {
            menu,
            uptime_item,
            style_items,
        }
    }

    fn update(&self) {
        let mtm = MainThreadMarker::new().unwrap();
        let duration = match uptime::uptime() {
            Ok(duration) => duration,
            Err(err) => {
                eprintln!("error reading uptime: {err}");
                return;
            }
        };

        let ivars = self.ivars().borrow();
        let Some(ui) = ivars.as_ref() else { return };
        let Some(button) = ui.status_item.button(mtm) else {
            return;
        };

        match ui.style {
            LayoutStyle::Text => {
                button.setAttributedTitle(&bar::attributed_title(&uptime::format_uptime(duration)));
                button.setImage(None);
                button.setImagePosition(NSCellImagePosition::NoImage);
            }
            style => {
                button.setImage(Some(&style.image(duration)));
                button.setImagePosition(NSCellImagePosition::ImageOnly);
            }
        }

        button.setToolTip(Some(&NSString::from_str(&format!(
            "System uptime is {}",
            uptime::human_uptime(duration)
        ))));

        ui.uptime_item
            .setTitle(&NSString::from_str(&uptime::expanded_uptime(duration)));

        // Each style row carries the image that style would install, drawn from
        // the current uptime — so the preview is the thing itself, not a mock
        // of it, and it stays current as the value ticks over.
        for (index, item) in ui.style_items.iter().enumerate() {
            let preview: Retained<NSImage> = ALL_STYLES[index].image(duration);
            item.setImage(Some(&preview));
        }
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

/// Ask before pulling the rug: `osascript` puts up the confirmation and, if it
/// is accepted, tells System Events to do the deed. Off the main thread, since
/// the dialog blocks until it is answered.
fn power_action(label: &'static str, command: &'static str) {
    std::thread::spawn(move || {
        let script = format!(
            r#"display dialog "Are you sure you want to {label} this Mac?" buttons {{"Cancel", "{label}"}} default button "Cancel" cancel button "Cancel" with icon caution
tell application "System Events" to {command}"#
        );
        // A cancelled dialog exits non-zero; only a launch failure is worth
        // reporting.
        if let Err(err) = Command::new("osascript").args(["-e", &script]).status() {
            eprintln!("error running {label} action: {err}");
        }
    });
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
