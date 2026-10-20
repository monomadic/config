//! The dashboard blocks at the top of the dropdown.
//!
//! Two custom `NSView`s, each in its own `NSMenuItem`: a header with the CPU
//! and GPU readings as big numbers over a strip of pills, and a table of the
//! busiest processes with their app icons. Views rather than attributed
//! titles because these are layouts, not lines — columns of numbers, pills
//! under them, a table — and none of it is interactive, so the native
//! highlight a custom view forgoes is not missed. Each is drawn in one pass
//! from a snapshot of the reading taken when the menu opens.
//!
//! No point sizes are hardcoded: every size is a ratio of the menu font's
//! point size, so the blocks follow the system text size like the status item
//! does. Colours are the system's semantic and accent colours, which resolve
//! at draw time, so the blocks follow light and dark appearance too.

use std::cell::RefCell;
use std::ffi::c_char;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{
    AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send,
};
use objc2_app_kit::{
    NSBezierPath, NSColor, NSCompositingOperation, NSEvent, NSFont, NSFontAttributeName,
    NSFontDescriptorSymbolicTraits, NSFontWeight, NSFontWeightBold, NSFontWeightMedium,
    NSFontWeightRegular, NSFontWeightSemibold, NSForegroundColorAttributeName, NSImage,
    NSKernAttributeName, NSRunningApplication, NSStringDrawing, NSTrackingArea,
    NSTrackingAreaOptions, NSView, NSWorkspace,
};
use objc2_core_foundation::{CFData, CFString};
use objc2_foundation::{NSMutableDictionary, NSNumber, NSPoint, NSRect, NSSize, NSString};
use objc2_io_kit::{IOObjectRelease, IORegistryEntryCreateCFProperty, IORegistryEntryFromPath};

use crate::gpu::Gpu;

/// Cores at or above this share of their capacity are drawn hot.
const HOT_CORE: f64 = 0.8;
/// Processes at or above a whole core are drawn hot.
const HOT_PROCESS: f64 = 1.0;

/// One line of the process table.
pub struct Row {
    pub pid: i32,
    pub name: String,
    pub share: f64,
    pub icon: Option<Retained<NSImage>>,
}

/// Everything the blocks draw, captured when the menu opens.
pub struct Snapshot {
    pub machine: String,
    pub cores: Vec<f64>,
    pub cpu: f64,
    pub gpu: Option<Gpu>,
    /// Whether the GPU column should appear at all — it is left out when the
    /// user has not asked for the GPU, and says "not reporting" when they
    /// have and the machine will not answer.
    pub wants_gpu: bool,
    pub rows: Vec<Row>,
}

/// Which of the two blocks a view draws.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Part {
    Header,
    Processes,
}

pub struct Ivars {
    snapshot: Snapshot,
    part: Part,
    /// The process row under the pointer, which swaps its meter for Quit
    /// and Kill buttons.
    hovered: Option<usize>,
    tracking: Option<Retained<NSTrackingArea>>,
}

/// What a row's buttons do to its process.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    /// Ask nicely: the app's own quit, or SIGTERM for anything that is not
    /// an app.
    Quit,
    /// SIGKILL, or AppKit's force-terminate for an app.
    Kill,
}

const SIGTERM: i32 = 15;
const SIGKILL: i32 = 9;

unsafe extern "C" {
    fn kill(pid: i32, signal: i32) -> i32;
}

