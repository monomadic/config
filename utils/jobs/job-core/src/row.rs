//! One job as a menu row, in the same idiom as `free-disk-space-widget`'s
//! volume rows: an outline SF Symbol on the left with a small caption under
//! it, the job name with a value small and right-aligned, and a progress bar
//! underneath. A running job gets one line more — the last thing it printed.
//!
//! A menu item can host a view, but then AppKit draws none of it, so the row
//! draws its own text with the menu font (`NSFont::menuFontOfSize(0.0)`) and
//! takes every measurement from that font, which keeps it sized with the
//! system text size. The row tracks the mouse to paint the native-looking
//! highlight, because AppKit only highlights items it draws itself.
//!
//! All rows in one menu share a [`Layout`], so names and values line up down
//! the menu instead of following each job's own width.

use std::cell::Cell;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

/// How long a job must be quiet before silence is worth mentioning, when
/// there is no live process to ask instead. Deliberately longer than any
/// reasonable progress-reporting interval.
const SILENT_WARN: Duration = Duration::from_secs(45 * 60);

use crate::observe::{Root, Run, Snapshot, State};

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{
    NSBezierPath, NSColor, NSCompositingOperation, NSEvent, NSFont, NSFontAttributeName,
    NSGraphicsContext,
    NSFontWeightRegular, NSForegroundColorAttributeName, NSImage, NSImageSymbolConfiguration,
    NSStringDrawing, NSTrackingArea, NSTrackingAreaOptions, NSView,
};
use objc2_foundation::{NSMutableDictionary, NSPoint, NSRect, NSSize, NSString};

/// What the row is showing, which decides its symbol and its colour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Running,
    Paused,
    Queued,
    Done,
    Failed,
}

impl Kind {
    fn symbol(self) -> &'static str {
        match self {
            Kind::Running => "play",
            Kind::Paused => "pause",
            Kind::Queued => "clock",
            Kind::Done => "checkmark",
            Kind::Failed => "xmark",
        }
    }

    fn tint(self) -> Retained<NSColor> {
        match self {
            Kind::Done => NSColor::systemGreenColor(),
            Kind::Failed => NSColor::systemRedColor(),
            _ => NSColor::labelColor().colorWithAlphaComponent(0.85),
        }
    }
}

/// How the bar under the name is drawn.
#[derive(Clone, Copy, PartialEq)]
pub enum Progress {
    /// A real fraction, parsed out of what the job printed.
    Fraction(f64),
    /// Something is happening but the job doesn't say how far along: drawn as
    /// diagonal stripes, so it reads as motion without claiming a position.
    Unknown,
    /// The empty track, for a job that hasn't started. Keeps a queued row the
    /// same shape as a running one rather than leaving a gap where the bar
    /// would be.
    Track,
    /// No bar at all.
    None,
}

/// A button on a row. The destination is worked out when the row is built, so
/// pressing it is one `rename` — the row needs no callback into the app, and
/// both GUIs get identical behaviour because there is no behaviour to differ.
#[derive(Clone)]
pub struct Action {
    pub glyph: Glyph,
    /// Where the job's folder goes. Moving it *is* the command; the runner is
    /// watching its own folder and does the signalling.
    pub to: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    Pause,
    Resume,
    Stop,
}

/// Everything one row draws. Built by the apps from a `Snapshot`.
#[derive(Clone)]
pub struct RowSpec {
    pub kind: Kind,
    /// Small text under the icon: elapsed, queue position, or how long a
    /// finished job took.
    pub caption: String,
    pub name: String,
    /// Right-aligned: a percentage, `queued`, `24h ago`.
    pub value: String,
    /// Draws the value in red — a failure, or a job that has gone quiet.
    pub alert: bool,
    pub progress: Progress,
    /// The last line the job printed, under the bar.
    pub log: Option<String>,
    /// Opened in Finder when the row is clicked.
    pub path: Option<PathBuf>,
    /// Reveal the path in its parent folder rather than opening it.
    pub reveal: bool,
    /// The job's own folder, and the buttons that move it.
    pub dir: Option<PathBuf>,
    pub actions: Vec<Action>,
}

impl RowSpec {
    pub fn new(kind: Kind, name: impl Into<String>) -> Self {
        Self {
            kind,
            caption: String::new(),
            name: name.into(),
            value: String::new(),
            alert: false,
            progress: Progress::None,
            log: None,
            path: None,
            reveal: false,
            dir: None,
            actions: Vec::new(),
        }
    }

    /// Attach the controls for a job whose folder is at `dir`.
    pub fn actions(mut self, dir: PathBuf, actions: Vec<Action>) -> Self {
        self.dir = Some(dir);
        self.actions = actions;
        self
    }

