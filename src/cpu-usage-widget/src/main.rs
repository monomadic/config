//! cpu-usage-widget — CPU (and optionally GPU) load in the menu bar.
//!
//! Rust talking to AppKit directly via [objc2](https://github.com/madsmtm/objc2),
//! in the same family as `battery-widget` and `free-disk-space-widget`: no
//! wrapper library, no vendored fork, no `.app` bundle. The dropdown is a real
//! `NSMenu` assigned to the status item, rebuilt each time it opens so the
//! readings in it are never stale.

mod bar;
mod cpu;
mod dashboard;
mod gpu;
mod procs;

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2::{
    AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel,
};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSCellImagePosition, NSColor,
    NSControlStateValueOff, NSControlStateValueOn, NSFont, NSFontAttributeName,
    NSForegroundColorAttributeName, NSMenu, NSMenuDelegate, NSMenuItem, NSMutableParagraphStyle,
    NSParagraphStyleAttributeName, NSStatusBar, NSStatusItem, NSTextAlignment, NSTextTab,
    NSVariableStatusItemLength,
};
use objc2_foundation::{
    NSArray, NSMutableAttributedString, NSMutableDictionary, NSObject, NSObjectProtocol, NSRange,
    NSRunLoop, NSRunLoopCommonModes, NSString, NSTimer,
};

use bar::Gauge;
use dashboard::{DashboardView, Part, Snapshot};

pub const UPDATE_INTERVAL_SECONDS: f64 = 2.0;

/// Room left at the right of a value-bearing row for the submenu chevron
/// AppKit draws there, in ems of the menu font.
const CHEVRON_ROOM_RATIO: f64 = 1.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum LayoutStyle {
    PerCore,
    Bar,
    BarText,
    Text,
}

const ALL_STYLES: [LayoutStyle; 4] = [
    LayoutStyle::PerCore,
    LayoutStyle::Bar,
    LayoutStyle::BarText,
    LayoutStyle::Text,
];

impl LayoutStyle {
    fn label(self) -> &'static str {
        match self {
            LayoutStyle::PerCore => "Per-core Bars",
            LayoutStyle::Bar => "Aggregate Bar",
            LayoutStyle::BarText => "Aggregate Bar and Text",
            LayoutStyle::Text => "Percentage Text",
        }
    }

    fn key(self) -> &'static str {
        match self {
            LayoutStyle::PerCore => "per_core",
            LayoutStyle::Bar => "bar",
            LayoutStyle::BarText => "bar_text",
            LayoutStyle::Text => "text",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        ALL_STYLES.iter().copied().find(|style| style.key() == key)
    }
}

/// Which meters share the status item. The GPU has no per-core breakdown, so
/// in the per-core layout it contributes its three utilization figures instead.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Sources {
    Cpu,
    Gpu,
    CpuAndGpu,
}

const ALL_SOURCES: [Sources; 3] = [Sources::Cpu, Sources::Gpu, Sources::CpuAndGpu];

impl Sources {
    fn label(self) -> &'static str {
        match self {
            Sources::Cpu => "CPU",
            Sources::Gpu => "GPU",
            Sources::CpuAndGpu => "CPU and GPU",
        }
    }

    fn key(self) -> &'static str {
        match self {
            Sources::Cpu => "cpu",
            Sources::Gpu => "gpu",
            Sources::CpuAndGpu => "cpu_gpu",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        ALL_SOURCES.iter().copied().find(|s| s.key() == key)
    }

    fn shows_cpu(self) -> bool {
        self != Sources::Gpu
    }

    fn shows_gpu(self) -> bool {
        self != Sources::Cpu
    }
}

#[derive(Clone, Copy)]
struct Settings {
    style: LayoutStyle,
    sources: Sources,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            style: LayoutStyle::PerCore,
            sources: Sources::Cpu,
        }
    }
}