define_class!(
    // SAFETY: NSView has no subclassing requirements beyond `initWithFrame:`,
    // which `new` goes through, and DashboardView does not implement Drop.
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[ivars = RefCell<Ivars>]
    pub struct DashboardView;

    impl DashboardView {
        // Top-down coordinates, so the layout reads the way the design does.
        #[unsafe(method(isFlipped))]
        fn is_flipped(&self) -> bool {
            true
        }

        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, _dirty: NSRect) {
            let ivars = self.ivars().borrow();
            // AppKit stretches the view to the item's width, which is wider
            // than the frame it was created with whenever the menu has
            // native rows wider than it; drawing against the bounds is what
            // keeps the right inset equal to the left one.
            let m = Metrics::new(self.bounds().size.width);
            match ivars.part {
                Part::Header => draw_header(&ivars.snapshot, &m),
                Part::Processes => draw_processes(&ivars.snapshot, &m, ivars.hovered),
            }
        }

        // One tracking area over the whole view, remade whenever AppKit says
        // the geometry moved. Menus are not key, so it has to be active
        // always, not merely in the key window.
        #[unsafe(method(updateTrackingAreas))]
        fn update_tracking_areas(&self) {
            let _: () = unsafe { msg_send![super(self), updateTrackingAreas] };
            if self.ivars().borrow().part != Part::Processes {
                return;
            }
            if let Some(old) = self.ivars().borrow_mut().tracking.take() {
                self.removeTrackingArea(&old);
            }
            let area = unsafe {
                NSTrackingArea::initWithRect_options_owner_userInfo(
                    NSTrackingArea::alloc(),
                    self.bounds(),
                    NSTrackingAreaOptions::MouseEnteredAndExited
                        | NSTrackingAreaOptions::MouseMoved
                        | NSTrackingAreaOptions::ActiveAlways
                        | NSTrackingAreaOptions::InVisibleRect,
                    Some(self),
                    None,
                )
            };
            self.addTrackingArea(&area);
            self.ivars().borrow_mut().tracking = Some(area);
        }

        #[unsafe(method(mouseEntered:))]
        fn mouse_entered(&self, event: &NSEvent) {
            self.hover(Some(event));
        }

        #[unsafe(method(mouseMoved:))]
        fn mouse_moved(&self, event: &NSEvent) {
            self.hover(Some(event));
        }

        #[unsafe(method(mouseExited:))]
        fn mouse_exited(&self, _event: &NSEvent) {
            self.hover(None);
        }

        #[unsafe(method(mouseUp:))]
        fn mouse_up(&self, event: &NSEvent) {
            let point = self.point_of(event);
            let m = Metrics::new(self.bounds().size.width);
            let hit = {
                let ivars = self.ivars().borrow();
                row_at(&m, &ivars.snapshot, point).and_then(|index| {
                    let top = m.rows_top() + index as f64 * m.row_height();
                    buttons(&m, top)
                        .into_iter()
                        .find(|(rect, _)| contains(rect, point))
                        .map(|(_, action)| (ivars.snapshot.rows[index].pid, action))
                })
            };
            if let Some((pid, action)) = hit {
                perform(pid, action);
            }
        }
    }
);

impl DashboardView {
    fn point_of(&self, event: &NSEvent) -> NSPoint {
        self.convertPoint_fromView(event.locationInWindow(), None)
    }

    /// Track which row the pointer is over; redraw only when that changes.
    fn hover(&self, event: Option<&NSEvent>) {
        let hovered = event.and_then(|event| {
            let m = Metrics::new(self.bounds().size.width);
            let ivars = self.ivars().borrow();
            row_at(&m, &ivars.snapshot, self.point_of(event))
        });
        let changed = {
            let mut ivars = self.ivars().borrow_mut();
            let changed = ivars.hovered != hovered;
            ivars.hovered = hovered;
            changed
        };
        if changed {
            self.setNeedsDisplay(true);
        }
    }
}

/// Quit or kill a process. Apps get AppKit's version, which goes through the
/// app's own quit (or its forced equivalent); anything else gets the signal.
fn perform(pid: i32, action: Action) {
    let done = match (
        NSRunningApplication::runningApplicationWithProcessIdentifier(pid),
        action,
    ) {
        (Some(app), Action::Quit) => app.terminate(),
        (Some(app), Action::Kill) => app.forceTerminate(),
        (None, _) => false,
    };
    if !done {
        let signal = match action {
            Action::Quit => SIGTERM,
            Action::Kill => SIGKILL,
        };
        // SAFETY: plain syscall; a pid that has gone away just fails.
        if unsafe { kill(pid, signal) } != 0 {
            eprintln!("could not signal pid {pid}");
        }
    }
}

