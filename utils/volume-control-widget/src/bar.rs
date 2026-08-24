//! The status item image: a speaker glyph, optionally a pill bar showing the
//! active output's volume, and optionally a small route tag or percentage.
//!
//! Everything renders into one image so AppKit never inserts its roomy
//! image/title gap, exactly as the sibling widgets do. In the ordinary state
//! the image is a template, so macOS tints it for the menu bar; the alert
//! states (expected device missing, fallback caught) render pre-tinted in
//! red or orange and opt out of templating.
//!
//! No point constants: sizes derive from the menu bar font and the status
//! bar's own thickness, so the widget follows the system text size.

use objc2::Message;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_app_kit::{
    NSBezierPath, NSColor, NSCompositingOperation, NSFont, NSFontAttributeName,
    NSFontWeightMedium, NSForegroundColorAttributeName, NSImage, NSImageSymbolConfiguration,
    NSRectFillUsingOperation, NSStatusBar, NSStringDrawing,
};
use objc2_foundation::{NSMutableDictionary, NSPoint, NSRect, NSSize, NSString};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tint {
    Normal,
    Red,
    Orange,
}

impl Tint {
    fn ink(self) -> Retained<NSColor> {
        match self {
            Tint::Normal => NSColor::blackColor(),
            Tint::Red => NSColor::systemRedColor(),
            Tint::Orange => NSColor::systemOrangeColor(),
        }
    }
}

#[derive(Clone, Copy)]
pub struct BarFill {
    /// 0..1 of the active output's volume setting.
    pub level: f64,
    /// Muted routes draw their fill translucent — set, but not passing.
    pub dim: bool,
}

pub struct Chip {
    /// SF Symbol name for the state glyph.
    pub symbol: &'static str,
    pub bar: Option<BarFill>,
    /// Route tag (`DJM`, `INT`) or a percentage, drawn after the bar.
    pub text: Option<String>,
    pub tint: Tint,
}

pub fn chip_image(chip: Chip) -> Retained<NSImage> {
    let font = NSFont::menuBarFontOfSize(0.0);
    let em = font.pointSize();
    let height = NSStatusBar::systemStatusBar().thickness();

    let ink = chip.tint.ink();
    let symbol = tinted_symbol(chip.symbol, em, &ink);
    let symbol_size = symbol.as_ref().map(|image| image.size()).unwrap_or_default();

    let gap = (em * 0.35).round();
    let bar_width = chip.bar.map(|_| (em * 3.0).round()).unwrap_or(0.0);
    let bar_height = (font.xHeight() * 0.72).round().max(3.0);

    let text_font = NSFont::monospacedDigitSystemFontOfSize_weight((em * 0.82).round(), unsafe {
        NSFontWeightMedium
    });
    let text = chip.text.map(|text| {
        let attrs = attributes(&text_font, &ink);
        let string = NSString::from_str(&text);
        let size = unsafe { string.sizeWithAttributes(Some(&attrs)) };
        (string, attrs, size)
    });

    let mut width = symbol_size.width.ceil();
    if bar_width > 0.0 {
        width += gap + bar_width;
    }
    if let Some((_, _, size)) = &text {
        width += gap + size.width.ceil();
    }

    let bar = chip.bar;
    let ink_for_draw = ink.clone();
    let handler = block2::RcBlock::new(move |_bounds: NSRect| -> objc2::runtime::Bool {
        let mut x = 0.0;
        if let Some(symbol) = &symbol {
            symbol.drawInRect_fromRect_operation_fraction(
                rect(
                    0.0,
                    ((height - symbol_size.height) / 2.0).round(),
                    symbol_size.width,
                    symbol_size.height,
                ),
                NSRect::ZERO,
                NSCompositingOperation::SourceOver,
                1.0,
            );
            x += symbol_size.width.ceil();
        }

        if let Some(fill) = bar {
            x += gap;
            let y = ((height - bar_height) / 2.0).round();
            let radius = bar_height / 2.0;
            let pill = |w: f64| {
                NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                    rect(x, y, w, bar_height),
                    radius,
                    radius,
                )
                .fill();
            };
            ink_for_draw.colorWithAlphaComponent(0.3).set();
            pill(bar_width);
            let level = fill.level.clamp(0.0, 1.0);
            if level > 0.0 {
                let alpha = if fill.dim { 0.55 } else { 1.0 };
                ink_for_draw.colorWithAlphaComponent(alpha).set();
                pill((bar_width * level).max(bar_height));
            }
            x += bar_width;
        }

        if let Some((string, attrs, size)) = &text {
            x += gap;
            unsafe {
                string.drawAtPoint_withAttributes(
                    NSPoint {
                        x,
                        y: ((height - size.height) / 2.0).round(),
                    },
                    Some(attrs),
                )
            };
        }

        objc2::runtime::Bool::YES
    });

    let image = NSImage::imageWithSize_flipped_drawingHandler(
        NSSize {
            width: width.max(1.0),
            height,
        },
        false,
        &handler,
    );
    image.setTemplate(chip.tint == Tint::Normal);
    image
}

/// An SF Symbol flooded with the ink colour through its own alpha — the same
/// template-style tinting the disk widget uses for its badge, which works for
/// both the black (template) and the alert inks.
fn tinted_symbol(name: &str, point_size: f64, ink: &NSColor) -> Option<Retained<NSImage>> {
    let symbol =
        NSImage::imageWithSystemSymbolName_accessibilityDescription(&NSString::from_str(name), None)?;
    let config = NSImageSymbolConfiguration::configurationWithPointSize_weight(point_size, unsafe {
        objc2_app_kit::NSFontWeightRegular
    });
    let symbol = symbol.imageWithSymbolConfiguration(&config).unwrap_or(symbol);
    let size = symbol.size();
    let ink = ink.retain();
    let handler = block2::RcBlock::new(move |bounds: NSRect| -> objc2::runtime::Bool {
        symbol.drawInRect_fromRect_operation_fraction(
            bounds,
            NSRect::ZERO,
            NSCompositingOperation::SourceOver,
            1.0,
        );
        ink.set();
        NSRectFillUsingOperation(bounds, NSCompositingOperation::SourceAtop);
        objc2::runtime::Bool::YES
    });
    Some(NSImage::imageWithSize_flipped_drawingHandler(
        size, false, &handler,
    ))
}

fn attributes(
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
