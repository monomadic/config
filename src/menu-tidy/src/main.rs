//! menu-tidy: a minimal menu bar tidier.
//!
//! macOS has no API to hide another app's status item. What it has is a layout
//! rule: status items are packed right-to-left, and any item pushed left of the
//! frontmost app's menus stops being drawn. That suppression is the hiding
//! mechanism — the icons still exist and still run, they just aren't rendered.
//!
//! So hiding is done by widening an invisible spacer item until its neighbours
//! on the left spill past that boundary. The spacer stops being drawn too, which
//! is exactly why it is a separate item from the marker: the marker keeps its
//! natural width, so it is always drawn, and because a status item's position
//! depends only on the items to its *right*, it never moves when icons on its
//! left appear or disappear.

use std::cell::RefCell;
use std::time::Instant;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy, NSCellImagePosition, NSColor,
    NSControlStateValueOff, NSControlStateValueOn, NSEvent, NSEventMask, NSEventModifierFlags,
    NSEventType, NSFont, NSFontAttributeName, NSForegroundColorAttributeName, NSImage, NSMenu,
    NSMenuDelegate, NSMenuItem, NSScreen, NSStatusBar, NSStatusItem, NSStringDrawing,
    NSVariableStatusItemLength,
};
use objc2_core_foundation::{CFBoolean, CFDictionary, CFString};
use objc2_core_graphics::{
    CGRectMakeWithDictionaryRepresentation, CGWindowListCopyWindowInfo, CGWindowListOption,
    kCGWindowBounds, kCGWindowIsOnscreen,
};
use objc2_foundation::{
    NSMutableDictionary, NSObject, NSObjectProtocol, NSPoint, NSRect, NSSize, NSString, NSTimer,
    NSUserDefaults,
};

/// Seconds the mouse must stay away from the menu bar before auto-collapse.
const HIDE_AFTER_SECONDS: f64 = 5.0;
/// Drives auto-hide, hover detection and animation frames.
const POLL_INTERVAL_SECONDS: f64 = 1.0 / 30.0;

const MARKER_FONT_SIZE: f64 = 12.0;
const MARKER_HEIGHT: f64 = 22.0;
/// Canvas is sized for the largest the glyph ever gets, so the item's width
/// never changes — a width change would re-lay out the whole menu bar 30x a
/// second and drag the neighbouring icons around with it.
const MAX_SCALE: f64 = 1.45;
const MARKER_PADDING: f64 = 4.0;
/// Alpha of the resting marker. Drawn into a template image, so this reads as
/// a dimmed version of whatever tint the menu bar is using.
const RESTING_ALPHA: f64 = 0.4;

/// Initial bounce: big, fast, and quickly spent.
const POP_AMPLITUDE: f64 = 0.3;
const POP_DECAY: f64 = 4.0;
const POP_FREQUENCY: f64 = 2.2;
/// The throb it settles into.
const THROB_AMPLITUDE: f64 = 0.08;
const THROB_FREQUENCY: f64 = 1.1;
/// Redraw only when the scale moves by this much, so a resting marker is idle.
const SCALE_QUANTUM: f64 = 0.01;
/// Height of the "near the menu bar" strip at the top of each screen.
const MENU_BAR_STRIP: f64 = 30.0;

/// Collapsed spacer width, as a multiple of screen width. It only has to be
/// wide enough to push its left-hand neighbours past the boundary; the spacer
/// being suppressed at this width is expected, not a problem.
const COLLAPSED_SCREENS: f64 = 1.0;
const COLLAPSED_FALLBACK: f64 = 1600.0;

const SPACER_AUTOSAVE: &str = "menu-tidy-spacer";
const TOGGLE_AUTOSAVE: &str = "menu-tidy";
const SPACER_POSITION_KEY: &str = "NSStatusItem Preferred Position menu-tidy-spacer";
const TOGGLE_POSITION_KEY: &str = "NSStatusItem Preferred Position menu-tidy";
const STYLE_KEY: &str = "marker-style";

/// Slack when matching a frame against the window list.
const BOUNDS_TOLERANCE: f64 = 2.0;

/// A marker design: what to draw when collapsed (icons hidden) and expanded.
struct Style {
    key: &'static str,
    label: &'static str,
    collapsed: &'static str,
    expanded: &'static str,
}