impl Settings {
    /// `key=value` lines, in the same place the other widgets keep theirs.
    fn path() -> Option<PathBuf> {
        let home = std::env::var_os("HOME")?;
        Some(PathBuf::from(home).join(".config/cpu-usage-widget/settings"))
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
                "sources" => {
                    if let Some(sources) = Sources::from_key(value.trim()) {
                        settings.sources = sources;
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
            "style={}\nsources={}\n",
            self.style.key(),
            self.sources.key()
        );
        if let Err(err) = fs::write(&path, body) {
            eprintln!("error saving settings: {err}");
        }
    }
}

/// The most recent readings, kept so the dropdown and the redraw both work
/// from one sample rather than each taking their own.
#[derive(Default)]
struct Reading {
    cores: Vec<f64>,
    gpu: Option<gpu::Gpu>,
    /// The busiest processes over the last interval, busiest first.
    processes: Vec<procs::Process>,
}

impl Reading {
    fn cpu(&self) -> f64 {
        if self.cores.is_empty() {
            return 0.0;
        }
        self.cores.iter().map(|c| c.clamp(0.0, 1.0)).sum::<f64>() / self.cores.len() as f64
    }
}

/// The meters to draw, in top-to-bottom order. A GPU the machine will not
/// report simply drops out, leaving the CPU on its own rather than an empty
/// item.
fn gauges(reading: &Reading, sources: Sources) -> Vec<Gauge> {
    let mut gauges = Vec::new();
    if sources.shows_cpu() {
        gauges.push(Gauge {
            label: "C",
            columns: reading.cores.clone(),
            value: reading.cpu(),
        });
    }
    if sources.shows_gpu()
        && let Some(gpu) = reading.gpu
    {
        // The driver publishes no per-core GPU load — `num_cores` is static
        // configuration, and utilization only ever comes whole-device. These
        // three are every utilization figure there is, headline first.
        gauges.push(Gauge {
            label: "G",
            columns: vec![gpu.device, gpu.renderer, gpu.tiler],
            value: gpu.device,
        });
    }
    if gauges.is_empty() {
        gauges.push(Gauge {
            label: "C",
            columns: reading.cores.clone(),
            value: reading.cpu(),
        });
    }
    gauges
}

struct Ui {
    status_item: Retained<NSStatusItem>,
    settings: Settings,
    sampler: cpu::Sampler,
    processes: procs::Sampler,
    reading: Reading,
    /// Looked up once; it does not change while the widget runs.
    machine: String,
    /// The two dashboard views while the menu is open, so each tick can hand
    /// them a fresh snapshot. `None` while it is closed.
    dashboard: Option<[Retained<DashboardView>; 2]>,
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

        #[unsafe(method(sourcesAction:))]
        fn sources_action(&self, sender: &NSMenuItem) {
            self.apply(|settings| settings.sources = ALL_SOURCES[sender.tag() as usize]);
        }

        #[unsafe(method(openActivityMonitor:))]
        fn open_activity_monitor(&self, _sender: &NSMenuItem) {
            open_application("Activity Monitor");
        }

        #[unsafe(method(quitAction:))]
        fn quit_action(&self, _sender: &NSMenuItem) {
            let mtm = MainThreadMarker::new().unwrap();
            NSApplication::sharedApplication(mtm).terminate(None);
        }
    }

    unsafe impl NSObjectProtocol for Widget {}

    // macOS calls this immediately before showing the menu, which is what keeps
    // the readings in it current without a second timer.
    unsafe impl NSMenuDelegate for Widget {
        #[unsafe(method(menuNeedsUpdate:))]
        fn menu_needs_update(&self, menu: &NSMenu) {
            self.rebuild_menu(menu);
        }