    pub fn caption(mut self, caption: impl Into<String>) -> Self {
        self.caption = caption.into();
        self
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn alert(mut self, alert: bool) -> Self {
        self.alert = alert;
        self
    }

    pub fn progress(mut self, progress: Progress) -> Self {
        self.progress = progress;
        self
    }

    pub fn log(mut self, log: Option<String>) -> Self {
        self.log = log;
        self
    }

    pub fn reveal(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self.reveal = true;
        self
    }

    pub fn open(mut self, path: PathBuf) -> Self {
        self.path = Some(path);
        self.reveal = false;
        self
    }

    fn has_bar(&self) -> bool {
        self.progress != Progress::None
    }
}

/// A labelled group of rows. The label is a plain menu item — headers and
/// actions stay ordinary menu items at ordinary size; only the job rows are
/// custom views sharing one height.
pub struct Section {
    pub label: Option<String>,
    pub rows: Vec<RowSpec>,
}

/// A snapshot as menu sections: what is running, then the queue, then the last
/// few outcomes. Shared so a local queue and a remote one can't drift into
/// being shown two different ways.
pub fn sections(snapshot: &Snapshot, max_queued: usize, max_recent: usize) -> Vec<Section> {
    let mut sections = Vec::new();
    let root = snapshot.root.clone();

    // Running and paused first — the jobs with a process behind them.
    let mut rows: Vec<RowSpec> = snapshot
        .jobs
        .iter()
        .filter(|job| matches!(job.state, State::Running | State::Paused))
        .map(|job| active_row(job, &root))
        .collect();
    if !rows.is_empty() {
        sections.push(Section {
            label: None,
            rows: std::mem::take(&mut rows),
        });
    }

    // The queue: staged folders, then anything still sitting in the inbox.
    // Inbox entries only pile up when nothing is watching the folder, so they
    // are worth showing rather than hiding as an implementation detail.
    let ready: Vec<&Run> = snapshot.in_state(State::Ready).collect();
    let queued = ready.len() + snapshot.inbox.len();
    for (index, job) in ready.iter().take(max_queued).enumerate() {
        rows.push(
            RowSpec::new(Kind::Queued, job.name.clone())
                .caption(format!("{}", index + 1))
                .value(if index == 0 { "next".to_string() } else { String::new() })
                .progress(Progress::Track)
                .reveal(job.dir.clone())
                // A queued job can be held back before it ever starts.
                .actions(job.dir.clone(), holdable(job, &root)),
        );
    }
    for name in snapshot.inbox.iter().take(max_queued.saturating_sub(rows.len())) {
        rows.push(
            RowSpec::new(Kind::Queued, name.clone())
                .value("not picked up".to_string())
                .progress(Progress::Track),
        );
    }
    if queued > rows.len() {
        rows.push(
            RowSpec::new(Kind::Queued, format!("… {} more queued", queued - rows.len()))
                .progress(Progress::Track),
        );
    }
    if !rows.is_empty() {
        sections.push(Section {
            label: Some(format!("Queued: {queued}")),
            rows: std::mem::take(&mut rows),
        });
    }

    for outcome in snapshot.recent.iter().take(max_recent) {
        let kind = if outcome.ok { Kind::Done } else { Kind::Failed };
        rows.push(
            RowSpec::new(kind, outcome.name.clone())
                .caption(outcome.took().map(short_duration).unwrap_or_default())
                .value(ago_phrase(outcome.ago()))
                .alert(!outcome.ok)
                // A finished job's bar is full: it ran to its end, well or
                // badly, and the red fill says which.
                .progress(Progress::Fraction(1.0))
                .reveal(outcome.dir.clone()),
        );
    }
    if !rows.is_empty() {
        sections.push(Section {
            label: Some("Recent".to_string()),
            rows,
        });
    }

    sections
}

/// The buttons a not-yet-started job gets: hold it, or drop it out of the
/// queue entirely.
fn holdable(job: &Run, root: &Option<Root>) -> Vec<Action> {
    let Some(root) = root else { return Vec::new() };
    let Some(folder) = job.dir.file_name() else {
        return Vec::new();
    };
    vec![Action {
        glyph: Glyph::Pause,
        to: root.paused().join(folder),
    }]
}

/// A job with a process behind it — running, or suspended mid-flight.
fn active_row(job: &Run, root: &Option<Root>) -> RowSpec {
    let elapsed = job.elapsed();
    let paused = job.state == State::Paused;
    let stalled = job.is_stalled();
    let silent = job.silent_for();
    // Silence only means something when nothing else says the job is fine.
    //
    // A well-behaved encoder prints on its own schedule — topaz-encode logs
    // every 5%, which at 0.05x is one line every half hour — so five minutes
    // of quiet is not evidence of anything. Where the job is on this machine
    // its process settles the question outright, and where it isn't, the bar
    // has to clear the slowest sane reporting interval.
    let quiet = !paused
        && job.alive() != Some(true)
        && silent.is_some_and(|since| since > SILENT_WARN);

    // The value answers "how is it going": paused says so, a stalled job says
    // so, then a percentage if the job reports one, else the clock.
    let value = if paused {
        "paused".to_string()
    } else if stalled {
        "not running".to_string()
    } else if quiet {
        format!("no output {}", ago(silent.unwrap_or_default()))
    } else if let Some(progress) = job.progress {
        format!("{}%", (progress * 100.0).round() as i64)
    } else {
        elapsed.map(duration).unwrap_or_default()
    };

    RowSpec::new(if paused { Kind::Paused } else { Kind::Running }, job.name.clone())
        .caption(elapsed.map(short_duration).unwrap_or_default())
        .value(value)
        .alert(quiet || stalled)
        .progress(match (paused || stalled, job.progress) {
            // A stopped job's bar shouldn't animate: freeze it where it got to,
            // or show a flat track when there is no figure to freeze.
            (true, Some(fraction)) => Progress::Fraction(fraction),
            (true, None) => Progress::Track,
            (false, Some(fraction)) => Progress::Fraction(fraction),
            (false, None) => Progress::Unknown,
        })
        .log(job.last_line.clone())
        .open(job.dir.clone())
        .actions(job.dir.clone(), controls(job, root))
}

/// Pause/resume and stop, as the folder moves they are.
fn controls(job: &Run, root: &Option<Root>) -> Vec<Action> {
    let Some(root) = root else { return Vec::new() };
    let Some(folder) = job.dir.file_name() else {
        return Vec::new();
    };
    let paused = job.state == State::Paused;
    vec![
        Action {
            glyph: if paused { Glyph::Resume } else { Glyph::Pause },
            to: if paused {
                root.running().join(folder)
            } else {
                root.paused().join(folder)
            },
        },
        // Stopping files the job where a stopped job belongs, which is also
        // the signal for the runner to terminate it.
        Action {
            glyph: Glyph::Stop,
            to: root.failed().join(folder),
        },
    ]
}

/// `4:07` under an hour, `1:04:07` beyond it.
pub fn duration(duration: Duration) -> String {
    let total = duration.as_secs();
    let (hours, minutes, seconds) = (total / 3600, (total % 3600) / 60, total % 60);
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

/// The caption under the icon, where there is only room for two figures:
/// `4m`, `1:38`, `26h`.
pub fn short_duration(duration: Duration) -> String {
    let total = duration.as_secs();
    match total {
        0..=3599 => format!("{}m", total / 60),
        3600..=86399 => format!("{}:{:02}", total / 3600, (total % 3600) / 60),
        _ => format!("{}h", total / 3600),
    }
}

/// The same, as something that can end a sentence: `just now`, `12m ago`.
pub fn ago_phrase(duration: Duration) -> String {
    match duration.as_secs() {
        0..=59 => "just now".to_string(),
        _ => format!("{} ago", ago(duration)),
    }
}

/// Coarse relative time: `just now`, `12m`, `3h`.
pub fn ago(duration: Duration) -> String {
    let total = duration.as_secs();
    match total {
        0..=59 => "just now".to_string(),
        60..=3599 => format!("{}m", total / 60),
        _ => format!("{}h", total / 3600),
    }
}

/// Shared column geometry for one menu's worth of rows.
pub struct Layout {
    font: Retained<NSFont>,
    detail_font: Retained<NSFont>,
    caption_font: Retained<NSFont>,
    log_font: Retained<NSFont>,
    width: f64,
    icon_x: f64,
    icon_scrim: f64,
    icon_size: f64,
    text_left: f64,
    text_right: f64,
    bar_height: f64,
    line_gap: f64,
    height: f64,
    /// Centre of the leftmost button; buttons march rightwards from here.
    button_x: f64,
    button_diameter: f64,
    button_gap: f64,
}

/// Sized from the widest name and value in the set, and clamped so a menu of
/// long encode filenames doesn't stretch off the screen — names ellipsise
/// instead.
pub fn layout<'a>(specs: impl IntoIterator<Item = &'a RowSpec> + Clone) -> Layout {
    let font = NSFont::menuFontOfSize(0.0);
    let em = font.pointSize();
    let detail_font =
        NSFont::monospacedDigitSystemFontOfSize_weight((em * 0.82).round(), unsafe {
            NSFontWeightRegular
        });
    let caption_font =
        NSFont::monospacedDigitSystemFontOfSize_weight((em * 0.66).round(), unsafe {
            NSFontWeightRegular
        });
    let log_font = NSFont::monospacedSystemFontOfSize_weight((em * 0.74).round(), unsafe {
        NSFontWeightRegular
    });

    let left = (em * 1.1).round();
    let right = (em * 1.0).round();
    let gap = (em * 0.9).round();
    let icon_size = (em * 1.05).round();
    let bar_height = (em * 0.28).round().max(3.0);
    let line_gap = (em * 0.42).round();
    let button_diameter = (em * 1.45).round();
    let button_gap = (em * 0.35).round();

    let widest = |measure: &dyn Fn(&RowSpec) -> f64| {
        specs.clone().into_iter().map(measure).fold(0.0, f64::max)
    };
    let name_width = widest(&|spec: &RowSpec| text_size(&font, &spec.name).width);
    let value_width = widest(&|spec: &RowSpec| text_size(&detail_font, &spec.value).width);
    let caption_width = widest(&|spec: &RowSpec| text_size(&caption_font, &spec.caption).width);

    let icon_scrim = (em * 1.9).max(caption_width).round();
    let text_left = left + icon_scrim + gap;
    // Wide enough to be worth reading, narrow enough to stay a menu.
    let text_width = (name_width + gap * 2.0 + value_width).clamp(em * 16.0, em * 34.0);
    // The button column is sized by the busiest row, so the controls line up
    // down the menu instead of following each row's own count.
    let most_buttons = specs
        .clone()
        .into_iter()
        .map(|spec| spec.actions.len())
        .max()
        .unwrap_or(0);
    let button_column = if most_buttons > 0 {
        gap + most_buttons as f64 * button_diameter + (most_buttons as f64 - 1.0) * button_gap
    } else {
        0.0
    };
    let width = text_left + text_width + button_column + right;

    let mut layout = Layout {
        font,
        detail_font,
        caption_font,
        log_font,
        width,
        icon_x: left,
        icon_scrim,
        icon_size,
        text_left,
        text_right: text_left + text_width,
        bar_height,
        line_gap,
        height: 0.0,
        button_x: text_left + text_width + gap + button_diameter / 2.0,
        button_diameter,
        button_gap,
    };
    let tallest = specs
        .into_iter()
        .map(|spec| content_height(&layout, spec))
        .fold(0.0, f64::max);
    layout.height = (tallest + em * 0.75).round();
    layout
}

impl Layout {
    /// One height for every row in the menu — the tallest row's — so the list
    /// reads as a list. Shorter rows centre their content in it rather than
    /// each row sizing to its own contents and the column going ragged.
    pub fn height(&self) -> f64 {
        self.height
    }