const STYLES: [Style; 7] = [
    Style { key: "triangle", label: "Triangle", collapsed: "◀", expanded: "▶" },
    Style { key: "chevron", label: "Chevron", collapsed: "‹", expanded: "›" },
    Style { key: "chevron_bold", label: "Chevron (bold)", collapsed: "❮", expanded: "❯" },
    Style { key: "angle", label: "Angle", collapsed: "⟨", expanded: "⟩" },
    Style { key: "arrow", label: "Arrow", collapsed: "←", expanded: "→" },
    Style { key: "dots", label: "Dots", collapsed: "•••", expanded: "•" },
    Style { key: "bars", label: "Bars", collapsed: "❙❙", expanded: "❙" },
];

const DEFAULT_STYLE: usize = 0;

fn load_style() -> usize {
    let defaults = NSUserDefaults::standardUserDefaults();
    let Some(stored) = defaults.stringForKey(&NSString::from_str(STYLE_KEY)) else {
        return DEFAULT_STYLE;
    };
    let stored = stored.to_string();
    STYLES
        .iter()
        .position(|s| s.key == stored)
        .unwrap_or(DEFAULT_STYLE)
}

fn save_style(index: usize) {
    let defaults = NSUserDefaults::standardUserDefaults();
    unsafe {
        defaults.setObject_forKey(
            Some(&NSString::from_str(STYLES[index].key)),
            &NSString::from_str(STYLE_KEY),
        );
    }
}

struct Ui {
    /// Invisible item to the left of the marker. Widening this is what hides
    /// the icons; it is expected to stop being drawn itself, which is fine
    /// because it never draws anything.
    spacer: Retained<NSStatusItem>,
    /// The marker the user clicks. Always natural width, so it is always drawn
    /// and — since only items to its right affect its position — never moves.
    toggle: Retained<NSStatusItem>,
    menu: Retained<NSMenu>,
    style_items: Vec<Retained<NSMenuItem>>,
    style: usize,
    expanded: bool,
    last_near: Instant,
    /// Fixed canvas for the current style, wide enough for the biggest frame.
    canvas: NSSize,
    hovered: bool,
    /// When the current bounce began; `None` while at rest.
    animating_since: Option<Instant>,
    /// Last rendered (scale, alpha), to skip redundant redraws.
    last_render: Option<(f64, f64)>,
}

define_class!(
    // SAFETY: NSObject has no subclassing requirements and Tidy does not
    // implement Drop.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = RefCell<Option<Ui>>]
    struct Tidy;

    unsafe impl NSObjectProtocol for Tidy {}

    impl Tidy {
        #[unsafe(method(toggleAction:))]
        fn toggle_action(&self, _sender: &NSObject) {
            let mtm = MainThreadMarker::new().unwrap();
            let event = NSApplication::sharedApplication(mtm).currentEvent();
            let wants_menu = event
                .map(|e| {
                    e.r#type() == NSEventType::RightMouseUp
                        || e.modifierFlags().contains(NSEventModifierFlags::Control)
                })
                .unwrap_or(false);
            if wants_menu {
                self.show_menu();
            } else {
                let expanded = self
                    .ivars()
                    .borrow()
                    .as_ref()
                    .map(|ui| ui.expanded)
                    .unwrap_or(false);
                self.set_expanded(!expanded);
            }
        }

        #[unsafe(method(styleAction:))]
        fn style_action(&self, sender: &NSMenuItem) {
            let index = sender.tag() as usize;
            if index >= STYLES.len() {
                return;
            }
            if let Some(ui) = self.ivars().borrow_mut().as_mut() {
                ui.style = index;
                ui.canvas = canvas_for(widest_glyph(index));
                ui.last_render = None;
            }
            save_style(index);
            self.refresh_style_checks();
            self.refresh_marker();
        }

        #[unsafe(method(tick:))]
        fn tick(&self, _timer: &NSTimer) {
            let hovered = self.pointer_over_marker();
            let near_bar = mouse_near_menu_bar();

            let collapse_due = {
                let mut ivars = self.ivars().borrow_mut();
                let Some(ui) = ivars.as_mut() else { return };

                if hovered != ui.hovered {
                    ui.hovered = hovered;
                    // Restart the bounce each time the pointer arrives.
                    if hovered && ui.animating_since.is_none() {
                        ui.animating_since = Some(Instant::now());
                    }
                }
                // Animate while the pointer is on it, and keep throbbing for as
                // long as the icons are out; settle back once they hide again.
                let should_animate = ui.hovered || ui.expanded;
                if should_animate && ui.animating_since.is_none() {
                    ui.animating_since = Some(Instant::now());
                } else if !should_animate {
                    ui.animating_since = None;
                }

                if !ui.expanded {
                    false
                } else if near_bar {
                    ui.last_near = Instant::now();
                    false
                } else {
                    ui.last_near.elapsed().as_secs_f64() >= HIDE_AFTER_SECONDS
                }
            };

            if collapse_due {
                self.set_expanded(false);
            } else {
                self.refresh_marker();
            }
        }

        #[unsafe(method(debugTick:))]
        fn debug_tick(&self, _timer: &NSTimer) {
            self.debug_geometry("poll");
        }
    }

    unsafe impl NSMenuDelegate for Tidy {
        #[unsafe(method(menuDidClose:))]
        fn menu_did_close(&self, _menu: &NSMenu) {
            // Detach the menu again so plain left clicks go back to toggling
            // instead of opening the menu.
            let toggle = self.ivars().borrow().as_ref().map(|ui| ui.toggle.clone());
            if let Some(toggle) = toggle {
                toggle.setMenu(None);
            }
        }
    }
);

