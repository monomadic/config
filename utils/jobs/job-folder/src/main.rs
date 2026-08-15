//! job-folder — the jobs queue and its menu in one process.
//!
//! `job-daemon` and `job-monitor` are two programs that never speak: the runner
//! writes a folder, the menu reads it back, and the folder in between is the
//! whole protocol. That buys something real — a monitor on another machine sees
//! exactly what the runner sees, over nothing more than SMB — and it costs
//! exactly what you would expect. A pause is a `rename` the runner has to
//! notice. The row you pressed doesn't change until a poll comes round. The
//! order of the queue is the alphabet, so changing it means renaming folders.
//! And every question about a job — is it alive, has it gone quiet, did it
//! stall — is answered by inference, because nothing here is its parent.
//!
//! This is the other trade. One process runs the jobs and draws the menu, so
//! the queue is a `Vec<Job>` behind a mutex and a button press is a method
//! call: pause is `SIGSTOP` on the way back from the click, reordering is a
//! splice, and "is it running" is not a question — we are holding the child.
//! The menu updates itself while it is open, because the model it is drawing is
//! in the same address space.
//!
//! What is given up is the network. There is no state on disk for a second
//! machine to read, so nothing can watch this queue from anywhere else, and
//! nothing survives the process but the payload folders. That is the deal:
//! [`job-monitor`](../job-monitor) for a queue you share, this for one you
//! stand in front of.

mod notify;
mod queue;
mod rows;

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

use job_core::icon;
use job_core::row::{self, JobRow, Layout};
use notify::Notifier;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSApplicationDelegate, NSCellImagePosition,
    NSControlStateValueOff, NSControlStateValueOn, NSMenu, NSMenuDelegate, NSMenuItem, NSStatusBar,
    NSStatusItem, NSVariableStatusItemLength,
};
use objc2_foundation::{NSNotification, NSObject, NSObjectProtocol, NSString, NSTimer, ns_string};
use queue::{Event, Jobs, Phase};

/// The icon blinks on this, and it is also how often an open menu catches up
/// with a job that is only printing. Commands don't wait for it: they redraw
/// the rows themselves, from the click.
const TICK: f64 = 0.6;

const MAX_RECENT_LISTED: usize = 5;

struct Ui {
    status_item: Retained<NSStatusItem>,
    jobs: Arc<Jobs>,
    notifier: Notifier,
    muted: bool,
    style: icon::Style,
    blink_on: bool,
    /// The rows currently in the menu, with the keys they were built from. Held
    /// so a redraw can hand each one a fresh spec instead of the menu being
    /// rebuilt under the pointer.
    rows: Vec<(rows::Key, Retained<JobRow>)>,
    open: bool,
}

