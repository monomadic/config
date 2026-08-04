//! The menu bar item's drawn contents: an SF Symbol glyph and the uptime, side
//! by side or stacked, as one image.
//!
//! There are no point constants here. Everything is a ratio of the menu bar
//! font (`NSFont::menuBarFontOfSize(0.0)`) or of the status bar's own thickness
//! (`NSStatusBar::thickness()`). Change the system text size and the widget
//! follows.
//!
//! Drawing the glyph and the value into a single image rather than leaving the
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

/// Side by side, the value is set a little smaller than the glyph — the same
/// ratio `free-disk-space-widget` uses for its Icon and Text style, so the two
/// widgets read as one set.
const ROW_TEXT_RATIO: f64 = 0.84;

/// Stacked, both runs have to share the height of the menu bar, so both come
/// down: the glyph enough to leave room, the value a step further so the mark
/// stays the dominant one.
const STACKED_GLYPH_RATIO: f64 = 0.72;
const STACKED_TEXT_RATIO: f64 = 0.58;

/// The menu bar font. Size comes from the system, not from us.
fn menu_bar_font() -> Retained<NSFont> {
    NSFont::menuBarFontOfSize(0.0)
}

/// Monospaced digits, so the item keeps one width as the hours tick over.
fn digit_font(size: f64) -> Retained<NSFont> {
    NSFont::monospacedDigitSystemFontOfSize_weight(size, unsafe { NSFontWeightRegular })
}

/// The plain menu bar title, at full menu bar size, for the text-only style.
pub fn attributed_title(text: &str) -> Retained<NSMutableAttributedString> {
    let attrs = NSMutableDictionary::<NSString, AnyObject>::new();
    unsafe {
        attrs.setObject_forKey(
            &digit_font(menu_bar_font().pointSize()),
            ProtocolObject::from_ref(NSFontAttributeName),
        );
    }

    unsafe {
        NSMutableAttributedString::initWithString_attributes(
            NSMutableAttributedString::alloc(),
            &NSString::from_str(text),
            Some(&attrs),
        )
    }
}

/// A laid-out run: the string, its attributes, and both the box the advance
/// width describes and the box the ink actually lands in.
struct Run {
    text: Retained<NSString>,
    attrs: Retained<NSMutableDictionary<NSString, AnyObject>>,
    typographic: NSSize,
    ink: NSRect,
    descender: f64,
}

impl Run {
    fn new(text: &str, font: &NSFont, color: &NSColor) -> Self {
        let attrs = glyph_attributes(font, color);
        let text = NSString::from_str(text);
        let typographic = unsafe { text.sizeWithAttributes(Some(&attrs)) };
        let ink = ink_bounds(&text, &attrs).unwrap_or(NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            size: typographic,
        });
        Run {
            text,
            attrs,
            typographic,
            ink,
            descender: font.descender(),
        }
    }

    /// Where the mark lands, as opposed to how much room the run claims. SF
    /// Symbol glyphs carry side bearings, and left as slack they show up as
    /// dead space in the menu bar.
    fn ink_width(&self) -> f64 {
        self.ink.size.width.ceil()
    }

    fn draw(&self, x: f64, y: f64) {
        // Shift by the bearing so the mark starts where the caller asked,
        // rather than wherever the advance box happens to put it.
        unsafe {
            self.text.drawAtPoint_withAttributes(
                NSPoint {
                    x: x - self.ink.origin.x,
                    y,
                },
                Some(&self.attrs),
            )
        };
    }

    /// Draw with the *ink's* bottom edge at `y`. `drawAtPoint` takes the
    /// advance box's lower-left corner, which sits a descender below the
    /// baseline; the ink's own offset is measured from that baseline.
    fn draw_ink_bottom(&self, x: f64, y: f64) {
        self.draw(x, y - self.ink.origin.y + self.descender);
    }
}

/// Glyph on the left, value on the right, vertically centred in the status bar.
pub fn icon_text_image(glyph: &str, text: &str) -> Retained<NSImage> {
    let em = menu_bar_font().pointSize();
    let height = NSStatusBar::systemStatusBar().thickness();
    let gap = (em * 0.35).round();
    let ink = NSColor::blackColor();

    let glyph = Run::new(glyph, &menu_bar_font(), &ink);
    let text = Run::new(text, &digit_font(em * ROW_TEXT_RATIO), &ink);

    let text_x = glyph.ink_width() + gap;
    let width = text_x + text.typographic.width.ceil();
    let glyph_y = ((height - glyph.typographic.height) / 2.0).round();
    let text_y = ((height - text.typographic.height) / 2.0).round();

    image(width, height, move || {
        glyph.draw(0.0, glyph_y);
        text.draw(text_x, text_y);
    })
}

/// Glyph above the value, both centred on the same column. Two runs stacked in
/// a 24pt bar leaves each of them little room, so this style sets both well
/// below menu bar size and then, if the pair still overruns the bar, scales
/// them together until they fit — the widget stays legible at any system text
/// size rather than being clipped at large ones.
pub fn stacked_image(glyph: &str, text: &str) -> Retained<NSImage> {
    let em = menu_bar_font().pointSize();
    let height = NSStatusBar::systemStatusBar().thickness();
    let ink = NSColor::blackColor();

    // A hair of air between the two, and the same again above and below.
    let gap = (em * 0.06).round().max(1.0);
    let padding = (em * 0.06).round().max(1.0);

    let mut scale = 1.0;
    let (glyph, text) = loop {
        let glyph = Run::new(
            glyph,
            &NSFont::systemFontOfSize(em * STACKED_GLYPH_RATIO * scale),
            &ink,
        );
        let text = Run::new(text, &digit_font(em * STACKED_TEXT_RATIO * scale), &ink);

        // The glyph is measured by its ink here: an SF Symbol's advance box
        // carries the font's full line height, which stacked would show up as
        // a gap wide enough to push the value out of the bar.
        let content = glyph.ink.size.height + gap + text.typographic.height;
        let room = height - padding * 2.0;
        if content <= room || scale < 0.5 {
            break (glyph, text);
        }
        scale *= room / content;
    };

    let content_height = glyph.ink.size.height + gap + text.typographic.height;
    let width = glyph.ink_width().max(text.typographic.width.ceil());
    let glyph_x = ((width - glyph.ink_width()) / 2.0).round();
    let text_x = ((width - text.typographic.width) / 2.0).round();

    let bottom = ((height - content_height) / 2.0).round();
    let text_y = bottom;
    let glyph_bottom = bottom + text.typographic.height + gap;

    image(width, height, move || {
        glyph.draw_ink_bottom(glyph_x, glyph_bottom);
        text.draw(text_x, text_y);
    })
}

/// A template image, so macOS tints it to match the menu bar in either
/// appearance; block-based, so AppKit re-renders it at the current backing
/// scale.
fn image(width: f64, height: f64, draw: impl Fn() + 'static) -> Retained<NSImage> {
    let handler = block2::RcBlock::new(move |_bounds: NSRect| -> objc2::runtime::Bool {
        draw();
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
