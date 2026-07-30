//! One mounted volume as a menu row, Finder-sidebar style: an outline SF
//! Symbol for the volume kind on a dark disc at the left, name with the free
//! amount small and right-aligned, a capacity bar underneath, and — for
//! anything that isn't a system volume — an always-visible eject mark on a
//! light disc in its own column at the far right.
//!
//! A menu item can host a view, but then AppKit draws none of it — so the row
//! draws its own text with the menu font (`NSFont::menuFontOfSize(0.0)`) and
//! takes every measurement from that font, which keeps it sized with the
//! system text size. Two separate click targets, as in Finder: the row opens
//! the volume in Finder, the eject button unmounts it. Both are drawn by the
//! row itself, which tracks the mouse to paint the native-looking row
//! highlight and to brighten the eject circle under the pointer — AppKit only
//! highlights items it draws.
//!
//! All rows in one menu share a `Layout`, so the text column and the eject
//! buttons line up down the menu instead of following each name.

use std::cell::Cell;
use std::path::PathBuf;
use std::process::Command;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_app_kit::{
    NSBezierPath, NSColor, NSCompositingOperation, NSEvent, NSFont, NSFontAttributeName,
    NSFontWeightRegular, NSForegroundColorAttributeName, NSImage, NSImageSymbolConfiguration,
    NSStringDrawing, NSTrackingArea, NSTrackingAreaOptions, NSView,
};
use objc2_foundation::{NSMutableDictionary, NSPoint, NSRect, NSSize, NSString, ns_string};

use crate::volumes::{self, Volume, VolumeKind, format_bytes};

/// Below this much free space the amount and the bar fill turn red.
const LOW_SPACE_RATIO: f64 = 0.10;

/// Outline symbols, not the filled ones: the dark scrim behind them supplies
/// the weight the fill would have, without the icons out-shouting the names.
fn symbol_name(kind: VolumeKind) -> &'static NSString {
    match kind {
        VolumeKind::Internal => ns_string!("internaldrive"),
        VolumeKind::External => ns_string!("externaldrive"),
        VolumeKind::Network => ns_string!("network"),
    }
}

/// Shared column geometry for one menu's worth of rows.
pub struct Layout {
    font: Retained<NSFont>,
    detail_font: Retained<NSFont>,
    pub height: f64,
    width: f64,
    icon_x: f64,
    /// Diameter of the dark disc behind the icon; the glyph sits inside it.
    icon_scrim: f64,
    icon_size: f64,
    text_left: f64,
    /// Right edge of the text column: the free amount and the bar end here.
    text_right: f64,
    /// Centre of the eject circle; zero-diameter when no row needs one.
    button_center_x: f64,
    button_diameter: f64,
    bar_height: f64,
}

pub fn layout(volumes: &[Volume]) -> Layout {
    let font = NSFont::menuFontOfSize(0.0);
    let em = font.pointSize();
    let detail_font = NSFont::monospacedDigitSystemFontOfSize_weight((em * 0.78).round(), unsafe {
        NSFontWeightRegular
    });

    let height = (em * 3.4).round();
    let left = (em * 1.1).round();
    let right = (em * 1.0).round();
    let gap = (em * 0.9).round();
    // The two discs carry the row's left and right ends. The eject disc is a
    // button and the lighter of the two, so it wins on contrast at a smaller
    // size; the volume icon balances it with a wide, quiet disc and a mark
    // large enough to identify the drive at a glance.
    let icon_scrim = (em * 2.0).round();
    let icon_size = (em * 1.4).round();
    let button_diameter = (em * 1.45).round();
    let bar_height = (em * 0.28).round().max(3.0);

    let widest = |text: &dyn Fn(&Volume) -> String| {
        volumes
            .iter()
            .map(|volume| text_size(&font, &text(volume)).width)
            .fold(0.0, f64::max)
    };
    let name_width = widest(&|volume: &Volume| volume.name.clone());
    let detail_width = volumes
        .iter()
        .map(|volume| text_size(&detail_font, &format_bytes(volume.free)).width)
        .fold(0.0, f64::max);

    let text_left = left + icon_scrim + gap;
    // The name and amount share the top line; the bar takes the full column,
    // so the column just needs to fit both with breathing room between.
    let text_width = (name_width + gap * 3.0 + detail_width).max(em * 13.0);

    let button_column = if volumes.iter().any(Volume::unmountable) {
        gap + button_diameter
    } else {
        0.0
    };
    let width = text_left + text_width + button_column + right;

    Layout {
        font,
        detail_font,
        height,
        width,
        icon_x: left,
        icon_scrim,
        icon_size,
        text_left,
        text_right: text_left + text_width,
        button_center_x: width - right - button_diameter / 2.0,
        button_diameter,
        bar_height,
    }
}

