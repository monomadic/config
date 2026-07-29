//! job-folder — a menu bar face for the `~/jobs` drop folder.
//!
//! It is the runner as well as the display: the job loop lives in
//! `runner.rs` and keeps the same on-disk contract as the shell
//! `utils/job-runner/job-runner`, so `.job` scripts and `send-job` work
//! unchanged. Only one of the two should be active at a time — they share
//! the `$JOBS_DIR/.lock` directory, so nothing runs twice either way.

mod clock;
mod icon;
mod runner;

use std::cell::RefCell;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{
    DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel,
};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSCellImagePosition, NSControlStateValueOff,
    NSControlStateValueOn, NSMenu, NSMenuDelegate, NSMenuItem, NSStatusBar, NSStatusItem,
    NSVariableStatusItemLength,
};
use objc2_foundation::{NSObject, NSObjectProtocol, NSString, NSTimer, ns_string};
use runner::State;

/// Blink period for the running cursor and the error mark.
const BLINK_INTERVAL: f64 = 0.6;
const MAX_QUEUE_LISTED: usize = 5;

struct Ui {
    status_item: Retained<NSStatusItem>,
    state: Arc<Mutex<State>>,
    blink_on: bool,
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
            self.refresh_icon();
        }

        #[unsafe(method(openJobs:))]
        fn open_jobs(&self, _sender: &NSMenuItem) {
            open(&runner::jobs_dir());
        }

        #[unsafe(method(openFailed:))]
        fn open_failed(&self, _sender: &NSMenuItem) {
            open(&runner::err_dir());
        }

        #[unsafe(method(viewLog:))]
        fn view_log(&self, _sender: &NSMenuItem) {
            let path = self.with_state(|state| {
                state
                    .running
                    .as_ref()
                    .and_then(|job| runner::running_log_path(&job.name))
            });
            if let Some(path) = path {
                open(&path);
            }
        }

        #[unsafe(method(clearErrors:))]
        fn clear_errors(&self, _sender: &NSMenuItem) {
            runner::acknowledge_errors();
            if let Some(ui) = self.ivars().borrow_mut().as_mut() {
                if let Ok(mut state) = ui.state.lock() {
                    state.errors = 0;
                }
            }
            self.refresh_icon();
        }

        #[unsafe(method(togglePause:))]
        fn toggle_pause(&self, _sender: &NSMenuItem) {
            let paused = self.with_state(|state| state.paused);
            runner::set_paused(!paused);
        }

        #[unsafe(method(revealRecent:))]
        fn reveal_recent(&self, sender: &NSMenuItem) {
            let index = sender.tag() as usize;
            let path =
                self.with_state(|state| state.recent.get(index).map(|job| job.artifact.clone()));
            if let Some(path) = path {
                reveal(&path);
            }
        }
    }

    unsafe impl NSObjectProtocol for Widget {}

    // Rebuilding on demand keeps the menu correct without rebuilding it on
    // every tick — macOS calls this immediately before the menu is shown.
    unsafe impl NSMenuDelegate for Widget {
        #[unsafe(method(menuNeedsUpdate:))]
        fn menu_needs_update(&self, menu: &NSMenu) {
            self.rebuild_menu(menu);
        }
    }
);