fn collapsed_length(mtm: MainThreadMarker) -> f64 {
    NSScreen::mainScreen(mtm)
        .map(|s| s.frame().size.width * COLLAPSED_SCREENS)
        .filter(|w| *w > 0.0)
        .unwrap_or(COLLAPSED_FALLBACK)
}

fn mouse_near_menu_bar() -> bool {
    let mtm = MainThreadMarker::new().unwrap();
    let location = NSEvent::mouseLocation();
    NSScreen::screens(mtm).iter().any(|screen| {
        let frame = screen.frame();
        location.x >= frame.origin.x
            && location.x <= frame.origin.x + frame.size.width
            && location.y >= frame.origin.y + frame.size.height - MENU_BAR_STRIP
    })
}

/// The wider of a style's two glyphs, so the canvas fits both states and the
/// marker keeps one width throughout.
fn widest_glyph(index: usize) -> &'static str {
    let style = &STYLES[index];
    let collapsed = glyph_size(style.collapsed, MARKER_FONT_SIZE * MAX_SCALE).width;
    let expanded = glyph_size(style.expanded, MARKER_FONT_SIZE * MAX_SCALE).width;
    if collapsed >= expanded {
        style.collapsed
    } else {
        style.expanded
    }
}

fn glyph_attributes(font_size: f64, alpha: f64) -> Retained<NSMutableDictionary<NSString, AnyObject>> {
    let attrs = NSMutableDictionary::<NSString, AnyObject>::new();
    let font = NSFont::systemFontOfSize(font_size);
    let color = NSColor::blackColor().colorWithAlphaComponent(alpha);
    unsafe {
        attrs.setObject_forKey(&font, ProtocolObject::from_ref(NSFontAttributeName));
        attrs.setObject_forKey(&*color, ProtocolObject::from_ref(NSForegroundColorAttributeName));
    }
    attrs
}

fn glyph_size(glyph: &str, font_size: f64) -> NSSize {
    let attrs = glyph_attributes(font_size, 1.0);
    unsafe { NSString::from_str(glyph).sizeWithAttributes(Some(&attrs)) }
}

/// Canvas big enough for the glyph at full stretch, so the item never resizes.
fn canvas_for(glyph: &str) -> NSSize {
    let largest = glyph_size(glyph, MARKER_FONT_SIZE * MAX_SCALE);
    NSSize {
        width: largest.width + MARKER_PADDING * 2.0,
        height: MARKER_HEIGHT,
    }
}