// The one widget, reachable from the row-button handler.
//
// `Retained` is neither `Send` nor `Sync`, and `job_core::row::on_call` wants a
// handler that is both — but it only ever calls it from the main thread, which
// is the thread this is stored on. A thread-local says precisely that, and is
// empty when read from anywhere else rather than being unsound.
thread_local! {
    static WIDGET: RefCell<Option<Retained<Widget>>> = const { RefCell::new(None) };
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
            self.post_events();
            self.refresh_icon();
            self.refresh_rows();
        }

        #[unsafe(method(togglePause:))]
        fn toggle_pause(&self, _sender: &NSMenuItem) {
            self.with_ui(|ui| {
                let paused = ui.jobs.read(|queue| queue.paused);
                ui.jobs.set_paused(!paused);
            });
            self.refresh_icon();
        }

        #[unsafe(method(setConcurrency:))]
        fn set_concurrency(&self, sender: &NSMenuItem) {
            let concurrency = sender.tag() as usize;
            self.with_ui(|ui| ui.jobs.set_concurrency(concurrency));
            save(CONCURRENCY, &concurrency.to_string());
        }

        #[unsafe(method(clearFinished:))]
        fn clear_finished(&self, _sender: &NSMenuItem) {
            self.with_ui(|ui| ui.jobs.clear_finished());
            self.refresh_icon();
        }

        #[unsafe(method(openFolder:))]
        fn open_folder(&self, _sender: &NSMenuItem) {
            self.with_ui(|ui| open(&ui.jobs.root));
        }

        #[unsafe(method(setStyle:))]
        fn set_style(&self, sender: &NSMenuItem) {
            let style = icon::ALL_STYLES[sender.tag() as usize];
            self.with_ui(|ui| ui.style = style);
            save(STYLE, style.key());
            self.refresh_icon();
        }

        #[unsafe(method(toggleNotifications:))]
        fn toggle_notifications(&self, _sender: &NSMenuItem) {
            let muted = self.with_ui(|ui| {
                ui.muted = !ui.muted;
                ui.muted
            });
            if muted == Some(true) {
                save(MUTED, "");
            } else {
                let _ = fs::remove_file(config_dir().join(MUTED));
            }
        }
    }

    unsafe impl NSObjectProtocol for Widget {}

    unsafe impl NSMenuDelegate for Widget {
        #[unsafe(method(menuNeedsUpdate:))]
        fn menu_needs_update(&self, menu: &NSMenu) {
            self.rebuild_menu(menu);
        }

        #[unsafe(method(menuWillOpen:))]
        fn menu_will_open(&self, _menu: &NSMenu) {
            self.with_ui(|ui| ui.open = true);
        }

        #[unsafe(method(menuDidClose:))]
        fn menu_did_close(&self, _menu: &NSMenu) {
            self.with_ui(|ui| {
                ui.open = false;
                // The views belong to the menu that has just thrown them away.
                ui.rows.clear();
            });
        }
    }

    unsafe impl NSApplicationDelegate for Widget {
        /// The queue dies with this process, so quitting has to say so to the
        /// jobs: an encode nothing is watching would carry on for hours with no
        /// row left to stop it from.
        #[unsafe(method(applicationWillTerminate:))]
        fn will_terminate(&self, _notification: &NSNotification) {
            self.with_ui(|ui| ui.jobs.shutdown());
        }
    }
);