    pub fn width(&self) -> f64 {
        self.width
    }
}

/// The height one row's contents need: the name, plus a bar and a log line if
/// it has them.
fn content_height(layout: &Layout, spec: &RowSpec) -> f64 {
    let mut height = text_size(&layout.font, &spec.name).height;
    if spec.has_bar() {
        height += layout.line_gap + layout.bar_height;
    }
    if spec.log.is_some() {
        height += layout.line_gap + text_size(&layout.log_font, "Xg").height;
    }
    height
}

pub struct RowIvars {
    spec: RowSpec,
    font: Retained<NSFont>,
    detail_font: Retained<NSFont>,
    caption_font: Retained<NSFont>,
    log_font: Retained<NSFont>,
    icon_x: f64,
    icon_scrim: f64,
    icon_size: f64,
    text_left: f64,
    text_right: f64,
    bar_height: f64,
    line_gap: f64,
    button_x: f64,
    button_diameter: f64,
    button_gap: f64,
    hovered: Cell<bool>,
    /// Which button the pointer is over, so it can brighten under it.
    hot_button: Cell<Option<usize>>,
}

define_class!(
    // SAFETY: NSView imposes no subclassing requirements beyond initialising
    // through the superclass, and JobRow does not implement Drop.
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[name = "JobCoreJobRow"]
    #[ivars = RowIvars]
    pub struct JobRow;

    impl JobRow {
        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, _dirty: NSRect) {
            let ivars = self.ivars();
            let bounds = self.bounds();

            if ivars.hovered.get() && ivars.spec.path.is_some() {
                draw_highlight(bounds);
            }

            self.draw_icon(bounds);

            // The block of text is centred on its visible marks rather than
            // its boxes: the leading above the caps otherwise reads as extra
            // padding and drags everything low.
            let name_size = text_size(&ivars.font, &ivars.spec.name);
            let log_height = ivars
                .spec
                .log
                .as_ref()
                .map(|_| ivars.line_gap + text_size(&ivars.log_font, "Xg").height)
                .unwrap_or(0.0);
            let bar_block = if ivars.spec.has_bar() {
                ivars.line_gap + ivars.bar_height
            } else {
                0.0
            };
            let content = name_size.height + bar_block + log_height;
            let lift = (ivars.font.pointSize() * 0.12).round();
            let name_y = bounds.size.height - ((bounds.size.height - content) / 2.0).round()
                - name_size.height
                + lift;

            // Name and value share a baseline; the name is truncated to
            // whatever the value leaves it.
            let value_size = text_size(&ivars.detail_font, &ivars.spec.value);
            let value_gap = if ivars.spec.value.is_empty() {
                0.0
            } else {
                ivars.font.pointSize()
            };
            let name_room = ivars.text_right - ivars.text_left - value_size.width - value_gap;
            let name = truncate(&ivars.font, &ivars.spec.name, name_room);
            draw_text(
                &name,
                &ivars.font,
                &NSColor::labelColor(),
                NSPoint {
                    x: ivars.text_left,
                    y: name_y,
                },
            );

            if !ivars.spec.value.is_empty() {
                let color = if ivars.spec.alert {
                    NSColor::systemRedColor()
                } else {
                    NSColor::secondaryLabelColor()
                };
                draw_text(
                    &ivars.spec.value,
                    &ivars.detail_font,
                    &color,
                    NSPoint {
                        x: ivars.text_right - value_size.width,
                        // Align to the name's baseline, not its box.
                        y: name_y + ivars.font.descender() - ivars.detail_font.descender(),
                    },
                );
            }

            let mut y = name_y;
            if ivars.spec.has_bar() {
                y -= ivars.line_gap + ivars.bar_height;
                self.draw_bar(y);
            }

            self.draw_buttons(bounds);

            if let Some(log) = ivars.spec.log.as_ref() {
                let log_size = text_size(&ivars.log_font, "Xg");
                y -= ivars.line_gap + log_size.height;
                let text = truncate(
                    &ivars.log_font,
                    log,
                    ivars.text_right - ivars.text_left,
                );
                draw_text(
                    &text,
                    &ivars.log_font,
                    &NSColor::tertiaryLabelColor(),
                    NSPoint {
                        x: ivars.text_left,
                        y,
                    },
                );
            }
        }

        #[unsafe(method(mouseEntered:))]
        fn mouse_entered(&self, event: &NSEvent) {
            self.ivars().hovered.set(true);
            self.track_pointer(event);
        }

        #[unsafe(method(mouseMoved:))]
        fn mouse_moved(&self, event: &NSEvent) {
            self.track_pointer(event);
        }

        #[unsafe(method(mouseExited:))]
        fn mouse_exited(&self, _event: &NSEvent) {
            self.ivars().hovered.set(false);
            self.ivars().hot_button.set(None);
            self.setNeedsDisplay(true);
        }

        // Two kinds of target, as in Finder's sidebar: the buttons command the
        // job, the rest of the row opens its folder.
        #[unsafe(method(mouseUp:))]
        fn mouse_up(&self, event: &NSEvent) {
            let ivars = self.ivars();
            if let Some(index) = self.button_at(event) {
                let Some(action) = ivars.spec.actions.get(index) else {
                    return;
                };
                let Some(dir) = ivars.spec.dir.as_ref() else {
                    return;
                };
                self.dismiss_menu();
                // The command *is* the move. Whoever is watching the folder —
                // the runner, here or on another machine — does the rest.
                if let Err(err) = std::fs::rename(dir, &action.to) {
                    eprintln!("job: could not move {} — {err}", dir.display());
                }
                return;
            }

            let Some(path) = ivars.spec.path.clone() else {
                return;
            };
            self.dismiss_menu();
            let mut command = Command::new("open");
            if ivars.spec.reveal {
                command.arg("-R");
            }
            let _ = command.arg(path).spawn();
        }
    }
);