/// The glyph centred in a fixed canvas, scaled and faded. Template image, so
/// macOS tints it to the menu bar and the alpha reads as a dimmed version.
fn marker_image(glyph: &str, canvas: NSSize, scale: f64, alpha: f64) -> Retained<NSImage> {
    let glyph = glyph.to_string();
    let handler = block2::RcBlock::new(move |_bounds: NSRect| -> objc2::runtime::Bool {
        let font_size = MARKER_FONT_SIZE * scale;
        let attrs = glyph_attributes(font_size, alpha);
        let text = NSString::from_str(&glyph);
        let size = unsafe { text.sizeWithAttributes(Some(&attrs)) };
        let origin = NSPoint {
            x: (canvas.width - size.width) / 2.0,
            y: (canvas.height - size.height) / 2.0,
        };
        unsafe { text.drawAtPoint_withAttributes(origin, Some(&attrs)) };
        objc2::runtime::Bool::YES
    });
    let image = NSImage::imageWithSize_flipped_drawingHandler(canvas, false, &handler);
    image.setTemplate(true);
    image
}

/// Bounce out hard, then settle into a slow throb.
fn animation_scale(elapsed: f64) -> f64 {
    let tau = std::f64::consts::TAU;
    let pop = POP_AMPLITUDE * (-elapsed * POP_DECAY).exp() * (tau * POP_FREQUENCY * elapsed).sin();
    let throb = THROB_AMPLITUDE * (tau * THROB_FREQUENCY * elapsed).sin();
    1.0 + pop + throb
}

fn key_ptr(key: &CFString) -> *const core::ffi::c_void {
    key as *const CFString as *const core::ffi::c_void
}

