mod battery;

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use battery::{BatteryInfo, BatteryState, read_battery};
use objc2::Message;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{
    AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel,
};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSBezierPath, NSCellImagePosition, NSColor,
    NSControlStateValueOff, NSControlStateValueOn, NSFont, NSFontAttributeName,
    NSAttributedStringAttachmentConveniences, NSAttributedStringNSStringDrawing,
    NSCompositingOperation, NSFontWeightRegular,
    NSForegroundColorAttributeName, NSImage, NSMenu, NSMenuItem, NSMutableParagraphStyle,
    NSParagraphStyleAttributeName, NSRectFillUsingOperation, NSStatusBar, NSStatusItem,
    NSStringDrawing, NSTextAlignment, NSTextAttachment, NSTextTab, NSVariableStatusItemLength,
};
use objc2_foundation::{
    NSArray, NSAttributedString, NSMutableAttributedString, NSMutableDictionary, NSObject, NSPoint,
    NSRange, NSRect, NSSize, NSString, NSTimer, ns_string,
};

const BATTERY_ICON: &str = "\u{1006E8}"; // SF Symbols battery.100
const LOW_BATTERY_ICON: &str = "\u{1006EA}"; // battery.0
const BOLT_ICON: &str = "\u{1002E6}"; // bolt.fill

const TITLE_FONT_SIZE: f64 = 14.0;
const LOW_BATTERY_THRESHOLD: i32 = 10;
const UPDATE_INTERVAL_SECONDS: f64 = 10.0;

// Animation. The battery itself is only re-read every UPDATE_INTERVAL_SECONDS;
// the animation timer just re-renders the cached reading, and only while some
// pulse is actually running.
const ANIMATION_INTERVAL_SECONDS: f64 = 1.0 / 20.0;
const LOW_PULSE_THRESHOLD: i32 = 8;
const FULL_PULSE_THRESHOLD: i32 = 95; // charge level, not battery health
const BOLT_YELLOW_THRESHOLD: i32 = 60; // above this the bolt reads as healthy
const BAR_PULSE_PERIOD: f64 = 2.6; // slow breath for the charging/low/full bar
const BOLT_THROB_PERIOD: f64 = 0.6; // rapid throb, critical charge only
const BAR_PULSE_MIN_ALPHA: f64 = 0.3;
const BOLT_THROB_MIN_ALPHA: f64 = 0.35;
const IDLE_BOLT_ALPHA: f64 = 0.45;

// Bar image geometry, in points. The button centers image+title as one block,
// but the bar image is full-bleed while the title's glyphs carry side bearing,
// so the content ends up sitting left of centre in its capsule. BAR_OUTER_PAD
// is transparent padding added to the image's outer edge (away from the text)
// to cancel that out.
const BAR_TRACK_WIDTH: f64 = 40.0;
const BAR_OUTER_PAD: f64 = 3.0;
const BAR_IMAGE_HEIGHT: f64 = 16.0;
const BAR_HEIGHT: f64 = 6.0;
const BAR_RADIUS: f64 = 3.0;

// A bolt drawn to the left of the track, as part of the same image. Drawing it
// rather than setting it as a title run keeps the whole widget one centred
// image — a text run's side bearing is what pushed the content off-centre.
const BOLT_SLOT_GAP: f64 = 4.0;
const BOLT_GLYPH_SIZE: f64 = 9.0;

/// Which edge of the bar image gets the balancing padding.
#[derive(Clone, Copy, PartialEq, Eq)]
enum BarPad {
    None,
    Leading,
    Trailing,
}

impl BarPad {
    fn image_width(self) -> f64 {
        match self {
            BarPad::None => BAR_TRACK_WIDTH,
            _ => BAR_TRACK_WIDTH + BAR_OUTER_PAD,
        }
    }

