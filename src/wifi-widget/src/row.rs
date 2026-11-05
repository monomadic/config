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
    /// Everything on the network line; lifted when the status line shows.
    content: Retained<NSView>,
    spinner: Retained<NSProgressIndicator>,
    status: Retained<NSTextField>,
    symbol: Cell<&'static str>,
    saved: Cell<bool>,
    progress: RefCell<Option<(crate::join::Phase, String)>>,
}
/// Row height without and with the join status line beneath it.
const LINE: f64 = 30.;
const STATUS: f64 = 14.;
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
                    let lift = self.ivars().content.frame().origin.y;
                    NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(frame(278. + i as f64 * 5., 15. + lift, 3., h), 1., 1.).fill();
                }
            }
        }

        #[unsafe(method(openSettings:))]
        fn open_settings(&self,_sender:&NSButton) {
            // The menu stays open: the row itself shows the join's progress.
            crate::join::start(self.ivars().name.borrow().clone(), None);
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
            content: NSView::initWithFrame(NSView::alloc(mtm), frame(0., 0., 320., LINE)),
            spinner: NSProgressIndicator::new(mtm),
            status: NSTextField::labelWithString(&NSString::from_str(""), mtm),
            symbol: Cell::new("wifi"),
            saved: Cell::new(false),
            progress: RefCell::new(None),
        });
        let this: Retained<Self> =
            unsafe { msg_send![super(this),initWithFrame:frame(0.,0.,320.,LINE)] };
        let content = this.ivars().content.clone();
        this.addSubview(&content);
        let surface = HoverSurface::new(frame(14., 0., 292., 30.), false, mtm);
        content.addSubview(&surface);
        let button = &this.ivars().button;
        button.setFrame(frame(45., 0., 198., 30.));
        button.setBordered(false);
        button.setAlignment(NSTextAlignment::Left);
        button.setFont(Some(&NSFont::systemFontOfSize(12.)));
        unsafe {
            button.setTarget(Some(this.as_ref()));
            button.setAction(Some(sel!(openSettings:)));
        }
        content.addSubview(button);
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
        content.addSubview(image);
        let label = &this.ivars().detail;
        label.setFrame(frame(247., 13., 50., 15.));
        label.setFont(Some(&NSFont::systemFontOfSize(9.)));
        label.setAlignment(NSTextAlignment::Center);
        label.setTextColor(Some(&NSColor::secondaryLabelColor()));
        content.addSubview(label);
        let security = &this.ivars().security;
        security.setFrame(frame(260., 1., 52., 12.));
        security.setAlignment(NSTextAlignment::Center);
        security.setFont(Some(&NSFont::systemFontOfSize(8.)));
        security.setTextColor(Some(&NSColor::secondaryLabelColor()));
        content.addSubview(security);
        let spinner = &this.ivars().spinner;
        spinner.setFrame(frame(22., 7., 16., 16.));
        spinner.setStyle(NSProgressIndicatorStyle::Spinning);
        spinner.setControlSize(NSControlSize::Small);
        spinner.setIndeterminate(true);
        spinner.setDisplayedWhenStopped(false);
        spinner.setHidden(true);
        content.addSubview(spinner);
        let status = &this.ivars().status;
        status.setFrame(frame(45., 1., 260., 13.));
        status.setFont(Some(&NSFont::systemFontOfSize(9.)));
        status.setTextColor(Some(&NSColor::secondaryLabelColor()));
        status.setLineBreakMode(NSLineBreakMode::ByTruncatingTail);
        status.setHidden(true);
        this.addSubview(status);
        this
    }
    pub fn set_reading(&self, reading: Option<(wifi_widget::wifi::Band, i32)>) {
        self.ivars().strength.set(reading.map(|r| r.1));
        self.ivars().detail.setStringValue(&NSString::from_str(
            reading.map(|r| r.0.label()).unwrap_or("—"),
        ));
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
    }
    pub fn set_saved(&self, saved: bool) {
        self.ivars().saved.set(saved);
        let color = if saved && self.ivars().strength.get().is_some() {
            NSColor::whiteColor()
        } else {
            NSColor::secondaryLabelColor()
        };
        if !self.ivars().progress.borrow().as_ref().is_some_and(|p| p.0 != crate::join::Phase::Working) {
            self.ivars().image.setContentTintColor(Some(&color));
        }
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
        self.ivars().symbol.set(if hotspot { "personalhotspot" } else { "wifi" });
        self.show_symbol();
        self.ivars().button.setTitle(&NSString::from_str(
            &name.chars().filter(|c| !c.is_control()).collect::<String>(),
        ));
        self.ivars()
            .button
            .setAccessibilityLabel(Some(&NSString::from_str(&format!(
                "{name}, nearby network. Join network"
            ))));
    }
}
impl KnownRow {
    /// Shows a join on this row: a spinner in place of the icon while it
    /// runs, a check or warning when it ends, and the system's last word on a
    /// line beneath. Returns true when the row changed height, so the caller
    /// can tell the menu to re-lay it out.
    pub fn set_progress(&self, progress: Option<(crate::join::Phase, String)>) -> bool {
        use crate::join::Phase;
        if *self.ivars().progress.borrow() == progress {
            return false;
        }
        let was_tall = self.ivars().progress.borrow().is_some();
        let tall = progress.is_some();
        let ivars = self.ivars();
        let working = matches!(progress, Some((Phase::Working, _)));
        ivars.spinner.setHidden(!working);
        if working { unsafe { ivars.spinner.startAnimation(None) }; } else { unsafe { ivars.spinner.stopAnimation(None) }; }
        ivars.image.setHidden(working);
        match &progress {
            Some((phase, detail)) => {
                set_label(&ivars.status, detail);
                ivars.status.setTextColor(Some(&*if *phase == Phase::Failed { NSColor::systemOrangeColor() } else { NSColor::secondaryLabelColor() }));
            }
            None => set_label(&ivars.status, ""),
        }
        ivars.status.setHidden(!tall);
        *ivars.progress.borrow_mut() = progress;
        self.show_symbol();
        if was_tall != tall {
            let height = if tall { LINE + STATUS } else { LINE };
            let mut own = self.frame();
            own.size.height = height;
            self.setFrame(own);
            ivars.content.setFrameOrigin(NSPoint { x: 0., y: if tall { STATUS } else { 0. } });
        }
        self.setNeedsDisplay(true);
        was_tall != tall
    }
    fn show_symbol(&self) {
        use crate::join::Phase;
        let (name, tint) = match self.ivars().progress.borrow().as_ref().map(|p| p.0) {
            Some(Phase::Connected) => ("checkmark.circle.fill", Some(NSColor::systemGreenColor())),
            Some(Phase::Failed) => ("exclamationmark.triangle.fill", Some(NSColor::systemOrangeColor())),
            _ => (self.ivars().symbol.get(), None),
        };
        let image = &self.ivars().image;
        image.setImage(NSImage::imageWithSystemSymbolName_accessibilityDescription(&NSString::from_str(name), None).as_deref());
        if let Some(tint) = tint {
            image.setContentTintColor(Some(&tint));
        } else {
            // set_saved owns the resting tint; reapply it from the title colour.
            self.set_saved(self.ivars().saved.get());
        }
    }
}
fn set_label(field: &NSTextField, text: &str) {
    if field.stringValue().to_string() != text {
        field.setStringValue(&NSString::from_str(text));
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

/// Quiet placeholder under a section header, sized so it never outweighs the
/// 11pt heading above it the way a plain 13pt menu item does.
pub fn placeholder_row(text: &str, mtm: MainThreadMarker) -> Retained<NSView> {
    let view = NSView::new(mtm);
    view.setFrame(frame(0., 0., 320., 20.));
    let label = NSTextField::labelWithString(&NSString::from_str(text), mtm);
    label.setFrame(frame(21., 2., 284., 15.));
    label.setFont(Some(&NSFont::systemFontOfSize(11.)));
    label.setTextColor(Some(&NSColor::tertiaryLabelColor()));
    view.addSubview(&label);
    view
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
    chevron.setContentTintColor(Some(&NSColor::secondaryLabelColor()));
    chevron.setAccessibilityElement(false);
    view.addSubview(&chevron);
    (view, detail)
}

/// Hover is read from the pointer at draw time, never latched. Menu items hide
/// and reshuffle while the menu is open, and a view hidden (or a menu closed)
/// under the pointer gets no `mouseExited:` — a latched flag then stayed on.
pub struct HoverIvars {
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
        fn entered(&self, _: &NSEvent) { self.refresh(); }
        #[unsafe(method(mouseExited:))]
        fn exited(&self, _: &NSEvent) { self.refresh(); }
        #[unsafe(method(viewDidMoveToWindow))]
        fn moved_to_window(&self) { self.refresh(); }
        #[unsafe(method(viewDidUnhide))]
        fn unhidden(&self) { self.refresh(); }
        #[unsafe(method(drawRect:))]
        fn draw(&self, _: NSRect) {
            if self.ivars().button.borrow().as_ref().is_some_and(|button| !button.isEnabled()) { return; }
            if !self.pointer_inside() { return; }
            let mut bounds = self.bounds();
            bounds.origin.x += 0.5; bounds.origin.y += 0.5;
            bounds.size.width -= 1.; bounds.size.height -= 1.;
            let radius = if self.ivars().circle { bounds.size.width / 2. } else { 5. };
            let path = NSBezierPath::bezierPathWithRoundedRect_xRadius_yRadius(bounds, radius, radius);
            NSColor::labelColor().colorWithAlphaComponent(0.09).set(); path.fill();
            if self.ivars().circle { NSColor::whiteColor().colorWithAlphaComponent(0.35).set(); path.setLineWidth(0.5); path.stroke(); }
        }
    }
);
impl HoverSurface {
    fn new(bounds: NSRect, circle: bool, mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(HoverIvars {
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

    fn pointer_inside(&self) -> bool {
        let Some(window) = self.window() else { return false };
        if self.isHiddenOrHasHiddenAncestor() { return false; }
        let local = self.convertPoint_fromView(window.convertPointFromScreen(NSEvent::mouseLocation()), None);
        self.mouse_inRect(local, self.bounds())
    }

    fn refresh(&self) {
        let inside = self.pointer_inside();
        if let Some(button) = self.ivars().button.borrow().as_ref() {
            let tint = if inside { NSColor::whiteColor() } else { NSColor::secondaryLabelColor() };
            button.setContentTintColor(Some(&tint));
        }
        self.setNeedsDisplay(true);
    }
}
