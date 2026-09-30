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
use std::sync::OnceLock;
use std::time::Duration;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{
    AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, Message, define_class, msg_send,
};
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
            Kind::Failed => NSColor::systemRedColor(),
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

/// A button on a row: what it draws, and what pressing it does.
#[derive(Clone)]
pub struct Action {
    pub glyph: Glyph,
    pub act: Act,
}

impl Action {
    /// A button that calls the app back with `token`. The app answers from
    /// its own model, so the row has the new state before it redraws.
    pub fn call(glyph: Glyph, token: u64) -> Self {
        Self {
            glyph,
            act: Act::Call(token),
        }
    }

    /// The small "log" pill, opening `path`.
    pub fn log(path: PathBuf) -> Self {
        Self {
            glyph: Glyph::Log,
            act: Act::Open(path),
        }
    }
}

/// Where [`Act::Call`] presses go. Set once, by the app that owns the queue.
///
/// Called on the main thread, from inside menu tracking, so a handler must
/// return promptly: signalling a process group is instant, and anything that
/// blocks here blocks the menu.
static ON_CALL: OnceLock<Box<dyn Fn(u64) + Send + Sync + 'static>> = OnceLock::new();

pub fn on_call(handler: impl Fn(u64) + Send + Sync + 'static) {
    let _ = ON_CALL.set(Box::new(handler));
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    /// Open this path — a log, not a command.
    Open(PathBuf),
    /// Hand this token to the app's [`on_call`] handler. Nothing here knows
    /// what the token means: the row's job is to notice the press.
    Call(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Glyph {
    Pause,
    Resume,
    Stop,
    /// Send a queued job to the front. Only means anything where the queue
    /// order is the app's to change.
    Top,
    /// Run a finished job again.
    Retry,
    /// Drawn as a small labelled pill rather than a symbol: opening a log is
    /// not a command and shouldn't look like one.
    Log,
}

impl Glyph {
    /// The SF Symbol the button draws, where it draws one. Pause and resume
    /// draw *different* glyphs: they shared `playpause` on the reasoning that
    /// the row's value already said which way it would go, which asked you to
    /// read the far end of the row to find out what the button under your
    /// pointer did. Stop is an x: stopping *is* killing (SIGTERM to the
    /// process group), so there is only the one verb.
    fn symbol(self) -> Option<&'static str> {
        match self {
            Glyph::Pause => Some("pause.circle.fill"),
            Glyph::Resume => Some("play.circle.fill"),
            Glyph::Stop => Some("xmark.circle.fill"),
            Glyph::Top => Some("arrow.up.circle.fill"),
            Glyph::Retry => Some("arrow.clockwise.circle.fill"),
            Glyph::Log => None,
        }
    }

    fn label(self) -> Option<&'static str> {
        match self {
            Glyph::Log => Some("log"),
            _ => None,
        }
    }
}