/// The process row under `point`, if any.
fn row_at(m: &Metrics, snapshot: &Snapshot, point: NSPoint) -> Option<usize> {
    if point.y < m.rows_top() || point.x < 0.0 || point.x > m.width {
        return None;
    }
    let index = ((point.y - m.rows_top()) / m.row_height()) as usize;
    (index < snapshot.rows.len()).then_some(index)
}

fn contains(rect: &NSRect, point: NSPoint) -> bool {
    point.x >= rect.origin.x
        && point.x <= rect.origin.x + rect.size.width
        && point.y >= rect.origin.y
        && point.y <= rect.origin.y + rect.size.height
}

/// The Quit and Kill buttons of the row starting at `top`, Kill outermost.
fn buttons(m: &Metrics, top: f64) -> [(NSRect, Action); 2] {
    let h = (m.em * 1.5).round();
    let w = (m.em * 3.1).round();
    let gap = (m.em * 0.4).round();
    let y = top + ((m.row_height() - h) / 2.0).round();
    let kill_x = m.width - w;
    [
        (rect(kill_x - gap - w, y, w, h), Action::Quit),
        (rect(kill_x, y, w, h), Action::Kill),
    ]
}

impl DashboardView {
    /// Replace what the view draws and have it redraw. The frame is left
    /// alone: the row count is the same from one tick to the next.
    pub fn set_snapshot(&self, snapshot: Snapshot) {
        self.ivars().borrow_mut().snapshot = snapshot;
        self.setNeedsDisplay(true);
    }

    pub fn new(mtm: MainThreadMarker, snapshot: Snapshot, part: Part) -> Retained<Self> {
        let m = Metrics::new(0.0);
        let height = match part {
            Part::Header => m.header_height(&snapshot),
            Part::Processes => m.processes_height(&snapshot),
        };
        let frame = NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            size: NSSize {
                width: m.width,
                height,
            },
        };
        let this = Self::alloc(mtm).set_ivars(RefCell::new(Ivars {
            snapshot,
            part,
            hovered: None,
            tracking: None,
        }));
        unsafe { msg_send![super(this), initWithFrame: frame] }
    }
}

/// Every dimension, as a ratio of the menu font's point size.
struct Metrics {
    em: f64,
    width: f64,
    /// Left inset, chosen to line the content up with native item titles.
    /// There is no right inset: AppKit stretches the view to the item width
    /// less the menu's own trailing margin, and the submenu chevron on the
    /// native rows ends exactly at the view's right edge, so running the
    /// content to that edge is what lines the two up.
    inset: f64,
}

/// The width the views are created with, in ems. AppKit widens them to the
/// menu, whose width the native rows set.
const WIDTH_RATIO: f64 = 24.6;

impl Metrics {
    /// `width` is the view's current width; zero asks for the minimum the
    /// view is created with.
    fn new(width: f64) -> Self {
        let em = NSFont::menuFontOfSize(0.0).pointSize();
        Metrics {
            em,
            width: width.max((em * WIDTH_RATIO).round()),
            inset: (em * 1.08).round(),
        }
    }

    // -- header rows, top-down --
    fn title_top(&self) -> f64 {
        (self.em * 0.6).round()
    }
    fn badge_top(&self) -> f64 {
        self.title_top() + (self.em * 1.9).round()
    }
    fn badge_height(&self) -> f64 {
        (self.em * 1.25).round()
    }
    fn number_top(&self) -> f64 {
        self.badge_top() + self.badge_height() + (self.em * 0.3).round()
    }
    fn number_height(&self) -> f64 {
        (self.em * 3.0).round()
    }
    fn strip_top(&self) -> f64 {
        self.number_top() + self.number_height() + (self.em * 0.45).round()
    }
    fn strip_height(&self) -> f64 {
        (self.em * 2.3).round()
    }
    fn caption_top(&self) -> f64 {
        self.strip_top() + self.strip_height() + (self.em * 0.35).round()
    }
    fn header_height(&self, _snapshot: &Snapshot) -> f64 {
        self.caption_top() + (self.em * 1.1).round() + (self.em * 0.5).round()
    }