impl Widget {
    fn new(mtm: MainThreadMarker, jobs: Arc<Jobs>) -> Retained<Self> {
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
            jobs,
            notifier: Notifier::new(),
            muted: config_dir().join(MUTED).exists(),
            style: load(STYLE)
                .and_then(|text| icon::Style::from_key(text.trim()))
                .unwrap_or(icon::Style::Cursors),
            blink_on: true,
            rows: Vec::new(),
            open: false,
        });
        this
    }

    fn with_ui<T>(&self, act: impl FnOnce(&mut Ui) -> T) -> Option<T> {
        let mut ivars = self.ivars().borrow_mut();
        ivars.as_mut().map(act)
    }

    fn jobs(&self) -> Option<Arc<Jobs>> {
        self.with_ui(|ui| Arc::clone(&ui.jobs))
    }

    /// Banners for jobs that finished. Drained here rather than posted from the
    /// job's own thread: this is the one part of the app that cares which
    /// thread it is on, so it is the one part that runs on the main one.
    fn post_events(&self) {
        let Some(jobs) = self.jobs() else { return };
        for event in jobs.take_events() {
            let Event::Finished { name, ok } = event;
            self.with_ui(|ui| {
                if ui.muted {
                    return;
                }
                let title = if ok { "Job finished" } else { "Job failed" };
                ui.notifier.post(title, &name);
            });
        }
    }

    fn refresh_icon(&self) {
        let mtm = MainThreadMarker::new().unwrap();
        let mut ivars = self.ivars().borrow_mut();
        let Some(ui) = ivars.as_mut() else { return };

        let (running, queued, failed) = ui.jobs.read(|queue| {
            (
                queue.jobs.iter().filter(|job| job.phase == Phase::Running).count(),
                queue.queued(),
                queue.failures(),
            )
        });

        // Blink while anything wants attention. Never for an unreachable folder:
        // this app *is* the folder's reason to exist, and it made it itself.
        ui.blink_on = if running > 0 || failed > 0 {
            !ui.blink_on
        } else {
            true
        };

        let image = icon::draw(ui.style, &icon::IconState {
            running,
            queued,
            failed,
            blink_on: ui.blink_on,
            connected: true,
        });
        if let Some(button) = ui.status_item.button(mtm) {
            button.setImage(Some(&image));
            button.setImagePosition(NSCellImagePosition::ImageOnly);
        }
    }

    /// Bring an open menu up to date.
    ///
    /// While the shape of the list holds — the same jobs, with the same buttons
    /// — each row is handed a new spec and redraws itself, so a percentage
    /// climbing does not disturb a menu you are pointing at. When the shape
    /// changes, and only then, the menu is rebuilt.
    fn refresh_rows(&self) {
        let open = self.with_ui(|ui| ui.open).unwrap_or(false);
        if !open {
            return;
        }
        let Some(jobs) = self.jobs() else { return };
        let fresh = jobs.read(|queue| rows::rows(queue, MAX_RECENT_LISTED));

        let same = self
            .with_ui(|ui| {
                ui.rows.len() == fresh.len()
                    && ui
                        .rows
                        .iter()
                        .zip(fresh.iter())
                        .all(|((key, _), row)| *key == row.key)
            })
            .unwrap_or(false);

        if same {
            self.with_ui(|ui| {
                for ((_, view), row) in ui.rows.iter().zip(fresh.into_iter()) {
                    view.update(row.spec);
                }
            });
            return;
        }

        let mtm = MainThreadMarker::new().unwrap();
        let menu = self.with_ui(|ui| ui.status_item.menu(mtm)).flatten();
        if let Some(menu) = menu {
            self.rebuild_menu(&menu);
        }
    }

    fn rebuild_menu(&self, menu: &NSMenu) {
        let mtm = MainThreadMarker::new().unwrap();
        let Some(jobs) = self.jobs() else { return };
        let (specs, paused, concurrency, finished) = jobs.read(|queue| {
            (
                rows::rows(queue, MAX_RECENT_LISTED),
                queue.paused,
                queue.concurrency,
                queue.jobs.iter().filter(|job| job.phase.finished()).count(),
            )
        });

        menu.removeAllItems();
        let mut views = Vec::new();

        let info = |title: &str| {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(title));
            item.setEnabled(false);
            menu.addItem(&item);
        };
        let action = |title: &str, selector, enabled: bool, tag: isize| -> Retained<NSMenuItem> {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(title));
            item.setEnabled(enabled);
            item.setTag(tag);
            unsafe {
                item.setTarget(Some(self.as_ref()));
                item.setAction(Some(selector));
            }
            menu.addItem(&item);
            item
        };

        if specs.is_empty() {
            info("Idle");
        } else {
            let layout: Layout = row::layout(specs.iter().map(|row| &row.spec));
            for row in &specs {
                let view = JobRow::new(row.spec.clone(), &layout, mtm);
                let item = NSMenuItem::new(mtm);
                item.setEnabled(true);
                item.setView(Some(&view));
                menu.addItem(&item);
                views.push((row.key, view));
            }
        }

        menu.addItem(&NSMenuItem::separatorItem(mtm));

        // One verb for the whole queue: what is running carries on, nothing new
        // starts. Suspending the jobs themselves is what the row buttons are
        // for, and conflating the two would make this the button that stops
        // your encode.
        action(
            if paused { "Resume Queue" } else { "Hold Queue" },
            sel!(togglePause:),
            true,
            0,
        );

        let concurrency_menu = NSMenu::new(mtm);
        concurrency_menu.setAutoenablesItems(false);
        for slots in 1..=queue::max_concurrency() {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(&format!("{slots}")));
            item.setTag(slots as isize);
            item.setEnabled(true);
            item.setState(if slots == concurrency {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            });
            unsafe {
                item.setTarget(Some(self.as_ref()));
                item.setAction(Some(sel!(setConcurrency:)));
            }
            concurrency_menu.addItem(&item);
        }
        let concurrency_root = NSMenuItem::new(mtm);
        // "Workers", not "Run N at a Time": the number is a property of the
        // queue that stays true while you are not looking at it, and reads as
        // one at a glance rather than as a sentence about right now.
        concurrency_root.setTitle(&NSString::from_str(&format!(
            "{concurrency} Worker{}",
            if concurrency == 1 { "" } else { "s" }
        )));
        concurrency_root.setEnabled(true);
        concurrency_root.setSubmenu(Some(&concurrency_menu));
        menu.addItem(&concurrency_root);

        action("Clear Finished", sel!(clearFinished:), finished > 0, 0);

        menu.addItem(&NSMenuItem::separatorItem(mtm));
        action("Jobs Folder…", sel!(openFolder:), true, 0);

        // Each style shows itself: the same icon it would put in the bar, in a
        // representative state, so the choice is made by looking rather than by
        // reading four names and guessing.
        let style_menu = NSMenu::new(mtm);
        style_menu.setAutoenablesItems(false);
        let preview = icon::IconState {
            running: 1,
            queued: 2,
            ..icon::IconState::default()
        };
        let current = self.with_ui(|ui| ui.style).unwrap_or(icon::Style::Cursors);
        for (index, style) in icon::ALL_STYLES.iter().enumerate() {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(style.label()));
            item.setTag(index as isize);
            item.setEnabled(true);
            item.setImage(Some(&icon::draw(*style, &preview)));
            item.setState(if *style == current {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            });
            unsafe {
                item.setTarget(Some(self.as_ref()));
                item.setAction(Some(sel!(setStyle:)));
            }
            style_menu.addItem(&item);
        }
        let style_root = NSMenuItem::new(mtm);
        style_root.setTitle(ns_string!("Icon"));
        style_root.setEnabled(true);
        style_root.setSubmenu(Some(&style_menu));
        menu.addItem(&style_root);

        let notifications = action("Notifications", sel!(toggleNotifications:), true, 0);
        notifications.setState(if self.with_ui(|ui| ui.muted).unwrap_or(false) {
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

        self.with_ui(|ui| ui.rows = views);
    }
}

