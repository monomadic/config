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

use std::cell::{Cell, RefCell};
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

/// What the row is showing, drawn as the symbol at the left of the row.
///
/// This is the row's answer to "what is this job doing", and it has to be
/// legible without reading anything: a paused job that says so only in small
/// grey text at the right-hand end is a paused job you will not notice you
/// paused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Running,
    Paused,
    /// In `_running` with nothing running it. Its own state rather than a
    /// flavour of running: the row is reporting a queue that needs a hand, not
    /// a job that is getting on with it.
    Stalled,
    Queued,
    Done,
    Failed,
}

impl Kind {
    /// The SF Symbol at the left of the row. Plain shapes, not the `.fill`
    /// circles the buttons use: this one is a label and mustn't read as
    /// something to press.
    fn symbol(self) -> &'static str {
        match self {
            Kind::Running => "play.fill",
            Kind::Paused => "pause.fill",
            Kind::Stalled => "exclamationmark.triangle.fill",
            Kind::Queued => "clock",
            Kind::Done => "checkmark",
            Kind::Failed => "xmark",
        }
    }

    fn tint(self) -> Retained<NSColor> {
        match self {
            Kind::Running => NSColor::labelColor().colorWithAlphaComponent(0.8),
            // A paused job is deliberately quieter than a running one — the
            // whole row dims — but the symbol still has to carry across the
            // menu, so it keeps full strength.
            Kind::Paused => NSColor::labelColor().colorWithAlphaComponent(0.8),
            Kind::Stalled | Kind::Failed => NSColor::systemRedColor(),
            Kind::Queued | Kind::Done => NSColor::tertiaryLabelColor(),
        }
    }

    /// Whether the row draws itself at reduced strength: a suspended job should
    /// look suspended at a glance down the menu.
    fn dimmed(self) -> bool {
        self == Kind::Paused
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

/// A button on a row. What it does is worked out when the row is built, so
/// pressing it is one `rename` (or one `open`) — the row needs no callback
/// into the app, and both GUIs get identical behaviour because there is no
/// behaviour to differ.
#[derive(Clone)]
pub struct Action {
    pub glyph: Glyph,
    pub act: Act,
    /// Where the job came from, for a button that toggles. Pressing pause moves
    /// the folder to `_paused`; pressing it again has to put the folder back
    /// where it was, and that is `_running` for a job that had started and
    /// `_ready` for one that never did — a distinction the row cannot recover
    /// from the destination alone, so it is recorded when the row is built.
    ///
    /// `None` for the buttons that only go one way: stop, and the log.
    pub back: Option<PathBuf>,
}

impl Action {
    fn oneway(glyph: Glyph, act: Act) -> Self {
        Self {
            glyph,
            act,
            back: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    /// Move the job's folder here. The move *is* the command; the runner is
    /// watching its own folder and does the signalling.
    Move(PathBuf),
    /// Open this path — a log, not a command.
    Open(PathBuf),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    Pause,
    Resume,
    Stop,
    /// Drawn as a small labelled pill rather than a symbol: opening a log is
    /// not one of the folder-move verbs and shouldn't look like one.
    Log,
}

impl Glyph {
    /// The SF Symbol the button draws, where it draws one. Pause and resume
    /// draw *different* glyphs: they shared `playpause` on the reasoning that
    /// the row's value already said which way it would go, which asked you to
    /// read the far end of the row to find out what the button under your
    /// pointer did. Stop is an x: in this system stopping *is* killing (the
    /// move to `_failed` SIGTERMs the process group), so there is only the one
    /// verb.
    fn symbol(self) -> Option<&'static str> {
        match self {
            Glyph::Pause => Some("pause.circle.fill"),
            Glyph::Resume => Some("play.circle.fill"),
            Glyph::Stop => Some("xmark.circle.fill"),
            Glyph::Log => None,
        }
    }

    /// What this button becomes once it has been pressed and the row is showing
    /// the move it hasn't seen confirmed yet.
    fn toggled(self) -> Self {
        match self {
            Glyph::Pause => Glyph::Resume,
            Glyph::Resume => Glyph::Pause,
            other => other,
        }
    }

    fn label(self) -> Option<&'static str> {
        match self {
            Glyph::Log => Some("log"),
            _ => None,
        }
    }
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

/// A group of rows — active, queued, or recent. Groups carry no header text;
/// the menu separates them with ordinary separators, and each row's own icon
/// and value say what it is.
pub struct Section {
    pub rows: Vec<RowSpec>,
}

/// A snapshot as menu sections: what is running, then the queue, then the last
/// few outcomes. Shared so a local queue and a remote one can't drift into
/// being shown two different ways.
pub fn sections(snapshot: &Snapshot, max_queued: usize, max_recent: usize) -> Vec<Section> {
    let mut sections = Vec::new();
    let root = snapshot.root.clone();

    // Running and paused first — the jobs with a process behind them — then
    // the queue behind them. They are one section: a job moves between these
    // states while you watch, and a rule dividing them would put a line
    // through the middle of the thing being looked at.
    let mut rows: Vec<RowSpec> = snapshot
        .jobs
        .iter()
        .filter(|job| matches!(job.state, State::Running | State::Paused))
        .map(|job| active_row(job, &root))
        .collect();

    // The queue: staged folders, then anything still sitting in the inbox.
    // Inbox entries only pile up when nothing is watching the folder, so they
    // are worth showing rather than hiding as an implementation detail.
    let ready: Vec<&Run> = snapshot.in_state(State::Ready).collect();
    let queued = ready.len() + snapshot.inbox.len();
    // Counted from where the queue starts, not from the top of the section:
    // the active jobs above share this list but are not part of the queue, and
    // must not eat into how much of it gets shown.
    let active = rows.len();
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
    for name in snapshot
        .inbox
        .iter()
        .take(max_queued.saturating_sub(rows.len() - active))
    {
        rows.push(
            RowSpec::new(Kind::Queued, name.clone())
                .value("not picked up".to_string())
                .progress(Progress::Track),
        );
    }
    let listed = rows.len() - active;
    if queued > listed {
        rows.push(
            RowSpec::new(Kind::Queued, format!("… {} more queued", queued - listed))
                .progress(Progress::Track),
        );
    }
    if !rows.is_empty() {
        sections.push(Section {
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
        sections.push(Section { rows });
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
        act: Act::Move(root.paused().join(folder)),
        // Back to the queue it was waiting in, not to `_running`: this job
        // never started, and putting it there would claim it had.
        back: Some(root.ready().join(folder)),
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

    let kind = match (paused, stalled) {
        (true, _) => Kind::Paused,
        (false, true) => Kind::Stalled,
        (false, false) => Kind::Running,
    };

    RowSpec::new(kind, job.name.clone())
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

/// Pause/resume and stop, as the folder moves they are, plus the job's log
/// where it has written one.
fn controls(job: &Run, root: &Option<Root>) -> Vec<Action> {
    let Some(root) = root else { return Vec::new() };
    let Some(folder) = job.dir.file_name() else {
        return Vec::new();
    };
    let paused = job.state == State::Paused;
    let mut actions = vec![
        Action {
            glyph: if paused { Glyph::Resume } else { Glyph::Pause },
            act: Act::Move(if paused {
                root.running().join(folder)
            } else {
                root.paused().join(folder)
            }),
            back: Some(if paused {
                root.paused().join(folder)
            } else {
                root.running().join(folder)
            }),
        },
        // Stopping files the job where a stopped job belongs, which is also
        // the signal for the runner to terminate it.
        Action::oneway(Glyph::Stop, Act::Move(root.failed().join(folder))),
    ];
    if let Some(log) = job.log_path() {
        actions.push(Action::oneway(Glyph::Log, Act::Open(log)));
    }
    actions
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
    log_font: Retained<NSFont>,
    button_font: Retained<NSFont>,
    caption_font: Retained<NSFont>,
    width: f64,
    /// The state symbol's column: left edge, and how wide it is — the symbol
    /// or the widest caption under one, whichever needs more.
    icon_x: f64,
    icon_width: f64,
    icon_size: f64,
    text_left: f64,
    text_right: f64,
    bar_height: f64,
    line_gap: f64,
    height: f64,
    /// Left edge of the button column; buttons march rightwards from here,
    /// each as wide as its own glyph needs.
    button_x: f64,
    button_diameter: f64,
    button_gap: f64,
    /// The labelled pill is wider than a symbol button.
    label_width: f64,
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
    let log_font = NSFont::monospacedSystemFontOfSize_weight((em * 0.74).round(), unsafe {
        NSFontWeightRegular
    });
    let button_font = NSFont::systemFontOfSize_weight((em * 0.72).round(), unsafe {
        NSFontWeightRegular
    });
    let caption_font =
        NSFont::monospacedDigitSystemFontOfSize_weight((em * 0.64).round(), unsafe {
            NSFontWeightRegular
        });

    // Matched to where AppKit indents an ordinary menu item's title, so the
    // job rows sit in the same column as Pause, Icon and the rest rather than
    // hanging left of them. A view-backed item gets the menu's full width and
    // none of that inset, so it has to be reproduced here.
    let left = (em * 1.75).round();
    let right = (em * 1.0).round();
    let gap = (em * 0.9).round();
    let bar_height = (em * 0.28).round().max(3.0);
    let line_gap = (em * 0.42).round();
    let button_diameter = (em * 1.45).round();
    let button_gap = (em * 0.35).round();
    let label_width = (text_size(&button_font, "log").width + em * 0.9).round();

    let widest = |measure: &dyn Fn(&RowSpec) -> f64| {
        specs.clone().into_iter().map(measure).fold(0.0, f64::max)
    };
    let name_width = widest(&|spec: &RowSpec| text_size(&font, &spec.name).width);
    let value_width = widest(&|spec: &RowSpec| text_size(&detail_font, &spec.value).width);
    let caption_width =
        widest(&|spec: &RowSpec| text_size(&caption_font, &spec.caption).width);

    // The symbol and its caption share a column, sized by whichever is wider.
    // It sits in the indent an ordinary menu item leaves empty, and only pushes
    // the names right of that when the captions need the room.
    let icon_size = (em * 0.95).round();
    let icon_x = (em * 0.5).round();
    let icon_width = icon_size.max(caption_width);
    let text_left = left.max(icon_x + icon_width + (em * 0.55).round());
    // Wide enough to be worth reading, narrow enough to stay a menu.
    let text_width = (name_width + gap * 2.0 + value_width).clamp(em * 16.0, em * 34.0);
    // The button column is sized by the busiest row, so the controls line up
    // down the menu instead of following each row's own count.
    let button_span = |spec: &RowSpec| {
        let mut span = 0.0;
        for (index, action) in spec.actions.iter().enumerate() {
            if index > 0 {
                span += button_gap;
            }
            span += if action.glyph.label().is_some() {
                label_width
            } else {
                button_diameter
            };
        }
        span
    };
    let widest_buttons = widest(&button_span);
    let button_column = if widest_buttons > 0.0 {
        gap + widest_buttons
    } else {
        0.0
    };
    let width = text_left + text_width + button_column + right;

    let mut layout = Layout {
        font,
        detail_font,
        log_font,
        button_font,
        caption_font,
        width,
        icon_x,
        icon_width,
        icon_size,
        text_left,
        text_right: text_left + text_width,
        bar_height,
        line_gap,
        height: 0.0,
        button_x: text_left + text_width + gap,
        button_diameter,
        button_gap,
        label_width,
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
/// it has them — or the symbol and its caption, on the rows where that stack is
/// the taller of the two.
fn content_height(layout: &Layout, spec: &RowSpec) -> f64 {
    let mut height = text_size(&layout.font, &spec.name).height;
    if spec.has_bar() {
        height += layout.line_gap + layout.bar_height;
    }
    if spec.log.is_some() {
        height += layout.line_gap + text_size(&layout.log_font, "Xg").height;
    }
    height.max(icon_stack_height(layout, spec))
}

/// The symbol, and the caption under it where there is one.
fn icon_stack_height(layout: &Layout, spec: &RowSpec) -> f64 {
    if spec.caption.is_empty() {
        return layout.icon_size;
    }
    layout.icon_size + (layout.line_gap * 0.5) + text_size(&layout.caption_font, "0").height
}

/// What a row shows after one of its buttons has moved the folder, before any
/// poll has confirmed it. The move *is* the command, and it has already
/// happened — this is not a guess about the future, it is the row catching up
/// with what it just did rather than sitting there looking untouched until the
/// next poll comes round (2s locally, up to a minute through the SMB directory
/// cache).
#[derive(Clone, Copy, PartialEq)]
enum Pending {
    Paused,
    Resuming,
}

/// The state a press of this button moves the job into.
fn pending_for(glyph: Glyph) -> Pending {
    match glyph {
        Glyph::Resume => Pending::Resuming,
        _ => Pending::Paused,
    }
}

pub struct RowIvars {
    spec: RowSpec,
    font: Retained<NSFont>,
    detail_font: Retained<NSFont>,
    log_font: Retained<NSFont>,
    button_font: Retained<NSFont>,
    caption_font: Retained<NSFont>,
    icon_x: f64,
    icon_width: f64,
    icon_size: f64,
    text_left: f64,
    text_right: f64,
    bar_height: f64,
    line_gap: f64,
    button_x: f64,
    button_diameter: f64,
    button_gap: f64,
    label_width: f64,
    hovered: Cell<bool>,
    /// Which button the pointer is over, so it can brighten under it.
    hot_button: Cell<Option<usize>>,
    /// Which button is being held down, so it darkens while pressed.
    held_button: Cell<Option<usize>>,
    /// Set once a toggle button has been pressed: the row draws the state it
    /// moved the job into, and the button offers the way back.
    pending: Cell<Option<Pending>>,
    /// Where the folder is now, once this row has moved it. The buttons and the
    /// click-to-open both work from here, because `spec.dir` no longer exists.
    moved_to: RefCell<Option<PathBuf>>,
    /// A move that failed, said where it happened. This used to go to stderr,
    /// which in a `.app` is a file nobody opens — a pause that silently didn't
    /// happen is the whole complaint about these buttons.
    error: RefCell<Option<String>>,
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

            // The block of text is centred on its visible marks rather than
            // its boxes: the leading above the caps otherwise reads as extra
            // padding and drags everything low.
            let name_size = text_size(&ivars.font, &ivars.spec.name);
            let has_line = ivars.spec.log.is_some() || ivars.error.borrow().is_some();
            let log_height = if has_line {
                ivars.line_gap + text_size(&ivars.log_font, "Xg").height
            } else {
                0.0
            };
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

            let kind = self.effective_kind();
            self.draw_state_symbol(kind, bounds);

            // Name and value share a baseline; the name is truncated to
            // whatever the value leaves it.
            let value = self.effective_value();
            let value_size = text_size(&ivars.detail_font, &value);
            let value_gap = if value.is_empty() {
                0.0
            } else {
                ivars.font.pointSize()
            };
            let name_room = ivars.text_right - ivars.text_left - value_size.width - value_gap;
            let name = truncate(&ivars.font, &ivars.spec.name, name_room);
            // A suspended job reads as suspended down the whole row, not just
            // in the word at the end of it.
            let name_color = if kind.dimmed() {
                NSColor::secondaryLabelColor()
            } else {
                NSColor::labelColor()
            };
            draw_text(
                &name,
                &ivars.font,
                &name_color,
                NSPoint {
                    x: ivars.text_left,
                    y: name_y,
                },
            );

            if !value.is_empty() {
                let color = if self.alerting() {
                    NSColor::systemRedColor()
                } else {
                    NSColor::secondaryLabelColor()
                };
                draw_text(
                    &value,
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

            // A failed move takes the log line's place: it is the more urgent
            // thing this row has to say, and it says it where you are already
            // looking rather than in a console you will never open.
            let failure = ivars.error.borrow().clone();
            if let Some(line) = failure.as_ref().or(ivars.spec.log.as_ref()) {
                let log_size = text_size(&ivars.log_font, "Xg");
                y -= ivars.line_gap + log_size.height;
                let text = truncate(&ivars.log_font, line, ivars.text_right - ivars.text_left);
                let color = if failure.is_some() {
                    NSColor::systemRedColor()
                } else {
                    NSColor::tertiaryLabelColor()
                };
                draw_text(
                    &text,
                    &ivars.log_font,
                    &color,
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

        #[unsafe(method(mouseDown:))]
        fn mouse_down(&self, event: &NSEvent) {
            self.ivars().held_button.set(self.button_at(event));
            self.setNeedsDisplay(true);
        }

        // Two kinds of target, as in Finder's sidebar: the buttons command the
        // job, the rest of the row opens its folder.
        #[unsafe(method(mouseUp:))]
        fn mouse_up(&self, event: &NSEvent) {
            let ivars = self.ivars();
            ivars.held_button.set(None);
            if let Some(index) = self.button_at(event) {
                self.press(index);
                return;
            }

            let Some(path) = self.open_target() else {
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
            log_font: layout.log_font.clone(),
            button_font: layout.button_font.clone(),
            caption_font: layout.caption_font.clone(),
            icon_x: layout.icon_x,
            icon_width: layout.icon_width,
            icon_size: layout.icon_size,
            text_left: layout.text_left,
            text_right: layout.text_right,
            bar_height: layout.bar_height,
            line_gap: layout.line_gap,
            button_x: layout.button_x,
            button_diameter: layout.button_diameter,
            button_gap: layout.button_gap,
            label_width: layout.label_width,
            hovered: Cell::new(false),
            hot_button: Cell::new(None),
            held_button: Cell::new(None),
            pending: Cell::new(None),
            moved_to: RefCell::new(None),
            error: RefCell::new(None),
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

    /// Put the row into a pointer state it would normally only reach under a
    /// live mouse, so `examples/render_rows` can draw the hover and pressed
    /// treatments. Nothing in the apps calls this: they have a real pointer.
    pub fn preview_pointer(&self, hot: Option<usize>, held: Option<usize>) {
        self.ivars().hovered.set(hot.is_some() || held.is_some());
        self.ivars().hot_button.set(hot);
        self.ivars().held_button.set(held);
    }

    /// Press button `index`.
    ///
    /// Pause and resume keep the menu open and show the move immediately: they
    /// are the two you press to *watch* something, and dismissing the menu to
    /// go and reopen it — then waiting out a poll — is why they felt like they
    /// did nothing. Stop and the log still dismiss, because both are the end of
    /// what you came to the menu to do.
    fn press(&self, index: usize) {
        let ivars = self.ivars();
        let Some(action) = ivars.spec.actions.get(index) else {
            return;
        };

        match &action.act {
            // The command *is* the move. Whoever is watching the folder — the
            // runner, here or on another machine — does the rest.
            Act::Move(to) => {
                let toggle = action.back.clone();
                // Pressing a toggle a second time puts the folder back where it
                // came from, so pause/resume works as many times as you like
                // without waiting for a poll in between.
                let (destination, next) = match (toggle, ivars.pending.get()) {
                    (Some(back), Some(_)) => (back, None),
                    (Some(_), None) => (to.clone(), Some(pending_for(action.glyph))),
                    (None, _) => (to.clone(), ivars.pending.get()),
                };
                let one_way = action.back.is_none();

                let Some(from) = self.source_dir() else {
                    return;
                };
                match std::fs::rename(&from, &destination) {
                    Ok(()) => {
                        *ivars.error.borrow_mut() = None;
                        *ivars.moved_to.borrow_mut() = Some(destination);
                        ivars.pending.set(next);
                        if one_way {
                            self.dismiss_menu();
                        } else {
                            self.setNeedsDisplay(true);
                        }
                    }
                    Err(err) => {
                        // Stays on screen: a share that went away mid-menu is
                        // exactly when you need to be told.
                        *ivars.error.borrow_mut() = Some(format!("could not move — {err}"));
                        self.setNeedsDisplay(true);
                    }
                }
            }
            Act::Open(path) => {
                self.dismiss_menu();
                let _ = Command::new("open").arg(path).spawn();
            }
        }
    }

    /// Where the job's folder is now: where this row put it, or where the
    /// snapshot found it.
    fn source_dir(&self) -> Option<PathBuf> {
        let ivars = self.ivars();
        ivars
            .moved_to
            .borrow()
            .clone()
            .or_else(|| ivars.spec.dir.clone())
    }

    /// What clicking the row opens. A row that has moved its own folder opens
    /// where the folder went, not the path that is no longer there.
    fn open_target(&self) -> Option<PathBuf> {
        let ivars = self.ivars();
        let path = ivars.spec.path.clone()?;
        match (ivars.moved_to.borrow().clone(), ivars.spec.dir.clone()) {
            (Some(moved), Some(dir)) if dir == path => Some(moved),
            _ => Some(path),
        }
    }

    /// The glyph a button draws now — flipped on the one that has been pressed,
    /// so it offers the way back rather than repeating what it just did.
    fn effective_glyph(&self, index: usize) -> Glyph {
        let ivars = self.ivars();
        let Some(action) = ivars.spec.actions.get(index) else {
            return Glyph::Stop;
        };
        if action.back.is_some() && ivars.pending.get().is_some() {
            return action.glyph.toggled();
        }
        action.glyph
    }

    /// What the row is showing *now*, which is what this row's own buttons have
    /// done to the job when a poll hasn't caught up with them yet.
    fn effective_kind(&self) -> Kind {
        match self.ivars().pending.get() {
            Some(Pending::Paused) => Kind::Paused,
            Some(Pending::Resuming) => Kind::Running,
            None => self.ivars().spec.kind,
        }
    }

    fn effective_value(&self) -> String {
        match self.ivars().pending.get() {
            Some(Pending::Paused) => "paused".to_string(),
            // Not "running": the folder has moved, and the runner has not
            // necessarily noticed yet. Claiming more than that is how a UI
            // starts lying about somebody's encode.
            Some(Pending::Resuming) => "resuming…".to_string(),
            None => self.ivars().spec.value.clone(),
        }
    }

    /// Red text. A row that has just been commanded is not in trouble, whatever
    /// the snapshot said a moment ago.
    fn alerting(&self) -> bool {
        self.ivars().spec.alert && self.ivars().pending.get().is_none()
    }

    /// The state symbol, with the caption under it — elapsed, queue position,
    /// or how long a finished job took.
    fn draw_state_symbol(&self, kind: Kind, bounds: NSRect) {
        let ivars = self.ivars();
        let caption = &ivars.spec.caption;
        let caption_size = text_size(&ivars.caption_font, caption);
        let stack = if caption.is_empty() {
            ivars.icon_size
        } else {
            ivars.icon_size + ivars.line_gap * 0.5 + caption_size.height
        };
        let top = ((bounds.size.height + stack) / 2.0).round();

        let centred = |width: f64| (ivars.icon_x + (ivars.icon_width - width) / 2.0).round();

        if let Some(image) = symbol_image(kind.symbol(), &kind.tint()) {
            let size = image.size();
            let scale = if size.width > 0.0 && size.height > 0.0 {
                (ivars.icon_size / size.width).min(ivars.icon_size / size.height)
            } else {
                1.0
            };
            let (width, height) = (size.width * scale, size.height * scale);
            image.drawInRect_fromRect_operation_fraction(
                rect(
                    centred(width),
                    (top - ivars.icon_size + (ivars.icon_size - height) / 2.0).round(),
                    width,
                    height,
                ),
                NSRect::ZERO,
                NSCompositingOperation::SourceOver,
                1.0,
            );
        }

        if !caption.is_empty() {
            draw_text(
                caption,
                &ivars.caption_font,
                &NSColor::tertiaryLabelColor(),
                NSPoint {
                    x: centred(caption_size.width),
                    y: (top - stack).round(),
                },
            );
        }
    }

    fn draw_bar(&self, y: f64) {
        let ivars = self.ivars();
        let width = ivars.text_right - ivars.text_left;
        let radius = ivars.bar_height / 2.0;
        let track = rect(ivars.text_left, y, width, ivars.bar_height);

        NSColor::labelColor().colorWithAlphaComponent(0.16).set();
        NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(track, radius, radius).fill();

        let kind = self.effective_kind();
        // A suspended job's bar is drawn back to nearly the track's own
        // strength: at a glance down the menu the difference between a job
        // working and a job stopped shouldn't be one word of grey text.
        let progress = match (kind, ivars.spec.progress) {
            (Kind::Paused, Progress::Unknown) => Progress::Track,
            (_, progress) => progress,
        };

        match progress {
            Progress::Fraction(fraction) => {
                let filled = (width * fraction.clamp(0.0, 1.0)).max(ivars.bar_height);
                let strength = if kind.dimmed() { 0.34 } else { 0.75 };
                if self.alerting() {
                    NSColor::systemRedColor().colorWithAlphaComponent(strength).set();
                } else {
                    NSColor::labelColor().colorWithAlphaComponent(strength).set();
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

    fn button_width(&self, index: usize) -> f64 {
        let ivars = self.ivars();
        match ivars.spec.actions.get(index).map(|action| action.glyph) {
            Some(glyph) if glyph.label().is_some() => ivars.label_width,
            _ => ivars.button_diameter,
        }
    }

    /// Button `index`'s box, laid out left to right, each as wide as it needs.
    fn button_rect(&self, index: usize, bounds: NSRect) -> NSRect {
        let ivars = self.ivars();
        let mut x = ivars.button_x;
        for earlier in 0..index {
            x += self.button_width(earlier) + ivars.button_gap;
        }
        let width = self.button_width(index);
        rect(
            x,
            (bounds.size.height - ivars.button_diameter) / 2.0,
            width,
            ivars.button_diameter,
        )
    }

    fn button_at(&self, event: &NSEvent) -> Option<usize> {
        let ivars = self.ivars();
        if ivars.spec.actions.is_empty() {
            return None;
        }
        let point = self.convertPoint_fromView(event.locationInWindow(), None);
        let bounds = self.bounds();
        // A little forgiveness beyond the drawn box.
        let slack = 3.0;
        (0..ivars.spec.actions.len()).find(|index| {
            let box_ = self.button_rect(*index, bounds);
            point.x >= box_.origin.x - slack
                && point.x <= box_.origin.x + box_.size.width + slack
                && point.y >= box_.origin.y - slack
                && point.y <= box_.origin.y + box_.size.height + slack
        })
    }

    fn track_pointer(&self, event: &NSEvent) {
        let hot = self.button_at(event);
        if hot != self.ivars().hot_button.get() {
            self.ivars().hot_button.set(hot);
        }
        self.setNeedsDisplay(true);
    }

    /// The SF `circle.fill` symbols, on a background that appears under the
    /// pointer and deepens while held — filled circles read at menu size where
    /// hand-drawn glyphs went muddy. `log` is a labelled pill instead: it opens
    /// something rather than commanding the job, and shouldn't be mistaken for
    /// one of the verbs.
    ///
    /// Every button gets the same background treatment. Tinting stop red on
    /// hover and leaving the others to shift opacity by a third of a step meant
    /// only the destructive button felt like a button at all.
    fn draw_buttons(&self, bounds: NSRect) {
        let ivars = self.ivars();
        let hot = ivars.hot_button.get();
        let held = ivars.held_button.get();

        for index in 0..ivars.spec.actions.len() {
            let hovered = hot == Some(index);
            let pressed = held == Some(index);
            let glyph = self.effective_glyph(index);
            let box_ = self.button_rect(index, bounds);

            let tint = match (glyph, hovered || pressed) {
                // Stopping is the destructive one, so it says so under the
                // pointer as well as lighting up like the rest.
                (Glyph::Stop, true) => NSColor::systemRedColor(),
                (_, true) => NSColor::labelColor(),
                _ => NSColor::labelColor().colorWithAlphaComponent(0.62),
            };
            let backing = match (pressed, hovered) {
                (true, _) => 0.30,
                (false, true) => 0.15,
                (false, false) => 0.0,
            };

            if let Some(label) = glyph.label() {
                let height = (ivars.button_diameter * 0.76).round();
                let pill = rect(
                    box_.origin.x,
                    box_.origin.y + (ivars.button_diameter - height) / 2.0,
                    box_.size.width,
                    height,
                );
                // The pill is a shape in its own right, so it keeps a resting
                // fill where the symbol buttons have none.
                NSColor::labelColor()
                    .colorWithAlphaComponent(0.13 + backing)
                    .set();
                NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                    pill,
                    height / 2.0,
                    height / 2.0,
                )
                .fill();

                let size = text_size(&ivars.button_font, label);
                draw_text(
                    label,
                    &ivars.button_font,
                    &tint,
                    NSPoint {
                        x: (pill.origin.x + (pill.size.width - size.width) / 2.0).round(),
                        y: (pill.origin.y + (pill.size.height - size.height) / 2.0).round(),
                    },
                );
                continue;
            }

            if backing > 0.0 {
                let disc = rect(
                    box_.origin.x,
                    box_.origin.y,
                    box_.size.width,
                    box_.size.height,
                );
                NSColor::labelColor().colorWithAlphaComponent(backing).set();
                NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                    disc,
                    disc.size.width / 2.0,
                    disc.size.height / 2.0,
                )
                .fill();
            }

            let Some(icon) = glyph.symbol().and_then(|name| symbol_image(name, &tint)) else {
                continue;
            };

            // Inside the backing disc rather than filling it, so the ring of
            // background reads as the button and the symbol as its label.
            let inset = (ivars.button_diameter * 0.16).round();
            let box_ = rect(
                box_.origin.x + inset,
                box_.origin.y + inset,
                box_.size.width - inset * 2.0,
                box_.size.height - inset * 2.0,
            );
            let size = icon.size();
            let scale = if size.width > 0.0 && size.height > 0.0 {
                (box_.size.width / size.width).min(box_.size.height / size.height)
            } else {
                1.0
            };
            let width = size.width * scale;
            let height = size.height * scale;
            icon.drawInRect_fromRect_operation_fraction(
                rect(
                    box_.origin.x + (box_.size.width - width) / 2.0,
                    box_.origin.y + (box_.size.height - height) / 2.0,
                    width,
                    height,
                ),
                NSRect::ZERO,
                NSCompositingOperation::SourceOver,
                1.0,
            );
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

/// An SF Symbol in one colour, ready to draw. `None` when the running system
/// doesn't have that symbol, which is a thing to skip rather than a thing to
/// fall back from — every symbol here has been in macOS since well before the
/// versions this runs on.
fn symbol_image(name: &str, tint: &NSColor) -> Option<Retained<NSImage>> {
    let symbol = NSImage::imageWithSystemSymbolName_accessibilityDescription(
        &NSString::from_str(name),
        None,
    )?;
    let config = NSImageSymbolConfiguration::configurationWithHierarchicalColor(tint);
    symbol.imageWithSymbolConfiguration(&config)
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
        // No log button: these fixtures have no log file on disk.
        assert_eq!(running.actions.len(), 2);
        assert_eq!(running.actions[0].glyph, Glyph::Pause);
        assert_eq!(
            running.actions[0].act,
            Act::Move(PathBuf::from("/j/_paused/2026-running"))
        );
        assert_eq!(running.actions[1].glyph, Glyph::Stop);
        assert_eq!(
            running.actions[1].act,
            Act::Move(PathBuf::from("/j/_failed/2026-running"))
        );

        // A paused job offers the way back, not another pause.
        let held = rows.iter().find(|row| row.name == "held").unwrap();
        assert_eq!(held.actions[0].glyph, Glyph::Resume);
        assert_eq!(
            held.actions[0].act,
            Act::Move(PathBuf::from("/j/_running/2026-held"))
        );

        // A job that hasn't started can be held, but there is nothing to stop.
        let waiting = rows.iter().find(|row| row.name == "waiting").unwrap();
        assert_eq!(waiting.actions.len(), 1);
        assert_eq!(
            waiting.actions[0].act,
            Act::Move(PathBuf::from("/j/_paused/2026-waiting"))
        );

        // Finished jobs get no controls at all.
        assert!(rows.iter().filter(|row| row.kind == Kind::Done).all(|row| row.actions.is_empty()));
    }

    /// Every toggle has to know where the job came from, and it is not always
    /// `_running`: sending a queued job back there would claim it had started.
    /// The row is the only thing that knows the difference, so it records it.
    #[test]
    fn a_toggle_knows_the_way_back() {
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
        let back = |name: &str| {
            rows.iter()
                .find(|row| row.name == name)
                .unwrap()
                .actions[0]
                .back
                .clone()
        };

        assert_eq!(back("running"), Some(PathBuf::from("/j/_running/2026-running")));
        assert_eq!(back("held"), Some(PathBuf::from("/j/_paused/2026-held")));
        // Never started, so it goes back to the queue it was waiting in.
        assert_eq!(back("waiting"), Some(PathBuf::from("/j/_ready/2026-waiting")));

        // Stopping is not a toggle: there is no coming back from `_failed`.
        let running = rows.iter().find(|row| row.name == "running").unwrap();
        assert_eq!(running.actions[1].glyph, Glyph::Stop);
        assert_eq!(running.actions[1].back, None);
    }

    /// Pause and resume were the same glyph, so the button under your pointer
    /// never said which way it went.
    #[test]
    fn pause_and_resume_do_not_look_alike() {
        assert_ne!(Glyph::Pause.symbol(), Glyph::Resume.symbol());
        assert_eq!(Glyph::Pause.toggled(), Glyph::Resume);
        assert_eq!(Glyph::Resume.toggled(), Glyph::Pause);
    }

    /// The row's state has to be visible as a symbol, not only as a word at the
    /// far right of it — and a stalled job is its own state, not a running one
    /// wearing red text.
    #[test]
    fn a_stalled_job_is_its_own_state() {
        let mut stalled = job("orphan", State::Running);
        stalled.started = Some(SystemTime::now() - Duration::from_secs(4 * 3600));
        stalled.last_output = Some(SystemTime::now() - Duration::from_secs(3 * 3600));
        stalled.local = false;

        let snapshot = Snapshot {
            root: Some(Root::new("/j")),
            connected: true,
            jobs: vec![stalled],
            ..Snapshot::default()
        };
        let row = &sections(&snapshot, 5, 5)[0].rows[0];
        assert_eq!(row.kind, Kind::Stalled);
        assert_eq!(row.value, "not running");
        assert_ne!(Kind::Stalled.symbol(), Kind::Running.symbol());
        assert_ne!(Kind::Paused.symbol(), Kind::Running.symbol());
    }

    /// A remote job that has printed nothing *yet* is not a stalled one however
    /// long it has been going: the log file appears with the first line, so a
    /// slow starter and a dead runner look identical from another machine.
    #[test]
    fn silence_escalates_rather_than_jumping_to_a_verdict() {
        let mut quiet = job("remote", State::Running);
        quiet.started = Some(SystemTime::now() - Duration::from_secs(6 * 3600));
        quiet.local = false;

        let never_logged = Snapshot {
            root: Some(Root::new("/j")),
            connected: true,
            jobs: vec![quiet.clone()],
            ..Snapshot::default()
        };
        let row = &sections(&never_logged, 5, 5)[0].rows[0];
        assert_eq!(row.kind, Kind::Running, "got {:?}", row.value);

        // Quiet for an hour: worth mentioning, not worth a verdict. Half an
        // hour between progress lines is normal for a slow encode.
        quiet.last_output = Some(SystemTime::now() - Duration::from_secs(60 * 60));
        let hushed = Snapshot {
            root: Some(Root::new("/j")),
            connected: true,
            jobs: vec![quiet.clone()],
            ..Snapshot::default()
        };
        let row = &sections(&hushed, 5, 5)[0].rows[0];
        assert_eq!(row.kind, Kind::Running);
        assert!(row.value.starts_with("no output"), "got {:?}", row.value);

        // Well past any sane reporting interval, silence is the verdict.
        quiet.last_output = Some(SystemTime::now() - Duration::from_secs(3 * 3600));
        let silent = Snapshot {
            root: Some(Root::new("/j")),
            connected: true,
            jobs: vec![quiet],
            ..Snapshot::default()
        };
        assert_eq!(sections(&silent, 5, 5)[0].rows[0].kind, Kind::Stalled);
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