    fn track_x(self) -> f64 {
        match self {
            BarPad::Leading => BAR_OUTER_PAD,
            _ => 0.0,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LayoutStyle {
    Text,
    IconText,
    BarText,
    IconBar,
    PercentBar,
    Bar,
    BarPower,
    SmartBar,
    SmartBarTimer,
}

const ALL_STYLES: [LayoutStyle; 9] = [
    LayoutStyle::Text,
    LayoutStyle::IconText,
    LayoutStyle::BarText,
    LayoutStyle::IconBar,
    LayoutStyle::PercentBar,
    LayoutStyle::Bar,
    LayoutStyle::BarPower,
    LayoutStyle::SmartBar,
    LayoutStyle::SmartBarTimer,
];

impl LayoutStyle {
    fn label(self) -> &'static str {
        match self {
            LayoutStyle::Text => "Text",
            LayoutStyle::IconText => "Icon and Text",
            LayoutStyle::BarText => "Bar and Text",
            LayoutStyle::IconBar => "Icon and Bar",
            LayoutStyle::PercentBar => "Percentage and Bar",
            LayoutStyle::Bar => "Bar",
            LayoutStyle::BarPower => "Bar and Power",
            LayoutStyle::SmartBar => "Smart Bar",
            LayoutStyle::SmartBarTimer => "Smart Bar and Timer",
        }
    }

    fn key(self) -> &'static str {
        match self {
            LayoutStyle::Text => "text",
            LayoutStyle::IconText => "icon_text",
            LayoutStyle::BarText => "bar_text",
            LayoutStyle::IconBar => "icon_bar",
            LayoutStyle::PercentBar => "percent_bar",
            LayoutStyle::Bar => "bar",
            LayoutStyle::BarPower => "bar_power",
            LayoutStyle::SmartBar => "smart_bar",
            LayoutStyle::SmartBarTimer => "smart_bar_timer",
        }
    }

    fn from_key(key: &str) -> Option<Self> {
        ALL_STYLES.iter().copied().find(|s| s.key() == key)
    }
}

fn style_config_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join(".config/battery-widget/style"))
}

fn load_style() -> LayoutStyle {
    style_config_path()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|key| LayoutStyle::from_key(key.trim()))
        .unwrap_or(LayoutStyle::SmartBar)
}

fn save_style(style: LayoutStyle) {
    let Some(path) = style_config_path() else {
        return;
    };
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    if let Err(err) = fs::write(&path, style.key()) {
        eprintln!("error saving style: {err}");
    }
}

/// One styled segment of the menu bar title.
struct Run {
    text: String,
    color: Option<Retained<NSColor>>,
    monospaced: bool,
}

impl Run {
    fn plain(text: impl Into<String>) -> Self {
        Run {
            text: text.into(),
            color: None,
            monospaced: false,
        }
    }

    fn colored(text: impl Into<String>, color: Retained<NSColor>) -> Self {
        Run {
            color: Some(color),
            ..Run::plain(text)
        }
    }
}

/// The drawn progress-bar image, and how it sits relative to the text.
struct BarSpec {
    percent: i32,
    fill: Option<Retained<NSColor>>, // None = template image, adapts to menu bar
    fill_alpha: f64,
    bolt: Option<Retained<NSColor>>,         // overlaid on the track
    leading_bolt: Option<Retained<NSColor>>, // in its own slot, left of the track
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pulse {
    None,
    Charging,
    Low,
    Full,
}

/// 0 → 1 → 0, once per `period`.
fn wave(t: f64, period: f64) -> f64 {
    0.5 - 0.5 * (std::f64::consts::TAU * t / period).cos()
}

/// The animated part of the title: which pulse is running and how far through
/// it we are. Rebuilt on every render from the elapsed time.
struct Anim {
    pulse: Pulse,
    bar_alpha: f64,
    bolt_color: Retained<NSColor>,
    charging: bool,
}

impl Anim {
    fn new(info: &BatteryInfo, t: f64) -> Self {
        // "Charger in", not "actively charging": at 100% pmset reports the
        // state as `charged`, which is still plugged in.
        let charging = info.on_ac;
        // Critical charge is the only state that animates the bolt itself.
        let critical = info.percent <= LOW_PULSE_THRESHOLD && !charging;

        // Charging is signalled by breathing the bar, not the bolt — one
        // moving thing in the widget at a time.
        let pulse = if info.percent > FULL_PULSE_THRESHOLD {
            Pulse::Full
        } else if critical {
            Pulse::Low
        } else if charging {
            Pulse::Charging
        } else {
            Pulse::None
        };

        let bar_alpha =
            BAR_PULSE_MIN_ALPHA + (1.0 - BAR_PULSE_MIN_ALPHA) * wave(t, BAR_PULSE_PERIOD);

        // The bolt is a charge-level readout, and solid unless things are dire:
        // yellow with plenty in the tank, plain white as it drains, then red
        // and throbbing fast once critical. White rather than a label colour so
        // it reads on both light and dark menu bars without going muddy grey.
        let bolt_color = if critical {
            let alpha =
                BOLT_THROB_MIN_ALPHA + (1.0 - BOLT_THROB_MIN_ALPHA) * wave(t, BOLT_THROB_PERIOD);
            NSColor::systemRedColor().colorWithAlphaComponent(alpha)
        } else if info.percent > BOLT_YELLOW_THRESHOLD {
            NSColor::systemYellowColor()
        } else {
            NSColor::whiteColor().colorWithAlphaComponent(IDLE_BOLT_ALPHA)
        };

        Anim {
            pulse,
            bar_alpha,
            bolt_color,
            charging,
        }
    }

