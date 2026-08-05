//! job-server — the job runner with a menu bar face.
//!
//! It is the runner as well as the display: the job loop lives in `runner.rs`
//! and keeps the same on-disk contract as the shell `job-server-cli`, so `.job`
//! scripts and `send-job` work unchanged. Only one runner may be active per
//! folder — they share `$JOBS_DIR/.lock`, so nothing runs twice either way.
//!
//! The menu never reads the runner's memory: it renders a `job_core` snapshot
//! polled off the folder itself, exactly like `job-monitor` does across the
//! network. Restarting mid-job therefore shows the job that is still running,
//! rather than an empty queue.

mod runner;

use std::cell::RefCell;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use job_core::icon;
use job_core::observe::{Observer, Snapshot};
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSCellImagePosition, NSControlStateValueOff,
    NSControlStateValueOn, NSMenu, NSMenuDelegate, NSMenuItem, NSStatusBar, NSStatusItem,
    NSVariableStatusItemLength,
};
use objc2_foundation::{NSObject, NSObjectProtocol, NSString, NSTimer, ns_string};

/// Blink period for the running cursor and the error mark.
const BLINK_INTERVAL: f64 = 0.6;
const MAX_QUEUE_LISTED: usize = 5;
/// How often the folder is re-read. Faster while there is something to watch,
/// slower when idle — the folder is local here, but the same loop runs over a
/// share in job-monitor, where every poll is a round trip.
const POLL_BUSY: Duration = Duration::from_secs(1);
const POLL_IDLE: Duration = Duration::from_secs(3);

struct Ui {
    status_item: Retained<NSStatusItem>,
    snapshot: Arc<Mutex<Snapshot>>,
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
            open(runner::root().path());
        }

        #[unsafe(method(openFailed:))]
        fn open_failed(&self, _sender: &NSMenuItem) {
            open(&runner::root().err());
        }

        #[unsafe(method(viewLog:))]
        fn view_log(&self, _sender: &NSMenuItem) {
            let path = self
                .with_snapshot(|snapshot| snapshot.running.first().and_then(|job| job.log_path()));
            if let Some(path) = path {
                open(&path);
            }
        }

        #[unsafe(method(clearErrors:))]
        fn clear_errors(&self, _sender: &NSMenuItem) {
            runner::acknowledge_errors();
            if let Some(ui) = self.ivars().borrow_mut().as_mut()
                && let Ok(mut snapshot) = ui.snapshot.lock()
            {
                snapshot.errors = 0;
            }
            self.refresh_icon();
        }

        #[unsafe(method(togglePause:))]
        fn toggle_pause(&self, _sender: &NSMenuItem) {
            let paused = self.with_snapshot(|snapshot| snapshot.paused);
            runner::set_paused(!paused);
        }

        #[unsafe(method(revealRecent:))]
        fn reveal_recent(&self, sender: &NSMenuItem) {
            let index = sender.tag() as usize;
            let path = self
                .with_snapshot(|snapshot| snapshot.recent.get(index).map(|job| job.dir.clone()));
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
    fn new(mtm: MainThreadMarker, snapshot: Arc<Mutex<Snapshot>>) -> Retained<Self> {
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
            snapshot,
            blink_on: true,
        });
        this
    }

    fn with_snapshot<T>(&self, read: impl FnOnce(&Snapshot) -> T) -> T
    where
        T: Default,
    {
        let ivars = self.ivars().borrow();
        let Some(ui) = ivars.as_ref() else {
            return T::default();
        };
        let Ok(snapshot) = ui.snapshot.lock() else {
            return T::default();
        };
        read(&snapshot)
    }

    fn refresh_icon(&self) {
        let mtm = MainThreadMarker::new().unwrap();
        let mut ivars = self.ivars().borrow_mut();
        let Some(ui) = ivars.as_mut() else { return };

        let (running, queued, errors) = {
            let Ok(snapshot) = ui.snapshot.lock() else {
                return;
            };
            (
                !snapshot.running.is_empty(),
                snapshot.queued.len(),
                snapshot.errors > 0,
            )
        };

        // Only animate when there is something to say; a quiet menu bar
        // should stay perfectly still.
        ui.blink_on = if running || errors { !ui.blink_on } else { true };

        let image = icon::draw(&icon::IconState {
            running,
            queued,
            errors,
            blink_on: ui.blink_on,
            ..icon::IconState::default()
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
        let Ok(snapshot) = ui.snapshot.lock() else {
            return;
        };

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

        match (snapshot.running.first(), snapshot.paused) {
            (Some(job), _) => {
                let label = match job.elapsed() {
                    Some(elapsed) => format!("Running: {} — {}", job.name, duration(elapsed)),
                    None => format!("Running: {}", job.name),
                };
                info(label);
                // A claimed job with no runner behind it: the run folder was
                // left in _running when something killed the runner.
                if snapshot.stalled {
                    info("    ⚠ runner not responding".to_string());
                }
            }
            (None, true) => info("Paused".to_string()),
            (None, false) => info("Idle".to_string()),
        }

        // The queue excludes whatever is currently running: a claimed job has
        // moved into _running, so it never appears here.
        if !snapshot.queued.is_empty() {
            info(format!("Queued: {}", snapshot.queued.len()));
            for name in snapshot.queued.iter().take(MAX_QUEUE_LISTED) {
                info(format!("    {name}"));
            }
            if snapshot.queued.len() > MAX_QUEUE_LISTED {
                info(format!(
                    "    … {} more",
                    snapshot.queued.len() - MAX_QUEUE_LISTED
                ));
            }
        }

        if !snapshot.recent.is_empty() {
            menu.addItem(&NSMenuItem::separatorItem(mtm));
            info("Recent".to_string());
            for (index, job) in snapshot.recent.iter().enumerate() {
                let mark = if job.ok { "✓" } else { "✗" };
                let item = action(
                    format!("{mark}  {} · {} ago", job.name, ago(job.ago())),
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
        let has_log = snapshot
            .running
            .first()
            .is_some_and(|job| job.log_path().is_some());
        action("View running job log".to_string(), sel!(viewLog:), has_log);
        action(
            "Clear error badge".to_string(),
            sel!(clearErrors:),
            snapshot.errors > 0,
        );

        menu.addItem(&NSMenuItem::separatorItem(mtm));
        let pause = action("Pause queue".to_string(), sel!(togglePause:), true);
        pause.setState(if snapshot.paused {
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
fn duration(duration: Duration) -> String {
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

/// Re-read the folder on a background thread. Even locally this is kept off
/// the main thread on purpose: the same code path serves a mounted share in
/// job-monitor, where a `read_dir` can block until the mount gives up.
fn spawn_poller(snapshot: Arc<Mutex<Snapshot>>) {
    thread::spawn(move || {
        let mut observer = Observer::new(runner::root());
        loop {
            let fresh = observer.poll(runner::read_ack());
            let busy = fresh.is_busy();
            if let Ok(mut shared) = snapshot.lock() {
                *shared = fresh;
            }
            thread::sleep(if busy { POLL_BUSY } else { POLL_IDLE });
        }
    });
}

fn main() {
    let mtm = MainThreadMarker::new().expect("must run on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let snapshot = Arc::new(Mutex::new(Snapshot::default()));
    runner::spawn();
    spawn_poller(Arc::clone(&snapshot));

    let widget = Widget::new(mtm, snapshot);
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
