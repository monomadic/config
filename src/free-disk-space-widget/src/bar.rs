//! Menu bar text and the progress-bar image.
//!
//! There are no point constants here. The font is whatever macOS says the menu
//! bar font is (`NSFont::menuBarFontOfSize(0.0)`), the canvas is one text line
//! tall but never taller than the status bar itself
//! (`NSStatusBar::thickness()`), and the bar's own size is expressed as ratios
//! of the font's metrics. Change the system text size and the widget follows.

use objc2::AnyThread;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_app_kit::{
    NSBezierPath, NSColor, NSFont, NSFontAttributeName, NSFontWeightRegular,
    NSForegroundColorAttributeName, NSImage, NSStatusBar, NSStringDrawing,
};
use objc2_core_foundation::CFAttributedString;
use objc2_core_text::{CTLine, CTLineBoundsOptions};
use objc2_foundation::{
    NSMutableAttributedString, NSMutableDictionary, NSPoint, NSRect, NSSize, NSString,
};

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

/// The bar's three segments as 0..1 fractions of capacity, left to right:
/// used space drawn solid, purgeable space translucent, and whatever remains
/// is the dim track — the genuinely free part.
#[derive(Clone, Copy)]
pub struct Fill {
    pub used: f64,
    pub purgeable: f64,
}

/// Extra alpha for the purgeable segment. Drawn over the 0.3 track it
/// composites to roughly 0.55 — clearly brighter than free, clearly dimmer
/// than used.
const PURGEABLE_ALPHA: f64 = 0.35;

/// One rounded pill over another: track at 0.3, purgeable segment translucent,
/// used segment solid. Pills below one bar-height degenerate into lopsided
/// blobs, so that is the floor for any non-empty segment.
fn draw_segments(fill: Fill, ink: &NSColor, x: f64, y: f64, width: f64, height: f64) {
    let radius = height / 2.0;
    let pill = |w: f64| {
        NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(rect(x, y, w, height), radius, radius)
            .fill();
    };

    ink.colorWithAlphaComponent(0.3).set();
    pill(width);

    let used = fill.used.clamp(0.0, 1.0);
    let through_purgeable = (used + fill.purgeable.max(0.0)).clamp(0.0, 1.0);
    if through_purgeable > used {
        ink.colorWithAlphaComponent(PURGEABLE_ALPHA).set();
        pill((width * through_purgeable).max(height));
    }
    if used > 0.0 {
        ink.set();
        pill((width * used).max(height));
    }
}

/// A rounded track with a rounded fill, optionally with a glyph drawn to its
/// left. A template image, so macOS tints it to match the menu bar in either
/// appearance. Block-based, so AppKit re-renders at the current backing scale
/// and appearance.
///
/// Drawing the glyph into the image rather than leaving it in the button's
/// title is what keeps the item tight: a status item button that carries both
/// a title and an image pads generously around each, and the glyph's own left
/// side bearing lands on top of that.
pub fn bar_image(fill: Fill, glyph: Option<&str>) -> Retained<NSImage> {
    let font = font();
    let em = font.pointSize();

    // One text line tall, clipped to the status bar so the image can never
    // force the item taller than the bar it sits in.
    let line_height = (font.ascender() - font.descender()).round();
    let height = line_height.min(NSStatusBar::systemStatusBar().thickness());
    let bar_width = (em * 3.0).round();
    let bar_height = (font.xHeight() * 0.75).round().max(3.0);

    let ink = NSColor::blackColor();

    // Measure the glyph by its *ink*, not its advance: SF Symbol glyphs carry
    // side bearings, and left as slack they show up as dead space in the menu
    // bar. The canvas is the inked width, and the draw origin is shifted by
    // the bearing so the mark starts hard against the edge.
    let glyph = glyph.map(|glyph| {
        let attrs = glyph_attributes(&font, &ink);
        let text = NSString::from_str(glyph);
        let typographic = unsafe { text.sizeWithAttributes(Some(&attrs)) };
        let ink_bounds = ink_bounds(&text, &attrs).unwrap_or(NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            size: typographic,
        });
        (text, attrs, typographic, ink_bounds)
    });
    let gap = if glyph.is_some() {
        (em * 0.35).round()
    } else {
        0.0
    };
    let glyph_width = glyph
        .as_ref()
        .map(|(_, _, _, ink)| ink.size.width.ceil())
        .unwrap_or(0.0);
    let bar_x = glyph_width + gap;
    let width = bar_x + bar_width;

    let handler = block2::RcBlock::new(move |_bounds: NSRect| -> objc2::runtime::Bool {
        if let Some((text, attrs, typographic, ink_bounds)) = &glyph {
            unsafe {
                text.drawAtPoint_withAttributes(
                    NSPoint {
                        x: -ink_bounds.origin.x,
                        y: ((height - typographic.height) / 2.0).round(),
                    },
                    Some(attrs),
                )
            };
        }

        let y = ((height - bar_height) / 2.0).round();
        draw_segments(fill, &ink, bar_x, y, bar_width, bar_height);

        objc2::runtime::Bool::YES
    });

    let image =
        NSImage::imageWithSize_flipped_drawingHandler(NSSize { width, height }, false, &handler);
    image.setTemplate(true);
    image
}