impl JobRow {
    pub fn new(spec: RowSpec, layout: &Layout, mtm: MainThreadMarker) -> Retained<Self> {
        let height = layout.height();
        let this = Self::alloc(mtm).set_ivars(RowIvars {
            spec,
            font: layout.font.clone(),
            detail_font: layout.detail_font.clone(),
            caption_font: layout.caption_font.clone(),
            log_font: layout.log_font.clone(),
            icon_x: layout.icon_x,
            icon_scrim: layout.icon_scrim,
            icon_size: layout.icon_size,
            text_left: layout.text_left,
            text_right: layout.text_right,
            bar_height: layout.bar_height,
            line_gap: layout.line_gap,
            button_x: layout.button_x,
            button_diameter: layout.button_diameter,
            button_gap: layout.button_gap,
            hovered: Cell::new(false),
            hot_button: Cell::new(None),
        });
        let frame = NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            size: NSSize {
                width: layout.width,
                height,
            },
        };
        let this: Retained<Self> = unsafe { msg_send![super(this), initWithFrame: frame] };

        // InVisibleRect keeps the area matched to the bounds for us, and
        // ActiveAlways is required because a menu never makes us key.
        let tracking = unsafe {
            NSTrackingArea::initWithRect_options_owner_userInfo(
                NSTrackingArea::alloc(),
                NSRect::ZERO,
                NSTrackingAreaOptions::MouseEnteredAndExited
                    | NSTrackingAreaOptions::MouseMoved
                    | NSTrackingAreaOptions::ActiveAlways
                    | NSTrackingAreaOptions::InVisibleRect,
                Some(this.as_ref()),
                None,
            )
        };
        this.addTrackingArea(&tracking);
        this
    }

    /// The state symbol with its caption underneath, like the label on a
    /// drive in the volume rows this borrows from.
    fn draw_icon(&self, bounds: NSRect) {
        let ivars = self.ivars();
        let caption_size = text_size(&ivars.caption_font, &ivars.spec.caption);
        let lift = if ivars.spec.caption.is_empty() {
            0.0
        } else {
            (caption_size.height * 0.45).round()
        };
        let center_y = bounds.size.height / 2.0 + lift;

        if !ivars.spec.caption.is_empty() {
            draw_text(
                &ivars.spec.caption,
                &ivars.caption_font,
                &NSColor::secondaryLabelColor(),
                NSPoint {
                    x: (ivars.icon_x + (ivars.icon_scrim - caption_size.width) / 2.0).round(),
                    y: (center_y - ivars.icon_size / 2.0 - caption_size.height).round(),
                },
            );
        }

        let name = NSString::from_str(ivars.spec.kind.symbol());
        let Some(symbol) = NSImage::imageWithSystemSymbolName_accessibilityDescription(&name, None)
        else {
            return;
        };
        // Hierarchical colour bakes the tint in, so the template image can be
        // drawn directly; rows are rebuilt each time the menu opens, which
        // re-resolves it for the current appearance.
        let config =
            NSImageSymbolConfiguration::configurationWithHierarchicalColor(&ivars.spec.kind.tint());
        let Some(icon) = symbol.imageWithSymbolConfiguration(&config) else {
            return;
        };

        let size = icon.size();
        let scale = if size.width > 0.0 && size.height > 0.0 {
            (ivars.icon_size / size.width).min(ivars.icon_size / size.height)
        } else {
            1.0
        };
        let width = size.width * scale;
        let height = size.height * scale;
        icon.drawInRect_fromRect_operation_fraction(
            rect(
                ivars.icon_x + (ivars.icon_scrim - width) / 2.0,
                (center_y - height / 2.0).round(),
                width,
                height,
            ),
            NSRect::ZERO,
            NSCompositingOperation::SourceOver,
            1.0,
        );
    }

    fn draw_bar(&self, y: f64) {
        let ivars = self.ivars();
        let width = ivars.text_right - ivars.text_left;
        let radius = ivars.bar_height / 2.0;
        let track = rect(ivars.text_left, y, width, ivars.bar_height);

        NSColor::labelColor().colorWithAlphaComponent(0.16).set();
        NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(track, radius, radius).fill();

        match ivars.spec.progress {
            Progress::Fraction(fraction) => {
                let filled = (width * fraction.clamp(0.0, 1.0)).max(ivars.bar_height);
                if ivars.spec.alert {
                    NSColor::systemRedColor().colorWithAlphaComponent(0.75).set();
                } else {
                    NSColor::labelColor().colorWithAlphaComponent(0.75).set();
                }
                NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                    rect(ivars.text_left, y, filled, ivars.bar_height),
                    radius,
                    radius,
                )
                .fill();
            }
            // Diagonal stripes: motion without a claim about how far along.
            Progress::Unknown => {
                // Save/restore rather than resetting the clip by hand: the
                // menu may have set one, and it is not ours to throw away.
                NSGraphicsContext::saveGraphicsState_class();
                NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(track, radius, radius)
                    .addClip();
                NSColor::labelColor().colorWithAlphaComponent(0.34).set();
                let pitch = ivars.bar_height * 2.4;
                let lean = ivars.bar_height;
                let mut x = ivars.text_left - lean;
                while x < ivars.text_right + lean {
                    let stripe = NSBezierPath::new();
                    stripe.moveToPoint(NSPoint { x, y });
                    stripe.lineToPoint(NSPoint {
                        x: x + pitch / 2.0,
                        y,
                    });
                    stripe.lineToPoint(NSPoint {
                        x: x + pitch / 2.0 + lean,
                        y: y + ivars.bar_height,
                    });
                    stripe.lineToPoint(NSPoint {
                        x: x + lean,
                        y: y + ivars.bar_height,
                    });
                    stripe.closePath();
                    stripe.fill();
                    x += pitch;
                }
                NSGraphicsContext::restoreGraphicsState_class();
            }
            // The track alone is the whole drawing for these two.
            Progress::Track | Progress::None => {}
        }
    }

    /// The centre of button `index`, laid out left to right.
    fn button_center(&self, index: usize, bounds: NSRect) -> NSPoint {
        let ivars = self.ivars();
        NSPoint {
            x: ivars.button_x + index as f64 * (ivars.button_diameter + ivars.button_gap),
            y: bounds.size.height / 2.0,
        }
    }

    fn button_at(&self, event: &NSEvent) -> Option<usize> {
        let ivars = self.ivars();
        if ivars.spec.actions.is_empty() {
            return None;
        }
        let point = self.convertPoint_fromView(event.locationInWindow(), None);
        let bounds = self.bounds();
        // A little forgiveness beyond the drawn circle.
        let reach = ivars.button_diameter / 2.0 + 3.0;
        (0..ivars.spec.actions.len()).find(|index| {
            let center = self.button_center(*index, bounds);
            let (dx, dy) = (point.x - center.x, point.y - center.y);
            dx * dx + dy * dy <= reach * reach
        })
    }

    fn track_pointer(&self, event: &NSEvent) {
        let hot = self.button_at(event);
        if hot != self.ivars().hot_button.get() {
            self.ivars().hot_button.set(hot);
        }
        self.setNeedsDisplay(true);
    }

    /// Translucent circles that brighten under the pointer, with the glyph
    /// drawn inside — the same affordance the volume rows use for eject.
    fn draw_buttons(&self, bounds: NSRect) {
        let ivars = self.ivars();
        let hot = ivars.hot_button.get();
        let diameter = ivars.button_diameter;

        for (index, action) in ivars.spec.actions.iter().enumerate() {
            let hovered = hot == Some(index);
            let center = self.button_center(index, bounds);

            NSColor::labelColor()
                .colorWithAlphaComponent(if hovered { 0.26 } else { 0.13 })
                .set();
            NSBezierPath::bezierPathWithOvalInRect(rect(
                center.x - diameter / 2.0,
                center.y - diameter / 2.0,
                diameter,
                diameter,
            ))
            .fill();

            let ink = match (action.glyph, hovered) {
                // Stopping is the destructive one, so it says so under the
                // pointer rather than looking like everything else.
                (Glyph::Stop, true) => NSColor::systemRedColor(),
                (_, true) => NSColor::labelColor(),
                _ => NSColor::secondaryLabelColor(),
            };
            ink.set();

            match action.glyph {
                Glyph::Pause => {
                    let bar = diameter * 0.13;
                    let tall = diameter * 0.42;
                    for side in [-1.0, 1.0] {
                        NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                            rect(
                                center.x + side * bar * 1.6 - bar / 2.0,
                                center.y - tall / 2.0,
                                bar,
                                tall,
                            ),
                            bar / 2.0,
                            bar / 2.0,
                        )
                        .fill();
                    }
                }
                Glyph::Resume => {
                    let size = diameter * 0.4;
                    let triangle = NSBezierPath::new();
                    // Nudged right so the triangle looks centred, which it
                    // isn't when its bounding box is.
                    let left = center.x - size * 0.35 + size * 0.08;
                    triangle.moveToPoint(NSPoint { x: left, y: center.y - size / 2.0 });
                    triangle.lineToPoint(NSPoint { x: left, y: center.y + size / 2.0 });
                    triangle.lineToPoint(NSPoint { x: left + size * 0.8, y: center.y });
                    triangle.closePath();
                    triangle.fill();
                }
                Glyph::Stop => {
                    let size = diameter * 0.34;
                    NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                        rect(center.x - size / 2.0, center.y - size / 2.0, size, size),
                        size * 0.18,
                        size * 0.18,
                    )
                    .fill();
                }
            }
        }
    }

    fn dismiss_menu(&self) {
        if let Some(menu) = self
            .enclosingMenuItem()
            .and_then(|item| unsafe { item.menu() })
        {
            menu.cancelTracking();
        }
    }
}