    // -- process rows --
    fn section_top(&self) -> f64 {
        (self.em * 0.55).round()
    }
    fn rows_top(&self) -> f64 {
        self.section_top() + (self.em * 1.75).round()
    }
    fn row_height(&self) -> f64 {
        (self.em * 2.2).round()
    }
    fn processes_height(&self, snapshot: &Snapshot) -> f64 {
        let rows = snapshot.rows.len().max(1) as f64;
        self.rows_top() + rows * self.row_height() + (self.em * 0.3).round()
    }

    fn pill_width(&self) -> f64 {
        (self.em * 0.6).round()
    }
    fn pill_gap(&self) -> f64 {
        (self.em * 0.6).round()
    }
}

// ---- drawing ---------------------------------------------------------------

fn draw_header(s: &Snapshot, m: &Metrics) {
    let right = m.width;

    // Machine name, centred.
    let title = attrs(
        &system(m.em * 0.85, unsafe { NSFontWeightBold }),
        &NSColor::secondaryLabelColor(),
        0.0,
    );
    let size = measure(&s.machine, &title);
    draw(
        &s.machine,
        ((m.width - size.width) / 2.0).round(),
        m.title_top(),
        &title,
    );

    // Badges: "CPU" in the CPU accent, "GPU" in the GPU accent.
    let badge_font = system(m.em * 0.77, unsafe { NSFontWeightBold });
    let cpu_badge = badge("CPU", &badge_font, &NSColor::systemBlueColor(), m);
    cpu_badge.draw_at(m.inset, m.badge_top());
    if s.wants_gpu {
        let gpu_badge = badge("GPU", &badge_font, &NSColor::systemGreenColor(), m);
        gpu_badge.draw_at(right - gpu_badge.width, m.badge_top());
    }

    // Big numbers. "%" shares the number's baseline, which in flipped
    // coordinates means offsetting it by the difference in ascenders.
    let big = system(m.em * 2.6, unsafe { NSFontWeightBold });
    let big_unit = system(m.em * 1.4, unsafe { NSFontWeightRegular });
    let number_y = m.number_top();
    let baseline = number_y + big.ascender();
    let label = NSColor::labelColor();
    let secondary = NSColor::secondaryLabelColor();
    let big_attrs = attrs(&big, &label, -0.02 * big.pointSize());
    let unit_attrs = attrs(&big_unit, &secondary, 0.0);
    let cpu_text = format!("{:.0}", s.cpu.clamp(0.0, 1.0) * 100.0);
    let cpu_width = measure(&cpu_text, &big_attrs).width;
    draw(&cpu_text, m.inset, number_y, &big_attrs);
    draw(
        "%",
        m.inset + cpu_width + (m.em * 0.2).round(),
        baseline - big_unit.ascender(),
        &unit_attrs,
    );

    if s.wants_gpu {
        let mid = system(m.em * 1.7, unsafe { NSFontWeightBold });
        let mid_unit = system(m.em * 1.0, unsafe { NSFontWeightRegular });
        let mid_attrs = attrs(&mid, &label, 0.0);
        let mid_unit_attrs = attrs(&mid_unit, &secondary, 0.0);
        let (value, unit) = match s.gpu {
            Some(gpu) => (format!("{:.0}", gpu.device.clamp(0.0, 1.0) * 100.0), "%"),
            None => ("—".to_string(), ""),
        };
        let unit_width = measure(unit, &mid_unit_attrs).width;
        let gap = if unit.is_empty() {
            0.0
        } else {
            (m.em * 0.15).round()
        };
        let value_width = measure(&value, &mid_attrs).width;
        let x = right - unit_width - gap - value_width;
        draw(&value, x, baseline - mid.ascender(), &mid_attrs);
        draw(
            unit,
            x + value_width + gap,
            baseline - mid_unit.ascender(),
            &mid_unit_attrs,
        );
    }

    // Per-core pills, a cluster from the left. Hot cores go red.
    let pill_w = m.pill_width();
    let gap = m.pill_gap();
    let track = label.colorWithAlphaComponent(0.12);
    let strip_top = m.strip_top();
    let strip_h = m.strip_height();
    for (i, load) in s.cores.iter().enumerate() {
        let x = m.inset + i as f64 * (pill_w + gap);
        let color = if *load >= HOT_CORE {
            NSColor::systemRedColor()
        } else {
            NSColor::systemBlueColor()
        };
        pill_column(
            x,
            strip_top,
            pill_w,
            strip_h,
            load.clamp(0.0, 1.0),
            &track,
            &color,
        );
    }

    // The GPU's three figures as pills from the right: device, renderer, tiler.
    if s.wants_gpu {
        let figures = s.gpu.map_or([0.0; 3], |g| [g.device, g.renderer, g.tiler]);
        let colors = [
            NSColor::systemGreenColor(),
            NSColor::systemPurpleColor(),
            NSColor::systemPinkColor(),
        ];
        for (i, (load, color)) in figures.iter().zip(&colors).enumerate() {
            let x = right - pill_w - (2 - i) as f64 * (pill_w + gap);
            pill_column(
                x,
                strip_top,
                pill_w,
                strip_h,
                load.clamp(0.0, 1.0),
                &track,
                color,
            );
        }
    }

    // Captions under the strips, italic.
    let caption = attrs(&italic(m.em * 0.85), &secondary, 0.0);
    let cores_text = format!("{} cores", s.cores.len());
    draw(&cores_text, m.inset, m.caption_top(), &caption);
    if s.wants_gpu {
        let text = match s.gpu {
            Some(g) => format!(
                "render {:.0} - tiler {:.0}",
                g.renderer.clamp(0.0, 1.0) * 100.0,
                g.tiler.clamp(0.0, 1.0) * 100.0
            ),
            None => "not reporting".to_string(),
        };
        let w = measure(&text, &caption).width;
        draw(&text, right - w, m.caption_top(), &caption);
    }
}