        // Once the menu is gone, the ticks stop redrawing into it.
        #[unsafe(method(menuDidClose:))]
        fn menu_did_close(&self, _menu: &NSMenu) {
            if let Some(ui) = self.ivars().borrow_mut().as_mut() {
                ui.dashboard = None;
            }
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
            sampler: cpu::Sampler::default(),
            processes: procs::Sampler::default(),
            reading: Reading::default(),
            machine: dashboard::machine_name(),
            dashboard: None,
        });
        this
    }

    fn apply(&self, change: impl FnOnce(&mut Settings)) {
        let settings = {
            let mut ivars = self.ivars().borrow_mut();
            let Some(ui) = ivars.as_mut() else { return };
            change(&mut ui.settings);
            // A GPU meter switched on from the menu should appear straight
            // away rather than on the next tick.
            ui.reading.gpu = ui.settings.sources.shows_gpu().then(gpu::read).flatten();
            ui.settings
        };
        settings.save();
        self.render();
    }

    /// Take a fresh sample and redraw. Utilization is a rate, so the very first
    /// tick after launch reports zeros — that is the baseline being primed.
    fn update(&self) {
        {
            let mut ivars = self.ivars().borrow_mut();
            let Some(ui) = ivars.as_mut() else { return };
            match ui.sampler.sample() {
                Ok(cores) => ui.reading.cores = cores,
                Err(err) => eprintln!("error reading cpu usage: {err}"),
            }
            ui.reading.processes = ui.processes.sample();
            // Only walk the IO registry when something is actually showing
            // the GPU: the status item, or the open dashboard.
            let wants_gpu = ui.settings.sources.shows_gpu() || ui.dashboard.is_some();
            ui.reading.gpu = wants_gpu.then(gpu::read).flatten();

            // The menu does not rebuild while it is open, so an open
            // dashboard is redrawn from each new reading instead.
            if let Some([header, processes]) = &ui.dashboard {
                let (fresh_header, fresh_processes) = snapshots(ui);
                header.set_snapshot(fresh_header);
                processes.set_snapshot(fresh_processes);
            }
        }
        self.render();
    }

    fn render(&self) {
        let mtm = MainThreadMarker::new().unwrap();
        let ivars = self.ivars().borrow();
        let Some(ui) = ivars.as_ref() else { return };
        let Some(button) = ui.status_item.button(mtm) else {
            return;
        };

        let gauges = gauges(&ui.reading, ui.settings.sources);
        let stacked = gauges.len() > 1;

        // A lone percentage is just the button's title; anything else — every
        // drawn meter, and stacked text that has to line up — is one image.
        let (image, title) = match ui.settings.style {
            LayoutStyle::PerCore => (Some(bar::columns_image(&gauges)), String::new()),
            LayoutStyle::Bar => (Some(bar::bars_image(&gauges, false)), String::new()),
            LayoutStyle::BarText if stacked => {
                (Some(bar::bars_image(&gauges, true)), String::new())
            }
            LayoutStyle::BarText => (
                Some(bar::bars_image(&gauges, false)),
                bar::percent(gauges[0].value),
            ),
            LayoutStyle::Text if stacked => (Some(bar::text_image(&gauges)), String::new()),
            LayoutStyle::Text => (None, bar::percent(gauges[0].value)),
        };

        button.setAttributedTitle(&bar::attributed_title(&title));
        match image {
            Some(image) => {
                button.setImage(Some(&image));
                button.setImagePosition(if title.is_empty() {
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

        button.setToolTip(Some(&NSString::from_str(&tooltip(&ui.reading))));
    }

    fn rebuild_menu(&self, menu: &NSMenu) {
        let mtm = MainThreadMarker::new().unwrap();
        menu.removeAllItems();

        let (settings, header, processes) = {
            let mut ivars = self.ivars().borrow_mut();
            let Some(ui) = ivars.as_mut() else { return };
            // The dashboard shows the GPU whenever the machine reports one,
            // whatever the status item is set to show; so make sure there is
            // a reading for it to draw.
            if ui.reading.gpu.is_none() {
                ui.reading.gpu = gpu::read();
            }
            let (header, processes) = snapshots(ui);
            (ui.settings, header, processes)
        };

        let header = DashboardView::new(mtm, header, Part::Header);
        let processes = DashboardView::new(mtm, processes, Part::Processes);
        for (view, rule_after) in [(&header, true), (&processes, false)] {
            let item = NSMenuItem::new(mtm);
            item.setEnabled(false);
            item.setView(Some(view));
            menu.addItem(&item);
            if rule_after {
                menu.addItem(&NSMenuItem::separatorItem(mtm));
            }
        }
        if let Some(ui) = self.ivars().borrow_mut().as_mut() {
            ui.dashboard = Some([header, processes]);
        }

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

        menu.addItem(&NSMenuItem::separatorItem(mtm));
        self.add_submenu(
            menu,
            "Style",
            &ALL_STYLES.map(|style| style.label()),
            ALL_STYLES.iter().position(|s| *s == settings.style),
            sel!(styleAction:),
            mtm,
        );
        self.add_submenu(
            menu,
            "Show",
            &ALL_SOURCES.map(|sources| sources.label()),
            ALL_SOURCES.iter().position(|s| *s == settings.sources),
            sel!(sourcesAction:),
            mtm,
        );

        menu.addItem(&NSMenuItem::separatorItem(mtm));
        action("Open Activity Monitor", sel!(openActivityMonitor:));

        // Our own selector rather than `terminate:`: recent macOS decorates
        // menu items it recognises as standard actions with an SF Symbol, and
        // an unfamiliar action is the way to opt out of that.
        menu.addItem(&NSMenuItem::separatorItem(mtm));
        let quit = action("Quit", sel!(quitAction:));
        quit.setImage(None);
    }

    /// A submenu whose root row also shows the current choice, dimmed, flush
    /// right where the chevron sits.
    fn add_submenu(
        &self,
        menu: &NSMenu,
        title: &str,
        labels: &[&str],
        selected: Option<usize>,
        selector: objc2::runtime::Sel,
        mtm: MainThreadMarker,
    ) {
        let submenu = NSMenu::new(mtm);
        submenu.setAutoenablesItems(false);
        for (index, label) in labels.iter().enumerate() {
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
        let current = selected.map_or("", |index| labels[index]);
        root.setAttributedTitle(Some(&value_row_title(title, current)));
        root.setEnabled(true);
        root.setSubmenu(Some(&submenu));
        menu.addItem(&root);
    }
}

/// What the two dashboard views draw, from the current reading: the header's
/// snapshot carries no rows, the process table's carries them with icons.
fn snapshots(ui: &Ui) -> (Snapshot, Snapshot) {
    let reading = &ui.reading;
    let wants_gpu = reading.gpu.is_some() || ui.settings.sources.shows_gpu();
    let header = Snapshot {
        machine: ui.machine.clone(),
        cores: reading.cores.clone(),
        cpu: reading.cpu(),
        gpu: reading.gpu,
        wants_gpu,
        rows: Vec::new(),
    };
    let processes = Snapshot {
        machine: String::new(),
        cores: Vec::new(),
        cpu: 0.0,
        gpu: None,
        wants_gpu,
        rows: reading
            .processes
            .iter()
            .map(|p| dashboard::Row {
                pid: p.pid,
                name: p.name.clone(),
                share: p.share,
                icon: p.path.as_deref().and_then(dashboard::icon_for),
            })
            .collect(),
    };
    (header, processes)
}

/// The menu font — the one macOS sets menu item titles in, a size apart from
/// the menu *bar* font the status item is drawn at.
fn menu_font() -> Retained<NSFont> {
    NSFont::menuFontOfSize(0.0)
}

/// A setting row: the label flush left, its current value dimmed and flush
/// right against the same edge the dashboard blocks end at, leaving room for
/// the submenu chevron. A right-aligned tab stop is what lines the values up
/// across rows, the construction the battery widget's style rows use.
fn value_row_title(label: &str, value: &str) -> Retained<NSMutableAttributedString> {
    let font = menu_font();
    let em = font.pointSize();
    let title = NSMutableAttributedString::initWithString(
        NSMutableAttributedString::alloc(),
        &NSString::from_str(&format!("{label}\t{value}")),
    );
    let whole = NSRange {
        location: 0,
        length: title.length(),
    };
    let value_length = value.encode_utf16().count();
    let value_range = NSRange {
        location: whole.length - value_length,
        length: value_length,
    };

    let paragraph = NSMutableParagraphStyle::new();
    let right_edge = dashboard::content_right_edge() - (em * CHEVRON_ROOM_RATIO).round();
    let tab = unsafe {
        NSTextTab::initWithTextAlignment_location_options(
            NSTextTab::alloc(),
            NSTextAlignment::Right,
            right_edge,
            &NSMutableDictionary::new(),
        )
    };
    paragraph.setTabStops(Some(&NSArray::from_retained_slice(&[tab])));
    unsafe {
        title.addAttribute_value_range(NSFontAttributeName, &font, whole);
        title.addAttribute_value_range(NSParagraphStyleAttributeName, &paragraph, whole);
        title.addAttribute_value_range(
            NSForegroundColorAttributeName,
            &NSColor::secondaryLabelColor(),
            value_range,
        );
    }
    title
}

fn tooltip(reading: &Reading) -> String {
    let mut text = format!(
        "CPU {} across {} cores",
        bar::percent(reading.cpu()),
        reading.cores.len()
    );
    if let Some(gpu) = reading.gpu {
        text += &format!("\nGPU {}", bar::percent(gpu.device));
    }
    text
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

    // Added for the common modes rather than scheduled in the default one, so
    // the ticks keep coming while the menu is open — menu tracking runs the
    // run loop in its own mode, and a default-mode timer would pause for as
    // long as the dropdown is up, which is exactly when the numbers matter.
    unsafe {
        let timer = NSTimer::timerWithTimeInterval_target_selector_userInfo_repeats(
            UPDATE_INTERVAL_SECONDS,
            &widget,
            sel!(tick:),
            None,
            true,
        );
        NSRunLoop::mainRunLoop().addTimer_forMode(&timer, NSRunLoopCommonModes);
    }

    app.run();
}
