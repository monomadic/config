//! The menu bar icon: a terminal prompt, in one of several selectable styles.
//!
//! Every style shares a vocabulary: a chevron prompt, a blinking block for a
//! running job, a dim resting underscore for an idle one, and red —
//! reserved for failures. A failed job draws the style's own job mark in red
//! rather than a glyph of its own: an `!` small enough to fit a menu bar is
//! a smudge, and colour alone carries the meaning at that size. When no
//! watched folder is reachable the whole icon becomes a blinking solid
//! folder: not a subtle tell but an error state, since a monitor with nothing
//! to read has nothing true to say.
//!
//! The styles differ in how they spend width. `Classic` is the original
//! fixed-width icon: one cursor slot for the aggregate state, queued jobs as
//! up to three fading dots. The other three give every job its own indicator
//! and let the icon widen with the queue, up to a per-style cap:
//!
//! - `Cursors` — a lane of cursors after the prompt, one per job, inside the
//!   screen frame. Idle is pixel-identical to `Classic` idle.
//! - `Screen` — the frame itself widens and jobs render inside it as blocks,
//!   a terminal showing its own queue.
//! - `Equalizer` — no frame; a thin bar per job after the chevron. Densest,
//!   so it carries the largest cap.
//!
//! Everything is drawn rather than glyph-based, so it stays crisp at any
//! backing scale. Geometry is authored on the original 14-unit grid and
//! scaled up to [`HEIGHT`] points at draw time. Neutral states are template
//! images that macOS tints to match the menu bar; failure states mix red with
//! the label colour, so they are drawn as regular images instead.

use objc2::rc::Retained;
use objc2_app_kit::{
    NSBezierPath, NSColor, NSImage, NSLineCapStyle, NSLineJoinStyle,
};
use objc2_foundation::{NSPoint, NSRect, NSSize};

/// Drawn height, in points. The geometry below is authored on a 14-unit grid
/// (the icon's original size, which read too small in the bar) and scaled.
pub const HEIGHT: f64 = 20.0;
const UNIT: f64 = 14.0;
const S: f64 = HEIGHT / UNIT;

const DIM: f64 = 0.4;

// Per-style caps on how many jobs get their own indicator; past the cap the
// last slot becomes a dim `+` (or an ellipsis inside Screen's frame).
const CLASSIC_QUEUE_DOTS: usize = 3;
const CURSORS_CAP: usize = 6;
const SCREEN_CAP: usize = 6;
const EQUALIZER_CAP: usize = 10;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Style {
    Classic,
    Cursors,
    Screen,
    Equalizer,
}

pub const ALL_STYLES: [Style; 4] = [
    Style::Classic,
    Style::Cursors,
    Style::Screen,
    Style::Equalizer,
];

impl Style {
    pub fn label(self) -> &'static str {
        match self {
            Style::Classic => "Classic",
            Style::Cursors => "Cursor per Job",
            Style::Screen => "Queue on Screen",
            Style::Equalizer => "Equalizer",
        }
    }

    pub fn key(self) -> &'static str {
        match self {
            Style::Classic => "classic",
            Style::Cursors => "cursors",
            Style::Screen => "screen",
            Style::Equalizer => "equalizer",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        ALL_STYLES.iter().copied().find(|style| style.key() == key)
    }
}

#[derive(Clone, Copy)]
pub struct IconState {
    pub running: usize,
    pub queued: usize,
    /// Unacknowledged failures (a stalled job counts as one).
    pub failed: usize,
    pub blink_on: bool,
    /// False when no watched folder can be reached.
    pub connected: bool,
}