pub struct RowIvars {
    name: String,
    detail: String,
    path: PathBuf,
    used_ratio: f64,
    low: bool,
    ejectable: bool,
    kind: VolumeKind,
    font: Retained<NSFont>,
    detail_font: Retained<NSFont>,
    icon_x: f64,
    icon_scrim: f64,
    icon_size: f64,
    text_left: f64,
    text_right: f64,
    button_center_x: f64,
    button_diameter: f64,
    bar_height: f64,
    hovered: Cell<bool>,
    eject_hovered: Cell<bool>,
}

define_class!(
    // SAFETY: NSView imposes no subclassing requirements beyond initialising
    // through the superclass, and VolumeRow does not implement Drop.
    #[unsafe(super(NSView))]
    #[thread_kind = MainThreadOnly]
    #[name = "FreeDiskSpaceVolumeRow"]
    #[ivars = RowIvars]
    pub struct VolumeRow;

    impl VolumeRow {
        #[unsafe(method(drawRect:))]
        fn draw_rect(&self, _dirty: NSRect) {
            let ivars = self.ivars();
            let bounds = self.bounds();

            if ivars.hovered.get() {
                draw_highlight(bounds);
            }

            self.draw_icon(bounds);

            // Top line: name left, free amount right, sharing a baseline.
            // Below it the bar spans the whole text column.
            let name_size = text_size(&ivars.font, &ivars.name);
            let line_gap = (ivars.font.pointSize() * 0.45).round();
            let content = name_size.height + line_gap + ivars.bar_height;
            let name_y = bounds.size.height - ((bounds.size.height - content) / 2.0).round()
                - name_size.height;

            draw_text(
                &ivars.name,
                &ivars.font,
                &NSColor::labelColor(),
                NSPoint {
                    x: ivars.text_left,
                    y: name_y,
                },
            );

            let detail_color = if ivars.low {
                NSColor::systemRedColor()
            } else {
                NSColor::secondaryLabelColor()
            };
            let detail_size = text_size(&ivars.detail_font, &ivars.detail);
            draw_text(
                &ivars.detail,
                &ivars.detail_font,
                &detail_color,
                NSPoint {
                    x: ivars.text_right - detail_size.width,
                    // Align to the name's baseline, not its box.
                    y: name_y + ivars.font.descender() - ivars.detail_font.descender(),
                },
            );

            let bar_y = name_y - line_gap - ivars.bar_height;
            let bar_width = ivars.text_right - ivars.text_left;
            let radius = ivars.bar_height / 2.0;
            NSColor::labelColor().colorWithAlphaComponent(0.16).set();
            NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                rect(ivars.text_left, bar_y, bar_width, ivars.bar_height),
                radius,
                radius,
            )
            .fill();

            // The bar fills with used space, so a full bar means a full disk.
            if ivars.used_ratio > 0.0 {
                let filled = (bar_width * ivars.used_ratio).max(ivars.bar_height);
                if ivars.low {
                    NSColor::systemRedColor().set();
                } else {
                    NSColor::labelColor().colorWithAlphaComponent(0.75).set();
                }
                NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
                    rect(ivars.text_left, bar_y, filled, ivars.bar_height),
                    radius,
                    radius,
                )
                .fill();
            }

            if ivars.ejectable {
                self.draw_eject(bounds);
            }
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
            self.ivars().eject_hovered.set(false);
            self.setNeedsDisplay(true);
        }

        // One view, two targets: the eject circle unmounts, the rest of the
        // row opens the volume in Finder. The menu closes first either way —
        // unmounting is asynchronous, and a menu left open would keep showing
        // a volume that is on its way out.
        #[unsafe(method(mouseUp:))]
        fn mouse_up(&self, event: &NSEvent) {
            self.dismiss_menu();
            if self.point_in_eject(event) {
                volumes::unmount(&self.ivars().path);
            } else {
                let _ = Command::new("open").arg(&self.ivars().path).spawn();
            }
        }
    }
);