fn draw_processes(s: &Snapshot, m: &Metrics, hovered: Option<usize>) {
    let right = m.width;
    let label = NSColor::labelColor();
    let tertiary = NSColor::tertiaryLabelColor();

    // Section title with its sampling window on the right.
    let heading = system(m.em * 0.85, unsafe { NSFontWeightSemibold });
    let heading_attrs = attrs(&heading, &label, 0.06 * heading.pointSize());
    draw("TOP PROCESSES", m.inset, m.section_top(), &heading_attrs);
    let window_attrs = attrs(
        &system(m.em * 0.85, unsafe { NSFontWeightRegular }),
        &tertiary,
        0.0,
    );
    let window = format!("{:.0}s", crate::UPDATE_INTERVAL_SECONDS);
    draw(
        &window,
        right - measure(&window, &window_attrs).width,
        m.section_top(),
        &window_attrs,
    );

    let row_h = m.row_height();
    let icon = (m.em * 1.55).round();
    let gap = (m.em * 0.77).round();
    let meter_w = (m.em * 3.4).round();
    let meter_h = (m.em * 0.38).round().max(3.0);
    let share_w = (m.em * 3.1).round();
    let name_font = system(m.em, unsafe { NSFontWeightRegular });
    let name_attrs = attrs(&name_font, &label, 0.0);
    let share_attrs = attrs(&system(m.em, unsafe { NSFontWeightMedium }), &label, 0.0);
    let track = label.colorWithAlphaComponent(0.12);

    if s.rows.is_empty() {
        let waiting = attrs(&italic(m.em), &tertiary, 0.0);
        draw(
            "sampling…",
            m.inset + icon + gap,
            m.rows_top() + ((row_h - m.em * 1.2) / 2.0).round(),
            &waiting,
        );
        return;
    }

    for (i, row) in s.rows.iter().enumerate() {
        let top = m.rows_top() + i as f64 * row_h;
        let is_hovered = hovered == Some(i);
        if is_hovered {
            let pad = (m.em * 0.5).round();
            label.colorWithAlphaComponent(0.07).set();
            pill(
                rect(m.inset - pad, top, right - m.inset + pad * 2.0, row_h),
                (m.em * 0.45).round(),
            );
        }
        let icon_rect = rect(m.inset, top + ((row_h - icon) / 2.0).round(), icon, icon);
        match &row.icon {
            Some(image) => unsafe {
                image.drawInRect_fromRect_operation_fraction_respectFlipped_hints(
                    icon_rect,
                    NSRect::ZERO,
                    NSCompositingOperation::SourceOver,
                    1.0,
                    true,
                    None,
                );
            },
            None => {
                label.colorWithAlphaComponent(0.1).set();
                pill(icon_rect, (icon * 0.28).round());
                let initial: String = row.name.chars().next().into_iter().collect();
                let initial_attrs = attrs(
                    &system(m.em * 0.85, unsafe { NSFontWeightSemibold }),
                    &NSColor::secondaryLabelColor(),
                    0.0,
                );
                let size = measure(&initial, &initial_attrs);
                draw(
                    &initial,
                    icon_rect.origin.x + ((icon - size.width) / 2.0).round(),
                    icon_rect.origin.y + ((icon - size.height) / 2.0).round(),
                    &initial_attrs,
                );
            }
        }

        let share_x = right - share_w;
        let meter_x = share_x - gap - meter_w;
        let name_x = m.inset + icon + gap;
        let text_y = top + ((row_h - measure("X", &name_attrs).height) / 2.0).round();

        // A hovered row trades its meter and share for the two buttons.
        if is_hovered {
            let controls = buttons(m, top);
            let name = truncate(
                &row.name,
                &name_attrs,
                controls[0].0.origin.x - gap - name_x,
            );
            draw(&name, name_x, text_y, &name_attrs);
            let font = system(m.em * 0.85, unsafe { NSFontWeightSemibold });
            for (button, action) in controls {
                let (color, text) = match action {
                    Action::Quit => (label.clone(), "Quit"),
                    Action::Kill => (NSColor::systemRedColor(), "Kill"),
                };
                color.colorWithAlphaComponent(0.16).set();
                pill(button, button.size.height / 2.0);
                let attrs = attrs(&font, &color, 0.0);
                let size = measure(text, &attrs);
                draw(
                    text,
                    button.origin.x + ((button.size.width - size.width) / 2.0).round(),
                    button.origin.y + ((button.size.height - size.height) / 2.0).round(),
                    &attrs,
                );
            }
            continue;
        }

        let name = truncate(&row.name, &name_attrs, meter_x - gap - name_x);
        draw(&name, name_x, text_y, &name_attrs);

        let meter_y = top + ((row_h - meter_h) / 2.0).round();
        track.set();
        pill(rect(meter_x, meter_y, meter_w, meter_h), meter_h / 2.0);
        if row.share > 0.0 {
            if row.share >= HOT_PROCESS {
                NSColor::systemOrangeColor()
            } else {
                NSColor::systemBlueColor()
            }
            .set();
            pill(
                rect(
                    meter_x,
                    meter_y,
                    (meter_w * row.share.min(1.0)).max(meter_h),
                    meter_h,
                ),
                meter_h / 2.0,
            );
        }

        let share = crate::bar::cores_percent(row.share);
        draw(
            &share,
            right - measure(&share, &share_attrs).width,
            text_y,
            &share_attrs,
        );
    }
}