/// The rounded highlight a native menu item gets, drawn to the same insets.
fn draw_highlight(bounds: NSRect) {
    NSColor::labelColor().colorWithAlphaComponent(0.12).set();
    NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
        rect(5.0, 0.0, bounds.size.width - 10.0, bounds.size.height),
        5.0,
        5.0,
    )
    .fill();
}

/// Cut text to fit `room`, with an ellipsis. Encode job names are long and
/// front-loaded with the collection they came from, so the tail is what gets
/// dropped.
fn truncate(font: &NSFont, text: &str, room: f64) -> String {
    if room <= 0.0 || text_size(font, text).width <= room {
        return text.to_string();
    }
    let chars: Vec<char> = text.chars().collect();
    let mut keep = chars.len();
    while keep > 1 {
        keep -= 1;
        let candidate: String = chars[..keep].iter().collect::<String>() + "…";
        if text_size(font, &candidate).width <= room {
            return candidate;
        }
    }
    "…".to_string()
}

fn text_attributes(
    font: &NSFont,
    color: Option<&NSColor>,
) -> Retained<NSMutableDictionary<NSString, AnyObject>> {
    let attrs = NSMutableDictionary::<NSString, AnyObject>::new();
    unsafe {
        attrs.setObject_forKey(font, ProtocolObject::from_ref(NSFontAttributeName));
        if let Some(color) = color {
            attrs.setObject_forKey(
                color,
                ProtocolObject::from_ref(NSForegroundColorAttributeName),
            );
        }
    }
    attrs
}

