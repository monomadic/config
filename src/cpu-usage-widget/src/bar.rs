//! Menu bar text and the drawn load images.
//!
//! There are no point constants here. The font is whatever macOS says the menu
//! bar font is (`NSFont::menuBarFontOfSize(0.0)`), the canvas is exactly the
//! status bar's own thickness, and every bar, gap and pad is expressed as a
//! ratio of the font's metrics. Change the system text size and the widget
//! follows. Same approach as `free-disk-space-widget`.
//!
//! Everything drawn here is a template image, so macOS tints it to match the
//! menu bar in light and dark appearance.

use objc2::AnyThread;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_app_kit::{
    NSBezierPath, NSColor, NSFont, NSFontAttributeName, NSFontWeightRegular,
    NSForegroundColorAttributeName, NSImage, NSStatusBar, NSStringDrawing,
};
use objc2_foundation::{
    NSMutableAttributedString, NSMutableDictionary, NSPoint, NSRect, NSSize, NSString,
};

/// One meter's worth of load. `columns` is what the per-core layout draws — a
/// list for the CPU, a single entry for the GPU — and `value` is the one number
/// every other layout shows.
pub struct Gauge {
    /// A single letter, drawn only when more than one gauge shares the item.
    pub label: &'static str,
    pub columns: Vec<f64>,
    pub value: f64,
}

/// Alpha for the unfilled part of any track. Low enough to read as a groove,
/// high enough to still show the meter's extent when the load is zero.
const TRACK_ALPHA: f64 = 0.28;

/// The menu bar font, with monospaced digits so the title keeps one width as
/// the number changes. Size comes from the system, not from us.
pub fn font() -> Retained<NSFont> {
    let size = NSFont::menuBarFontOfSize(0.0).pointSize();
    NSFont::monospacedDigitSystemFontOfSize_weight(size, unsafe { NSFontWeightRegular })
}

pub fn attributed_title(text: &str) -> Retained<NSMutableAttributedString> {
    let attrs = NSMutableDictionary::<NSString, AnyObject>::new();
    unsafe {
        attrs.setObject_forKey(&font(), ProtocolObject::from_ref(NSFontAttributeName));
    }
    unsafe {
        NSMutableAttributedString::initWithString_attributes(
            NSMutableAttributedString::alloc(),
            &NSString::from_str(text),
            Some(&attrs),
        )
    }
}

pub fn percent(ratio: f64) -> String {
    format!("{:.0}%", ratio.clamp(0.0, 1.0) * 100.0)
}

/// One vertical bar per column: a faint full-height track with a solid fill
/// rising from the bottom. Gauges are separated by a wider gap than the
/// columns within one, so a trailing GPU column reads as its own meter rather
/// than as one more core.
pub fn columns_image(gauges: &[Gauge]) -> Retained<NSImage> {
    let font = font();
    let em = font.pointSize();
    let height = status_bar_height();

    let width = (em * 0.18).round().max(2.0);
    let gap = width;
    let group_gap = (width + gap) * 2.0;
    let radius = width / 2.0;

    // One text line tall, less a hair of padding top and bottom.
    let pad = (em * 0.1).round().max(1.0);
    let track = (line_height(&font).min(height) - pad * 2.0).max(width * 2.0);
    let bottom = ((height - track) / 2.0).round();

    // Resolve every column to an absolute x before the closure, so drawing is
    // a flat walk and the gauge grouping never has to be re-derived.
    let mut placed: Vec<(f64, f64)> = Vec::new();
    let mut x = 0.0;
    for (index, gauge) in gauges.iter().enumerate() {
        if index > 0 {
            x += group_gap;
        }
        for (column, ratio) in gauge.columns.iter().enumerate() {
            if column > 0 {
                x += gap;
            }
            placed.push((x, ratio.clamp(0.0, 1.0)));
            x += width;
        }
    }
    let total_width = x.max(width);

    image(total_width, height, move |ink| {
        for (x, ratio) in &placed {
            ink.colorWithAlphaComponent(TRACK_ALPHA).set();
            pill(rect(*x, bottom, width, track), radius);

            if *ratio > 0.0 {
                // Columns shorter than their own width degenerate into
                // lopsided blobs, so that is the floor for any live core.
                let fill = (track * ratio).max(width);
                ink.set();
                pill(rect(*x, bottom, width, fill), radius);
            }
        }
    })
}