fn open(path: &std::path::Path) {
    let _ = Command::new("open").arg(path).spawn();
}

const STYLE: &str = "style";
const MUTED: &str = "muted";
const CONCURRENCY: &str = "concurrency";

/// Preferences about how *this* app looks and behaves, kept away from the jobs
/// folder: the folder is the work, not the settings.
fn config_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(".config/job-folder")
}

fn load(name: &str) -> Option<String> {
    fs::read_to_string(config_dir().join(name)).ok()
}

fn save(name: &str, value: &str) {
    let _ = fs::create_dir_all(config_dir());
    let _ = fs::write(config_dir().join(name), value);
}

fn main() {
    let mtm = MainThreadMarker::new().expect("must run on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let jobs = Jobs::start(queue::root());
    if let Some(concurrency) = load(CONCURRENCY).and_then(|text| text.trim().parse::<usize>().ok()) {
        jobs.set_concurrency(concurrency);
    }

    let widget = Widget::new(mtm, Arc::clone(&jobs));
    WIDGET.with(|slot| *slot.borrow_mut() = Some(widget.clone()));
    app.setDelegate(Some(ProtocolObject::from_ref(&*widget)));

    // Where a row button ends up. It runs here, on the main thread, off the
    // click — the command is applied to the queue and the menu redrawn before
    // this returns, which is the entire difference between this app and
    // watching a folder.
    row::on_call(move |token| {
        if let Some((id, verb)) = queue::untoken(token) {
            jobs.command(id, verb);
        }
        WIDGET.with(|slot| {
            if let Some(widget) = slot.borrow().as_ref() {
                widget.refresh_rows();
                widget.refresh_icon();
            }
        });
    });

    widget.refresh_icon();

    unsafe {
        NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
            TICK,
            &widget,
            sel!(tick:),
            None,
            true,
        );
    }

    app.run();
}

