//! system-uptime-widget — how long this Mac has been up, in the menu bar.
//!
//! Rust talking to AppKit directly via [objc2](https://github.com/madsmtm/objc2),
//! in the same family as `battery-widget` and `free-disk-space-widget`: no
//! wrapper library, no vendored fork, no `.app` bundle. The dropdown is a real
//! `NSMenu` assigned to the status item, so macOS presents it natively.
//!
//! Sizing is macOS's business, not ours: the glyph is set in the menu bar font
//! and the value in the same compact size the disk widget uses.

mod bar;
mod uptime;

use std::cell::RefCell;
use std::process::Command;

use objc2::rc::Retained;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSCellImagePosition, NSMenu, NSMenuItem,
    NSStatusBar, NSStatusItem, NSVariableStatusItemLength,
};
use objc2_foundation::{NSObject, NSObjectProtocol, NSString, NSTimer};

const UPTIME_ICON: &str = "\u{102754}"; // SF Symbols clock.arrow.circlepath

/// The value only ever changes by the minute, and by the hour once the machine
/// has been up for one.
const UPDATE_INTERVAL_SECONDS: f64 = 30.0;

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
        status_item.setMenu(Some(&this.build_menu(mtm)));

        *this.ivars().borrow_mut() = Some(Ui { status_item });
        this
    }

    fn build_menu(&self, mtm: MainThreadMarker) -> Retained<NSMenu> {
        let menu = NSMenu::new(mtm);
        menu.setAutoenablesItems(false);

        // Our own selectors rather than `terminate:`: recent macOS decorates
        // menu items it recognises as standard actions with an SF Symbol, and
        // an unfamiliar action is the way to opt out of that.
        let action = |title: &str, selector| {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(title));
            item.setEnabled(true);
            unsafe {
                item.setTarget(Some(self.as_ref()));
                item.setAction(Some(selector));
            }
            menu.addItem(&item);
        };

        action("Reboot", sel!(rebootAction:));
        action("Shutdown", sel!(shutdownAction:));
        menu.addItem(&NSMenuItem::separatorItem(mtm));
        action("Quit", sel!(quitAction:));

        menu
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

        button.setImage(Some(&bar::icon_text_image(
            UPTIME_ICON,
            &uptime::format_uptime(duration),
        )));
        button.setImagePosition(NSCellImagePosition::ImageOnly);
        button.setToolTip(Some(&NSString::from_str(&format!(
            "System uptime is {}",
            uptime::human_uptime(duration)
        ))));
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