/// Everything one row draws.
#[derive(Clone)]
pub struct RowSpec {
    pub kind: Kind,
    /// Small text under the icon: elapsed, queue position, or how long a
    /// finished job took.
    pub caption: String,
    pub name: String,
    /// Right-aligned: a percentage, `queued`, `24h ago`.
    pub value: String,
    /// Draws the value in red — a failure.
    pub alert: bool,
    pub progress: Progress,
    /// The last line the job printed, under the bar.
    pub log: Option<String>,
    /// Opened in Finder when the row is clicked.
    pub path: Option<PathBuf>,
    /// Reveal the path in its parent folder rather than opening it.
    pub reveal: bool,
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
            actions: Vec::new(),
        }
    }

    pub fn actions(mut self, actions: Vec<Action>) -> Self {
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

pub struct RowIvars {
    /// Replaceable, so a row can be redrawn from a model that changed under it
    /// without the menu being torn down and rebuilt — see [`JobRow::update`].
    spec: RefCell<RowSpec>,
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
            let spec = ivars.spec.borrow();
            let bounds = self.bounds();

            if ivars.hovered.get() && spec.path.is_some() {
                draw_highlight(bounds);
            }

            // The block of text is centred on its visible marks rather than
            // its boxes: the leading above the caps otherwise reads as extra
            // padding and drags everything low.
            let name_size = text_size(&ivars.font, &spec.name);
            let has_line = spec.log.is_some();
            let log_height = if has_line {
                ivars.line_gap + text_size(&ivars.log_font, "Xg").height
            } else {
                0.0
            };
            let bar_block = if spec.has_bar() {
                ivars.line_gap + ivars.bar_height
            } else {
                0.0
            };
            let content = name_size.height + bar_block + log_height;
            let lift = (ivars.font.pointSize() * 0.12).round();
            let name_y = bounds.size.height - ((bounds.size.height - content) / 2.0).round()
                - name_size.height
                + lift;

            let kind = spec.kind;
            self.draw_state_symbol(kind, bounds);

            // Name and value share a baseline; the name is truncated to
            // whatever the value leaves it.
            let value = spec.value.clone();
            let value_size = text_size(&ivars.detail_font, &value);
            let value_gap = if value.is_empty() {
                0.0
            } else {
                ivars.font.pointSize()
            };
            let name_room = ivars.text_right - ivars.text_left - value_size.width - value_gap;
            let name = truncate(&ivars.font, &spec.name, name_room);
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
                let color = if spec.alert {
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
            if spec.has_bar() {
                y -= ivars.line_gap + ivars.bar_height;
                self.draw_bar(y);
            }

            self.draw_buttons(bounds);

            if let Some(line) = spec.log.as_ref() {
                let log_size = text_size(&ivars.log_font, "Xg");
                y -= ivars.line_gap + log_size.height;
                let text = truncate(&ivars.log_font, line, ivars.text_right - ivars.text_left);
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
        //
        // `mouseUp:` only. A menu tracks in its own modal run loop and routes
        // events to item views itself; a `mouseDown:` override that swallows
        // the event — added here for a pressed-button treatment — hung menu
        // tracking hard enough to beachball the machine. Hover is drawn from
        // the tracking area instead, which AppKit feeds without our
        // intercepting anything.
        #[unsafe(method(mouseUp:))]
        fn mouse_up(&self, event: &NSEvent) {
            let ivars = self.ivars();
            if let Some(index) = self.button_at(event) {
                self.press(index);
                return;
            }

            let Some(path) = ivars.spec.borrow().path.clone() else {
                return;
            };
            self.dismiss_menu();
            let mut command = Command::new("open");
            if ivars.spec.borrow().reveal {
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
            spec: RefCell::new(spec),
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

    /// Press button `index`. A call keeps the menu open — the app redraws the
    /// row from its model — and opening the log dismisses it.
    fn press(&self, index: usize) {
        let ivars = self.ivars();
        // Cloned, and the borrow released before anything is dispatched: an
        // `Act::Call` handler answers by rewriting this row's own spec, and a
        // borrow still held across that call would be a panic rather than a
        // paused job.
        let Some(action) = ivars.spec.borrow().actions.get(index).cloned() else {
            return;
        };

        match &action.act {
            Act::Open(path) => {
                self.dismiss_menu();
                let _ = Command::new("open").arg(path).spawn();
            }
            // Straight through to the app, on this thread, and the menu stays
            // open. There is nothing to guess at and nothing to undo: the
            // handler changes the model and hands every visible row its new
            // spec back before this returns, so the row is already right by the
            // time it redraws.
            Act::Call(token) => {
                // Held for the duration of the call. A handler is free to
                // answer by rebuilding the menu it was called from — moving a
                // job to the front of the queue reorders these very rows — and
                // that releases every row in it, including the one whose method
                // is running.
                let keep_alive = self.retain();
                if let Some(handler) = ON_CALL.get() {
                    handler(*token);
                }
                keep_alive.setNeedsDisplay(true);
            }
        }
    }

    /// Redraw this row against a fresh spec — the app has the new truth
    /// immediately, and tearing the menu down to say a job reached 46% would
    /// take the pointer's place in it with it.
    pub fn update(&self, spec: RowSpec) {
        *self.ivars().spec.borrow_mut() = spec;
        self.setNeedsDisplay(true);
    }

    /// The state symbol, with the caption under it — elapsed, queue position,
    /// or how long a finished job took.
    fn draw_state_symbol(&self, kind: Kind, bounds: NSRect) {
        let ivars = self.ivars();
        let spec = ivars.spec.borrow();
        let caption = &spec.caption;
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

        let kind = ivars.spec.borrow().kind;
        // A suspended job's bar is drawn back to nearly the track's own
        // strength: at a glance down the menu the difference between a job
        // working and a job stopped shouldn't be one word of grey text.
        let progress = match (kind, ivars.spec.borrow().progress) {
            (Kind::Paused, Progress::Unknown) => Progress::Track,
            (_, progress) => progress,
        };

        match progress {
            Progress::Fraction(fraction) => {
                let filled = (width * fraction.clamp(0.0, 1.0)).max(ivars.bar_height);
                let strength = if kind.dimmed() { 0.34 } else { 0.75 };
                if ivars.spec.borrow().alert {
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
        match ivars.spec.borrow().actions.get(index).map(|action| action.glyph) {
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
        if ivars.spec.borrow().actions.is_empty() {
            return None;
        }
        let point = self.convertPoint_fromView(event.locationInWindow(), None);
        let bounds = self.bounds();
        // A little forgiveness beyond the drawn box.
        let slack = 3.0;
        let count = ivars.spec.borrow().actions.len();
        (0..count).find(|index| {
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
    /// pointer — filled circles read at menu size where hand-drawn glyphs went
    /// muddy. `log` is a labelled pill instead: it opens something rather than
    /// commanding the job, and shouldn't be mistaken for one of the verbs.
    ///
    /// Every button gets the same background treatment. Tinting stop red on
    /// hover and leaving the others to shift opacity by a third of a step meant
    /// only the destructive button felt like a button at all.
    ///
    /// Hover only, no pressed state: drawing one means overriding `mouseDown:`,
    /// and a view in a menu item that swallows mouse-down hangs menu tracking.
    /// The row answering the press by changing state does the same job.
    fn draw_buttons(&self, bounds: NSRect) {
        let ivars = self.ivars();
        let hot = ivars.hot_button.get();

        let count = ivars.spec.borrow().actions.len();
        for index in 0..count {
            let hovered = hot == Some(index);
            let glyph = ivars.spec.borrow().actions[index].glyph;
            let box_ = self.button_rect(index, bounds);

            let tint = match (glyph, hovered) {
                // Stopping is the destructive one, so it says so under the
                // pointer as well as lighting up like the rest.
                (Glyph::Stop, true) => NSColor::systemRedColor(),
                (_, true) => NSColor::labelColor(),
                _ => NSColor::labelColor().colorWithAlphaComponent(0.62),
            };
            let backing = if hovered { 0.15 } else { 0.0 };

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

    /// Pause and resume were the same glyph, so the button under your pointer
    /// never said which way it went.
    #[test]
    fn pause_and_resume_do_not_look_alike() {
        assert_ne!(Glyph::Pause.symbol(), Glyph::Resume.symbol());
        assert_ne!(Kind::Paused.symbol(), Kind::Running.symbol());
    }
}
