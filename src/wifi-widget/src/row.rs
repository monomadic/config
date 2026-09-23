//! Compact saved-network row. Availability stays unclaimed until scans exist.
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{
    AnyThread, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send,
    rc::Retained, sel,
};
use objc2_app_kit::*;
use objc2_foundation::{
    NSMutableAttributedString, NSMutableDictionary, NSObjectProtocol, NSPoint, NSRect, NSSize,
    NSString,
};
use std::cell::{Cell, RefCell};
pub struct RowIvars {
    button: Retained<NSButton>,
    name: RefCell<String>,
    image: Retained<NSImageView>,
    detail: Retained<NSTextField>,
    security: Retained<NSTextField>,
    strength: Cell<Option<i32>>,
}
define_class!(
    #[unsafe(super(NSView))]
    #[thread_kind=MainThreadOnly]
    #[name="WiFiKnownNetworkRow"]
    #[ivars=RowIvars]
    pub struct KnownRow;
    unsafe impl NSObjectProtocol for KnownRow {}
    impl KnownRow {
        #[unsafe(method(drawRect:))]
        fn draw(&self, _: NSRect) {
            if let Some(rssi) = self.ivars().strength.get() {
                let active = if rssi >= -55 {4} else if rssi >= -67 {3} else if rssi >= -75 {2} else {1};
                for i in 0..4 {
                    NSColor::secondaryLabelColor().colorWithAlphaComponent(if i < active {1.0} else {0.22}).set();
                    let h = 3.0 + i as f64 * 2.0;
                    NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(frame(278. + i as f64 * 5., 15., 3., h), 1., 1.).fill();
                }
            }
        }

        #[unsafe(method(openSettings:))]
        fn open_settings(&self,_sender:&NSButton) {
            crate::join::start(self.ivars().name.borrow().clone(), None);
            if let Some(item)=self.enclosingMenuItem() && let Some(menu)=unsafe { item.menu() } { menu.cancelTracking(); }
        }
    }
);
impl KnownRow {
    pub fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let button = NSButton::new(mtm);
        let this = Self::alloc(mtm).set_ivars(RowIvars {
            button,
            name: RefCell::new(String::new()),
            image: NSImageView::new(mtm),
            detail: NSTextField::labelWithString(&NSString::from_str("—"), mtm),
            security: NSTextField::labelWithString(&NSString::from_str(""), mtm),
            strength: Cell::new(None),
        });
        let this: Retained<Self> =
            unsafe { msg_send![super(this),initWithFrame:frame(0.,0.,320.,30.)] };
        let surface = HoverSurface::new(frame(14., 0., 292., 30.), false, mtm);
        this.addSubview(&surface);
        let button = &this.ivars().button;
        button.setFrame(frame(45., 0., 198., 30.));
        button.setBordered(false);
        button.setAlignment(NSTextAlignment::Left);
        button.setFont(Some(&NSFont::systemFontOfSize(12.)));
        unsafe {
            button.setTarget(Some(this.as_ref()));
            button.setAction(Some(sel!(openSettings:)));
        }
        this.addSubview(button);
        let image = &this.ivars().image;
        image.setFrame(frame(21., 6., 18., 18.));
        image.setImageScaling(NSImageScaling::ScaleProportionallyUpOrDown);
        image.setImage(
            NSImage::imageWithSystemSymbolName_accessibilityDescription(
                &NSString::from_str("wifi"),
                None,
            )
            .as_deref(),
        );
        image.setContentTintColor(Some(&NSColor::secondaryLabelColor()));
        image.setAccessibilityElement(false);
        this.addSubview(image);
        let label = &this.ivars().detail;
        label.setFrame(frame(247., 13., 50., 15.));
        label.setFont(Some(&NSFont::systemFontOfSize(9.)));
        label.setAlignment(NSTextAlignment::Center);
        label.setTextColor(Some(&NSColor::secondaryLabelColor()));
        this.addSubview(label);
        let security = &this.ivars().security;
        security.setFrame(frame(260., 1., 52., 12.));
        security.setAlignment(NSTextAlignment::Center);
        security.setFont(Some(&NSFont::systemFontOfSize(8.)));
        security.setTextColor(Some(&NSColor::secondaryLabelColor()));
        this.addSubview(security);
        this
    }
    pub fn set_reading(&self, reading: Option<(wifi_widget::wifi::Band, i32)>) {
        self.ivars().strength.set(reading.map(|r| r.1));
        self.ivars().detail.setStringValue(&NSString::from_str(
            reading.map(|r| r.0.label()).unwrap_or("—"),
        ));
        self.ivars().detail.setToolTip(Some(&NSString::from_str(
            &reading
                .map(|r| {
                    format!(
                        "Strongest scanned access point: {}, {} dBm",
                        r.0.label(),
                        r.1
                    )
                })
                .unwrap_or_else(|| "No recent scan reading available".into()),
        )));
        let pill_width = self.ivars().detail.intrinsicContentSize().width + 4.;
        let name_width = (self.ivars().button.intrinsicContentSize().width + 2.)
            .min(250. - 45. - pill_width - 6.);
        self.ivars()
            .button
            .setFrame(frame(45., 0., name_width, 30.));
        self.ivars()
            .detail
            .setFrame(frame(45. + name_width + 6., 5.5, pill_width, 15.));
        self.ivars().detail.setHidden(reading.is_none());
        self.setNeedsDisplay(true);
    }
    pub fn set_security(&self, security: Option<&str>) {
        let text = security.unwrap_or("Unknown");
        self.ivars()
            .security
            .setStringValue(&NSString::from_str(text));
        self.ivars().security.setToolTip(Some(&NSString::from_str(&format!("Security advertised by the strongest scanned access point: {text}. Ent means enterprise authentication."))));
    }
    pub fn set_cached(&self, cached: bool) {
        self.ivars().button.setToolTip(Some(&NSString::from_str(if cached {
            "macOS cached scan result; confirming availability with a fresh scan. Click to join."
        } else { "Detected by the latest scan. Click to join." })));
    }
    pub fn set_saved(&self, saved: bool) {
        let color = if saved && self.ivars().strength.get().is_some() {
            NSColor::whiteColor()
        } else {
            NSColor::secondaryLabelColor()
        };
        self.ivars().image.setContentTintColor(Some(&color));
        let attrs = NSMutableDictionary::<NSString, AnyObject>::new();
        unsafe {
            attrs.setObject_forKey(
                &color,
                ProtocolObject::from_ref(NSForegroundColorAttributeName),
            );
            attrs.setObject_forKey(
                &NSFont::systemFontOfSize(12.),
                ProtocolObject::from_ref(NSFontAttributeName),
            );
            let title = NSMutableAttributedString::initWithString_attributes(
                NSMutableAttributedString::alloc(),
                &self.ivars().button.title(),
                Some(&attrs),
            );
            self.ivars().button.setAttributedTitle(&title);
        }
        self.ivars()
            .button
            .setAccessibilityLabel(Some(&NSString::from_str(&format!(
                "{}, {} network. Join network",
                self.ivars().name.borrow(),
                if saved { "saved nearby" } else { "nearby" }
            ))));
    }
    pub fn set_name(&self, name: &str) {
        *self.ivars().name.borrow_mut() = name.to_owned();
        let clean: String = name.chars().filter(|c| !c.is_control()).collect();
        if self.ivars().button.title().to_string() == clean {
            return;
        }
        // Saved profiles do not expose device type; this is a presentation hint only.
        let hotspot = clean.to_lowercase().contains("iphone");
        self.ivars().image.setImage(
            NSImage::imageWithSystemSymbolName_accessibilityDescription(
                &NSString::from_str(if hotspot { "personalhotspot" } else { "wifi" }),
                None,
            )
            .as_deref(),
        );
        self.ivars().button.setTitle(&NSString::from_str(
            &name.chars().filter(|c| !c.is_control()).collect::<String>(),
        ));
        self.ivars()
            .button
            .setAccessibilityLabel(Some(&NSString::from_str(&format!(
                "{name}, nearby network. Join network"
            ))));
        self.ivars().button.setToolTip(Some(&NSString::from_str(
            if hotspot { "Likely iPhone hotspot based on its name. Detected by the latest scan. Click to join." } else { "Nearby network. Detected by the latest scan. Click to join." },
        )));
    }
}
fn frame(x: f64, y: f64, width: f64, height: f64) -> NSRect {
    NSRect {
        origin: NSPoint { x, y },
        size: NSSize { width, height },
    }
}

