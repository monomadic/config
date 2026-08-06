//! job-monitor — a read-only menu bar view of one or more jobs folders,
//! normally mounted from another machine over SMB.
//!
//! It is deliberately a separate crate from the runner rather than a flag on
//! it: a binary with no job loop linked into it cannot claim a job however it
//! is launched, which is what makes watching someone else's folder safe.
//!
//! It is not, however, read-only. Every command in this system is a folder
//! move — pause, resume, stop, requeue — so the row buttons work here exactly
//! as they do locally, over SMB, with no protocol and no listening port. The
//! runner at the other end is watching its own folders and does the
//! signalling; this app only ever renames.
//!
//! Everything it shows comes from `job_core`'s observer, so a folder watched
//! from across the LAN reads exactly the way it does on the machine running it.

mod notify;
mod roots;

use std::cell::RefCell;
use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use job_core::icon;
use job_core::observe::{Observer, Root, Snapshot, State};
use job_core::row;
use notify::Notifier;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSCellImagePosition, NSControlStateValueOff,
    NSControlStateValueOn, NSMenu, NSMenuDelegate, NSMenuItem, NSStatusBar, NSStatusItem,
    NSVariableStatusItemLength,
};
use objc2_foundation::{NSObject, NSObjectProtocol, NSString, NSTimer, ns_string};

const BLINK_INTERVAL: f64 = 0.6;
const MAX_QUEUE_LISTED: usize = 5;
const MAX_RECENT_LISTED: usize = 5;

/// Poll intervals. macOS caches SMB directory listings for 30–60s by default
/// (`dir_cache_min` / `dir_cache_max` in `nsmb.conf`), so hammering the share
/// buys nothing — these are paced to be responsive without being pointless.
const POLL_BUSY: Duration = Duration::from_secs(2);
const POLL_IDLE: Duration = Duration::from_secs(8);
/// A folder that isn't there is checked more slowly still: the `read_dir` that
/// discovers a dead mount is the expensive one, since it blocks until the
/// mount gives up.
const POLL_OFFLINE: Duration = Duration::from_secs(15);


struct RootView {
    root: Root,
    snapshot: Snapshot,
}

struct Ui {
    status_item: Retained<NSStatusItem>,
    views: Arc<Mutex<Vec<RootView>>>,
    muted: Arc<AtomicBool>,
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

        #[unsafe(method(openFolder:))]
        fn open_folder(&self, sender: &NSMenuItem) {
            if let Some(root) = self.root_at(sender.tag()) {
                open(root.path());
            }
        }

        #[unsafe(method(openFailed:))]
        fn open_failed(&self, sender: &NSMenuItem) {
            if let Some(root) = self.root_at(sender.tag()) {
                open(&root.failed());
            }
        }

        #[unsafe(method(viewLog:))]
        fn view_log(&self, sender: &NSMenuItem) {
            let index = sender.tag() as usize;
            let path = self.with_views(|views| {
                views
                    .get(index)
                    .and_then(|view| view.snapshot.running().next())
                    .and_then(|job| job.log_path())
            });
            if let Some(path) = path {
                open(&path);
            }
        }


        #[unsafe(method(clearErrors:))]
        fn clear_errors(&self, _sender: &NSMenuItem) {
            acknowledge_errors();
            if let Some(ui) = self.ivars().borrow_mut().as_mut()
                && let Ok(mut views) = ui.views.lock()
            {
                for view in views.iter_mut() {
                    view.snapshot.errors = 0;
                }
            }
            self.refresh_icon();
        }

        #[unsafe(method(toggleNotifications:))]
        fn toggle_notifications(&self, _sender: &NSMenuItem) {
            let ivars = self.ivars().borrow();
            let Some(ui) = ivars.as_ref() else { return };
            let muted = !ui.muted.load(Ordering::Relaxed);
            ui.muted.store(muted, Ordering::Relaxed);
            set_muted(muted);
        }
    }

    unsafe impl NSObjectProtocol for Widget {}

    unsafe impl NSMenuDelegate for Widget {
        #[unsafe(method(menuNeedsUpdate:))]
        fn menu_needs_update(&self, menu: &NSMenu) {
            self.rebuild_menu(menu);
        }
    }
);

