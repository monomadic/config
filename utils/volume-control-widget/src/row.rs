//! One output device as a menu row: a bare device glyph at the left, the
//! name with its bus pill, a spec line (`24bit 48khz – 12 in, 10 out`), a
//! volume line whose head is the mute speaker — the macOS Sound slider's own
//! anatomy — and, where the disk widget keeps eject, a circular lock button
//! that cycles the device's rule: open padlock (no rule), filled padlock
//! (Always Mute), shield (Never Mute, read-only).
//!
//! Three click targets: the lock cycles the rule and the mute speaker
//! toggles mute, both leaving the menu open; anywhere else selects the
//! device as the default output and closes the menu. The row draws its own
//! hover highlight and text with the menu font, exactly as the disk widget's
//! rows do — a menu item hosting a view gets nothing drawn for it.

use std::cell::Cell;

use objc2::Message;
use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{
    NSBezierPath, NSColor, NSCompositingOperation, NSEvent, NSFont, NSFontAttributeName,
    NSFontWeightMedium, NSFontWeightRegular, NSForegroundColorAttributeName, NSImage,
    NSRectFillUsingOperation, NSStringDrawing, NSTrackingArea, NSTrackingAreaOptions, NSView,
};
use objc2_foundation::{NSMutableDictionary, NSPoint, NSRect, NSSize, NSString};

use crate::audio::{self, AudioObjectID, Transport};
use crate::settings::{self, Policy};

/// How a row presents, beyond its data.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RowState {
    Normal,
    /// The default output: blue row, white ink.
    Active,
    /// A Never Mute device that has disappeared: red.
    Missing,
    /// An Always Mute device that became the default output: amber.
    Fallback,
    /// Known but disconnected, no warning owed: dimmed.
    Away,
}

pub struct RowSpec {
    /// None when the device is not connected.
    pub device_id: Option<AudioObjectID>,
    pub uid: String,
    pub name: String,
    pub transport: Transport,
    pub symbol: &'static str,
    pub spec: String,
    pub volume: Option<f64>,
    pub muted: bool,
    pub can_mute: bool,
    pub state: RowState,
    pub policy: Policy,
}

/// Shared column geometry for one menu's worth of rows.
pub struct Layout {
    font: Retained<NSFont>,
    spec_font: Retained<NSFont>,
    pill_font: Retained<NSFont>,
    pub height: f64,
    width: f64,
    icon_x: f64,
    icon_column: f64,
    icon_size: f64,
    text_left: f64,
    text_right: f64,
    lock_center_x: f64,
    lock_diameter: f64,
    bar_height: f64,
    mute_size: f64,
}

pub fn layout(rows: &[RowSpec]) -> Layout {
    let font = NSFont::menuFontOfSize(0.0);
    let em = font.pointSize();
    let spec_font = NSFont::monospacedDigitSystemFontOfSize_weight((em * 0.72).round(), unsafe {
        NSFontWeightRegular
    });
    let pill_font = NSFont::monospacedDigitSystemFontOfSize_weight((em * 0.62).round(), unsafe {
        NSFontWeightMedium
    });

    let left = (em * 1.0).round();
    let right = (em * 1.0).round();
    let gap = (em * 0.8).round();
    let icon_size = (em * 1.9).round();
    let icon_column = (em * 2.2).round();
    let lock_diameter = (em * 1.45).round();
    let bar_height = (em * 0.28).round().max(3.0);
    let mute_size = (em * 0.85).round();

    let widest = rows
        .iter()
        .map(|row| {
            let mut width = text_size(&font, &row.name).width;
            if let Some(pill) = row.transport.pill() {
                width += text_size(&pill_font, pill).width + em * 1.0;
            }
            width.max(text_size(&spec_font, &row.spec).width)
        })
        .fold(0.0, f64::max);

    let text_left = left + icon_column + gap;
    let text_width = widest.max(em * 13.0);
    let width = text_left + text_width + gap + lock_diameter + right;

    // Three stacked lines: name, spec, bar. Height from the fonts plus
    // breathing room, like a Finder sidebar row that grew a slider.
    let name_height = text_size(&font, "Ag").height;
    let spec_height = text_size(&spec_font, "Ag").height;
    let height = (name_height + spec_height + bar_height + em * 1.5).round();

    Layout {
        font,
        spec_font,
        pill_font,
        height,
        width,
        icon_x: left,
        icon_column,
        icon_size,
        text_left,
        text_right: text_left + text_width,
        lock_center_x: width - right - lock_diameter / 2.0,
        lock_diameter,
        bar_height,
        mute_size,
    }
}

