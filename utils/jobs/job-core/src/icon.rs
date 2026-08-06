//! The menu bar icon: a terminal prompt in a small screen frame.
//!
//! Idle is a dim frame, dim chevron and a resting underscore. While a job
//! runs the frame and chevron come up to full strength and the underscore
//! becomes a blinking block cursor. Unacknowledged failures turn the chevron
//! red and blink a red `!` in the cursor slot, while the frame stays at full
//! strength. Queued jobs trail as faint dots, and only appear when something
//! is actually waiting.
//!
//! `job-monitor` draws the same icon with a doubled chevron, so two menu bars'
//! worth of job icons can be told apart at a glance — and when its folder is
//! unreachable the whole thing drops to dim with an empty cursor slot, which
//! is deliberately *not* how idle looks.
//!
//! Everything is drawn rather than glyph-based, so it stays crisp at any
//! backing scale. Neutral states are template images that macOS tints to
//! match the menu bar; the error state mixes red with the label colour, so
//! it is drawn as a regular image instead.

use objc2::rc::Retained;
use objc2_app_kit::{
    NSBezierPath, NSColor, NSImage, NSLineCapStyle, NSLineJoinStyle,
};
use objc2_foundation::{NSPoint, NSRect, NSSize};

// Icon geometry, in points. The frame is deliberately snug so the chevron
// can take most of its height and stay legible at menu bar size.
const HEIGHT: f64 = 14.0;
const BASE_WIDTH: f64 = 16.0;
const QUEUE_WIDTH: f64 = 11.0;
const MAX_QUEUE_DOTS: usize = 3;
const DIM: f64 = 0.4;

pub struct IconState {
    pub running: bool,
    pub queued: usize,
    pub errors: bool,
    pub blink_on: bool,
    /// Draw the doubled chevron: this is a monitor, watching someone else's
    /// folder, not the runner that owns it.
    pub remote: bool,
    /// Remote only — false when the folder can't be reached.
    pub connected: bool,
}

impl Default for IconState {
    fn default() -> Self {
        Self {
            running: false,
            queued: 0,
            errors: false,
            blink_on: true,
            remote: false,
            connected: true,
        }
    }
}

pub fn draw(state: &IconState) -> Retained<NSImage> {
    let dots = state.queued.min(MAX_QUEUE_DOTS);
    let width = if dots > 0 { BASE_WIDTH + QUEUE_WIDTH } else { BASE_WIDTH };
    let size = NSSize { width, height: HEIGHT };

    // Template images are drawn in black and tinted by macOS; the error
    // state needs real colour, so it resolves the label colour itself.
    let is_template = !state.errors;
    let ink = if is_template {
        NSColor::blackColor()
    } else {
        NSColor::labelColor()
    };
    let accent = if state.errors {
        NSColor::systemRedColor()
    } else {
        ink.clone()
    };

    let running = state.running;
    let errors = state.errors;
    let blink_on = state.blink_on;
    let remote = state.remote;
    // An unreachable folder knows nothing: no cursor, no queue, everything dim.
    let offline = remote && !state.connected;

    let handler = block2::RcBlock::new(move |_bounds: NSRect| -> objc2::runtime::Bool {
        let frame_color = if (running || errors) && !offline {
            ink.clone()
        } else {
            ink.colorWithAlphaComponent(DIM)
        };
        let chevron_color = if offline {
            ink.colorWithAlphaComponent(DIM)
        } else if errors {
            accent.clone()
        } else if running {
            ink.clone()
        } else {
            ink.colorWithAlphaComponent(DIM)
        };

        // Screen frame.
        frame_color.set();
        let frame = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
            rect(1.0, 1.5, 14.0, 11.0),
            2.4,
            2.4,
        );
        frame.setLineWidth(1.3);
        frame.stroke();

        // The prompt chevron — doubled for a monitor, which is watching a
        // prompt rather than being one. The second stroke sits where the
        // cursor would start, so the two icons stay the same width.
        chevron_color.set();
        let stroke_chevron = |x: f64| {
            let chevron = NSBezierPath::new();
            chevron.moveToPoint(NSPoint { x, y: 9.3 });
            chevron.lineToPoint(NSPoint { x: x + 2.8, y: 7.0 });
            chevron.lineToPoint(NSPoint { x, y: 4.7 });
            chevron.setLineWidth(1.7);
            chevron.setLineCapStyle(NSLineCapStyle::Round);
            chevron.setLineJoinStyle(NSLineJoinStyle::Round);
            chevron.stroke();
        };
        if remote {
            stroke_chevron(2.9);
            stroke_chevron(5.8);
        } else {
            stroke_chevron(4.6);
        }

        // Cursor slot: a blinking `!` when work has failed and nothing is
        // running, a blinking block while a job runs, a resting underscore
        // otherwise. When a job is running *and* older failures are
        // unacknowledged, the red chevron carries the error and the block
        // keeps showing progress.
        if offline {
            // Nothing: an empty slot is the tell that there is no answer from
            // the other end, as against idle's resting underscore.
        } else if errors && !running {
            if blink_on {
                accent.set();
                NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                    rect(9.6, 6.2, 1.6, 3.6),
                    0.8,
                    0.8,
                )
                .fill();
                NSBezierPath::bezierPathWithOvalInRect(rect(9.5, 3.6, 1.8, 1.8)).fill();
            }
        } else if running {
            if blink_on {
                ink.set();
                NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                    rect(9.2, 4.2, 2.2, 5.4),
                    0.5,
                    0.5,
                )
                .fill();
            }
        } else {
            ink.colorWithAlphaComponent(DIM).set();
            NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                rect(9.2, 4.1, 2.6, 1.3),
                0.6,
                0.6,
            )
            .fill();
        }

        // Queued jobs, fading with position.
        for index in 0..dots {
            let alpha = [0.6, 0.4, 0.25][index];
            ink.colorWithAlphaComponent(alpha).set();
            let x = 17.1 + index as f64 * 3.7;
            NSBezierPath::bezierPathWithOvalInRect(rect(x, 5.5, 3.0, 3.0)).fill();
        }

        objc2::runtime::Bool::YES
    });

    let image = NSImage::imageWithSize_flipped_drawingHandler(size, false, &handler);
    image.setTemplate(is_template);
    image
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
    NSRect {
        origin: NSPoint { x, y },
        size: NSSize { width, height },
    }
}