/// The v3 section header keeps Refresh on the same baseline as its label.
pub fn section(
    target: Option<&objc2::runtime::AnyObject>,
    action: Option<objc2::runtime::Sel>,
    mtm: MainThreadMarker,
) -> (
    Retained<NSView>,
    Retained<NSButton>,
    Retained<NSProgressIndicator>,
) {
    let view = NSView::new(mtm);
    view.setFrame(frame(0., 0., 320., 32.));
    let label = NSTextField::labelWithString(&NSString::from_str("Nearby Networks"), mtm);
    label.setFrame(frame(15., 5., 220., 17.));
    label.setFont(Some(&NSFont::systemFontOfSize(11.)));
    label.setTextColor(Some(&NSColor::secondaryLabelColor()));
    view.addSubview(&label);
    let button = unsafe {
        NSButton::buttonWithTitle_target_action(&NSString::from_str(""), target, action, mtm)
    };
    let surface = HoverSurface::new(frame(281., 3., 24., 24.), true, mtm);
    *surface.ivars().button.borrow_mut() = Some(button.clone());
    button.setContentTintColor(Some(&NSColor::secondaryLabelColor()));
    view.addSubview(&surface);
    button.setFrame(frame(286., 8., 14., 14.));
    button.setBordered(false);
    button.setImage(
        NSImage::imageWithSystemSymbolName_accessibilityDescription(
            &NSString::from_str("arrow.clockwise"),
            None,
        )
        .as_deref(),
    );
    button.setImagePosition(NSCellImagePosition::ImageOnly);
    button.setToolTip(Some(&NSString::from_str(
        "Refresh Wi-Fi readings and nearby networks",
    )));
    button.setFont(Some(&NSFont::systemFontOfSize(10.)));
    button.setAccessibilityLabel(Some(&NSString::from_str(
        "Refresh Wi-Fi readings and nearby networks",
    )));
    view.addSubview(&button);
    let spinner = NSProgressIndicator::new(mtm);
    spinner.setFrame(frame(285., 7., 16., 16.));
    spinner.setStyle(NSProgressIndicatorStyle::Spinning);
    spinner.setControlSize(NSControlSize::Small);
    spinner.setIndeterminate(true);
    spinner.setDisplayedWhenStopped(false);
    spinner.setHidden(true);
    spinner.setAccessibilityLabel(Some(&NSString::from_str("Scanning nearby networks")));
    view.addSubview(&spinner);
    (view, button, spinner)
}