    fn animating(&self) -> bool {
        self.pulse != Pulse::None
    }

    /// Charging breathes the bar in the menu bar's own foreground (white on a
    /// dark menu bar); low breathes it red; full fades a template bar in and
    /// out. The bolt stays out of it unless the charge is critical.
    fn apply_to_bar(&self, bar: &mut BarSpec) {
        let has_bolt = bar.bolt.is_some() || bar.leading_bolt.is_some();

        // A template image is drawn as a mask, which would repaint a coloured
        // bolt in the menu bar's tint. Once the image carries a bolt, the bar
        // has to carry an explicit colour — labelColor still tracks the
        // appearance.
        if has_bolt && bar.fill.is_none() {
            bar.fill = Some(NSColor::labelColor());
        }

        match self.pulse {
            Pulse::None => {}
            Pulse::Charging => bar.fill_alpha = self.bar_alpha,
            Pulse::Low => {
                bar.fill = Some(NSColor::systemRedColor());
                bar.fill_alpha = self.bar_alpha;
            }
            Pulse::Full => {
                // A template bar tracks the menu bar's tint, but only when the
                // image has no coloured bolt for the mask to flatten.
                if !self.charging && !has_bolt {
                    bar.fill = None;
                }
                bar.fill_alpha = self.bar_alpha;
            }
        }
    }
}

struct TitleSpec {
    runs: Vec<Run>,
    bar: Option<BarSpec>,
    bar_on_left: bool,
}

fn attributed_title(runs: &[Run]) -> Retained<NSMutableAttributedString> {
    attributed_runs(runs, None)
}

/// The runs, set. `fallback` colours the runs that carry no colour of their
/// own — the menu bar wants them left alone, so the status item passes `None`
/// and lets AppKit take the title as the bar's own; a preview drawn into an
/// image has no such owner, so it passes the label colour and gets ink that
/// tracks the appearance instead of the default black.
fn attributed_runs(
    runs: &[Run],
    fallback: Option<&NSColor>,
) -> Retained<NSMutableAttributedString> {
    let result = NSMutableAttributedString::new();
    for run in runs {
        let weight = unsafe { NSFontWeightRegular };
        let font = if run.monospaced {
            NSFont::monospacedSystemFontOfSize_weight(TITLE_FONT_SIZE, weight)
        } else {
            NSFont::monospacedDigitSystemFontOfSize_weight(TITLE_FONT_SIZE, weight)
        };

        let attrs = NSMutableDictionary::<NSString, AnyObject>::new();
        unsafe {
            attrs.setObject_forKey(&font, ProtocolObject::from_ref(NSFontAttributeName));
            if let Some(color) = run.color.as_deref().or(fallback) {
                attrs.setObject_forKey(
                    color,
                    ProtocolObject::from_ref(NSForegroundColorAttributeName),
                );
            }
        }

        let piece = unsafe {
            NSMutableAttributedString::initWithString_attributes(
                NSMutableAttributedString::alloc(),
                &NSString::from_str(&run.text),
                Some(&attrs),
            )
        };
        result.appendAttributedString(&piece);
    }
    result
}

/// How far the previews sit from the longest label, in ems of the menu font.
const STYLE_ROW_GAP_RATIO: f64 = 2.0;

/// Between the bar and the text inside a preview — the gap the menu bar item
/// itself gets from `NSCellImagePosition`, which a composed image has to
/// supply for itself.
const PREVIEW_INNER_GAP: f64 = 4.0;

/// The menu font — the one macOS sets menu item titles in, a size apart from
/// the menu *bar* font the status item is drawn at.
fn menu_font() -> Retained<NSFont> {
    NSFont::menuFontOfSize(0.0)
}

/// How wide `text` sets as a menu item title.
fn menu_label_width(text: &str) -> f64 {
    let attrs = NSMutableDictionary::<NSString, AnyObject>::new();
    unsafe {
        attrs.setObject_forKey(&menu_font(), ProtocolObject::from_ref(NSFontAttributeName));
        NSString::from_str(text)
            .sizeWithAttributes(Some(&attrs))
            .width
    }
}

/// Where the previews' right edge belongs: past the longest label, with room
/// for the widest preview. One figure for the whole menu is what puts the
/// previews in a column rather than leaving each to sit where its own label
/// ends.
fn style_column_right_edge(widest_label: f64, widest_preview: f64) -> f64 {
    widest_label + menu_font().pointSize() * STYLE_ROW_GAP_RATIO + widest_preview
}

/// An image in the menu's own text colour.
///
/// A neutral bar is a template image — black ink plus alpha, for macOS to tint
/// to the current appearance. `NSMenuItem`'s image well does that tinting and
/// so does the menu bar; drawing into another image does not, which would put
/// black ink on a dark menu. So the template is drawn and then flooded
/// source-atop with the label colour, which keeps the alpha and replaces the
/// black. Block-based, so the colour resolves at draw time and follows the
/// system between light and dark.
fn tinted(image: &NSImage) -> Retained<NSImage> {
    let source = image.retain();
    let handler = block2::RcBlock::new(move |bounds: NSRect| -> objc2::runtime::Bool {
        source.drawInRect(bounds);
        NSColor::labelColor().set();
        NSRectFillUsingOperation(bounds, NSCompositingOperation::SourceAtop);
        objc2::runtime::Bool::YES
    });

    NSImage::imageWithSize_flipped_drawingHandler(image.size(), false, &handler)
}

/// What the menu bar item would look like in `style`, as one image, for the
/// style menu to show beside that style's name.
///
/// The item itself splits its appearance between an attributed title and an
/// image, and lets `NSCellImagePosition` put the two together. A menu row has
/// only the one slot, so the preview has to compose them here: bar and text
/// side by side, in the order the style asks for, centred on each other.
///
/// The preview is the style drawn from the live reading, not a mock of it, but
/// it is drawn at rest — pulses are held at full so a row is never caught
/// mid-breath, which at menu refresh rate would read as rows fading at random
/// rather than as a pulse.
fn preview_image(info: &BatteryInfo, style: LayoutStyle) -> Retained<NSImage> {
    let mut anim = Anim::new(info, 0.0);
    anim.bar_alpha = 1.0;
    let spec = title_spec(info, style, &anim);

    let title = attributed_runs(&spec.runs, Some(&NSColor::labelColor()));
    let title_size = if spec.runs.is_empty() {
        NSSize {
            width: 0.0,
            height: 0.0,
        }
    } else {
        title.size()
    };

    let bar = spec.bar.as_ref().map(|bar| {
        let image = bar_image(bar, BarPad::None);
        // A neutral bar comes back as a template; inside a composed image it
        // has to carry its own colour.
        if image.isTemplate() {
            tinted(&image)
        } else {
            image
        }
    });
    let bar_size = bar.as_ref().map(|bar| bar.size()).unwrap_or(NSSize {
        width: 0.0,
        height: 0.0,
    });

    let gap = if bar.is_some() && !spec.runs.is_empty() {
        PREVIEW_INNER_GAP
    } else {
        0.0
    };
    let size = NSSize {
        width: title_size.width + gap + bar_size.width,
        height: title_size.height.max(bar_size.height).max(BAR_IMAGE_HEIGHT),
    };

    let bar_on_left = spec.bar_on_left;
    let handler = block2::RcBlock::new(move |_bounds: NSRect| -> objc2::runtime::Bool {
        let (bar_x, title_x) = if bar_on_left {
            (0.0, bar_size.width + gap)
        } else {
            (title_size.width + gap, 0.0)
        };

        if let Some(bar) = &bar {
            bar.drawInRect(NSRect {
                origin: NSPoint {
                    x: bar_x,
                    y: (size.height - bar_size.height) / 2.0,
                },
                size: bar_size,
            });
        }

        title.drawAtPoint(NSPoint {
            x: title_x,
            y: (size.height - title_size.height) / 2.0,
        });

        objc2::runtime::Bool::YES
    });

    NSImage::imageWithSize_flipped_drawingHandler(size, false, &handler)
}

/// A style menu row: the label flush left, the preview flush right against the
/// column's edge.
///
/// A right-aligned tab stop is what does it — the label, a tab, then the
/// preview as an attachment, so every preview finishes on the same line
/// however wide it is and however short its label. The alternative, a custom
/// `NSView` per row, would cost the rows their native highlight and checkmark.
fn style_row_title(label: &str, preview: &NSImage) -> Retained<NSMutableAttributedString> {
    let font = menu_font();

    let attachment = NSTextAttachment::new();
    attachment.setImage(Some(preview));

    // An attachment sits on the baseline, which hangs the preview below the
    // label it shares a row with. Drop it by half the difference so the two
    // centre on each other.
    let size = preview.size();
    attachment.setBounds(NSRect {
        origin: NSPoint {
            x: 0.0,
            y: font.capHeight() / 2.0 - size.height / 2.0,
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

/// The paragraph style the rows share: one right-aligned tab stop, at the
/// column's right edge.
fn style_row_paragraph(right_edge: f64) -> Retained<NSMutableParagraphStyle> {
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
fn apply_style_row_paragraph(
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

/// Draw the bar the mockup way: a rounded track with a rounded fill, and the
/// charging bolt overlaid with a dark halo. Neutral bars are template images
/// so macOS tints them to match the menu bar in any appearance. Block-based
/// so AppKit re-renders at the backing scale and current appearance.
fn bar_image(spec: &BarSpec, pad: BarPad) -> Retained<NSImage> {
    let leading = spec.leading_bolt.clone();
    // Size the slot to the glyph itself: a fixed slot leaves slack on one side
    // and the whole image ends up looking off-centre in the menu bar.
    let bolt_width = glyph_size(BOLT_ICON, BOLT_GLYPH_SIZE).width;
    let lead_width = if leading.is_some() {
        bolt_width + BOLT_SLOT_GAP
    } else {
        0.0
    };
    let size = NSSize {
        width: pad.image_width() + lead_width,
        height: BAR_IMAGE_HEIGHT,
    };
    let lead_x = pad.track_x();
    let track_x = lead_x + lead_width;
    let is_template = spec.fill.is_none();
    let fill = spec
        .fill
        .clone()
        .unwrap_or_else(NSColor::blackColor)
        .colorWithAlphaComponent(spec.fill_alpha.clamp(0.0, 1.0));
    let percent = spec.percent.clamp(0, 100);
    let bolt = spec.bolt.clone();

    let handler = block2::RcBlock::new(move |_bounds: NSRect| -> objc2::runtime::Bool {
        let bar_y = (BAR_IMAGE_HEIGHT - BAR_HEIGHT) / 2.0;
        let track_rect = NSRect {
            origin: NSPoint {
                x: track_x,
                y: bar_y,
            },
            size: NSSize {
                width: BAR_TRACK_WIDTH,
                height: BAR_HEIGHT,
            },
        };
        fill.colorWithAlphaComponent(0.3).set();
        NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(track_rect, BAR_RADIUS, BAR_RADIUS)
            .fill();

        if percent > 0 {
            let filled_width = (BAR_TRACK_WIDTH * percent as f64 / 100.0).max(BAR_RADIUS * 2.0);
            let fill_rect = NSRect {
                origin: NSPoint {
                    x: track_x,
                    y: bar_y,
                },
                size: NSSize {
                    width: filled_width,
                    height: BAR_HEIGHT,
                },
            };
            fill.set();
            NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                fill_rect, BAR_RADIUS, BAR_RADIUS,
            )
            .fill();
        }

        if let Some(color) = &leading {
            draw_glyph_centered(BOLT_ICON, BOLT_GLYPH_SIZE, color, lead_x + bolt_width / 2.0);
        }

        if let Some(color) = &bolt {
            // No halo: the dark outline bled into the glyph at menu bar size
            // and read as grey. The bolt carries its own alpha instead.
            draw_glyph_centered(BOLT_ICON, 8.0, color, track_x + BAR_TRACK_WIDTH / 2.0);
        }

        objc2::runtime::Bool::YES
    });

    let image = NSImage::imageWithSize_flipped_drawingHandler(size, false, &handler);
    image.setTemplate(is_template);
    image
}

fn glyph_attrs(
    font_size: f64,
    color: Option<&NSColor>,
) -> Retained<NSMutableDictionary<NSString, AnyObject>> {
    let attrs = NSMutableDictionary::<NSString, AnyObject>::new();
    let font = NSFont::systemFontOfSize(font_size);
    unsafe {
        attrs.setObject_forKey(&font, ProtocolObject::from_ref(NSFontAttributeName));
        if let Some(color) = color {
            attrs.setObject_forKey(
                color,
                ProtocolObject::from_ref(NSForegroundColorAttributeName),
            );
        }
    }
    attrs
}

/// Laid-out size of a glyph, measurable outside a drawing context so image
/// geometry can be derived from it.
fn glyph_size(glyph: &str, font_size: f64) -> NSSize {
    let attrs = glyph_attrs(font_size, None);
    unsafe { NSString::from_str(glyph).sizeWithAttributes(Some(&attrs)) }
}

fn draw_glyph_centered(glyph: &str, font_size: f64, color: &NSColor, center_x: f64) {
    let attrs = glyph_attrs(font_size, Some(color));
    let text = NSString::from_str(glyph);
    let size = glyph_size(glyph, font_size);
    let origin = NSPoint {
        x: center_x - size.width / 2.0,
        y: (BAR_IMAGE_HEIGHT - size.height) / 2.0,
    };
    unsafe { text.drawAtPoint_withAttributes(origin, Some(&attrs)) };
}

fn state_color(info: &BatteryInfo) -> Option<Retained<NSColor>> {
    match info.state {
        BatteryState::Charging => Some(NSColor::systemGreenColor()),
        BatteryState::Discharging if info.percent < LOW_BATTERY_THRESHOLD => {
            Some(NSColor::systemRedColor())
        }
        _ if info.low_power_mode => Some(NSColor::systemYellowColor()),
        _ => None,
    }
}

// power_text renders battery power flow compactly: "8.4w" discharging,
// "+42w" charging.
fn power_text(info: &BatteryInfo) -> String {
    let prefix = if info.state == BatteryState::Charging {
        "+"
    } else {
        ""
    };
    if info.watts >= 10.0 {
        format!("{prefix}{:.0}w", info.watts)
    } else {
        format!("{prefix}{:.1}w", info.watts)
    }
}

fn status_icon(info: &BatteryInfo) -> &'static str {
    match info.state {
        BatteryState::Charging => BOLT_ICON,
        _ if info.percent < LOW_BATTERY_THRESHOLD => LOW_BATTERY_ICON,
        _ => BATTERY_ICON,
    }
}

fn neutral_bar(percent: i32) -> Option<BarSpec> {
    Some(BarSpec {
        percent,
        fill: None,
        fill_alpha: 1.0,
        bolt: None,
        leading_bolt: None,
    })
}

fn smart_title(info: &BatteryInfo, anim: &Anim, with_timer: bool) -> TitleSpec {
    let accent = state_color(info);
    let accented = |text: String| match &accent {
        Some(color) => Run::colored(text, color.clone()),
        None => Run::plain(text),
    };

    let mut runs = vec![accented(power_text(info))];
    if with_timer {
        if let Some(time) = &info.time_remaining {
            runs.push(Run::colored(
                " · ".to_string(),
                NSColor::tertiaryLabelColor(),
            ));
            runs.push(Run {
                monospaced: true,
                ..Run::colored(time.clone(), NSColor::secondaryLabelColor())
            });
        }
    }

    TitleSpec {
        runs,
        bar: Some(BarSpec {
            percent: info.percent,
            fill: accent,
            fill_alpha: 1.0,
            bolt: anim.charging.then(|| anim.bolt_color.clone()),
            leading_bolt: None,
        }),
        bar_on_left: true,
    }
}

fn title_spec(info: &BatteryInfo, style: LayoutStyle, anim: &Anim) -> TitleSpec {
    let percent = format!("{}%", info.percent);
    let text_only = |runs: Vec<Run>| TitleSpec {
        runs,
        bar: None,
        bar_on_left: true,
    };

    let mut spec = match style {
        LayoutStyle::Text => text_only(vec![Run::plain(percent)]),
        LayoutStyle::IconText => {
            text_only(vec![Run::plain(format!("{} {percent}", status_icon(info)))])
        }
        LayoutStyle::BarText => TitleSpec {
            runs: vec![Run::plain(percent)],
            bar: neutral_bar(info.percent),
            bar_on_left: true,
        },
        // Bolt and bar are one image, so the button has nothing to centre but
        // that image — no title run, no side bearing, no lopsided capsule.
        LayoutStyle::IconBar => TitleSpec {
            runs: Vec::new(),
            bar: Some(BarSpec {
                leading_bolt: Some(anim.bolt_color.clone()),
                ..neutral_bar(info.percent).unwrap()
            }),
            bar_on_left: true,
        },
        LayoutStyle::PercentBar => TitleSpec {
            runs: vec![Run::plain(percent)],
            bar: neutral_bar(info.percent),
            bar_on_left: false,
        },
        LayoutStyle::Bar => TitleSpec {
            runs: Vec::new(),
            bar: neutral_bar(info.percent),
            bar_on_left: true,
        },
        LayoutStyle::BarPower => TitleSpec {
            runs: vec![Run::plain(power_text(info))],
            bar: neutral_bar(info.percent),
            bar_on_left: true,
        },
        LayoutStyle::SmartBar => smart_title(info, anim, false),
        LayoutStyle::SmartBarTimer => smart_title(info, anim, true),
    };

    if let Some(bar) = spec.bar.as_mut() {
        anim.apply_to_bar(bar);
    }
    spec
}

struct Ui {
    status_item: Retained<NSStatusItem>,
    info_status: Retained<NSMenuItem>,
    info_power: Retained<NSMenuItem>,
    info_health: Retained<NSMenuItem>,
    lpm_item: Retained<NSMenuItem>,
    style_items: Vec<Retained<NSMenuItem>>,
    style: LayoutStyle,
    last_info: Option<BatteryInfo>,
    started: Instant,
    animating: bool,
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements and Widget does not
    // implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = RefCell<Option<Ui>>]
    struct Widget;

    impl Widget {
        #[unsafe(method(tick:))]
        fn tick(&self, _timer: &NSTimer) {
            self.update();
        }

        #[unsafe(method(animate:))]
        fn animate(&self, _timer: &NSTimer) {
            let animating = self
                .ivars()
                .borrow()
                .as_ref()
                .is_some_and(|ui| ui.animating);
            if animating {
                self.render();
            }
        }

        #[unsafe(method(styleAction:))]
        fn style_action(&self, sender: &NSMenuItem) {
            let style = ALL_STYLES[sender.tag() as usize];
            if let Some(ui) = self.ivars().borrow_mut().as_mut() {
                ui.style = style;
            }
            save_style(style);
            self.refresh_style_checks();
            self.update();
        }

        #[unsafe(method(lpmAction:))]
        fn lpm_action(&self, _sender: &NSMenuItem) {
            let enabled = self
                .ivars()
                .borrow()
                .as_ref()
                .and_then(|ui| ui.last_info.as_ref().map(|i| i.low_power_mode))
                .unwrap_or(false);
            std::thread::spawn(move || toggle_low_power_mode(enabled));
        }
    }
);

impl Widget {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(RefCell::new(None));
        unsafe { msg_send![super(this), init] }
    }

    fn build_ui(&self, mtm: MainThreadMarker) {
        let status_bar = NSStatusBar::systemStatusBar();
        let status_item = status_bar.statusItemWithLength(NSVariableStatusItemLength);

        let menu = NSMenu::new(mtm);
        menu.setAutoenablesItems(false);

        let info_item = |title: &str| {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(title));
            item.setEnabled(false);
            menu.addItem(&item);
            item
        };
        let info_status = info_item("Battery: —");
        let info_power = info_item("Power draw: —");
        let info_health = info_item("Health: —");
        menu.addItem(&NSMenuItem::separatorItem(mtm));

        let style_menu = NSMenu::new(mtm);
        style_menu.setAutoenablesItems(false);
        let mut style_items = Vec::new();
        for (index, style) in ALL_STYLES.iter().enumerate() {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(style.label()));
            item.setTag(index as isize);
            item.setEnabled(true);
            unsafe {
                item.setTarget(Some(self.as_ref()));
                item.setAction(Some(sel!(styleAction:)));
            }
            style_menu.addItem(&item);
            style_items.push(item);
        }
        let style_root = NSMenuItem::new(mtm);
        style_root.setTitle(ns_string!("Style"));
        style_root.setEnabled(true);
        style_root.setSubmenu(Some(&style_menu));
        menu.addItem(&style_root);

        menu.addItem(&NSMenuItem::separatorItem(mtm));

        let lpm_item = NSMenuItem::new(mtm);
        lpm_item.setTitle(ns_string!("Low Power Mode: Off"));
        lpm_item.setEnabled(true);
        unsafe {
            lpm_item.setTarget(Some(self.as_ref()));
            lpm_item.setAction(Some(sel!(lpmAction:)));
        }
        menu.addItem(&lpm_item);

        menu.addItem(&NSMenuItem::separatorItem(mtm));

        let quit_item = NSMenuItem::new(mtm);
        quit_item.setTitle(ns_string!("Quit"));
        quit_item.setEnabled(true);
        unsafe { quit_item.setAction(Some(sel!(terminate:))) };
        menu.addItem(&quit_item);

        // The native menu path: macOS anchors and presents it instantly.
        status_item.setMenu(Some(&menu));

        *self.ivars().borrow_mut() = Some(Ui {
            status_item,
            info_status,
            info_power,
            info_health,
            lpm_item,
            style_items,
            style: load_style(),
            last_info: None,
            started: Instant::now(),
            animating: false,
        });
        self.refresh_style_checks();
    }

    fn update(&self) {
        let info = match read_battery() {
            Ok(info) => info,
            Err(err) => {
                eprintln!("error reading battery: {err}");
                return;
            }
        };

        {
            let mut ivars = self.ivars().borrow_mut();
            let Some(ui) = ivars.as_mut() else { return };
            ui.last_info = Some(info.clone());
        }
        self.render();

        let ivars = self.ivars().borrow();
        let Some(ui) = ivars.as_ref() else { return };

        let mut status_line = format!("Battery: {}%", info.percent);
        match (info.state, &info.time_remaining) {
            (BatteryState::Charging, Some(time)) => {
                status_line += &format!(" — {time} until full");
            }
            (BatteryState::Discharging, Some(time)) => {
                status_line += &format!(" — {time} remaining");
            }
            (BatteryState::Idle, _) if info.on_ac => status_line += " — charged",
            _ => {}
        }
        let source = if info.on_ac {
            "power adapter"
        } else {
            "battery"
        };
        ui.info_status.setTitle(&NSString::from_str(&status_line));
        ui.info_power.setTitle(&NSString::from_str(&format!(
            "Power draw: {} ({source})",
            power_text(&info)
        )));
        ui.info_health.setTitle(&NSString::from_str(&format!(
            "Health: {}% · {} cycles",
            info.health_percent, info.cycle_count
        )));
        ui.lpm_item
            .setTitle(&NSString::from_str(if info.low_power_mode {
                "Low Power Mode: On"
            } else {
                "Low Power Mode: Off"
            }));

        // Each style row carries the item that style would install, drawn from
        // the live reading. Refreshed here rather than in `render`: the reading
        // is what changes them, and `render` runs at animation rate. The
        // previews go in the title rather than in the item's own image well,
        // which is what lets them sit in a right-hand column — an item image is
        // drawn hard against the label, ragged down the menu as labels vary.
        let previews: Vec<Retained<NSImage>> = ALL_STYLES
            .iter()
            .map(|style| preview_image(&info, *style))
            .collect();

        // The column is measured across the whole menu, not row by row, so
        // every preview shares one right edge. Widths move with the reading —
        // "100%" is wider than "9%" — so this is remeasured on each update.
        let widest_label = ALL_STYLES
            .iter()
            .map(|style| menu_label_width(style.label()))
            .fold(0.0, f64::max);
        let widest_preview = previews
            .iter()
            .map(|preview| preview.size().width)
            .fold(0.0, f64::max);
        let paragraph = style_row_paragraph(style_column_right_edge(widest_label, widest_preview));

        for (index, item) in ui.style_items.iter().enumerate() {
            let title = style_row_title(ALL_STYLES[index].label(), &previews[index]);
            apply_style_row_paragraph(&title, &paragraph);
            item.setAttributedTitle(Some(&title));
        }
    }

    /// Redraw the menu bar item from the cached reading. Cheap enough to run
    /// at animation rate; `update` is what actually re-reads the battery.
    fn render(&self) {
        let (info, style, t) = {
            let ivars = self.ivars().borrow();
            let Some(ui) = ivars.as_ref() else { return };
            let Some(info) = ui.last_info.clone() else {
                return;
            };
            (info, ui.style, ui.started.elapsed().as_secs_f64())
        };

        let anim = Anim::new(&info, t);
        let spec = title_spec(&info, style, &anim);
        let title = attributed_title(&spec.runs);

        if let Some(ui) = self.ivars().borrow_mut().as_mut() {
            ui.animating = anim.animating();
        }

        let ivars = self.ivars().borrow();
        let Some(ui) = ivars.as_ref() else { return };
        let Some(button) = ui.status_item.button(MainThreadMarker::new().unwrap()) else {
            return;
        };
        button.setAttributedTitle(&title);
        match &spec.bar {
            Some(bar) => {
                let (position, pad) = if spec.runs.is_empty() {
                    (NSCellImagePosition::ImageOnly, BarPad::None)
                } else if spec.bar_on_left {
                    (NSCellImagePosition::ImageLeft, BarPad::Leading)
                } else {
                    (NSCellImagePosition::ImageRight, BarPad::Trailing)
                };
                button.setImage(Some(&bar_image(bar, pad)));
                button.setImagePosition(position);
            }
            None => {
                button.setImage(None);
                button.setImagePosition(NSCellImagePosition::NoImage);
            }
        }
    }

    fn refresh_style_checks(&self) {
        let ivars = self.ivars().borrow();
        let Some(ui) = ivars.as_ref() else { return };
        for (index, item) in ui.style_items.iter().enumerate() {
            item.setState(if ALL_STYLES[index] == ui.style {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            });
        }
    }
}

fn toggle_low_power_mode(currently_enabled: bool) {
    let target = if currently_enabled { "0" } else { "1" };
    let script =
        format!("do shell script \"pmset -a lowpowermode {target}\" with administrator privileges");
    if let Err(err) = Command::new("osascript").args(["-e", &script]).status() {
        eprintln!("error toggling low power mode: {err}");
    }
}

fn main() {
    let mtm = MainThreadMarker::new().expect("must run on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let widget = Widget::new(mtm);
    widget.build_ui(mtm);
    widget.update();

    unsafe {
        NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
            UPDATE_INTERVAL_SECONDS,
            &widget,
            sel!(tick:),
            None,
            true,
        );
        NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
            ANIMATION_INTERVAL_SECONDS,
            &widget,
            sel!(animate:),
            None,
            true,
        );
    }

    app.run();
}