impl Default for IconState {
    fn default() -> Self {
        Self {
            running: 0,
            queued: 0,
            failed: 0,
            blink_on: true,
            connected: true,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Glyph {
    Run,
    Queued,
    Failed,
}

/// The per-job indicators to draw, in display order (running, then queued,
/// then failed), plus whether anything was cut. When the queue overflows the
/// cap it is the queued jobs that give way first: a failure must never be
/// capped out of sight, since red is the only thing that makes it visible.
fn lane(state: &IconState, cap: usize) -> (Vec<Glyph>, bool) {
    let total = state.running + state.queued + state.failed;
    let over = total > cap;
    let failed = state.failed.min(cap);
    let running = state.running.min(cap - failed);
    let queued = state.queued.min(cap - failed - running);
    let mut glyphs = Vec::with_capacity(running + queued + failed);
    glyphs.extend(std::iter::repeat_n(Glyph::Run, running));
    glyphs.extend(std::iter::repeat_n(Glyph::Queued, queued));
    glyphs.extend(std::iter::repeat_n(Glyph::Failed, failed));
    (glyphs, over)
}

pub fn draw(style: Style, state: &IconState) -> Retained<NSImage> {
    let mut state = *state;
    // An unreachable folder knows nothing: whatever counts arrived with the
    // state are stale, so they are not drawn.
    let offline = !state.connected;
    if offline {
        state.running = 0;
        state.queued = 0;
        state.failed = 0;
    }

    let width = if offline { 16.0 } else { width_for(style, &state) };
    let size = NSSize { width: width * S, height: HEIGHT };

    // Template images are drawn in black and tinted by macOS; failure states
    // need real colour, so they resolve the label colour themselves.
    let is_template = state.failed == 0;
    let ink = if is_template {
        NSColor::blackColor()
    } else {
        NSColor::labelColor()
    };

    let handler = block2::RcBlock::new(move |_bounds: NSRect| -> objc2::runtime::Bool {
        if offline {
            draw_folder_gone(&state, &ink);
        } else {
            match style {
                Style::Classic => draw_classic(&state, &ink),
                Style::Cursors => draw_cursors(&state, &ink),
                Style::Screen => draw_screen(&state, &ink),
                Style::Equalizer => draw_equalizer(&state, &ink),
            }
        }
        objc2::runtime::Bool::YES
    });

    let image = NSImage::imageWithSize_flipped_drawingHandler(size, false, &handler);
    image.setTemplate(is_template);
    image
}

fn width_for(style: Style, state: &IconState) -> f64 {
    match style {
        Style::Classic => {
            if state.queued.min(CLASSIC_QUEUE_DOTS) > 0 { 27.0 } else { 16.0 }
        }
        Style::Cursors => cursors_frame_width(state) + 2.0,
        Style::Screen => screen_frame_width(state) + 2.0,
        Style::Equalizer => {
            let (glyphs, over) = lane(state, EQUALIZER_CAP);
            let slots = glyphs.len() + over as usize;
            if slots == 0 { 14.0 } else { 11.0 + 3.0 * slots as f64 }
        }
    }
}

fn screen_frame_width(state: &IconState) -> f64 {
    let (glyphs, _) = lane(state, SCREEN_CAP);
    let extra = SCREEN_PITCH * glyphs.len().saturating_sub(1) as f64;
    (LANE_X + SCREEN_PITCH + extra).max(14.0)
}

/// The frame ends a slot's-worth after the last cursor, not a whole pitch —
/// otherwise the icon carries a visible empty seat for a job that isn't there.
fn cursors_frame_width(state: &IconState) -> f64 {
    let (glyphs, over) = lane(state, CURSORS_CAP);
    let slots = glyphs.len() + over as usize;
    if slots == 0 {
        return 14.0;
    }
    let extra = CURSORS_PITCH * (slots - 1) as f64;
    (LANE_X + TRAILING + extra).max(14.0)
}

/// Where the first job mark sits, shared by Cursors and Screen so the prompt
/// keeps identical breathing room across styles.
const LANE_X: f64 = 9.6;
const CURSORS_PITCH: f64 = 3.9;
const SCREEN_PITCH: f64 = 4.2;
/// Mark width plus the gap to the frame's inner edge.
const TRAILING: f64 = 4.2;

// ---- shared strokes, all in unit space ----

fn rect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
    NSRect {
        origin: NSPoint { x: x * S, y: y * S },
        size: NSSize { width: width * S, height: height * S },
    }
}

fn point(x: f64, y: f64) -> NSPoint {
    NSPoint { x: x * S, y: y * S }
}

fn fill_rounded(x: f64, y: f64, width: f64, height: f64, radius: f64) {
    NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
        rect(x, y, width, height),
        radius * S,
        radius * S,
    )
    .fill();
}

fn fill_oval(x: f64, y: f64, width: f64, height: f64) {
    NSBezierPath::bezierPathWithOvalInRect(rect(x, y, width, height)).fill();
}

fn stroke_frame(width: f64) {
    let frame = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
        rect(1.0, 1.5, width, 11.0),
        2.4 * S,
        2.4 * S,
    );
    frame.setLineWidth(1.0 * S);
    frame.stroke();
}