/// The Style item: a plain menu row with the current choice dimmed on the
/// right, then a chevron. Attributed titles with a tab stop can't do this:
/// the tab lands in the text column, which then grows past the 320pt views
/// and widens the whole menu. Drawing it is the only way to keep the width.
pub fn style_row(mtm: MainThreadMarker) -> (Retained<NSView>, Retained<NSTextField>) {
    let view = NSView::new(mtm);
    view.setFrame(frame(0., 0., 320., 22.));
    view.addSubview(&HoverSurface::new(frame(14., 0., 292., 22.), false, mtm));
    let title = NSTextField::labelWithString(&NSString::from_str("Style"), mtm);
    title.setFrame(frame(21., 2., 120., 18.));
    title.setFont(Some(&NSFont::menuFontOfSize(0.)));
    title.setTextColor(Some(&NSColor::labelColor()));
    view.addSubview(&title);
    let detail = NSTextField::labelWithString(&NSString::from_str(""), mtm);
    detail.setFrame(frame(150., 2., 134., 18.));
    detail.setAlignment(NSTextAlignment::Right);
    detail.setFont(Some(&NSFont::menuFontOfSize(0.)));
    detail.setTextColor(Some(&NSColor::secondaryLabelColor()));
    view.addSubview(&detail);
    let chevron = NSImageView::new(mtm);
    chevron.setFrame(frame(290., 6., 10., 10.));
    chevron.setImageScaling(NSImageScaling::ScaleProportionallyUpOrDown);
    chevron.setImage(
        NSImage::imageWithSystemSymbolName_accessibilityDescription(
            &NSString::from_str("chevron.right"),
            None,
        )
        .as_deref(),
    );
    chevron.setContentTintColor(Some(&NSColor::labelColor()));
    chevron.setAccessibilityElement(false);
    view.addSubview(&chevron);
    (view, detail)
}