impl VolumeRow {
    pub fn new(volume: &Volume, layout: &Layout, mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(RowIvars {
            name: volume.name.clone(),
            detail: format_bytes(volume.free),
            path: volume.path.clone(),
            used_ratio: 1.0 - volume.free_ratio(),
            low: volume.free_ratio() < LOW_SPACE_RATIO,
            ejectable: volume.unmountable(),
            kind: volume.kind,
            font: layout.font.clone(),
            detail_font: layout.detail_font.clone(),
            icon_x: layout.icon_x,
            icon_scrim: layout.icon_scrim,
            icon_size: layout.icon_size,
            text_left: layout.text_left,
            text_right: layout.text_right,
            button_center_x: layout.button_center_x,
            button_diameter: layout.button_diameter,
            bar_height: layout.bar_height,
            hovered: Cell::new(false),
            eject_hovered: Cell::new(false),
        });
        let frame = NSRect {
            origin: NSPoint { x: 0.0, y: 0.0 },
            size: NSSize {
                width: layout.width,
                height: layout.height,
            },
        };
        let this: Retained<Self> = unsafe { msg_send![super(this), initWithFrame: frame] };

        // InVisibleRect keeps the area matched to the bounds for us, and
        // ActiveAlways is required because a menu never makes us key.
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

    /// The volume icon on a dark disc — the opposite polarity to the eject
    /// circle's light one, so the two ends of the row read as different kinds
    /// of thing while carrying the same weight.
    fn draw_icon(&self, bounds: NSRect) {
        let ivars = self.ivars();
        let center_y = bounds.size.height / 2.0;

        NSColor::blackColor().colorWithAlphaComponent(0.22).set();
        NSBezierPath::bezierPathWithOvalInRect(rect(
            ivars.icon_x,
            center_y - ivars.icon_scrim / 2.0,
            ivars.icon_scrim,
            ivars.icon_scrim,
        ))
        .fill();

        let Some(symbol) = NSImage::imageWithSystemSymbolName_accessibilityDescription(
            symbol_name(ivars.kind),
            None,
        ) else {
            return;
        };
        // Hierarchical colour bakes the tint in, so the template image can be
        // drawn directly; rows are rebuilt each time the menu opens, which
        // re-resolves it for the current appearance.
        let config = NSImageSymbolConfiguration::configurationWithHierarchicalColor(
            &NSColor::labelColor().colorWithAlphaComponent(0.85),
        );
        let Some(icon) = symbol.imageWithSymbolConfiguration(&config) else {
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
                ivars.icon_x + (ivars.icon_scrim - width) / 2.0,
                (center_y - height / 2.0).round(),
                width,
                height,
            ),
            NSRect::ZERO,
            NSCompositingOperation::SourceOver,
            1.0,
        );
    }

    /// The eject affordance: a translucent circle that brightens under the
    /// pointer, with the eject mark (triangle over a bar) drawn inside it.
    fn draw_eject(&self, bounds: NSRect) {
        let ivars = self.ivars();
        let hovered = ivars.eject_hovered.get();
        let diameter = ivars.button_diameter;
        let center = NSPoint {
            x: ivars.button_center_x,
            y: bounds.size.height / 2.0,
        };

        let circle_alpha = if hovered { 0.26 } else { 0.13 };
        NSColor::labelColor()
            .colorWithAlphaComponent(circle_alpha)
            .set();
        NSBezierPath::bezierPathWithOvalInRect(rect(
            center.x - diameter / 2.0,
            center.y - diameter / 2.0,
            diameter,
            diameter,
        ))
        .fill();

        let ink = if hovered {
            NSColor::labelColor()
        } else {
            NSColor::secondaryLabelColor()
        };
        ink.set();

        // Glyph metrics as fractions of the circle.
        let glyph_width = diameter * 0.42;
        let triangle_height = diameter * 0.26;
        let bar_height = (diameter * 0.09).max(1.5);
        let spacing = diameter * 0.09;
        let total = triangle_height + spacing + bar_height;
        let bottom = center.y - total / 2.0;

        let bar = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
            rect(
                center.x - glyph_width / 2.0,
                bottom,
                glyph_width,
                bar_height,
            ),
            bar_height / 2.0,
            bar_height / 2.0,
        );
        bar.fill();

        let triangle = NSBezierPath::new();
        let triangle_bottom = bottom + bar_height + spacing;
        triangle.moveToPoint(NSPoint {
            x: center.x - glyph_width / 2.0,
            y: triangle_bottom,
        });
        triangle.lineToPoint(NSPoint {
            x: center.x + glyph_width / 2.0,
            y: triangle_bottom,
        });
        triangle.lineToPoint(NSPoint {
            x: center.x,
            y: triangle_bottom + triangle_height,
        });
        triangle.closePath();
        triangle.fill();
    }

    fn point_in_eject(&self, event: &NSEvent) -> bool {
        let ivars = self.ivars();
        if !ivars.ejectable {
            return false;
        }
        let point = self.convertPoint_fromView(event.locationInWindow(), None);
        let bounds = self.bounds();
        let dx = point.x - ivars.button_center_x;
        let dy = point.y - bounds.size.height / 2.0;
        // A little forgiveness beyond the drawn circle.
        let reach = ivars.button_diameter / 2.0 + 3.0;
        dx * dx + dy * dy <= reach * reach
    }

    fn track_pointer(&self, event: &NSEvent) {
        let over_eject = self.point_in_eject(event);
        if over_eject != self.ivars().eject_hovered.get() {
            self.ivars().eject_hovered.set(over_eject);
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

/// The rounded highlight a native menu item gets, drawn to the same insets.
fn draw_highlight(bounds: NSRect) {
    NSColor::labelColor().colorWithAlphaComponent(0.12).set();
    NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(
        rect(5.0, 0.0, bounds.size.width - 10.0, bounds.size.height),
        5.0,
        5.0,
    )
    .fill();
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
