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
use objc2::Message;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2_app_kit::{
    NSBezierPath, NSColor, NSFont, NSFontAttributeName, NSFontWeightRegular,
    NSForegroundColorAttributeName, NSImage, NSMutableParagraphStyle,
    NSAttributedStringAttachmentConveniences, NSCompositingOperation,
    NSParagraphStyleAttributeName, NSRectFillUsingOperation, NSStatusBar, NSStringDrawing,
    NSTextAlignment, NSTextAttachment, NSTextTab,
};
use objc2_core_foundation::CFAttributedString;
use objc2_core_text::{CTLine, CTLineBoundsOptions};
use objc2_foundation::{
    NSArray, NSAttributedString, NSMutableAttributedString, NSMutableDictionary, NSPoint, NSRange, NSRect, NSSize,
    NSString,
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

/// Boxed, the value comes down far enough that the border around it still fits
/// inside the bar with air on either side of the rule.
const BOXED_TEXT_RATIO: f64 = 0.72;

/// With a progress bar underneath it, the value shares the bar's height with
/// the rule the same way the stacked style shares it with the glyph.
const PROGRESS_TEXT_RATIO: f64 = 0.62;

/// Beside a bar, the value is set at the same compact size the row style uses,
/// so the digits match `Icon and Text` and `free-disk-space-widget`.
const DAY_BAR_TEXT_RATIO: f64 = 0.84;

/// The gauge itself, in menu bar ems: `battery-widget`'s 40pt track and 6pt
/// rule at a 14pt menu bar font, so the two widgets read as one set.
const TRACK_RATIO: f64 = 2.85;
const DAY_BAR_RULE_RATIO: f64 = 0.42;

/// With a ledger under it the rule comes down — the pair still has to share the
/// height of the bar, and the bar is the mark that has to stay readable.
const DOTS_RULE_RATIO: f64 = 0.36;

/// One day, and one week: the week mark is three dots wide, which is the
/// narrowest it can be and still not read as a dot that has been squashed.
const DOT_RATIO: f64 = 0.21;
const WEEK_RATIO: f64 = 0.63;

/// How far the ledger counts. Past this the row would be longer than the gauge
/// above it and too long to count at a glance, so the style hands over to
/// [`day_bar_image`] — the same reading, carried by digits instead of marks.
/// Three weeks is where that happens whatever the remainder: 20 days is the
/// widest row below it, and it still fits inside four and a bit ems.
const DOTS_MAX_DAYS: u64 = 21;

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

/// How far the previews sit from the longest label, in ems of the menu font.
/// Wide enough that the two columns read as columns rather than as one run
/// that happens to have a gap in it.
const STYLE_ROW_GAP_RATIO: f64 = 2.0;

/// The menu font — the one macOS sets menu item titles in, which is a size
/// apart from the menu *bar* font the item itself is drawn at.
fn menu_font() -> Retained<NSFont> {
    NSFont::menuFontOfSize(0.0)
}

/// How wide `text` sets as a menu item title.
pub fn menu_label_width(text: &str) -> f64 {
    let attrs = NSMutableDictionary::<NSString, AnyObject>::new();
    unsafe {
        attrs.setObject_forKey(&menu_font(), ProtocolObject::from_ref(NSFontAttributeName));
        NSString::from_str(text).sizeWithAttributes(Some(&attrs)).width
    }
}

/// Where the previews' right edge belongs, given every label and every preview
/// in the menu: past the longest label, with room for the widest preview. One
/// figure for the whole menu is what puts the previews in a column — a row
/// with a short label and a narrow preview lines up with every other row.
pub fn style_column_right_edge(widest_label: f64, widest_preview: f64) -> f64 {
    widest_label + menu_font().pointSize() * STYLE_ROW_GAP_RATIO + widest_preview
}

/// A style menu row: the label set flush left, the preview flush right against
/// `right_edge`.
///
/// A right-aligned tab stop is what does it. The label, a tab, then the preview
/// as an attachment — the tab stop ends the run that follows it at
/// `right_edge`, so every preview finishes on the same line however wide it is
/// and however short its label. The alternative, a custom `NSView` per row,
/// would cost the rows their native highlight and checkmark.
pub fn style_row_title(label: &str, preview: &NSImage) -> Retained<NSMutableAttributedString> {
    let font = menu_font();

    let attachment = NSTextAttachment::new();
    attachment.setImage(Some(&tinted(preview)));

    // An attachment sits on the baseline by default, which hangs the preview
    // below the label it shares a row with. Drop it by half the difference
    // between the two so the preview centres on the label's own midline.
    let size = preview.size();
    let midline = font.capHeight() / 2.0;
    attachment.setBounds(NSRect {
        origin: NSPoint {
            x: 0.0,
            y: midline - size.height / 2.0,
        },
        size,
    });

    let title = NSMutableAttributedString::initWithString(
        NSMutableAttributedString::alloc(),
        &NSString::from_str(&format!("{label}\t")),
    );
    title.appendAttributedString(&NSAttributedString::attributedStringWithAttachment(&attachment));
    title
}

/// A preview in the menu's own text colour.
///
/// The previews are template images — black ink plus alpha, for macOS to tint
/// to whichever appearance the menu bar is in. `NSMenuItem`'s image well does
/// that tinting; a text attachment does not, it draws the image exactly as
/// given, which puts black ink on a dark menu. So the attachment gets a tinted
/// copy: the template drawn, then flooded source-atop with the label colour,
/// which keeps the alpha and replaces the black.
///
/// Block-based like the previews themselves, so the colour is resolved at draw
/// time and follows the system between light and dark.
fn tinted(preview: &NSImage) -> Retained<NSImage> {
    let source = preview.retain();
    let handler = block2::RcBlock::new(move |bounds: NSRect| -> objc2::runtime::Bool {
        source.drawInRect(bounds);
        NSColor::labelColor().set();
        NSRectFillUsingOperation(bounds, NSCompositingOperation::SourceAtop);
        objc2::runtime::Bool::YES
    });

    NSImage::imageWithSize_flipped_drawingHandler(preview.size(), false, &handler)
}

/// The paragraph style the rows share: one right-aligned tab stop, at the
/// column's right edge.
pub fn style_row_paragraph(right_edge: f64) -> Retained<NSMutableParagraphStyle> {
    let paragraph = NSMutableParagraphStyle::new();
    let tab = unsafe {
        NSTextTab::initWithTextAlignment_location_options(
            NSTextTab::alloc(),
            NSTextAlignment::Right,
            right_edge,
            &NSMutableDictionary::new(),
        )
    };
    paragraph.setTabStops(Some(&NSArray::from_retained_slice(&[tab])));
    paragraph
}

/// Stamp the shared paragraph style over a whole row title.
pub fn apply_style_row_paragraph(
    title: &NSMutableAttributedString,
    paragraph: &NSMutableParagraphStyle,
) {
    unsafe {
        title.addAttribute_value_range(
            NSParagraphStyleAttributeName,
            paragraph,
            NSRange {
                location: 0,
                length: title.length(),
            },
        );
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

/// The value on its own, drawn rather than set as the button's title. The menu
/// bar doesn't need this — the text style sets a title — but the style menu's
/// previews do, since a preview is an image whatever style it is showing.
pub fn text_image(text: &str) -> Retained<NSImage> {
    let em = menu_bar_font().pointSize();
    let height = NSStatusBar::systemStatusBar().thickness();
    let run = Run::new(text, &digit_font(em), &NSColor::blackColor());

    let width = run.typographic.width.ceil();
    let y = ((height - run.typographic.height) / 2.0).round();

    image(width, height, move || run.draw(0.0, y))
}

/// The value set small inside a thin rounded rule. The rule is drawn a little
/// lighter than the digits so the box reads as a container rather than as
/// another mark competing with the number.
pub fn boxed_image(text: &str) -> Retained<NSImage> {
    let em = menu_bar_font().pointSize();
    let height = NSStatusBar::systemStatusBar().thickness();
    let run = Run::new(
        text,
        &digit_font(em * BOXED_TEXT_RATIO),
        &NSColor::blackColor(),
    );

    let pad_x = (em * 0.30).round().max(2.0);
    let pad_y = (em * 0.10).round().max(1.0);
    let width = run.typographic.width.ceil() + pad_x * 2.0;
    let box_height = (run.typographic.height.ceil() + pad_y * 2.0).min(height);

    // Inset by half the rule so the stroke lands inside the image rather than
    // half outside it, where it would be clipped.
    let rule = 1.0;
    let rect = NSRect {
        origin: NSPoint {
            x: rule / 2.0,
            y: ((height - box_height) / 2.0).round() + rule / 2.0,
        },
        size: NSSize {
            width: width - rule,
            height: box_height - rule,
        },
    };
    let radius = (box_height * 0.30).round().max(2.0);
    let text_x = ((width - run.typographic.width) / 2.0).round();
    let text_y = ((height - run.typographic.height) / 2.0).round();

    image(width, height, move || {
        let path = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(rect, radius, radius);
        path.setLineWidth(rule);
        NSColor::blackColor().colorWithAlphaComponent(0.75).set();
        path.stroke();
        run.draw(text_x, text_y);
    })
}

/// The value over a progress bar filled to `fraction` — whole days in the
/// digits, the part of the day they drop in the bar. The track is drawn faint
/// and the fill solid; both are template alpha, so macOS tints them with the
/// menu bar's own colour in either appearance.
pub fn progress_image(text: &str, fraction: f64) -> Retained<NSImage> {
    let em = menu_bar_font().pointSize();
    let height = NSStatusBar::systemStatusBar().thickness();
    let run = Run::new(
        text,
        &digit_font(em * PROGRESS_TEXT_RATIO),
        &NSColor::blackColor(),
    );

    let rule = (em * 0.16).round().max(2.0);
    let gap = (em * 0.12).round().max(1.0);
    // A bar narrower than this reads as a dash rather than a gauge, so short
    // values ("5H") widen the item instead of shrinking the track.
    let width = run.typographic.width.ceil().max((em * 2.4).round());

    let content = run.typographic.height.ceil() + gap + rule;
    let bottom = ((height - content) / 2.0).round();
    let text_x = ((width - run.typographic.width) / 2.0).round();
    let text_y = bottom + rule + gap;

    let track = NSRect {
        origin: NSPoint { x: 0.0, y: bottom },
        size: NSSize {
            width,
            height: rule,
        },
    };

    image(width, height, move || {
        draw_track(track, fraction);
        run.draw(text_x, text_y);
    })
}

/// Whole days in the digits and the part-day in the bar, side by side — the
/// same split [`progress_image`] makes, laid out along the menu bar rather than
/// across it so the gauge gets its full length back.
pub fn day_bar_image(text: &str, fraction: f64) -> Retained<NSImage> {
    let em = menu_bar_font().pointSize();
    let height = NSStatusBar::systemStatusBar().thickness();
    let run = Run::new(
        text,
        &digit_font(em * DAY_BAR_TEXT_RATIO),
        &NSColor::blackColor(),
    );

    let gap = (em * 0.35).round();
    let track_width = (em * TRACK_RATIO).round();
    let rule = (em * DAY_BAR_RULE_RATIO).round().max(2.0);

    let text_width = run.typographic.width.ceil();
    let width = text_width + gap + track_width;
    let text_y = ((height - run.typographic.height) / 2.0).round();
    let track = NSRect {
        origin: NSPoint {
            x: text_width + gap,
            y: ((height - rule) / 2.0).round(),
        },
        size: NSSize {
            width: track_width,
            height: rule,
        },
    };

    image(width, height, move || {
        run.draw(0.0, text_y);
        draw_track(track, fraction);
    })
}

/// The glyph, the day in progress as a bar, and the days already behind it as a
/// ledger of marks below it. No digits: the marks are the count, one dot to the
/// day and one longer mark to the week.
///
/// The ledger's height is reserved whether or not there are any marks yet, so
/// the bar sits on one line for the whole first day rather than stepping up
/// when the first dot lands.
pub fn day_dots_image(glyph: &str, fraction: f64, days: u64) -> Retained<NSImage> {
    if days > DOTS_MAX_DAYS {
        return day_bar_image(&format!("{days}D"), fraction);
    }

    let em = menu_bar_font().pointSize();
    let height = NSStatusBar::systemStatusBar().thickness();
    let ink = NSColor::blackColor();

    let glyph = Run::new(glyph, &menu_bar_font(), &ink);
    let gap = (em * 0.35).round();
    let track_width = (em * TRACK_RATIO).round();
    let rule = (em * DOTS_RULE_RATIO).round().max(2.0);
    let row_gap = (em * 0.14).round().max(1.0);
    let dot = (em * DOT_RATIO).round().max(2.0);
    let week = (em * WEEK_RATIO).round().max(dot * 3.0);

    let marks = day_marks(days);
    let row_width = marks_width(&marks, dot, week);

    let bar_x = glyph.ink_width() + gap;
    let width = bar_x + track_width.max(row_width);

    let content = rule + row_gap + dot;
    let bottom = ((height - content) / 2.0).round();
    let track = NSRect {
        origin: NSPoint {
            x: bar_x,
            y: bottom + dot + row_gap,
        },
        size: NSSize {
            width: track_width,
            height: rule,
        },
    };
    let glyph_y = bottom + ((content - glyph.ink.size.height) / 2.0).round();

    image(width, height, move || {
        glyph.draw_ink_bottom(0.0, glyph_y);
        draw_track(track, fraction);

        NSColor::blackColor().set();
        let mut x = bar_x;
        for is_week in &marks {
            let mark_width = if *is_week { week } else { dot };
            let rect = NSRect {
                origin: NSPoint { x, y: bottom },
                size: NSSize {
                    width: mark_width,
                    height: dot,
                },
            };
            let radius = dot / 2.0;
            NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(rect, radius, radius).fill();
            x += mark_width + dot;
        }
    })
}

/// The ledger, left to right: one `true` per whole week, then one `false` per
/// day left over. Weeks lead, so the row reads coarse-to-fine the way the
/// digits it replaces do.
fn day_marks(days: u64) -> Vec<bool> {
    let weeks = days / 7;
    let rest = days % 7;
    std::iter::repeat_n(true, weeks as usize)
        .chain(std::iter::repeat_n(false, rest as usize))
        .collect()
}

/// How wide that ledger draws, marks plus the one-dot gaps between them.
fn marks_width(marks: &[bool], dot: f64, week: f64) -> f64 {
    if marks.is_empty() {
        return 0.0;
    }
    marks
        .iter()
        .map(|&is_week| if is_week { week } else { dot })
        .sum::<f64>()
        + dot * (marks.len() - 1) as f64
}

/// A rounded track with a solid fill to `fraction` — the gauge every bar style
/// here draws, and the one `battery-widget` draws for the same job. Both are
/// template alpha, so macOS tints them with the menu bar's own colour.
fn draw_track(rect: NSRect, fraction: f64) {
    let radius = rect.size.height / 2.0;
    NSColor::blackColor().colorWithAlphaComponent(0.28).set();
    NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(rect, radius, radius).fill();

    // A sliver of fill is worse than none: below one rule-width there is no
    // room for the rounded cap, so the fill starts at that width.
    let fill_width = (rect.size.width * fraction.clamp(0.0, 1.0)).round();
    if fill_width > 0.0 {
        let fill = NSRect {
            origin: rect.origin,
            size: NSSize {
                width: fill_width.max(rect.size.height),
                height: rect.size.height,
            },
        };
        NSColor::blackColor().set();
        NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(fill, radius, radius).fill();
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_carries_sevens_into_week_marks() {
        assert_eq!(day_marks(0), Vec::<bool>::new());
        assert_eq!(day_marks(3), vec![false, false, false]);
        assert_eq!(day_marks(7), vec![true]);
        assert_eq!(day_marks(12), vec![true, false, false, false, false, false]);
        assert_eq!(day_marks(21), vec![true, true, true]);
    }

    /// The ledger's whole reason to carry is that it stays countable: no row up
    /// to the cap may run past a hand's worth of marks.
    #[test]
    fn ledger_stays_countable_to_the_cap() {
        for days in 0..=DOTS_MAX_DAYS {
            assert!(day_marks(days).len() <= 9, "{days} days is too many marks");
        }
    }

    /// And it stays inside the item: at three dots to the week mark and one dot
    /// of gap, no row up to the cap is wider than four and a half ems.
    #[test]
    fn ledger_stays_inside_the_item() {
        for days in 0..=DOTS_MAX_DAYS {
            let width = marks_width(&day_marks(days), DOT_RATIO, WEEK_RATIO);
            assert!(width <= 4.5, "{days} days draws {width} ems wide");
        }
    }
}