fn stroke_chevron(x: f64) {
    let chevron = NSBezierPath::new();
    chevron.moveToPoint(point(x, 9.3));
    chevron.lineToPoint(point(x + 2.8, 7.0));
    chevron.lineToPoint(point(x, 4.7));
    chevron.setLineWidth(1.7 * S);
    chevron.setLineCapStyle(NSLineCapStyle::Round);
    chevron.setLineJoinStyle(NSLineJoinStyle::Round);
    chevron.stroke();
}

fn underscore(x: f64) {
    fill_rounded(x, 4.1, 2.6, 1.3, 0.6);
}

/// The overflow marker past a style's cap: a dim `+`.
fn stroke_plus(x: f64, center_y: f64) {
    let plus = NSBezierPath::new();
    plus.moveToPoint(point(x, center_y));
    plus.lineToPoint(point(x + 2.4, center_y));
    plus.moveToPoint(point(x + 1.2, center_y - 1.2));
    plus.lineToPoint(point(x + 1.2, center_y + 1.2));
    plus.setLineWidth(1.1 * S);
    plus.setLineCapStyle(NSLineCapStyle::Round);
    plus.stroke();
}

fn red() -> Retained<NSColor> {
    NSColor::systemRedColor()
}

// ---- the styles ----

/// No reachable folder: the icon stops being a terminal and becomes the
/// missing thing itself — a solid folder, blinking for attention. Nothing
/// else is drawn, because nothing else is known.
fn draw_folder_gone(state: &IconState, ink: &NSColor) {
    if !state.blink_on {
        return;
    }
    ink.set();
    // The tab first, then the body over it.
    fill_rounded(1.5, 7.6, 6.4, 4.0, 1.2);
    fill_rounded(1.5, 2.3, 13.0, 7.8, 1.2);
}

fn draw_classic(state: &IconState, ink: &NSColor) {
    let running = state.running > 0;
    let errors = state.failed > 0;
    let dots = state.queued.min(CLASSIC_QUEUE_DOTS);

    // Full strength whatever the queue is doing. The frame and the prompt are
    // what the icon *is*, not what it is reporting: fading them for an idle
    // queue made the whole thing read as disabled — and this app has a real
    // disabled state already, drawn as a missing folder.
    ink.set();
    stroke_frame(14.0);

    // The prompt stays the prompt: failures are carried by the badge, not by
    // recolouring the mark that says what this icon is.
    ink.set();
    stroke_chevron(4.6);

    // Cursor slot: a blinking block while a job runs, a resting underscore
    // otherwise.
    if running {
        if state.blink_on {
            ink.set();
            fill_rounded(9.2, 4.2, 2.2, 5.4, 0.5);
        }
    } else {
        ink.colorWithAlphaComponent(DIM).set();
        underscore(9.2);
    }

    for index in 0..dots {
        ink.colorWithAlphaComponent([0.6, 0.4, 0.25][index]).set();
        fill_oval(17.1 + index as f64 * 3.7, 5.5, 3.0, 3.0);
    }

    // The unacknowledged-failure badge, in the corner where every other
    // notification dot on the bar sits. Steady rather than blinking: it is a
    // count of something already over, not something happening.
    if errors {
        red().set();
        fill_oval(11.9, 8.9, 3.8, 3.8);
    }
}