/// A compact all-in-one status image: full-size disk glyph on the left, then
/// a smaller value over a bar on the right. Keeping all three marks in one
/// image avoids AppKit's roomy image/title gap, while stacking the value uses
/// menu-bar height that would otherwise go unused.
pub fn stacked_image(fill: Fill, glyph: &str, text: &str) -> Retained<NSImage> {
    compact_image(Some(fill), glyph, text)
}

/// The same full-size glyph and compact text used by `stacked_image`, without
/// the bar. Sharing this renderer keeps Icon and Text visually matched to the
/// stacked mode rather than letting AppKit size and space a separate title.
pub fn icon_text_image(glyph: &str, text: &str) -> Retained<NSImage> {
    compact_image(None, glyph, text)
}

fn compact_image(fill: Option<Fill>, glyph: &str, text: &str) -> Retained<NSImage> {
    let glyph_font = font();
    let em = glyph_font.pointSize();
    let text_font =
        NSFont::monospacedDigitSystemFontOfSize_weight(em * 0.84, unsafe { NSFontWeightRegular });
    let height = NSStatusBar::systemStatusBar().thickness();
    let bar_height = fill
        .map(|_| (text_font.xHeight() * 0.55).round().max(3.0))
        .unwrap_or(0.0);
    let vertical_gap = fill.map(|_| (em * 0.08).round().max(1.0)).unwrap_or(0.0);
    let horizontal_gap = (em * 0.35).round();

    let ink = NSColor::blackColor();

    let glyph_attrs = glyph_attributes(&glyph_font, &ink);
    let glyph = NSString::from_str(glyph);
    let glyph_size = unsafe { glyph.sizeWithAttributes(Some(&glyph_attrs)) };
    let glyph_ink = ink_bounds(&glyph, &glyph_attrs).unwrap_or(NSRect {
        origin: NSPoint { x: 0.0, y: 0.0 },
        size: glyph_size,
    });

    let text_attrs = glyph_attributes(&text_font, &ink);
    let text = NSString::from_str(text);
    let text_size = unsafe { text.sizeWithAttributes(Some(&text_attrs)) };
    let text_width = text_size.width.ceil();
    let column_width = if fill.is_some() {
        text_width.max((em * 2.0).round())
    } else {
        text_width
    };
    let column_x = glyph_ink.size.width.ceil() + horizontal_gap;
    let width = column_x + column_width;
    let content_height = text_size.height + vertical_gap + bar_height;
    let content_y = ((height - content_height) / 2.0).round();

    let handler = block2::RcBlock::new(move |_bounds: NSRect| -> objc2::runtime::Bool {
        unsafe {
            glyph.drawAtPoint_withAttributes(
                NSPoint {
                    x: -glyph_ink.origin.x,
                    y: ((height - glyph_size.height) / 2.0).round(),
                },
                Some(&glyph_attrs),
            );
            text.drawAtPoint_withAttributes(
                NSPoint {
                    x: column_x,
                    y: content_y + bar_height + vertical_gap,
                },
                Some(&text_attrs),
            );
        }

        if let Some(fill) = fill {
            draw_segments(fill, &ink, column_x, content_y, column_width, bar_height);
        }

        objc2::runtime::Bool::YES
    });

    let image =
        NSImage::imageWithSize_flipped_drawingHandler(NSSize { width, height }, false, &handler);
    image.setTemplate(true);
    image
}

/// The glyph-path bounds of a run — where the ink actually lands, as opposed
/// to the typographic box the advance width describes. `NSAttributedString`
/// bridges to `CFAttributedString`, so CoreText can measure it directly.
fn ink_bounds(text: &NSString, attrs: &NSMutableDictionary<NSString, AnyObject>) -> Option<NSRect> {
    let string = unsafe {
        NSMutableAttributedString::initWithString_attributes(
            NSMutableAttributedString::alloc(),
            text,
            Some(attrs),
        )
    };
    // SAFETY: NSAttributedString is toll-free bridged to CFAttributedString.
    let bridged: &CFAttributedString =
        unsafe { &*(Retained::as_ptr(&string) as *const CFAttributedString) };
    let line = unsafe { CTLine::with_attributed_string(bridged) };
    let bounds = unsafe { line.bounds_with_options(CTLineBoundsOptions::UseGlyphPathBounds) };
    (bounds.size.width > 0.0).then_some(bounds)
}

fn glyph_attributes(
    font: &NSFont,
    color: &NSColor,
) -> Retained<NSMutableDictionary<NSString, AnyObject>> {
    let attrs = NSMutableDictionary::<NSString, AnyObject>::new();
    unsafe {
        attrs.setObject_forKey(font, ProtocolObject::from_ref(NSFontAttributeName));
        attrs.setObject_forKey(
            color,
            ProtocolObject::from_ref(NSForegroundColorAttributeName),
        );
    }
    attrs
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
    NSRect {
        origin: NSPoint { x, y },
        size: NSSize { width, height },
    }
}