pub struct RowIvars {
    spec: RowSpec,
    muted: Cell<bool>,
    volume: Cell<Option<f64>>,
    policy: Cell<Policy>,
    font: Retained<NSFont>,
    spec_font: Retained<NSFont>,
    pill_font: Retained<NSFont>,
    icon_x: f64,
    icon_column: f64,
    icon_size: f64,
    text_left: f64,
    text_right: f64,
    lock_center_x: f64,
    lock_diameter: f64,
    bar_height: f64,
    mute_size: f64,
    hovered: Cell<bool>,
    lock_hovered: Cell<bool>,
}

define_class!(
    // SAFETY: NSView imposes no subclassing requirements beyond initialising
    // through the superclass, and DeviceRow does not implement Drop.
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[name = "VolumeControlDeviceRow"]
    #[ivars = RowIvars]
    pub struct DeviceRow;

    impl DeviceRow {
        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, _dirty: NSRect) {
            let ivars = self.ivars();
            let bounds = self.bounds();
            let state = ivars.spec.state;

            match state {
                RowState::Active => fill_row(bounds, &NSColor::systemBlueColor(), 1.0),
                RowState::Missing => fill_row(bounds, &NSColor::systemRedColor(), 0.16),
                RowState::Fallback => fill_row(bounds, &NSColor::systemOrangeColor(), 0.16),
                _ => {}
            }
            if ivars.hovered.get() && state != RowState::Active {
                fill_row(bounds, &NSColor::labelColor(), 0.10);
            }

            // The dashed border marks a mute-lock; missing Never Mute
            // devices borrow it in red so the broken promise reads as one.
            match (ivars.policy.get(), state) {
                (Policy::AlwaysMute, _) => {
                    let ink = if state == RowState::Fallback {
                        NSColor::systemOrangeColor()
                    } else if state == RowState::Active {
                        NSColor::whiteColor().colorWithAlphaComponent(0.6)
                    } else {
                        NSColor::labelColor().colorWithAlphaComponent(0.35)
                    };
                    stroke_dashed(bounds, &ink);
                }
                (Policy::NeverMute, RowState::Missing) => {
                    stroke_dashed(bounds, &NSColor::systemRedColor().colorWithAlphaComponent(0.6));
                }
                _ => {}
            }

            self.draw_icon(bounds);
            self.draw_text(bounds);
            self.draw_lock(bounds);
        }

        #[unsafe(method(mouseEntered:))]
        fn mouse_entered(&self, event: &NSEvent) {
            self.ivars().hovered.set(true);
            self.track_pointer(event);
        }

        #[unsafe(method(mouseMoved:))]
        fn mouse_moved(&self, event: &NSEvent) {
            self.track_pointer(event);
        }

        #[unsafe(method(mouseExited:))]
        fn mouse_exited(&self, _event: &NSEvent) {
            self.ivars().hovered.set(false);
            self.ivars().lock_hovered.set(false);
            self.setNeedsDisplay(true);
        }

        #[unsafe(method(mouseUp:))]
        fn mouse_up(&self, event: &NSEvent) {
            let ivars = self.ivars();
            if self.point_in_lock(event) {
                let next = settings::cycle_policy(
                    &ivars.spec.uid,
                    &ivars.spec.name,
                    ivars.spec.transport,
                );
                ivars.policy.set(next);
                // A device that just became Always Mute should go quiet now,
                // not at the next listener event.
                if next == Policy::AlwaysMute
                    && let Some(id) = ivars.spec.device_id
                {
                    audio::set_muted(id, true);
                    ivars.muted.set(true);
                }
                audio::mark_dirty();
                self.setNeedsDisplay(true);
                return;
            }
            if self.point_in_mute(event) {
                let Some(id) = ivars.spec.device_id else { return };
                // Locked devices don't toggle: Always Mute would re-mute
                // instantly, Never Mute is read-only by contract.
                if !ivars.spec.can_mute || ivars.policy.get() != Policy::None {
                    return;
                }
                let muted = !ivars.muted.get();
                audio::set_muted(id, muted);
                ivars.muted.set(muted);
                self.setNeedsDisplay(true);
                return;
            }
            let Some(id) = ivars.spec.device_id else { return };
            // A click on the bar itself sets the level — the row's one write
            // besides mute, and never on a read-only device.
            if let Some(fraction) = self.point_on_bar(event) {
                if ivars.policy.get() != Policy::NeverMute {
                    audio::set_volume(id, fraction);
                    ivars.volume.set(Some(fraction));
                    audio::mark_dirty();
                    self.setNeedsDisplay(true);
                }
                return;
            }
            if ivars.spec.state != RowState::Active {
                audio::set_default_output(id);
            }
            self.dismiss_menu();
        }
    }
);