fn draw_cursors(state: &IconState, ink: &NSColor) {
    let (glyphs, over) = lane(state, CURSORS_CAP);
    let slots = glyphs.len() + over as usize;

    ink.set();
    stroke_frame(cursors_frame_width(state));
    ink.set();
    stroke_chevron(4.6);

    if slots == 0 {
        ink.colorWithAlphaComponent(DIM).set();
        underscore(9.2);
        return;
    }

    let mut queue_index = 0usize;
    for (index, glyph) in glyphs.iter().enumerate() {
        let x = LANE_X + index as f64 * CURSORS_PITCH;
        match glyph {
            Glyph::Run => {
                if state.blink_on {
                    ink.set();
                    fill_rounded(x, 4.2, 2.2, 5.4, 0.5);
                }
            }
            Glyph::Queued => {
                let alpha = [0.6, 0.45, 0.32, 0.25][queue_index.min(3)];
                queue_index += 1;
                ink.colorWithAlphaComponent(alpha).set();
                underscore(x);
            }
            // The same block a running job gets, in red: at this size the
            // colour is the whole message.
            Glyph::Failed => {
                if state.blink_on {
                    red().set();
                    fill_rounded(x, 4.2, 2.2, 5.4, 0.5);
                }
            }
        }
    }
    if over {
        ink.colorWithAlphaComponent(DIM).set();
        stroke_plus(LANE_X + glyphs.len() as f64 * CURSORS_PITCH + 0.4, 7.0);
    }
}

fn draw_screen(state: &IconState, ink: &NSColor) {
    let (glyphs, over) = lane(state, SCREEN_CAP);

    ink.set();
    stroke_frame(screen_frame_width(state));
    ink.set();
    stroke_chevron(4.6);

    if glyphs.is_empty() {
        ink.colorWithAlphaComponent(DIM).set();
        underscore(LANE_X);
        return;
    }

    for (index, glyph) in glyphs.iter().enumerate() {
        let x = LANE_X + index as f64 * SCREEN_PITCH;
        // Past the cap the last block stands down for an ellipsis: the frame
        // stays the same width, the interior admits it isn't showing everything.
        if over && index == glyphs.len() - 1 {
            ink.colorWithAlphaComponent(0.5).set();
            for dot in 0..3 {
                fill_oval(x + dot as f64 * 1.15 - 0.05, 6.45, 1.1, 1.1);
            }
            continue;
        }
        match glyph {
            // Pulsing rather than blinking: the block never leaves, it just
            // breathes. A gap in a row of blocks reads as a job that vanished,
            // and at this size a hard blink is the most distracting thing on
            // the menu bar.
            Glyph::Run => {
                ink.colorWithAlphaComponent(if state.blink_on { 1.0 } else { 0.55 })
                    .set();
                fill_rounded(x, 4.3, 2.6, 5.4, 0.6);
            }
            Glyph::Queued => {
                ink.colorWithAlphaComponent(DIM).set();
                let outline = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                    rect(x + 0.35, 4.65, 1.9, 4.7),
                    0.5 * S,
                    0.5 * S,
                );
                outline.setLineWidth(0.9 * S);
                outline.stroke();
            }
            Glyph::Failed => {
                red().set();
                fill_rounded(x, 4.3, 2.6, 5.4, 0.6);
            }
        }
    }
}

fn draw_equalizer(state: &IconState, ink: &NSColor) {
    let (glyphs, over) = lane(state, EQUALIZER_CAP);
    let slots = glyphs.len() + over as usize;
    let running = state.running > 0;
    let errors = state.failed > 0;

    if errors && !running {
        red().set();
    } else {
        ink.set();
    }
    stroke_chevron(2.9);

    if slots == 0 {
        ink.colorWithAlphaComponent(DIM).set();
        underscore(9.4);
        return;
    }

    for (index, glyph) in glyphs.iter().enumerate() {
        let x = 10.6 + index as f64 * 3.0;
        match glyph {
            Glyph::Run => {
                // Two heights alternating on the blink clock: motion without
                // a blackout frame, out of phase with the neighbouring bar.
                ink.colorWithAlphaComponent(0.35).set();
                fill_rounded(x, 3.2, 2.0, 6.2, 1.0);
                let tall = (index % 2 == 0) == state.blink_on;
                ink.set();
                fill_rounded(x, 3.2, 2.0, if tall { 7.6 } else { 6.2 }, 1.0);
            }
            Glyph::Queued => {
                ink.colorWithAlphaComponent(0.35).set();
                fill_rounded(x, 3.2, 2.0, 3.4, 1.0);
            }
            // A bar at the running height, in red — steady, not bobbing:
            // nothing is moving any more.
            Glyph::Failed => {
                red().set();
                fill_rounded(x, 3.2, 2.0, 6.2, 1.0);
            }
        }
    }
    if over {
        ink.colorWithAlphaComponent(DIM).set();
        stroke_plus(10.6 + glyphs.len() as f64 * 3.0 + 0.5, 6.4);
    }
}
