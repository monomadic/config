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

pub fn attributed_title(
    text: &str,
    color: Option<&NSColor>,
) -> Retained<NSMutableAttributedString> {
    let attrs = NSMutableDictionary::<NSString, AnyObject>::new();
    unsafe {
        attrs.setObject_forKey(&font(), ProtocolObject::from_ref(NSFontAttributeName));
        if let Some(color) = color {
            attrs.setObject_forKey(
                color,
                ProtocolObject::from_ref(NSForegroundColorAttributeName),
            );
        }
    }

    unsafe {
        NSMutableAttributedString::initWithString_attributes(
            NSMutableAttributedString::alloc(),
            &NSString::from_str(text),
            Some(&attrs),
        )
    }
}

/// A rounded track with a rounded fill, optionally with a glyph drawn to its
/// left. With no colour it is a template image, so macOS tints it to match the
/// menu bar in either appearance; a colour (used when space runs low) is drawn
/// as-is. Block-based, so AppKit re-renders at the current backing scale and
/// appearance.
///
/// Drawing the glyph into the image rather than leaving it in the button's
/// title is what keeps the item tight: a status item button that carries both
/// a title and an image pads generously around each, and the glyph's own left
/// side bearing lands on top of that.
pub fn bar_image(ratio: f64, color: Option<&NSColor>, glyph: Option<&str>) -> Retained<NSImage> {
    let font = font();
    let em = font.pointSize();

    // One text line tall, clipped to the status bar so the image can never
    // force the item taller than the bar it sits in.
    let line_height = (font.ascender() - font.descender()).round();
    let height = line_height.min(NSStatusBar::systemStatusBar().thickness());
    let bar_width = (em * 3.0).round();
    let bar_height = (font.xHeight() * 0.75).round().max(3.0);
    let radius = bar_height / 2.0;

    let is_template = color.is_none();
    let ink = color
        .map(Retained::from)
        .unwrap_or_else(NSColor::blackColor);
    let ratio = ratio.clamp(0.0, 1.0);

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

        ink.colorWithAlphaComponent(0.3).set();
        NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
            rect(bar_x, y, bar_width, bar_height),
            radius,
            radius,
        )
        .fill();

        if ratio > 0.0 {
            // Below one bar-height the rounded fill degenerates into a
            // lopsided blob, so that is the floor.
            let filled = (bar_width * ratio).max(bar_height);
            ink.set();
            NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                rect(bar_x, y, filled, bar_height),
                radius,
                radius,
            )
            .fill();
        }

        objc2::runtime::Bool::YES
    });

    let image =
        NSImage::imageWithSize_flipped_drawingHandler(NSSize { width, height }, false, &handler);
    image.setTemplate(is_template);
    image
}

/// A compact all-in-one status image: full-size disk glyph on the left, then
/// a smaller value over a bar on the right. Keeping all three marks in one
/// image avoids AppKit's roomy image/title gap, while stacking the value uses
/// menu-bar height that would otherwise go unused.
pub fn stacked_image(
    ratio: f64,
    color: Option<&NSColor>,
    glyph: &str,
    text: &str,
) -> Retained<NSImage> {
    compact_image(Some(ratio), color, glyph, text)
}

/// The same full-size glyph and compact text used by `stacked_image`, without
/// the bar. Sharing this renderer keeps Icon and Text visually matched to the
/// stacked mode rather than letting AppKit size and space a separate title.
pub fn icon_text_image(color: Option<&NSColor>, glyph: &str, text: &str) -> Retained<NSImage> {
    compact_image(None, color, glyph, text)
}

fn compact_image(
    ratio: Option<f64>,
    color: Option<&NSColor>,
    glyph: &str,
    text: &str,
) -> Retained<NSImage> {
    let glyph_font = font();
    let em = glyph_font.pointSize();
    let text_font =
        NSFont::monospacedDigitSystemFontOfSize_weight(em * 0.84, unsafe { NSFontWeightRegular });
    let height = NSStatusBar::systemStatusBar().thickness();
    let bar_height = ratio
        .map(|_| (text_font.xHeight() * 0.55).round().max(3.0))
        .unwrap_or(0.0);
    let vertical_gap = ratio.map(|_| (em * 0.08).round().max(1.0)).unwrap_or(0.0);
    let horizontal_gap = (em * 0.35).round();

    let is_template = color.is_none();
    let ink = color
        .map(Retained::from)
        .unwrap_or_else(NSColor::blackColor);
    let ratio = ratio.map(|ratio| ratio.clamp(0.0, 1.0));

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
    let column_width = if ratio.is_some() {
        text_width.max((em * 2.0).round())
    } else {
        text_width
    };
    let column_x = glyph_ink.size.width.ceil() + horizontal_gap;
    let width = column_x + column_width;
    let content_height = text_size.height + vertical_gap + bar_height;
    let content_y = ((height - content_height) / 2.0).round();
    let radius = bar_height / 2.0;

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

        if let Some(ratio) = ratio {
            ink.colorWithAlphaComponent(0.3).set();
            NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                rect(column_x, content_y, column_width, bar_height),
                radius,
                radius,
            )
            .fill();

            if ratio > 0.0 {
                let filled = (column_width * ratio).max(bar_height);
                ink.set();
                NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                    rect(column_x, content_y, filled, bar_height),
                    radius,
                    radius,
                )
                .fill();
            }
        }

        objc2::runtime::Bool::YES
    });

    let image =
        NSImage::imageWithSize_flipped_drawingHandler(NSSize { width, height }, false, &handler);
    image.setTemplate(is_template);
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