impl Widget {
    fn new(
        mtm: MainThreadMarker,
        views: Arc<Mutex<Vec<RootView>>>,
        muted: Arc<AtomicBool>,
    ) -> Retained<Self> {
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
            views,
            muted,
            blink_on: true,
        });
        this
    }

    fn with_views<T>(&self, read: impl FnOnce(&[RootView]) -> T) -> T
    where
        T: Default,
    {
        let ivars = self.ivars().borrow();
        let Some(ui) = ivars.as_ref() else {
            return T::default();
        };
        let Ok(views) = ui.views.lock() else {
            return T::default();
        };
        read(&views)
    }

    fn root_at(&self, tag: isize) -> Option<Root> {
        self.with_views(|views| views.get(tag as usize).map(|view| view.root.clone()))
    }

    fn refresh_icon(&self) {
        let mtm = MainThreadMarker::new().unwrap();
        let mut ivars = self.ivars().borrow_mut();
        let Some(ui) = ivars.as_mut() else { return };

        let (running, queued, errors, connected) = {
            let Ok(views) = ui.views.lock() else { return };
            (
                views.iter().any(|view| view.snapshot.running().next().is_some()),
                views
                    .iter()
                    .map(|view| {
                        view.snapshot.in_state(State::Ready).count() + view.snapshot.inbox.len()
                    })
                    .sum(),
                views
                    .iter()
                    .any(|view| view.snapshot.errors > 0 || view.snapshot.stalled()),
                views.iter().any(|view| view.snapshot.connected),
            )
        };

        ui.blink_on = if (running || errors) && connected {
            !ui.blink_on
        } else {
            true
        };

        let image = icon::draw(&icon::IconState {
            running,
            queued,
            errors,
            blink_on: ui.blink_on,
            remote: true,
            connected,
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
        let Ok(views) = ui.views.lock() else { return };

        let info = |title: String| {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(&title));
            item.setEnabled(false);
            menu.addItem(&item);
        };
        let action = |title: String, selector, enabled: bool, tag: isize| -> Retained<NSMenuItem> {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(&title));
            item.setEnabled(enabled);
            item.setTag(tag);
            unsafe {
                item.setTarget(Some(self.as_ref()));
                item.setAction(Some(selector));
            }
            menu.addItem(&item);
            item
        };

        let multiple = views.len() > 1;
        for (index, view) in views.iter().enumerate() {
            let tag = index as isize;
            if index > 0 {
                menu.addItem(&NSMenuItem::separatorItem(mtm));
            }
            if multiple {
                info(view.root.label());
            }

            // A folder that isn't there is called that. Rendering an
            // unreachable share as an empty queue would read as "nothing to
            // do", which is the one wrong answer.
            if !view.snapshot.connected {
                info(format!("Not mounted — {}", view.root.path().display()));
                action("Open jobs folder".to_string(), sel!(openFolder:), true, tag);
                continue;
            }

            let sections = row::sections(&view.snapshot, MAX_QUEUE_LISTED, MAX_RECENT_LISTED);
            if sections.is_empty() {
                info("Idle".to_string());
            } else {
                let layout = row::layout(sections.iter().flat_map(|section| section.rows.iter()));
                for section in &sections {
                    if let Some(label) = section.label.as_ref() {
                        info(label.clone());
                    }
                    for spec in &section.rows {
                        let item = NSMenuItem::new(mtm);
                        item.setEnabled(true);
                        item.setView(Some(&row::JobRow::new(spec.clone(), &layout, mtm)));
                        menu.addItem(&item);
                    }
                }
                if view.snapshot.stalled() {
                    info("⚠ a job in _running is not running".to_string());
                }
            }

            action("Open jobs folder".to_string(), sel!(openFolder:), true, tag);
            action("Open failed jobs".to_string(), sel!(openFailed:), true, tag);
            let has_log = view
                .snapshot
                .running()
                .next()
                .is_some_and(|job| job.log_path().is_some());
            action(
                "View running job log".to_string(),
                sel!(viewLog:),
                has_log,
                tag,
            );
        }

        let errors = views.iter().any(|view| view.snapshot.errors > 0);
        menu.addItem(&NSMenuItem::separatorItem(mtm));
        action(
            "Clear error badge".to_string(),
            sel!(clearErrors:),
            errors,
            0,
        );
        let notifications = action(
            "Notifications".to_string(),
            sel!(toggleNotifications:),
            true,
            0,
        );
        notifications.setState(if ui.muted.load(Ordering::Relaxed) {
            NSControlStateValueOff
        } else {
            NSControlStateValueOn
        });

        menu.addItem(&NSMenuItem::separatorItem(mtm));
        let quit = NSMenuItem::new(mtm);
        quit.setTitle(ns_string!("Quit"));
        quit.setEnabled(true);
        unsafe { quit.setAction(Some(sel!(terminate:))) };
        menu.addItem(&quit);
    }
}