/// A rounded track with a rounded fill, one row per gauge. With `with_text`
/// the percentage is drawn into the image beside its own bar, and multi-gauge
/// rows pick up a leading letter so it is clear which row is which.
///
/// Drawing text into the image rather than leaving it in the button's title is
/// what keeps a stacked item tight: a status item button carrying both a title
/// and an image pads generously around each.
pub fn bars_image(gauges: &[Gauge], with_text: bool) -> Retained<NSImage> {
    let font = font();
    let em = font.pointSize();
    let height = status_bar_height();
    let stacked = gauges.len() > 1;

    let text_font = if stacked {
        NSFont::monospacedDigitSystemFontOfSize_weight((em * 0.78).round().max(8.0), unsafe {
            NSFontWeightRegular
        })
    } else {
        font.clone()
    };

    let bar_width = (em * 3.0).round();
    let bar_height = (text_font.xHeight() * 0.72).round().max(3.0);
    let radius = bar_height / 2.0;
    let gap = (em * 0.3).round().max(2.0);

    // Widths come from the widest value the meter can ever show, not from the
    // current one, so the item never twitches as the load changes.
    let attrs = text_attributes(&text_font);
    let value_width = with_text.then(|| measure("100%", &attrs).width.ceil());
    let label_width = (with_text && stacked).then(|| measure("C", &attrs).width.ceil());
    let text_height = measure("100%", &attrs).height;

    let bar_x = label_width.map_or(0.0, |width| width + gap);
    let width = bar_x + bar_width + value_width.map_or(0.0, |value| gap + value);

    let row_height = if with_text {
        bar_height.max(text_height)
    } else {
        bar_height
    };
    let row_gap = (em * 0.16).round().max(1.0);
    let rows = gauges.len().max(1) as f64;
    let content = rows * row_height + (rows - 1.0) * row_gap;
    let bottom = ((height - content) / 2.0).round();

    // Top row first on screen means last in bottom-up image coordinates.
    struct Row {
        y: f64,
        fill: f64,
        value: String,
        label: Option<String>,
    }
    let rows: Vec<Row> = gauges
        .iter()
        .rev()
        .enumerate()
        .map(|(index, gauge)| Row {
            y: bottom + index as f64 * (row_height + row_gap),
            fill: gauge.value.clamp(0.0, 1.0),
            value: percent(gauge.value),
            label: label_width.map(|_| gauge.label.to_string()),
        })
        .collect();

    image(width, height, move |ink| {
        for row in &rows {
            let bar_y = row.y + ((row_height - bar_height) / 2.0).round();
            ink.colorWithAlphaComponent(TRACK_ALPHA).set();
            pill(rect(bar_x, bar_y, bar_width, bar_height), radius);
            if row.fill > 0.0 {
                ink.set();
                pill(
                    rect(
                        bar_x,
                        bar_y,
                        (bar_width * row.fill).max(bar_height),
                        bar_height,
                    ),
                    radius,
                );
            }

            let text_y = row.y + ((row_height - text_height) / 2.0).round();
            if let Some(label) = &row.label {
                draw(
                    label,
                    bar_x - gap - measure(label, &attrs).width,
                    text_y,
                    &attrs,
                );
            }
            if let Some(value_width) = value_width {
                let right = bar_x + bar_width + gap + value_width;
                draw(
                    &row.value,
                    right - measure(&row.value, &attrs).width,
                    text_y,
                    &attrs,
                );
            }
        }
    })
}

/// Percentages alone, one labelled row per gauge. Only used when more than one
/// gauge shares the item — a lone percentage is just the button's title.
pub fn text_image(gauges: &[Gauge]) -> Retained<NSImage> {
    let font = font();
    let em = font.pointSize();
    let height = status_bar_height();

    let text_font =
        NSFont::monospacedDigitSystemFontOfSize_weight((em * 0.78).round().max(8.0), unsafe {
            NSFontWeightRegular
        });
    let attrs = text_attributes(&text_font);

    let gap = (em * 0.22).round().max(1.0);
    let label_width = measure("C", &attrs).width.ceil();
    let value_width = measure("100%", &attrs).width.ceil();
    let width = label_width + gap + value_width;
    let row_height = measure("100%", &attrs).height;
    let row_gap = (em * 0.1).round().max(1.0);
    let rows = gauges.len().max(1) as f64;
    let bottom = ((height - (rows * row_height + (rows - 1.0) * row_gap)) / 2.0).round();

    let rows: Vec<(f64, String, String)> = gauges
        .iter()
        .rev()
        .enumerate()
        .map(|(index, gauge)| {
            (
                bottom + index as f64 * (row_height + row_gap),
                gauge.label.to_string(),
                percent(gauge.value),
            )
        })
        .collect();

    image(width, height, move |_ink| {
        for (y, label, value) in &rows {
            draw(label, 0.0, *y, &attrs);
            draw(value, width - measure(value, &attrs).width, *y, &attrs);
        }
    })
}

/// The status bar's own height. Sizing the canvas to it means the image can
/// never force the item taller than the bar it sits in, and leaves the full
/// height available for stacked rows.
fn status_bar_height() -> f64 {
    NSStatusBar::systemStatusBar().thickness()
}

fn line_height(font: &NSFont) -> f64 {
    (font.ascender() - font.descender()).round()
}

/// Block-based image creation, so AppKit re-renders at the current backing
/// scale and appearance. The handler is handed the ink colour to draw with;
/// black plus `setTemplate` is what lets macOS tint the result.
fn image(width: f64, height: f64, draw: impl Fn(&NSColor) + 'static) -> Retained<NSImage> {
    let handler = block2::RcBlock::new(move |_bounds: NSRect| -> objc2::runtime::Bool {
        draw(&NSColor::blackColor());
        objc2::runtime::Bool::YES
    });
    let image =
        NSImage::imageWithSize_flipped_drawingHandler(NSSize { width, height }, false, &handler);
    image.setTemplate(true);
    image
}

fn pill(rect: NSRect, radius: f64) {
    NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(rect, radius, radius).fill();
}

fn text_attributes(font: &NSFont) -> Retained<NSMutableDictionary<NSString, AnyObject>> {
    let attrs = NSMutableDictionary::<NSString, AnyObject>::new();
    unsafe {
        attrs.setObject_forKey(font, ProtocolObject::from_ref(NSFontAttributeName));
        attrs.setObject_forKey(
            &NSColor::blackColor(),
            ProtocolObject::from_ref(NSForegroundColorAttributeName),
        );
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

fn rect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
    NSRect {
        origin: NSPoint { x, y },
        size: NSSize { width, height },
    }
}