impl DeviceRow {
    pub fn new(spec: RowSpec, layout: &Layout, mtm: MainThreadMarker) -> Retained<Self> {
        let muted = Cell::new(spec.muted);
        let volume = Cell::new(spec.volume);
        let policy = Cell::new(spec.policy);
        let this = Self::alloc(mtm).set_ivars(RowIvars {
            spec,
            muted,
            volume,
            policy,
            font: layout.font.clone(),
            spec_font: layout.spec_font.clone(),
            pill_font: layout.pill_font.clone(),
            icon_x: layout.icon_x,
            icon_column: layout.icon_column,
            icon_size: layout.icon_size,
            text_left: layout.text_left,
            text_right: layout.text_right,
            lock_center_x: layout.lock_center_x,
            lock_diameter: layout.lock_diameter,
            bar_height: layout.bar_height,
            mute_size: layout.mute_size,
            hovered: Cell::new(false),
            lock_hovered: Cell::new(false),
        });
        let frame = NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            size: NSSize {
                width: layout.width,
                height: layout.height,
            },
        };
        let this: Retained<Self> = unsafe { msg_send![super(this), initWithFrame: frame] };

        if this.ivars().spec.state == RowState::Away {
            this.setAlphaValue(0.45);
        }

        let tracking = unsafe {
            NSTrackingArea::initWithRect_options_owner_userInfo(
                NSTrackingArea::alloc(),
                NSRect::ZERO,
                NSTrackingAreaOptions::MouseEnteredAndExited
                    | NSTrackingAreaOptions::MouseMoved
                    | NSTrackingAreaOptions::ActiveAlways
                    | NSTrackingAreaOptions::InVisibleRect,
                Some(this.as_ref()),
                None,
            )
        };
        this.addTrackingArea(&tracking);
        this
    }

    fn ink(&self) -> Retained<NSColor> {
        match self.ivars().spec.state {
            RowState::Active => NSColor::whiteColor(),
            _ => NSColor::labelColor(),
        }
    }

    fn draw_icon(&self, bounds: NSRect) {
        let ivars = self.ivars();
        let tint = match ivars.spec.state {
            RowState::Active => NSColor::whiteColor(),
            RowState::Missing => NSColor::systemRedColor(),
            RowState::Fallback => NSColor::systemOrangeColor(),
            _ => NSColor::labelColor().colorWithAlphaComponent(0.85),
        };
        let Some(icon) = tinted_symbol(ivars.spec.symbol, &tint) else {
            return;
        };
        let size = icon.size();
        let scale = if size.width > 0.0 && size.height > 0.0 {
            (ivars.icon_size / size.width).min(ivars.icon_size / size.height)
        } else {
            1.0
        };
        let width = size.width * scale;
        let height = size.height * scale;
        icon.drawInRect_fromRect_operation_fraction(
            rect(
                ivars.icon_x + (ivars.icon_column - width) / 2.0,
                (bounds.size.height / 2.0 - height / 2.0).round(),
                width,
                height,
            ),
            NSRect::ZERO,
            NSCompositingOperation::SourceOver,
            1.0,
        );
    }

    fn draw_text(&self, bounds: NSRect) {
        let ivars = self.ivars();
        let ink = self.ink();
        let em = ivars.font.pointSize();

        let name_size = text_size(&ivars.font, &ivars.spec.name);
        let spec_size = text_size(&ivars.spec_font, &ivars.spec.spec);
        let line_gap = (em * 0.22).round();
        let has_bar = ivars.volume.get().is_some() && ivars.spec.state != RowState::Missing;
        let bar_line = if has_bar {
            ivars.bar_height.max(ivars.mute_size) + line_gap
        } else {
            0.0
        };
        let content = name_size.height + line_gap + spec_size.height + bar_line;
        let top = bounds.size.height - ((bounds.size.height - content) / 2.0).round();

        // Name line, with the bus pill inline after it.
        let name_y = top - name_size.height;
        draw_text(
            &ivars.spec.name,
            &ivars.font,
            &ink,
            NSPoint {
                x: ivars.text_left,
                y: name_y,
            },
        );
        if let Some(pill) = ivars.spec.transport.pill() {
            self.draw_pill(pill, ivars.text_left + name_size.width + em * 0.45, name_y, &ink);
        }

        // Spec line.
        let spec_color = match ivars.spec.state {
            RowState::Active => NSColor::whiteColor().colorWithAlphaComponent(0.8),
            RowState::Missing => NSColor::systemRedColor(),
            _ => NSColor::secondaryLabelColor(),
        };
        let spec_y = name_y - line_gap - spec_size.height;
        draw_text(
            &ivars.spec.spec,
            &ivars.spec_font,
            &spec_color,
            NSPoint {
                x: ivars.text_left,
                y: spec_y,
            },
        );

        // Volume line: mute speaker at the head, bar to the column's edge.
        if has_bar {
            let line_center = spec_y - line_gap - ivars.mute_size / 2.0;
            let muted = ivars.muted.get();
            let symbol = if muted {
                "speaker.slash.fill"
            } else {
                "speaker.wave.2.fill"
            };
            let mute_tint = match ivars.spec.state {
                RowState::Active => NSColor::whiteColor().colorWithAlphaComponent(0.9),
                _ if muted => NSColor::labelColor().colorWithAlphaComponent(0.8),
                _ => NSColor::secondaryLabelColor(),
            };
            if let Some(icon) = tinted_symbol(symbol, &mute_tint) {
                let size = icon.size();
                let scale = (ivars.mute_size / size.width)
                    .min(ivars.mute_size / size.height)
                    .min(1.0);
                let width = size.width * scale;
                let height = size.height * scale;
                icon.drawInRect_fromRect_operation_fraction(
                    rect(
                        ivars.text_left,
                        (line_center - height / 2.0).round(),
                        width,
                        height,
                    ),
                    NSRect::ZERO,
                    NSCompositingOperation::SourceOver,
                    1.0,
                );
            }

            let bar_x = ivars.text_left + ivars.mute_size + em * 0.5;
            let bar_width = ivars.text_right - bar_x;
            let bar_y = (line_center - ivars.bar_height / 2.0).round();
            let radius = ivars.bar_height / 2.0;
            let track = match ivars.spec.state {
                RowState::Active => NSColor::whiteColor().colorWithAlphaComponent(0.3),
                _ => NSColor::labelColor().colorWithAlphaComponent(0.16),
            };
            track.set();
            NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                rect(bar_x, bar_y, bar_width, ivars.bar_height),
                radius,
                radius,
            )
            .fill();

            let level = ivars.volume.get().unwrap_or(0.0).clamp(0.0, 1.0);
            if level > 0.0 {
                let fill = match ivars.spec.state {
                    RowState::Active => NSColor::whiteColor(),
                    _ if muted => NSColor::labelColor().colorWithAlphaComponent(0.4),
                    _ => NSColor::labelColor().colorWithAlphaComponent(0.75),
                };
                fill.set();
                NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                    rect(
                        bar_x,
                        bar_y,
                        (bar_width * level).max(ivars.bar_height),
                        ivars.bar_height,
                    ),
                    radius,
                    radius,
                )
                .fill();
            }
        }
    }

    /// The bus pill: a hairline rounded outline around small caps text.
    fn draw_pill(&self, text: &str, x: f64, name_y: f64, ink: &NSColor) {
        let ivars = self.ivars();
        let size = text_size(&ivars.pill_font, text);
        let pad = (ivars.pill_font.pointSize() * 0.45).round();
        let height = size.height + 2.0;
        let name_height = text_size(&ivars.font, "Ag").height;
        let y = name_y + ((name_height - height) / 2.0).round();

        let color = match ivars.spec.state {
            RowState::Active => NSColor::whiteColor().colorWithAlphaComponent(0.8),
            _ => NSColor::secondaryLabelColor(),
        };
        let outline = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
            rect(x, y, size.width + pad * 2.0, height),
            height / 2.0,
            height / 2.0,
        );
        color.colorWithAlphaComponent(0.5).set();
        outline.setLineWidth(1.0);
        outline.stroke();
        draw_text(
            text,
            &ivars.pill_font,
            &color,
            NSPoint {
                x: x + pad,
                y: y + 1.0,
            },
        );
        let _ = ink;
    }

    /// The lock button: eject's circle with the rule inside it. Open padlock
    /// on translucent grey means no rule; a filled padlock on a solid disc
    /// means Always Mute; a shield on blue means Never Mute.
    fn draw_lock(&self, bounds: NSRect) {
        let ivars = self.ivars();
        let diameter = ivars.lock_diameter;
        let center = NSPoint {
            x: ivars.lock_center_x,
            y: bounds.size.height / 2.0,
        };
        let hovered = ivars.lock_hovered.get();

        let (circle, glyph, symbol): (Retained<NSColor>, Retained<NSColor>, &str) =
            match ivars.policy.get() {
                Policy::None => (
                    NSColor::labelColor()
                        .colorWithAlphaComponent(if hovered { 0.26 } else { 0.13 }),
                    if hovered {
                        NSColor::labelColor()
                    } else {
                        NSColor::secondaryLabelColor()
                    },
                    "lock.open.fill",
                ),
                Policy::AlwaysMute => (
                    NSColor::labelColor().colorWithAlphaComponent(if hovered { 1.0 } else { 0.85 }),
                    NSColor::windowBackgroundColor(),
                    "lock.fill",
                ),
                Policy::NeverMute => (
                    NSColor::systemBlueColor()
                        .colorWithAlphaComponent(if hovered { 1.0 } else { 0.9 }),
                    NSColor::whiteColor(),
                    "checkmark.shield.fill",
                ),
            };

        circle.set();
        NSBezierPath::bezierPathWithOvalInRect(rect(
            center.x - diameter / 2.0,
            center.y - diameter / 2.0,
            diameter,
            diameter,
        ))
        .fill();

        if let Some(icon) = tinted_symbol(symbol, &glyph) {
            let target = diameter * 0.52;
            let size = icon.size();
            let scale = if size.width > 0.0 && size.height > 0.0 {
                (target / size.width).min(target / size.height)
            } else {
                1.0
            };
            let width = size.width * scale;
            let height = size.height * scale;
            icon.drawInRect_fromRect_operation_fraction(
                rect(center.x - width / 2.0, center.y - height / 2.0, width, height),
                NSRect::ZERO,
                NSCompositingOperation::SourceOver,
                1.0,
            );
        }
    }

    fn point_in_lock(&self, event: &NSEvent) -> bool {
        let ivars = self.ivars();
        let point = self.convertPoint_fromView(event.locationInWindow(), None);
        let bounds = self.bounds();
        let dx = point.x - ivars.lock_center_x;
        let dy = point.y - bounds.size.height / 2.0;
        let reach = ivars.lock_diameter / 2.0 + 3.0;
        dx * dx + dy * dy <= reach * reach
    }

    /// Where along the volume bar a click landed, as 0..1 — None when the
    /// point is off the volume line or the row draws no bar.
    fn point_on_bar(&self, event: &NSEvent) -> Option<f64> {
        let ivars = self.ivars();
        if ivars.volume.get().is_none() || ivars.spec.state == RowState::Missing {
            return None;
        }
        let point = self.convertPoint_fromView(event.locationInWindow(), None);
        let bounds = self.bounds();
        if point.y > bounds.size.height * 0.42 {
            return None;
        }
        let em = ivars.font.pointSize();
        let bar_x = ivars.text_left + ivars.mute_size + em * 0.5;
        (point.x >= bar_x - 2.0 && point.x <= ivars.text_right + 2.0)
            .then(|| ((point.x - bar_x) / (ivars.text_right - bar_x)).clamp(0.0, 1.0))
    }

    /// A forgiving circle around the mute speaker at the volume line's head.
    fn point_in_mute(&self, event: &NSEvent) -> bool {
        let ivars = self.ivars();
        if ivars.volume.get().is_none() || ivars.spec.state == RowState::Missing {
            return false;
        }
        let point = self.convertPoint_fromView(event.locationInWindow(), None);
        // The volume line sits in the lower third of the row; accept the
        // icon's column below the spec line rather than re-deriving exact
        // text geometry for hit testing.
        let bounds = self.bounds();
        point.x >= ivars.text_left - 4.0
            && point.x <= ivars.text_left + ivars.mute_size + 6.0
            && point.y <= bounds.size.height * 0.42
    }

    fn track_pointer(&self, event: &NSEvent) {
        let over_lock = self.point_in_lock(event);
        if over_lock != self.ivars().lock_hovered.get() {
            self.ivars().lock_hovered.set(over_lock);
        }
        self.setNeedsDisplay(true);
    }

    fn dismiss_menu(&self) {
        if let Some(menu) = self
            .enclosingMenuItem()
            .and_then(|item| unsafe { item.menu() })
        {
            menu.cancelTracking();
        }
    }
}