impl Tidy {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = Self::alloc(mtm).set_ivars(RefCell::new(None));
        unsafe { msg_send![super(this), init] }
    }

    fn build_ui(&self, mtm: MainThreadMarker) {
        let status_bar = NSStatusBar::systemStatusBar();

        // Seed the spacer just left of the marker on first run. Preferred
        // position counts up leftwards, so +1 puts it on the marker's left.
        let defaults = NSUserDefaults::standardUserDefaults();
        if defaults
            .objectForKey(&NSString::from_str(SPACER_POSITION_KEY))
            .is_none()
        {
            let toggle_pos = defaults.doubleForKey(&NSString::from_str(TOGGLE_POSITION_KEY));
            defaults.setDouble_forKey(toggle_pos + 1.0, &NSString::from_str(SPACER_POSITION_KEY));
        }

        let spacer = status_bar.statusItemWithLength(0.0);
        spacer.setAutosaveName(Some(&NSString::from_str(SPACER_AUTOSAVE)));
        if let Some(button) = spacer.button(mtm) {
            button.setTitle(&NSString::from_str(""));
            unsafe {
                button.setTarget(Some(self.as_ref()));
                button.setAction(Some(sel!(toggleAction:)));
            }
            let _ = button.sendActionOn(NSEventMask::LeftMouseUp | NSEventMask::RightMouseUp);
        }

        let toggle = status_bar.statusItemWithLength(NSVariableStatusItemLength);
        toggle.setAutosaveName(Some(&NSString::from_str(TOGGLE_AUTOSAVE)));
        if let Some(button) = toggle.button(mtm) {
            unsafe {
                button.setTarget(Some(self.as_ref()));
                button.setAction(Some(sel!(toggleAction:)));
            }
            let _ = button.sendActionOn(NSEventMask::LeftMouseUp | NSEventMask::RightMouseUp);
        }

        let style = load_style();

        let menu = NSMenu::new(mtm);
        menu.setAutoenablesItems(false);
        let delegate: &ProtocolObject<dyn NSMenuDelegate> = ProtocolObject::from_ref(self);
        menu.setDelegate(Some(delegate));

        let hint = NSMenuItem::new(mtm);
        hint.setTitle(&NSString::from_str(
            "⌘-drag icons to the left of the marker to tidy them",
        ));
        hint.setEnabled(false);
        menu.addItem(&hint);

        menu.addItem(&NSMenuItem::separatorItem(mtm));

        let style_menu = NSMenu::new(mtm);
        style_menu.setAutoenablesItems(false);
        let mut style_items = Vec::new();
        for (index, entry) in STYLES.iter().enumerate() {
            let item = NSMenuItem::new(mtm);
            item.setTitle(&NSString::from_str(&format!(
                "{}   {}  {}",
                entry.label, entry.collapsed, entry.expanded
            )));
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
        style_root.setTitle(&NSString::from_str("Marker"));
        style_root.setEnabled(true);
        style_root.setSubmenu(Some(&style_menu));
        menu.addItem(&style_root);

        menu.addItem(&NSMenuItem::separatorItem(mtm));

        let quit_item = NSMenuItem::new(mtm);
        quit_item.setTitle(&NSString::from_str("Quit menu-tidy"));
        quit_item.setEnabled(true);
        unsafe { quit_item.setAction(Some(sel!(terminate:))) };
        menu.addItem(&quit_item);

        *self.ivars().borrow_mut() = Some(Ui {
            spacer,
            toggle,
            menu,
            style_items,
            style,
            expanded: false,
            last_near: Instant::now(),
            canvas: canvas_for(widest_glyph(style)),
            hovered: false,
            animating_since: None,
            last_render: None,
        });

        self.refresh_style_checks();
        self.debug_screens();

        // Start expanded: on a fresh install nothing is arranged yet, and on
        // login it gives a glimpse of what is tucked away before the timer
        // collapses it.
        self.set_expanded(true);
    }

    fn set_expanded(&self, expanded: bool) {
        let spacer = {
            let mut ivars = self.ivars().borrow_mut();
            let Some(ui) = ivars.as_mut() else { return };
            ui.expanded = expanded;
            ui.last_near = Instant::now();
            ui.last_render = None;
            if !expanded && !ui.hovered {
                ui.animating_since = None;
            }
            ui.spacer.clone()
        };

        let mtm = MainThreadMarker::new().unwrap();
        // Zero length rather than setVisible(false): hiding the item outright
        // makes macOS re-place it on the way back, which strands icons between
        // the spacer and the marker. A zero-length item still reserves ~16pt,
        // which sits between the revealed icons and the marker and reads as a
        // separator.
        spacer.setLength(if expanded { 0.0 } else { collapsed_length(mtm) });
        self.refresh_marker();
        self.debug_geometry("set");
    }

    fn refresh_marker(&self) {
        let mtm = MainThreadMarker::new().unwrap();
        let (toggle, glyph, canvas, scale, alpha) = {
            let mut ivars = self.ivars().borrow_mut();
            let Some(ui) = ivars.as_mut() else { return };
            let style = &STYLES[ui.style];
            let glyph = if ui.expanded {
                style.expanded
            } else {
                style.collapsed
            };

            let scale = match ui.animating_since {
                Some(start) => animation_scale(start.elapsed().as_secs_f64()),
                None => 1.0,
            };
            // Full strength while it is being used; dimmed once the icons are
            // tucked away and the pointer has left.
            let alpha = if ui.hovered || ui.expanded {
                1.0
            } else {
                RESTING_ALPHA
            };

            let quantised = (scale / SCALE_QUANTUM).round() * SCALE_QUANTUM;
            if ui.last_render == Some((quantised, alpha)) {
                return;
            }
            ui.last_render = Some((quantised, alpha));
            (ui.toggle.clone(), glyph, ui.canvas, quantised, alpha)
        };

        if let Some(button) = toggle.button(mtm) {
            button.setImage(Some(&marker_image(glyph, canvas, scale, alpha)));
            button.setImagePosition(NSCellImagePosition::ImageOnly);
        }
    }

    /// Whether the pointer is over the marker itself.
    fn pointer_over_marker(&self) -> bool {
        let mtm = MainThreadMarker::new().unwrap();
        let ivars = self.ivars().borrow();
        let Some(ui) = ivars.as_ref() else {
            return false;
        };
        let Some(frame) = ui.toggle.button(mtm).and_then(|b| b.window()).map(|w| w.frame()) else {
            return false;
        };
        let p = NSEvent::mouseLocation();
        p.x >= frame.origin.x
            && p.x <= frame.origin.x + frame.size.width
            && p.y >= frame.origin.y
            && p.y <= frame.origin.y + frame.size.height
    }

    fn refresh_style_checks(&self) {
        let ivars = self.ivars().borrow();
        let Some(ui) = ivars.as_ref() else { return };
        for (index, item) in ui.style_items.iter().enumerate() {
            item.setState(if index == ui.style {
                NSControlStateValueOn
            } else {
                NSControlStateValueOff
            });
        }
    }

    fn show_menu(&self) {
        // Attach the menu just for this click; the next click on the button
        // then opens it. menuDidClose detaches it again.
        let parts = {
            let ivars = self.ivars().borrow();
            ivars
                .as_ref()
                .map(|ui| (ui.toggle.clone(), ui.menu.clone()))
        };
        let Some((toggle, menu)) = parts else { return };
        let mtm = MainThreadMarker::new().unwrap();
        toggle.setMenu(Some(&menu));
        if let Some(button) = toggle.button(mtm) {
            unsafe { button.performClick(None) };
        }
    }

    /// Whether the window server is actually drawing a given item.
    ///
    /// macOS silently stops rendering a status item pushed left of the
    /// frontmost app's menus, and neither the item's own frame nor
    /// `occlusionState` (8194 either way) reflects it. The lookup goes by
    /// bounds, not window id: a status item's drawn window belongs to Control
    /// Center, so our own window number finds nothing in the window list.
    fn item_is_drawn(&self, item: &NSStatusItem) -> Option<bool> {
        let mtm = MainThreadMarker::new().unwrap();
        let frame = item.button(mtm)?.window()?.frame();

        let info = CGWindowListCopyWindowInfo(CGWindowListOption::OptionAll, 0)?;
        for i in 0..info.count() {
            let dict: &CFDictionary = unsafe { &*(info.value_at_index(i) as *const CFDictionary) };
            let bounds = unsafe { dict.value(key_ptr(kCGWindowBounds)) };
            if bounds.is_null() {
                continue;
            }
            let mut rect = NSRect::default();
            let ok = unsafe {
                CGRectMakeWithDictionaryRepresentation(
                    Some(&*(bounds as *const CFDictionary)),
                    &mut rect,
                )
            };
            if !ok
                || (rect.origin.x - frame.origin.x).abs() > BOUNDS_TOLERANCE
                || (rect.size.width - frame.size.width).abs() > BOUNDS_TOLERANCE
            {
                continue;
            }
            let onscreen = unsafe { dict.value(key_ptr(kCGWindowIsOnscreen)) };
            if onscreen.is_null() {
                return Some(false);
            }
            let flag: &CFBoolean = unsafe { &*(onscreen as *const CFBoolean) };
            return Some(flag.value());
        }
        None
    }

    fn debug_screens(&self) {
        if std::env::var_os("MENU_TIDY_DEBUG").is_none() {
            return;
        }
        let mtm = MainThreadMarker::new().unwrap();
        for (i, screen) in NSScreen::screens(mtm).iter().enumerate() {
            let f = screen.frame();
            eprintln!(
                "[screens] #{i} frame=({:.0},{:.0} {:.0}x{:.0})",
                f.origin.x, f.origin.y, f.size.width, f.size.height
            );
        }
    }

    fn debug_geometry(&self, tag: &str) {
        if std::env::var_os("MENU_TIDY_DEBUG").is_none() {
            return;
        }
        let mtm = MainThreadMarker::new().unwrap();
        let (spacer, toggle, expanded, style, hovered, anim) = {
            let ivars = self.ivars().borrow();
            let Some(ui) = ivars.as_ref() else { return };
            (
                ui.spacer.clone(),
                ui.toggle.clone(),
                ui.expanded,
                STYLES[ui.style].key,
                ui.hovered,
                ui.last_render,
            )
        };
        let describe = |item: &NSStatusItem| {
            item.button(mtm)
                .and_then(|b| b.window())
                .map(|w| {
                    let f = w.frame();
                    format!("{:.0}..{:.0}", f.origin.x, f.origin.x + f.size.width)
                })
                .unwrap_or_else(|| "?".into())
        };
        eprintln!(
            "[{tag}] style={style} expanded={expanded} hover={hovered} render={anim:?} spacer={} drawn={:?} | toggle={} drawn={:?}",
            describe(&spacer),
            self.item_is_drawn(&spacer),
            describe(&toggle),
            self.item_is_drawn(&toggle),
        );
    }
}

fn main() {
    let mtm = MainThreadMarker::new().expect("must run on the main thread");
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let tidy = Tidy::new(mtm);
    tidy.build_ui(mtm);

    unsafe {
        NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
            POLL_INTERVAL_SECONDS,
            &tidy,
            sel!(tick:),
            None,
            true,
        );
        if std::env::var_os("MENU_TIDY_DEBUG").is_some() {
            NSTimer::scheduledTimerWithTimeInterval_target_selector_userInfo_repeats(
                2.0,
                &tidy,
                sel!(debugTick:),
                None,
                true,
            );
        }
    }

    app.run();
}
