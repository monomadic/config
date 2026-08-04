//! The menu bar item's drawn contents: an SF Symbol glyph and the uptime
//! beside it, as one image.
//!
//! There are no point constants here. The glyph is set in the menu bar font
//! (`NSFont::menuBarFontOfSize(0.0)`), the value in the same 0.84em compact
//! size `free-disk-space-widget` uses for its Icon and Text style, and the
//! canvas is the status bar's own thickness. Change the system text size and
//! the widget follows.
//!
//! Drawing the glyph and the text into a single image rather than leaving the
//! value in the button's title is what keeps the item tight: a status item
//! button carrying both a title and an image pads generously around each, and
//! the glyph's own left side bearing lands on top of that.

use objc2::AnyThread;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_app_kit::{
    NSColor, NSFont, NSFontAttributeName, NSFontWeightRegular, NSForegroundColorAttributeName,
    NSImage, NSStatusBar, NSStringDrawing,
};
use objc2_core_foundation::CFAttributedString;
use objc2_core_text::{CTLine, CTLineBoundsOptions};
use objc2_foundation::{
    NSMutableAttributedString, NSMutableDictionary, NSPoint, NSRect, NSSize, NSString,
};

/// The value is set a little smaller than the glyph, the same ratio the disk
/// widget's compact styles use, so the two widgets read as one set.
const COMPACT_TEXT_RATIO: f64 = 0.84;

/// The menu bar font. Size comes from the system, not from us.
fn glyph_font() -> Retained<NSFont> {
    NSFont::menuBarFontOfSize(0.0)
}

/// The value font: monospaced digits, so the item keeps one width as the hours
/// tick over.
fn text_font() -> Retained<NSFont> {
    let size = glyph_font().pointSize() * COMPACT_TEXT_RATIO;
    NSFont::monospacedDigitSystemFontOfSize_weight(size, unsafe { NSFontWeightRegular })
}

/// Glyph on the left, value on the right, vertically centred in the status
/// bar. A template image, so macOS tints it to match the menu bar in either
/// appearance; block-based, so AppKit re-renders it at the current backing
/// scale.
pub fn icon_text_image(glyph: &str, text: &str) -> Retained<NSImage> {
    let glyph_font = glyph_font();
    let text_font = text_font();
    let em = glyph_font.pointSize();
    let height = NSStatusBar::systemStatusBar().thickness();
    let horizontal_gap = (em * 0.35).round();

    let ink = NSColor::blackColor();

    // Measure the glyph by its *ink*, not its advance: SF Symbol glyphs carry
    // side bearings, and left as slack they show up as dead space in the menu
    // bar. The canvas is the inked width, and the draw origin is shifted by the
    // bearing so the mark starts hard against the edge.
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

    let text_x = glyph_ink.size.width.ceil() + horizontal_gap;
    let width = text_x + text_size.width.ceil();
    let text_y = ((height - text_size.height) / 2.0).round();

    let handler = block2::RcBlock::new(move |_bounds: NSRect| -> objc2::runtime::Bool {
        unsafe {
            glyph.drawAtPoint_withAttributes(
                NSPoint {
                    x: -glyph_ink.origin.x,
                    y: ((height - glyph_size.height) / 2.0).round(),
                },
                Some(&glyph_attrs),
            );
            text.drawAtPoint_withAttributes(NSPoint { x: text_x, y: text_y }, Some(&text_attrs));
        }

        objc2::runtime::Bool::YES
    });

    let image =
        NSImage::imageWithSize_flipped_drawingHandler(NSSize { width, height }, false, &handler);
    image.setTemplate(true);
    image
}

/// The glyph-path bounds of a run — where the ink actually lands, as opposed to
/// the typographic box the advance width describes. `NSAttributedString`
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