/// A rounded track with a rounded fill rising from the bottom.
fn pill_column(x: f64, top: f64, w: f64, h: f64, load: f64, track: &NSColor, color: &NSColor) {
    let radius = w / 2.0;
    track.set();
    pill(rect(x, top, w, h), radius);
    if load > 0.0 {
        let fill = (h * load).max(w);
        color.set();
        pill(rect(x, top + h - fill, w, fill), radius);
    }
}

/// A small capsule with its label inside, in one accent colour.
struct Badge {
    text: String,
    attrs: Retained<NSMutableDictionary<NSString, AnyObject>>,
    color: Retained<NSColor>,
    width: f64,
    height: f64,
    pad: f64,
}

fn badge(text: &str, font: &NSFont, color: &NSColor, m: &Metrics) -> Badge {
    let attrs = attrs(font, color, 0.04 * font.pointSize());
    let pad = (m.em * 0.5).round();
    let width = measure(text, &attrs).width.ceil() + pad * 2.0;
    Badge {
        text: text.to_string(),
        attrs,
        color: color.retain(),
        width,
        height: m.badge_height(),
        pad,
    }
}

impl Badge {
    fn draw_at(&self, x: f64, y: f64) {
        self.color.colorWithAlphaComponent(0.18).set();
        pill(
            rect(x, y, self.width, self.height),
            (self.height * 0.3).round(),
        );
        let size = measure(&self.text, &self.attrs);
        draw(
            &self.text,
            x + self.pad,
            y + ((self.height - size.height) / 2.0).round(),
            &self.attrs,
        );
    }
}

