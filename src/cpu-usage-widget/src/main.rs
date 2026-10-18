//! cpu-usage-widget — CPU (and optionally GPU) load in the menu bar.
//!
//! Rust talking to AppKit directly via [objc2](https://github.com/madsmtm/objc2),
//! in the same family as `battery-widget` and `free-disk-space-widget`: no
//! wrapper library, no vendored fork, no `.app` bundle. The dropdown is a real
//! `NSMenu` assigned to the status item, rebuilt each time it opens so the
//! readings in it are never stale.

mod bar;
mod cpu;
mod gpu;
mod procs;

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{
    AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel,
};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSAttributedStringAttachmentConveniences,
    NSCellImagePosition, NSControlStateValueOff, NSControlStateValueOn, NSFont,
    NSFontAttributeName, NSImage, NSMenu, NSMenuDelegate, NSMenuItem, NSMutableParagraphStyle,
    NSParagraphStyleAttributeName, NSStatusBar, NSStatusItem, NSStringDrawing, NSTextAlignment,
    NSTextAttachment, NSTextTab, NSVariableStatusItemLength,
};
use objc2_foundation::{
    NSArray, NSAttributedString, NSMutableAttributedString, NSMutableDictionary, NSObject,
    NSObjectProtocol, NSPoint, NSRange, NSRect, NSString, NSTimer,
};

use bar::Gauge;

const UPDATE_INTERVAL_SECONDS: f64 = 2.0;

/// Gap between the longest process name and the meter column, in ems of the
/// menu font — the same breathing room the battery widget's style rows use.
const PROCESS_ROW_GAP_RATIO: f64 = 1.5;

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
            // Only walk the IO registry when something is actually asking for
            // the GPU.
            ui.reading.gpu = ui.settings.sources.shows_gpu().then(gpu::read).flatten();
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

        let (settings, cpu, cores, gpu, processes) = {
            let ivars = self.ivars().borrow();
            let Some(ui) = ivars.as_ref() else { return };
            (
                ui.settings,
                ui.reading.cpu(),
                ui.reading.cores.len(),
                ui.reading.gpu,
                ui.reading
                    .processes
                    .iter()
                    .map(|p| (p.name.clone(), p.share))
                    .collect::<Vec<_>>(),
            )
        };

        let info = |title: String| {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(&title));
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

        info(format!("CPU: {} across {cores} cores", bar::percent(cpu)));
        match gpu {
            Some(gpu) => {
                info(format!("GPU: {}", bar::percent(gpu.device)));
                info(format!(
                    "    renderer {} · tiler {}",
                    bar::percent(gpu.renderer),
                    bar::percent(gpu.tiler)
                ));
            }
            // Only worth saying when the GPU was asked for and did not answer.
            None if settings.sources.shows_gpu() => info("GPU: not reporting".to_string()),
            None => {}
        }

        // Empty only on the first tick, before there is an interval to rank.
        if !processes.is_empty() {
            menu.addItem(&NSMenuItem::separatorItem(mtm));
            info("Top processes".to_string());
            let paragraph = process_row_paragraph(&processes);
            for (name, share) in &processes {
                let item = NSMenuItem::new(mtm);
                let title = process_row_title(name, &bar::meter_image(*share));
                apply_row_paragraph(&title, &paragraph);
                item.setAttributedTitle(Some(&title));
                item.setEnabled(false);
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
            "Show",
            ALL_SOURCES.iter().map(|sources| sources.label()),
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

/// The menu font — the one macOS sets menu item titles in, a size apart from
/// the menu *bar* font the status item is drawn at.
fn menu_font() -> Retained<NSFont> {
    NSFont::menuFontOfSize(0.0)
}

/// How wide `text` sets as a menu item title.
fn menu_label_width(text: &str) -> f64 {
    let attrs = NSMutableDictionary::<NSString, AnyObject>::new();
    unsafe {
        attrs.setObject_forKey(&menu_font(), ProtocolObject::from_ref(NSFontAttributeName));
        NSString::from_str(text)
            .sizeWithAttributes(Some(&attrs))
            .width
    }
}

/// A process row: the name flush left, its meter flush right against a column
/// edge shared by every row.
///
/// A right-aligned tab stop is what does it — the name, a tab, then the meter
/// as an attachment, so every meter finishes on the same line however short
/// its name. The alternative, a custom `NSView` per row, would cost the rows
/// their native look. Same construction as the battery widget's style rows.
fn process_row_title(name: &str, meter: &NSImage) -> Retained<NSMutableAttributedString> {
    let font = menu_font();

    let attachment = NSTextAttachment::new();
    attachment.setImage(Some(meter));

    // An attachment sits on the baseline, which hangs the meter below the
    // name it shares a row with. Drop it by half the difference so the two
    // centre on each other.
    let size = meter.size();
    attachment.setBounds(NSRect {
        origin: NSPoint {
            x: 0.0,
            y: font.capHeight() / 2.0 - size.height / 2.0,
        },
        size,
    });

    let title = NSMutableAttributedString::initWithString(
        NSMutableAttributedString::alloc(),
        &NSString::from_str(&format!("{name}\t")),
    );
    title.appendAttributedString(&NSAttributedString::attributedStringWithAttachment(
        &attachment,
    ));
    title
}

/// The paragraph style the process rows share: one right-aligned tab stop
/// past the longest name, with room for the meter. One figure for the whole
/// list is what puts the meters in a column rather than leaving each to sit
/// where its own name ends.
fn process_row_paragraph(processes: &[(String, f64)]) -> Retained<NSMutableParagraphStyle> {
    let widest_name = processes
        .iter()
        .map(|(name, _)| menu_label_width(name))
        .fold(0.0, f64::max);
    let right_edge = widest_name
        + menu_font().pointSize() * PROCESS_ROW_GAP_RATIO
        + bar::meter_image(0.0).size().width;

    let paragraph = NSMutableParagraphStyle::new();
    let tab = unsafe {
        NSTextTab::initWithTextAlignment_location_options(
            NSTextTab::alloc(),
            NSTextAlignment::Right,
            right_edge,
            &NSMutableDictionary::new(),
        )
    };
    paragraph.setTabStops(Some(&NSArray::from_retained_slice(&[tab])));
    paragraph
}

/// Stamp the shared paragraph style over a whole row title.
fn apply_row_paragraph(title: &NSMutableAttributedString, paragraph: &NSMutableParagraphStyle) {
    unsafe {
        title.addAttribute_value_range(
            NSParagraphStyleAttributeName,
            paragraph,
            NSRange {
                location: 0,
                length: title.length(),
            },
        );
    }
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