pub struct HoverIvars {
    hovered: Cell<bool>,
    circle: bool,
    button: RefCell<Option<Retained<NSButton>>>,
}
define_class!(
    #[unsafe(super(NSView))]
    #[thread_kind=MainThreadOnly]
    #[name="WiFiHoverSurface"]
    #[ivars=HoverIvars]
    struct HoverSurface;
    unsafe impl NSObjectProtocol for HoverSurface {}
    impl HoverSurface {
        #[unsafe(method(mouseEntered:))]
        fn entered(&self, _: &NSEvent) { self.ivars().hovered.set(true); if let Some(button) = self.ivars().button.borrow().as_ref() { button.setContentTintColor(Some(&NSColor::whiteColor())); } self.setNeedsDisplay(true); }
        #[unsafe(method(mouseExited:))]
        fn exited(&self, _: &NSEvent) { self.ivars().hovered.set(false); if let Some(button) = self.ivars().button.borrow().as_ref() { button.setContentTintColor(Some(&NSColor::secondaryLabelColor())); } self.setNeedsDisplay(true); }
        #[unsafe(method(drawRect:))]
        fn draw(&self, _: NSRect) {
            if self.ivars().button.borrow().as_ref().is_some_and(|button| !button.isEnabled()) { return; }
            let mut bounds = self.bounds();
            bounds.origin.x += 0.5; bounds.origin.y += 0.5;
            bounds.size.width -= 1.; bounds.size.height -= 1.;
            let radius = if self.ivars().circle { bounds.size.width / 2. } else { 5. };
            let path = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(bounds, radius, radius);
            if self.ivars().hovered.get() { NSColor::labelColor().colorWithAlphaComponent(0.09).set(); path.fill();
                if self.ivars().circle { NSColor::whiteColor().colorWithAlphaComponent(0.35).set(); path.setLineWidth(0.5); path.stroke(); }
            }

        }
    }
);
impl HoverSurface {
    fn new(bounds: NSRect, circle: bool, mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(HoverIvars {
            hovered: Cell::new(false),
            circle,
            button: RefCell::new(None),
        });
        let this: Retained<Self> = unsafe { msg_send![super(this), initWithFrame: bounds] };
        let tracking = unsafe {
            NSTrackingArea::initWithRect_options_owner_userInfo(
                objc2::AnyThread::alloc(),
                NSRect::ZERO,
                NSTrackingAreaOptions::MouseEnteredAndExited
                    | NSTrackingAreaOptions::ActiveAlways
                    | NSTrackingAreaOptions::InVisibleRect,
                Some(this.as_ref()),
                None,
            )
        };
        this.addTrackingArea(&tracking);
        this
    }
}