// ---- text helpers ------------------------------------------------------------

fn system(size: f64, weight: NSFontWeight) -> Retained<NSFont> {
    NSFont::systemFontOfSize_weight(size.round(), weight)
}

fn italic(size: f64) -> Retained<NSFont> {
    let regular = system(size, unsafe { NSFontWeightRegular });
    let descriptor = regular
        .fontDescriptor()
        .fontDescriptorWithSymbolicTraits(NSFontDescriptorSymbolicTraits::TraitItalic);
    NSFont::fontWithDescriptor_size(&descriptor, size.round()).unwrap_or(regular)
}

fn attrs(
    font: &NSFont,
    color: &NSColor,
    kern: f64,
) -> Retained<NSMutableDictionary<NSString, AnyObject>> {
    let attrs = NSMutableDictionary::<NSString, AnyObject>::new();
    unsafe {
        attrs.setObject_forKey(font, ProtocolObject::from_ref(NSFontAttributeName));
        attrs.setObject_forKey(
            color,
            ProtocolObject::from_ref(NSForegroundColorAttributeName),
        );
        if kern != 0.0 {
            attrs.setObject_forKey(
                &NSNumber::new_f64(kern),
                ProtocolObject::from_ref(NSKernAttributeName),
            );
        }
    }
    attrs
}

fn measure(text: &str, attrs: &NSMutableDictionary<NSString, AnyObject>) -> NSSize {
    unsafe { NSString::from_str(text).sizeWithAttributes(Some(attrs)) }
}

fn draw(text: &str, x: f64, y: f64, attrs: &NSMutableDictionary<NSString, AnyObject>) {
    unsafe {
        NSString::from_str(text).drawAtPoint_withAttributes(NSPoint { x, y }, Some(attrs));
    }
}

/// Shorten `text` with an ellipsis until it fits `width`.
fn truncate(text: &str, attrs: &NSMutableDictionary<NSString, AnyObject>, width: f64) -> String {
    if measure(text, attrs).width <= width {
        return text.to_string();
    }
    let mut chars: Vec<char> = text.trim_end().chars().collect();
    while !chars.is_empty() {
        chars.pop();
        let candidate: String = chars.iter().collect::<String>().trim_end().to_string() + "…";
        if measure(&candidate, attrs).width <= width {
            return candidate;
        }
    }
    "…".to_string()
}

fn pill(rect: NSRect, radius: f64) {
    NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(rect, radius, radius).fill();
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
    NSRect {
        origin: NSPoint { x, y },
        size: NSSize { width, height },
    }
}