fn text_size(font: &NSFont, text: &str) -> NSSize {
    let attrs = text_attributes(font, None);
    unsafe { NSString::from_str(text).sizeWithAttributes(Some(&attrs)) }
}

fn draw_text(text: &str, font: &NSFont, color: &NSColor, origin: NSPoint) {
    let attrs = text_attributes(font, Some(color));
    unsafe { NSString::from_str(text).drawAtPoint_withAttributes(origin, Some(&attrs)) };
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
    NSRect {
        origin: NSPoint { x, y },
        size: NSSize { width, height },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::observe::{Root, Run, Snapshot, State, Status};
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn job(name: &str, state: State) -> Run {
        Run {
            name: name.to_string(),
            dir: PathBuf::from(format!("/j/{}/2026-{name}", state.dir_name())),
            state,
            started: None,
            last_line: None,
            last_output: None,
            progress: None,
            status: None,
            local: true,
        }
    }

    /// Every button is a destination, and the destination is the command. If
    /// these point at the wrong folder the UI silently does the wrong thing to
    /// somebody's encode, so they are worth pinning down.
    #[test]
    fn buttons_are_the_folder_moves_they_claim_to_be() {
        let snapshot = Snapshot {
            root: Some(Root::new("/j")),
            connected: true,
            jobs: vec![
                job("running", State::Running),
                job("held", State::Paused),
                job("waiting", State::Ready),
            ],
            ..Snapshot::default()
        };
        let rows: Vec<RowSpec> = sections(&snapshot, 5, 5)
            .into_iter()
            .flat_map(|section| section.rows)
            .collect();

        let running = rows.iter().find(|row| row.name == "running").unwrap();
        assert_eq!(running.actions.len(), 2);
        assert_eq!(running.actions[0].glyph, Glyph::Pause);
        assert_eq!(running.actions[0].to, PathBuf::from("/j/_paused/2026-running"));
        assert_eq!(running.actions[1].glyph, Glyph::Stop);
        assert_eq!(running.actions[1].to, PathBuf::from("/j/_failed/2026-running"));

        // A paused job offers the way back, not another pause.
        let held = rows.iter().find(|row| row.name == "held").unwrap();
        assert_eq!(held.actions[0].glyph, Glyph::Resume);
        assert_eq!(held.actions[0].to, PathBuf::from("/j/_running/2026-held"));

        // A job that hasn't started can be held, but there is nothing to stop.
        let waiting = rows.iter().find(|row| row.name == "waiting").unwrap();
        assert_eq!(waiting.actions.len(), 1);
        assert_eq!(waiting.actions[0].to, PathBuf::from("/j/_paused/2026-waiting"));

        // Finished jobs get no controls at all.
        assert!(rows.iter().filter(|row| row.kind == Kind::Done).all(|row| row.actions.is_empty()));
    }

    /// A slow encoder that logs every 5% can go half an hour between lines.
    /// Calling that "no output" in red taught the menu to cry wolf about jobs
    /// that were working perfectly well.
    #[test]
    fn a_quiet_but_live_job_is_not_reported_as_silent() {
        let mut running = job("slow", State::Running);
        running.started = Some(SystemTime::now() - Duration::from_secs(3 * 3600));
        running.last_output = Some(SystemTime::now() - Duration::from_secs(31 * 60));
        running.last_line = Some("progress: 30% (9:46 / 32:32) at 0.0496x".to_string());

        // Our own process group, so the liveness check is definitively true.
        // (Our *pid* would not do: a test binary is rarely a group leader.)
        running.status = Some(Status { pgid: unsafe { libc::getpgrp() } });
        running.local = true;
        let snapshot = Snapshot {
            root: Some(Root::new("/j")),
            connected: true,
            jobs: vec![running.clone()],
            ..Snapshot::default()
        };
        let row = &sections(&snapshot, 5, 5)[0].rows[0];
        assert!(!row.alert, "a live job should not be flagged red");
        assert!(
            !row.value.contains("no output"),
            "half an hour between progress lines is normal, got {:?}",
            row.value
        );

        // With no process to ask — a folder watched over SMB — silence has to
        // clear a much higher bar before it counts.
        running.status = None;
        running.local = false;
        let remote = Snapshot {
            root: Some(Root::new("/j")),
            connected: true,
            jobs: vec![running],
            ..Snapshot::default()
        };
        let row = &sections(&remote, 5, 5)[0].rows[0];
        assert!(!row.value.contains("no output"), "got {:?}", row.value);
    }
}