fn fill_row(bounds: NSRect, color: &NSColor, alpha: f64) {
    color.colorWithAlphaComponent(alpha).set();
    row_path(bounds).fill();
}

fn stroke_dashed(bounds: NSRect, ink: &NSColor) {
    ink.set();
    let path = row_path(NSRect {
        origin: NSPoint {
            x: bounds.origin.x + 0.5,
            y: bounds.origin.y + 0.5,
        },
        size: NSSize {
            width: bounds.size.width - 1.0,
            height: bounds.size.height - 1.0,
        },
    });
    path.setLineWidth(1.0);
    let pattern: [f64; 2] = [4.0, 3.0];
    unsafe { path.setLineDash_count_phase(pattern.as_ptr(), 2, 0.0) };
    path.stroke();
}

/// The native menu highlight's insets and radius, shared by fill and border.
fn row_path(bounds: NSRect) -> Retained<NSBezierPath> {
    NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
        rect(
            bounds.origin.x + 5.0,
            bounds.origin.y,
            bounds.size.width - 10.0,
            bounds.size.height,
        ),
        6.0,
        6.0,
    )
}

/// An SF Symbol flooded with a colour through its own alpha, as the disk
/// widget tints its badge — resolved at draw time for the current appearance.
fn tinted_symbol(name: &str, tint: &NSColor) -> Option<Retained<NSImage>> {
    let symbol =
        NSImage::imageWithSystemSymbolName_accessibilityDescription(&NSString::from_str(name), None)?;
    let size = symbol.size();
    let tint = tint.retain();
    let handler = block2::RcBlock::new(move |bounds: NSRect| -> objc2::runtime::Bool {
        symbol.drawInRect_fromRect_operation_fraction(
            bounds,
            NSRect::ZERO,
            NSCompositingOperation::SourceOver,
            1.0,
        );
        tint.set();
        NSRectFillUsingOperation(bounds, NSCompositingOperation::SourceAtop);
        objc2::runtime::Bool::YES
    });
    Some(NSImage::imageWithSize_flipped_drawingHandler(
        size, false, &handler,
    ))
}

fn text_attributes(
    font: &NSFont,
    color: Option<&NSColor>,
) -> Retained<NSMutableDictionary<NSString, AnyObject>> {
    let attrs = NSMutableDictionary::<NSString, AnyObject>::new();
    unsafe {
        attrs.setObject_forKey(font, ProtocolObject::from_ref(NSFontAttributeName));
        if let Some(color) = color {
            attrs.setObject_forKey(
                color,
                ProtocolObject::from_ref(NSForegroundColorAttributeName),
            );
        }
    }
    attrs
}

fn text_size(font: &NSFont, text: &str) -> NSSize {
    let attrs = text_attributes(font, None);
    unsafe { NSString::from_str(text).sizeWithAttributes(Some(&attrs)) }
}

fn draw_text(text: &str, font: &NSFont, color: &NSColor, origin: NSPoint) {
    let attrs = text_attributes(font, Some(color));
    unsafe { NSString::from_str(text).drawAtPoint_withAttributes(origin, Some(&attrs)) };
}

fn rect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
    NSRect {
        origin: NSPoint { x, y },
        size: NSSize { width, height },
    }
}