// ---- lookups -----------------------------------------------------------------

/// Where a native row's value should end, measured from its text origin,
/// which sits where the views' content starts: the created width less the
/// inset, which AppKit then follows with its submenu-chevron column.
pub fn content_right_edge() -> f64 {
    let m = Metrics::new(0.0);
    m.width - m.inset * 2.0
}

/// The icon of the app a process belongs to: the `.app` bundle enclosing its
/// executable, or the executable itself for anything that is not bundled.
pub fn icon_for(path: &str) -> Option<Retained<NSImage>> {
    let bundle = path
        .match_indices(".app/")
        .next()
        .map(|(at, _)| &path[..at + 4]);
    let target = bundle.unwrap_or(path);
    let workspace = NSWorkspace::sharedWorkspace();
    Some(workspace.iconForFile(&NSString::from_str(target)))
}

/// The Mac's marketing name and chip, "MacBook Pro M4", from the device tree's
/// `product` node (`product-name` also carries the size and year in
/// parentheses, which are dropped) and the CPU brand string. Either part may be missing; what is known is shown.
pub fn machine_name() -> String {
    let product = product_name().map(|name| match name.split_once(" (") {
        Some((short, _)) => short.trim().to_string(),
        None => name,
    });
    let chip = chip_name().map(|chip| chip.trim_start_matches("Apple ").to_string());
    match (product, chip) {
        (Some(p), Some(c)) => format!("{p} {c}"),
        (Some(p), None) => p,
        (None, Some(c)) => c,
        (None, None) => "Mac".to_string(),
    }
}

fn product_name() -> Option<String> {
    let mut path = [0 as c_char; 512];
    for (slot, byte) in path.iter_mut().zip(b"IODeviceTree:/product") {
        *slot = *byte as c_char;
    }
    // SAFETY: `path` is a NUL-terminated io_string_t; the entry is released
    // below.
    let entry = unsafe { IORegistryEntryFromPath(objc2_io_kit::kIOMainPortDefault, &mut path) };
    if entry == 0 {
        return None;
    }
    let key = CFString::from_str("product-name");
    // SAFETY: `entry` is a live registry entry until the release below.
    let value = unsafe { IORegistryEntryCreateCFProperty(entry, Some(&key), None, 0) };
    IOObjectRelease(entry);
    let data = value?.downcast::<CFData>().ok()?;
    let bytes = data.to_vec();
    let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
    String::from_utf8(bytes[..end].to_vec()).ok()
}

fn chip_name() -> Option<String> {
    let name = std::ffi::CString::new("machdep.cpu.brand_string").ok()?;
    let mut size = 0usize;
    // SAFETY: a null buffer asks for the size; the second call fills a buffer
    // of exactly that size.
    unsafe {
        if sysctlbyname(
            name.as_ptr(),
            std::ptr::null_mut(),
            &mut size,
            std::ptr::null_mut(),
            0,
        ) != 0
            || size == 0
        {
            return None;
        }
        let mut buffer = vec![0u8; size];
        if sysctlbyname(
            name.as_ptr(),
            buffer.as_mut_ptr().cast(),
            &mut size,
            std::ptr::null_mut(),
            0,
        ) != 0
        {
            return None;
        }
        buffer.truncate(size.saturating_sub(1));
        String::from_utf8(buffer).ok()
    }
}

unsafe extern "C" {
    fn sysctlbyname(
        name: *const c_char,
        oldp: *mut std::ffi::c_void,
        oldlenp: *mut usize,
        newp: *mut std::ffi::c_void,
        newlen: usize,
    ) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every Mac has a product name in its device tree and a chip string.
    #[test]
    fn names_the_machine() {
        let product = product_name();
        let chip = chip_name();
        eprintln!("product={product:?} chip={chip:?} name={}", machine_name());
        assert!(
            product.is_some_and(|p| p.starts_with("Mac")),
            "no product name"
        );
        assert!(chip.is_some_and(|c| c.starts_with("Apple")), "no chip");
    }
}