impl Widget {
    fn new(mtm: MainThreadMarker, state: Arc<Mutex<State>>) -> Retained<Self> {
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
            state,
            blink_on: true,
        });
        this
    }

    fn with_state<T>(&self, read: impl FnOnce(&State) -> T) -> T
    where
        T: Default,
    {
        let ivars = self.ivars().borrow();
        let Some(ui) = ivars.as_ref() else {
            return T::default();
        };
        let Ok(state) = ui.state.lock() else {
            return T::default();
        };
        read(&state)
    }

    fn refresh_icon(&self) {
        let mtm = MainThreadMarker::new().unwrap();
        let mut ivars = self.ivars().borrow_mut();
        let Some(ui) = ivars.as_mut() else { return };

        let (running, queued, errors) = {
            let Ok(state) = ui.state.lock() else { return };
            (state.running.is_some(), state.queued.len(), state.errors > 0)
        };

        // Only animate when there is something to say; a quiet menu bar
        // should stay perfectly still.
        ui.blink_on = if running || errors { !ui.blink_on } else { true };

        let image = icon::draw(&icon::IconState {
            running,
            queued,
            errors,
            blink_on: ui.blink_on,
        });
        if let Some(button) = ui.status_item.button(mtm) {
            button.setImage(Some(&image));
            button.setImagePosition(NSCellImagePosition::ImageOnly);
        }
    }

    fn rebuild_menu(&self, menu: &NSMenu) {
        let mtm = MainThreadMarker::new().unwrap();
        menu.removeAllItems();

        let ivars = self.ivars().borrow();
        let Some(ui) = ivars.as_ref() else { return };
        let Ok(state) = ui.state.lock() else { return };

        let info = |title: String| {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(&title));
            item.setEnabled(false);
            menu.addItem(&item);
        };
        let action = |title: String, selector, enabled: bool| -> Retained<NSMenuItem> {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(&title));
            item.setEnabled(enabled);
            unsafe {
                item.setTarget(Some(self.as_ref()));
                item.setAction(Some(selector));
            }
            menu.addItem(&item);
            item
        };

        match (&state.running, state.paused) {
            (Some(job), _) => info(format!(
                "Running: {} — {}",
                job.name,
                elapsed(job.started.elapsed())
            )),
            (None, true) => info("Paused".to_string()),
            (None, false) => info("Idle".to_string()),
        }

        // The queue excludes whatever is currently running: a claimed job is
        // renamed out of the scan set, so it never appears here.
        if !state.queued.is_empty() {
            info(format!("Queued: {}", state.queued.len()));
            for name in state.queued.iter().take(MAX_QUEUE_LISTED) {
                info(format!("    {name}"));
            }
            if state.queued.len() > MAX_QUEUE_LISTED {
                info(format!(
                    "    … {} more",
                    state.queued.len() - MAX_QUEUE_LISTED
                ));
            }
        }

        if !state.recent.is_empty() {
            menu.addItem(&NSMenuItem::separatorItem(mtm));
            info("Recent".to_string());
            for (index, job) in state.recent.iter().enumerate() {
                let mark = if job.ok { "✓" } else { "✗" };
                let item = action(
                    format!("{mark}  {} · {} ago", job.name, ago(job.finished.elapsed())),
                    sel!(revealRecent:),
                    true,
                );
                item.setTag(index as isize);
            }
        }

        menu.addItem(&NSMenuItem::separatorItem(mtm));
        action("Open jobs folder".to_string(), sel!(openJobs:), true);
        action("Open failed jobs".to_string(), sel!(openFailed:), true);

        // The log only exists once the running job has written something —
        // it is created lazily, so a silent job leaves no empty file.
        let has_log = state
            .running
            .as_ref()
            .is_some_and(|job| runner::running_log_path(&job.name).is_some());
        action("View running job log".to_string(), sel!(viewLog:), has_log);
        action(
            "Clear error badge".to_string(),
            sel!(clearErrors:),
            state.errors > 0,
        );

        menu.addItem(&NSMenuItem::separatorItem(mtm));
        let pause = action("Pause queue".to_string(), sel!(togglePause:), true);
        pause.setState(if state.paused {
            NSControlStateValueOn
        } else {
            NSControlStateValueOff
        });

        menu.addItem(&NSMenuItem::separatorItem(mtm));
        let quit = NSMenuItem::new(mtm);
        quit.setTitle(ns_string!("Quit"));
        quit.setEnabled(true);
        unsafe { quit.setAction(Some(sel!(terminate:))) };
        menu.addItem(&quit);
    }
}

/// `4:07` under an hour, `1:04:07` beyond it.
fn elapsed(duration: Duration) -> String {
    let total = duration.as_secs();
    let (hours, minutes, seconds) = (total / 3600, (total % 3600) / 60, total % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

/// Coarse relative time for the recent list: `just now`, `12m`, `3h`.
fn ago(duration: Duration) -> String {
    let total = duration.as_secs();
    match total {
        0..=59 => "just now".to_string(),
        60..=3599 => format!("{}m", total / 60),
        _ => format!("{}h", total / 3600),
    }
}

fn open(path: &std::path::Path) {
    let _ = Command::new("open").arg(path).spawn();
}

fn reveal(path: &std::path::Path) {
    let _ = Command::new("open").arg("-R").arg(path).spawn();
}

fn main() {
    let mtm = MainThreadMarker::new().expect("must run on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let state = Arc::new(Mutex::new(State::default()));
    runner::spawn(Arc::clone(&state));

    let widget = Widget::new(mtm, state);
    widget.refresh_icon();

    unsafe {
        NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
            BLINK_INTERVAL,
            &widget,
            sel!(tick:),
            None,
            true,
        );
    }

    app.run();
}