fn open(path: &std::path::Path) {
    let _ = Command::new("open").arg(path).spawn();
}

/// Acknowledgement and mute live on *this* machine. Nothing about how one
/// person reads a shared folder belongs in the shared folder.
fn ack_file() -> std::path::PathBuf {
    roots::config_dir().join("ack")
}

fn read_ack() -> i64 {
    fs::read_to_string(ack_file())
        .ok()
        .and_then(|text| text.trim().parse::<i64>().ok())
        .unwrap_or(0)
}

fn acknowledge_errors() {
    let _ = fs::create_dir_all(roots::config_dir());
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_secs() as i64)
        .unwrap_or(0);
    let _ = fs::write(ack_file(), now.to_string());
}

fn mute_file() -> std::path::PathBuf {
    roots::config_dir().join("muted")
}

fn read_muted() -> bool {
    mute_file().exists()
}

fn set_muted(muted: bool) {
    let _ = fs::create_dir_all(roots::config_dir());
    if muted {
        let _ = fs::write(mute_file(), "");
    } else {
        let _ = fs::remove_file(mute_file());
    }
}

/// One thread per root, so a share that has gone away blocks only its own
/// polling and the other folders carry on updating.
fn spawn_poller(
    index: usize,
    root: Root,
    views: Arc<Mutex<Vec<RootView>>>,
    muted: Arc<AtomicBool>,
    notifier: Arc<Notifier>,
) {
    thread::spawn(move || {
        let mut observer = Observer::new(root.clone());
        let label = root.label();
        // Established on the first poll and never notified about: launching
        // the app should not replay everything that happened while it was shut.
        let mut baseline = true;
        let mut last_finished = i64::MIN;
        let mut was_connected = true;
        let mut was_stalled = false;

        loop {
            let snapshot = observer.poll(read_ack());
            let newest = snapshot
                .recent
                .iter()
                .map(|outcome| outcome.finished)
                .max()
                .unwrap_or(i64::MIN);

            if baseline {
                baseline = false;
                last_finished = newest;
                was_connected = snapshot.connected;
                was_stalled = snapshot.stalled();
            } else if !muted.load(Ordering::Relaxed) {
                if snapshot.connected != was_connected {
                    if snapshot.connected {
                        notifier.post(&label, "Jobs folder is back");
                    } else {
                        notifier.post(&label, "Jobs folder is not reachable");
                    }
                }
                if snapshot.connected {
                    for outcome in snapshot
                        .recent
                        .iter()
                        .filter(|outcome| outcome.finished > last_finished)
                    {
                        let (title, body) = if outcome.ok {
                            ("Job finished", format!("{} · {label}", outcome.name))
                        } else {
                            ("Job failed", format!("{} · {label}", outcome.name))
                        };
                        notifier.post(title, &body);
                    }
                    if snapshot.stalled() && !was_stalled {
                        notifier.post(&label, "A job stopped running");
                    }
                }
                was_connected = snapshot.connected;
                was_stalled = snapshot.stalled();
            }
            if newest > last_finished {
                last_finished = newest;
            }

            let interval = if !snapshot.connected {
                POLL_OFFLINE
            } else if snapshot.is_busy() {
                POLL_BUSY
            } else {
                POLL_IDLE
            };

            if let Ok(mut views) = views.lock()
                && let Some(view) = views.get_mut(index)
            {
                view.snapshot = snapshot;
            }
            thread::sleep(interval);
        }
    });
}

fn main() {
    let mtm = MainThreadMarker::new().expect("must run on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let watched = roots::configured();
    let views: Vec<RootView> = watched
        .iter()
        .map(|root| RootView {
            root: root.clone(),
            // Starts disconnected rather than idle: nothing has been read yet,
            // and "idle" would be a claim we cannot make.
            snapshot: Snapshot::default(),
        })
        .collect();
    let views = Arc::new(Mutex::new(views));
    let muted = Arc::new(AtomicBool::new(read_muted()));
    let notifier = Arc::new(Notifier::new());

    for (index, root) in watched.into_iter().enumerate() {
        spawn_poller(
            index,
            root,
            Arc::clone(&views),
            Arc::clone(&muted),
            Arc::clone(&notifier),
        );
    }

    let widget = Widget::new(mtm, views, muted);
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
