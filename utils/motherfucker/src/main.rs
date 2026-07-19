//! motherfucker — cache-free minimalist Spotlight replacement.
//!
//! One resident process. A Carbon global hotkey summons a non-activating
//! NSPanel with the system's dark vibrancy material. App discovery is a
//! readdir on every summon; nothing is cached, nothing is drawn by us but
//! text. See DESIGN.md for the visual spec ("black glass, tint selection").

#![allow(non_snake_case)]
#![allow(deprecated)] // NSApplicationActivateIgnoringOtherApps: no-op on 14+, harmless
#![allow(unused_unsafe)] // AppKit bindings are unsafe-heavy; whole bodies are wrapped

mod apps;
mod config;
mod hotkey;
mod stats;

use config::{Action, Config, Mode};

use std::cell::{Cell, OnceCell, RefCell};
use std::ffi::c_void;
use std::path::PathBuf;

use objc2::rc::Retained;
use objc2::runtime::{ProtocolObject, Sel};
use objc2::{
    declare_class, msg_send, msg_send_id, mutability::MainThreadOnly, sel, ClassType,
    DeclaredClass,
};
use objc2_app_kit::{
    NSAppearance, NSAppearanceCustomization, NSAppearanceNameVibrantDark, NSApplication,
    NSApplicationActivationOptions,
    NSApplicationActivationPolicy, NSApplicationDelegate, NSAutoresizingMaskOptions,
    NSBackingStoreType, NSColor, NSControl, NSControlTextEditingDelegate, NSEvent, NSEventMask,
    NSEventModifierFlags, NSFocusRingType, NSFont, NSFontAttributeName,
    NSForegroundColorAttributeName, NSPanel, NSRunningApplication, NSScreen, NSTextField,
    NSTextFieldDelegate, NSTextView, NSVisualEffectBlendingMode, NSVisualEffectMaterial,
    NSVisualEffectState, NSVisualEffectView, NSView, NSWindowCollectionBehavior,
    NSWindowDelegate, NSWindowStyleMask, NSWorkspace, NSWorkspaceOpenConfiguration,
};
use objc2_foundation::{
    MainThreadMarker, NSData, NSMutableAttributedString, NSNotification, NSObject,
    NSObjectProtocol, NSPoint, NSRange, NSRect, NSSize, NSString, NSURL,
};
use objc2_quartz_core::CALayer;

// ---- visual spec (black glass) ----
// Colors, panel width, padding, corner radii, and font sizes moved to
// config::Style (same defaults); the structural metrics below stay compiled in.
const INPUT_H: f64 = 58.0;
const ROW_H: f64 = 40.0;
const ROWS_PAD: f64 = 12.0;
const MAX_ROWS: usize = 6;
// Soft ranking bonus for already-running apps. Worth just over one column of
// first-hit position (8 per column in apps::match_positions), so a running app
// wins near-ties but a clearly earlier match on a cold app still outranks it.
const RUNNING_BONUS: i32 = 12;
// CPU sampling: minimum interval for a trustworthy percentage.
const CPU_MIN_INTERVAL: f64 = 0.25;
// Whole-tree CPU (Activity Monitor scale: 100 = one full core) at which a
// row's gauge turns red.
const CPU_ALERT_PCT: f64 = 70.0;

// State glyph column metrics (glyph strings themselves live in
// config::Icons — SF Symbols as text, zero I/O).
const GLYPH_COL_W: f64 = 24.0;
const GLYPH_PT: f64 = 16.0;

struct Entry {
    name: String,
    path: Option<PathBuf>,
    running: Option<Retained<NSRunningApplication>>,
    /// Char indices in `name` matched by the query (for highlighting).
    matched: Vec<usize>,
    /// On-screen window count (running apps; picks the state glyph).
    windows: u32,
    /// Slim CPU gauge, running apps only.
    stats: Option<RowStats>,
    /// `[shortcuts]` entry: shell command run via `sh -c` on activation.
    command: Option<String>,
}

struct RowStats {
    /// Whole-tree CPU on the Activity Monitor scale (100 = one full core).
    cpu_pct: f64,
}

fn state_glyph<'a>(entry: &Entry, icons: &'a config::Icons) -> &'a str {
    if entry.running.is_some() {
        match entry.windows {
            0 => &icons.running_none,
            1 => &icons.running_one,
            _ => &icons.running_many,
        }
    } else {
        &icons.installed
    }
}

/// Location tag for installed rows: label + SF Symbol name.
fn location_for<'a>(
    path: &std::path::Path,
    icons: &'a config::Icons,
) -> (&'static str, &'a str) {
    let s = path.to_string_lossy();
    if s.contains("/Utilities/") {
        ("Utilities", &icons.utilities)
    } else if s.starts_with("/System/") {
        ("System", &icons.system)
    } else {
        ("Applications", &icons.applications)
    }
}

/// Last CPU-time sample per pid, for computing a percentage between looks.
struct CpuSample {
    cpu_secs: f64,
    at: std::time::Instant,
    pct: Option<f64>,
}

#[derive(Default)]
struct State {
    /// Swappable at runtime by the refresh-config action (global hotkeys
    /// aside — those are registered once at launch).
    config: RefCell<Config>,
    panel: OnceCell<Retained<Panel>>,
    field: OnceCell<Retained<NSTextField>>,
    glyph: OnceCell<Retained<NSTextField>>,
    rows_area: OnceCell<Retained<NSView>>,
    entries: RefCell<Vec<Entry>>,
    selected: Cell<usize>,
    top_y: Cell<f64>,
    hiding: Cell<bool>,
    cpu_samples: RefCell<std::collections::HashMap<i32, CpuSample>>,
    /// Repeating stats-refresh timer, alive only while the panel is visible.
    stats_timer: RefCell<Option<Retained<objc2::runtime::AnyObject>>>,
    /// App-directory scan, cached for the lifetime of one open panel. Filled on
    /// the first keystroke after a summon and cleared on hide, so a freshly
    /// installed app still appears next summon without re-scanning per keystroke.
    installed_cache: RefCell<Option<Vec<apps::InstalledApp>>>,
}

declare_class!(
    struct Panel;

    unsafe impl ClassType for Panel {
        type Super = NSPanel;
        type Mutability = MainThreadOnly;
        const NAME: &'static str = "MFPanel";
    }

    impl DeclaredClass for Panel {}

    unsafe impl Panel {
        #[method(canBecomeKeyWindow)]
        fn can_become_key_window(&self) -> bool {
            true
        }
    }
);

#[derive(Default)]
struct RowIvars {
    index: Cell<usize>,
    /// Raw pointer to the Delegate (alive for the process lifetime).
    delegate: Cell<usize>,
}

declare_class!(
    struct RowView;

    unsafe impl ClassType for RowView {
        type Super = NSView;
        type Mutability = MainThreadOnly;
        const NAME: &'static str = "MFRowView";
    }

    impl DeclaredClass for RowView {
        type Ivars = RowIvars;
    }

    unsafe impl RowView {
        #[method(mouseDown:)]
        fn mouse_down(&self, _event: &NSEvent) {
            let ptr = self.ivars().delegate.get();
            if ptr != 0 {
                let delegate = unsafe { &*(ptr as *const Delegate) };
                delegate.select_row(self.ivars().index.get());
            }
        }
    }
);

declare_class!(
    struct Delegate;

    unsafe impl ClassType for Delegate {
        type Super = NSObject;
        type Mutability = MainThreadOnly;
        const NAME: &'static str = "MFDelegate";
    }

    impl DeclaredClass for Delegate {
        type Ivars = State;
    }

    unsafe impl NSObjectProtocol for Delegate {}

    unsafe impl NSApplicationDelegate for Delegate {
        #[method(applicationDidFinishLaunching:)]
        fn app_did_finish_launching(&self, _notification: &NSNotification) {
            self.setup();
        }
    }

    unsafe impl NSWindowDelegate for Delegate {
        #[method(windowDidResignKey:)]
        fn window_did_resign_key(&self, _notification: &NSNotification) {
            self.hide();
        }
    }

    unsafe impl NSControlTextEditingDelegate for Delegate {
        #[method(controlTextDidChange:)]
        fn control_text_did_change(&self, _notification: &NSNotification) {
            self.ivars().selected.set(0);
            self.refresh();
        }

        #[method(control:textView:doCommandBySelector:)]
        fn do_command(&self, _control: &NSControl, _text_view: &NSTextView, command: Sel) -> bool {
            if command == sel!(moveUp:) {
                self.move_selection(-1);
                true
            } else if command == sel!(moveDown:) {
                self.move_selection(1);
                true
            } else if command == sel!(insertNewline:) {
                self.execute(false);
                true
            } else if command == sel!(cancelOperation:) {
                self.hide();
                true
            } else {
                false
            }
        }
    }

    unsafe impl NSTextFieldDelegate for Delegate {}

    unsafe impl Delegate {
        /// Re-render while visible: fired one-shot when CPU needs a second
        /// sample, and by the repeating stats timer (~1s) so gauges stay live.
        #[method(refreshTick)]
        fn refresh_tick(&self) {
            if self.ivars().panel.get().is_some_and(|p| p.isVisible()) {
                self.refresh();
            }
        }
    }
);

/// Configured `(r, g, b)` at the given alpha.
fn rgba(c: (f64, f64, f64), alpha: f64) -> Retained<NSColor> {
    unsafe { NSColor::colorWithSRGBRed_green_blue_alpha(c.0, c.1, c.2, alpha) }
}

fn set_layer_bg(layer: &CALayer, color: &NSColor) {
    unsafe {
        let cg: *mut c_void = msg_send![color, CGColor];
        let _: () = msg_send![layer, setBackgroundColor: cg];
    }
}

/// App titles display with a capitalized first letter ("kitty" → "Kitty").
/// Char count is unchanged, so match indices stay valid.
fn display_name(raw: &str) -> String {
    let mut chars = raw.chars();
    match chars.next() {
        Some(c) if c.is_lowercase() => {
            let mut s: String = c.to_uppercase().collect();
            s.push_str(chars.as_str());
            s
        }
        _ => raw.to_string(),
    }
}

/// Name text with matched characters in a brighter color.
fn attributed_name(
    text: &str,
    matched: &[usize],
    font: &NSFont,
    base: &NSColor,
    hi: &NSColor,
) -> Retained<NSMutableAttributedString> {
    unsafe {
        let ns = NSString::from_str(text);
        let attr = NSMutableAttributedString::initWithString(
            NSMutableAttributedString::alloc(),
            &ns,
        );
        let full = NSRange::new(0, ns.length());
        let _: () = msg_send![&*attr, addAttribute: NSFontAttributeName, value: font, range: full];
        let _: () =
            msg_send![&*attr, addAttribute: NSForegroundColorAttributeName, value: base, range: full];
        let mut utf16_pos = 0usize;
        for (char_idx, ch) in text.chars().enumerate() {
            let len = ch.len_utf16();
            if matched.contains(&char_idx) {
                let range = NSRange::new(utf16_pos, len);
                let _: () = msg_send![&*attr, addAttribute: NSForegroundColorAttributeName, value: hi, range: range];
            }
            utf16_pos += len;
        }
        attr
    }
}

/// Resolve a UI font: the configured `family` at `size` when set (falling
/// back to the system font if the name doesn't resolve), otherwise the
/// system font at `weight`. A named family carries its own weight, so
/// `weight` only applies to the system-font path.
fn resolve_font(family: &str, weight: f64, size: f64) -> Retained<NSFont> {
    if !family.is_empty() {
        if let Some(f) =
            unsafe { NSFont::fontWithName_size(&NSString::from_str(family), size) }
        {
            return f;
        }
    }
    unsafe { msg_send_id![NSFont::class(), systemFontOfSize: size, weight: weight] }
}

fn make_label(
    mtm: MainThreadMarker,
    text: &str,
    font: &NSFont,
    color: &NSColor,
) -> Retained<NSTextField> {
    let label = unsafe { NSTextField::labelWithString(&NSString::from_str(text), mtm) };
    unsafe { label.setFont(Some(font)) };
    unsafe { label.setTextColor(Some(color)) };
    unsafe { label.sizeToFit() };
    label
}

impl Delegate {
    fn new(mtm: MainThreadMarker, config: Config) -> Retained<Self> {
        let this = mtm.alloc::<Self>().set_ivars(State {
            config: RefCell::new(config),
            ..State::default()
        });
        unsafe { msg_send_id![super(this), init] }
    }

    fn setup(&self) {
        unsafe { self.setup_impl() }
    }

    unsafe fn setup_impl(&self) {
        let mtm = MainThreadMarker::new().unwrap();
        let cfg = self.ivars().config.borrow();
        let style = NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel;
        let rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(cfg.style.width, 200.0));
        let panel: Retained<Panel> = unsafe {
            msg_send_id![
                mtm.alloc::<Panel>(),
                initWithContentRect: rect,
                styleMask: style,
                backing: NSBackingStoreType::NSBackingStoreBuffered,
                defer: false,
            ]
        };
        panel.setOpaque(false);
        unsafe { panel.setBackgroundColor(Some(&NSColor::clearColor())) };
        // No window shadow: with a borderless window the shadow is computed
        // from the glass backdrop's rectangular bounds and shows as a black
        // box around the rounded panel. Liquid Glass draws its own edge.
        panel.setHasShadow(false);
        panel.setLevel(25); // NSStatusWindowLevel: above normal windows and menus
        panel.setCollectionBehavior(
            NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary,
        );
        panel.setHidesOnDeactivate(false);
        unsafe {
            panel.setAppearance(NSAppearance::appearanceNamed(NSAppearanceNameVibrantDark).as_deref());
        }
        panel.setDelegate(Some(ProtocolObject::from_ref(self)));

        let content = panel.contentView().unwrap();
        let bounds = content.bounds();
        let resize_mask = NSAutoresizingMaskOptions::NSViewWidthSizable
            | NSAutoresizingMaskOptions::NSViewHeightSizable;

        // All content lives in one container; the material view hosts it.
        let container = unsafe { NSView::initWithFrame(mtm.alloc(), bounds) };
        container.setAutoresizingMask(resize_mask);

        // Material: Liquid Glass (macOS 26+) tinted black, or dark vibrancy
        // as fallback. NSGlassEffectView isn't in the bindings yet, so it is
        // instantiated by name and every selector is guarded — worst case we
        // degrade to the vibrancy path, never crash.
        let glass = objc2::runtime::AnyClass::get("NSGlassEffectView").and_then(|cls| {
            let ok: bool = msg_send![cls, instancesRespondToSelector: sel!(setContentView:)];
            if !ok {
                return None;
            }
            let view: Retained<NSView> = unsafe { msg_send_id![cls, new] };
            Some(view)
        });

        if let Some(glass) = &glass {
            // Rim removal: the glass sits inside a clipping wrapper and
            // extends RIM_CLIP px beyond it on every side, so the material's
            // built-in edge highlight falls outside the visible shape and is
            // cut off entirely.
            const RIM_CLIP: f64 = 2.0;
            let wrapper = unsafe { NSView::initWithFrame(mtm.alloc(), bounds) };
            wrapper.setAutoresizingMask(resize_mask);
            wrapper.setWantsLayer(true);
            if let Some(layer) = wrapper.layer() {
                layer.setCornerRadius(cfg.style.panel_corner_radius);
                layer.setMasksToBounds(true);
                let curve = NSString::from_str("continuous");
                let _: () = msg_send![&*layer, setCornerCurve: &*curve];
            }
            glass.setFrame(NSRect::new(
                NSPoint::new(-RIM_CLIP, -RIM_CLIP),
                NSSize::new(
                    bounds.size.width + 2.0 * RIM_CLIP,
                    bounds.size.height + 2.0 * RIM_CLIP,
                ),
            ));
            glass.setAutoresizingMask(resize_mask);
            unsafe {
                let radius = cfg.style.panel_corner_radius + RIM_CLIP;
                let _: () = msg_send![&**glass, setCornerRadius: radius];
                let tint_color = rgba(cfg.style.panel_background, cfg.style.panel_opacity);
                let _: () = msg_send![&**glass, setTintColor: &*tint_color];
            }
            wrapper.addSubview(glass);
            content.addSubview(&wrapper);
            content.addSubview(&container);
        } else {
            let effect = unsafe { NSVisualEffectView::initWithFrame(mtm.alloc(), bounds) };
            unsafe {
                effect.setMaterial(NSVisualEffectMaterial::HUDWindow);
                effect.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
                effect.setState(NSVisualEffectState::Active);
            }
            effect.setAutoresizingMask(resize_mask);
            effect.setWantsLayer(true);
            if let Some(layer) = effect.layer() {
                layer.setCornerRadius(cfg.style.panel_corner_radius);
                layer.setMasksToBounds(true);
                // Apple's continuous corner curve — squircle, not circular arc.
                let curve = NSString::from_str("continuous");
                let _: () = msg_send![&*layer, setCornerCurve: &*curve];
            }
            content.addSubview(&effect);

            let tint = unsafe { NSView::initWithFrame(mtm.alloc(), bounds) };
            tint.setAutoresizingMask(resize_mask);
            tint.setWantsLayer(true);
            if let Some(layer) = tint.layer() {
                set_layer_bg(
                    &layer,
                    &rgba(cfg.style.panel_background, cfg.style.panel_opacity),
                );
            }
            effect.addSubview(&tint);
            content.addSubview(&container);
        }

        // Input: glyph + borderless field.
        let glyph_font = unsafe { NSFont::systemFontOfSize(34.0) };
        let glyph = make_label(
            mtm,
            &cfg.icons.search,
            &glyph_font,
            &rgba(cfg.style.panel_foreground, 0.85),
        );
        container.addSubview(&glyph);

        let field = unsafe { NSTextField::new(mtm) };
        unsafe {
            field.setBezeled(false);
            field.setBordered(false);
            field.setDrawsBackground(false);
            field.setFont(Some(&resolve_font(
                &cfg.style.font_family,
                0.0,
                cfg.style.input_font_size,
            )));
            field.setTextColor(Some(&rgba(cfg.style.panel_foreground, 1.0)));
            field.setFocusRingType(NSFocusRingType::None);
            field.setDelegate(Some(ProtocolObject::from_ref(self)));
        }
        unsafe { field.sizeToFit() };
        container.addSubview(&field);

        // Results container.
        let rows_area = unsafe { NSView::initWithFrame(mtm.alloc(), bounds) };
        container.addSubview(&rows_area);

        let ivars = self.ivars();
        ivars.panel.set(panel).ok();
        ivars.field.set(field).ok();
        ivars.glyph.set(glyph).ok();
        ivars.rows_area.set(rows_area).ok();

        // Key monitor for the configurable in-panel bindings ([keys] in the
        // config): chords with modifiers don't route reliably through the
        // text system in a borderless, menu-less panel. Local monitor = our
        // process only, our key window only.
        let this_ptr = self as *const Delegate as usize;
        let block = block2::RcBlock::new(
            move |event: std::ptr::NonNull<NSEvent>| -> *mut NSEvent {
                let delegate = unsafe { &*(this_ptr as *const Delegate) };
                if delegate.handle_key_event(unsafe { event.as_ref() }) {
                    std::ptr::null_mut()
                } else {
                    event.as_ptr()
                }
            },
        );
        let monitor = unsafe {
            NSEvent::addLocalMonitorForEventsMatchingMask_handler(NSEventMask::KeyDown, &block)
        };
        // Monitor and block live for the process lifetime.
        std::mem::forget(block);
        std::mem::forget(monitor);
    }

    /// Returns true if the event matched a configured binding and should be
    /// swallowed.
    fn handle_key_event(&self, event: &NSEvent) -> bool {
        let ivars = self.ivars();
        let Some(panel) = ivars.panel.get() else {
            return false;
        };
        if !panel.isVisible() || !panel.isKeyWindow() {
            return false;
        }
        let flags = unsafe { event.modifierFlags() };
        let cmd = flags.contains(NSEventModifierFlags::NSEventModifierFlagCommand);
        let ctrl = flags.contains(NSEventModifierFlags::NSEventModifierFlagControl);
        let opt = flags.contains(NSEventModifierFlags::NSEventModifierFlagOption);
        let shift = flags.contains(NSEventModifierFlags::NSEventModifierFlagShift);
        let chars = unsafe { event.charactersIgnoringModifiers() }
            .map(|s| s.to_string().to_lowercase())
            .unwrap_or_default();

        let action = ivars.config.borrow().binds.iter().find_map(|(chord, action)| {
            (chord.cmd == cmd
                && chord.ctrl == ctrl
                && chord.opt == opt
                && chord.shift == shift
                && config::event_chars(chord.key) == chars)
                .then_some(*action)
        });
        match action {
            Some(a) => {
                self.perform(a);
                true
            }
            None => false,
        }
    }

    fn perform(&self, action: Action) {
        let ivars = self.ivars();
        match action {
            Action::Open => self.execute(false),
            Action::LaunchNew => self.execute(true),
            Action::Reveal => self.reveal(),
            Action::Clear => {
                if let Some(field) = ivars.field.get() {
                    unsafe { field.setStringValue(&NSString::from_str("")) };
                    ivars.selected.set(0);
                    self.refresh();
                }
            }
            Action::Dismiss => self.hide(),
            Action::SelectAll => {
                if let (Some(panel), Some(field)) = (ivars.panel.get(), ivars.field.get()) {
                    unsafe {
                        if let Some(editor) = panel.fieldEditor_forObject(true, Some(field)) {
                            let _: () = msg_send![
                                &*editor,
                                selectAll: std::ptr::null::<objc2::runtime::AnyObject>()
                            ];
                        }
                    }
                }
            }
            Action::MoveUp => self.move_selection(-1),
            Action::MoveDown => self.move_selection(1),
            Action::RefreshConfig => self.reload_config(),
        }
    }

    /// Re-read the config file and re-apply it live. Global hotkeys and the
    /// panel's own chrome (background opacity, corner radius) are wired once
    /// at launch and still need a restart; everything else — layout, colors,
    /// fonts, icons, shortcuts, in-panel binds — updates immediately.
    fn reload_config(&self) {
        let ivars = self.ivars();
        *ivars.config.borrow_mut() = config::load();
        let cfg = ivars.config.borrow();
        if let Some(field) = ivars.field.get() {
            unsafe {
                field.setFont(Some(&resolve_font(
                    &cfg.style.font_family,
                    0.0,
                    cfg.style.input_font_size,
                )));
                field.setTextColor(Some(&rgba(cfg.style.panel_foreground, 1.0)));
            }
        }
        if let Some(glyph) = ivars.glyph.get() {
            unsafe {
                glyph.setStringValue(&NSString::from_str(&cfg.icons.search));
                glyph.setTextColor(Some(&rgba(cfg.style.panel_foreground, 0.85)));
                glyph.sizeToFit();
            }
        }
        drop(cfg);
        // Rebuild the results with the fresh style/layout.
        self.refresh();
    }

    fn toggle(&self) {
        let Some(panel) = self.ivars().panel.get() else {
            return;
        };
        if panel.isVisible() {
            self.hide();
        } else {
            self.show();
        }
    }

    fn show(&self) {
        unsafe { self.show_impl() }
    }

    unsafe fn show_impl(&self) {
        let mtm = MainThreadMarker::new().unwrap();
        let ivars = self.ivars();
        let (Some(panel), Some(field)) = (ivars.panel.get(), ivars.field.get()) else {
            return;
        };
        let Some(screen) = NSScreen::mainScreen(mtm) else {
            return;
        };

        field.setStringValue(&NSString::from_str(""));
        ivars.selected.set(0);

        let vf = screen.visibleFrame();
        ivars.top_y.set(vf.origin.y + vf.size.height * 0.72);
        self.refresh();
        let h = panel.frame().size.height;
        let x = vf.origin.x + (vf.size.width - ivars.config.borrow().style.width) / 2.0;
        panel.setFrameOrigin(NSPoint::new(x, ivars.top_y.get() - h));

        // Live stats: refresh on an interval while the panel is up.
        if ivars.stats_timer.borrow().is_none() {
            let nil = std::ptr::null::<objc2::runtime::AnyObject>();
            let timer: Retained<objc2::runtime::AnyObject> = msg_send_id![
                objc2::class!(NSTimer),
                scheduledTimerWithTimeInterval: ivars.config.borrow().stats_interval,
                target: self,
                selector: sel!(refreshTick),
                userInfo: nil,
                repeats: true
            ];
            *ivars.stats_timer.borrow_mut() = Some(timer);
        }

        panel.makeKeyAndOrderFront(None);
        panel.makeFirstResponder(Some(field));
        // Caret matches the search text color.
        if let Some(editor) = panel.fieldEditor_forObject(true, Some(field)) {
            let text_view: Retained<NSTextView> = unsafe { Retained::cast(editor) };
            unsafe {
                text_view.setInsertionPointColor(Some(&rgba(
                    ivars.config.borrow().style.panel_foreground,
                    1.0,
                )))
            };
        }
    }

    fn hide(&self) {
        let ivars = self.ivars();
        if ivars.hiding.replace(true) {
            return;
        }
        if let Some(timer) = ivars.stats_timer.borrow_mut().take() {
            let _: () = unsafe { msg_send![&*timer, invalidate] };
        }
        // Drop the cached scan so the next summon re-reads the app directories.
        ivars.installed_cache.borrow_mut().take();
        if let Some(panel) = ivars.panel.get() {
            panel.orderOut(None);
        }
        ivars.hiding.set(false);
    }

    fn query(&self) -> String {
        self.ivars()
            .field
            .get()
            .map(|f| unsafe { f.stringValue() }.to_string())
            .unwrap_or_default()
    }

    /// Recompute results for the current query. The app-directory scan is done
    /// once per summon and cached (see `installed_cache`); every keystroke after
    /// that reuses it, so only re-filtering and re-ranking run per keystroke.
    fn refresh(&self) {
        let query = self.query();
        let running = running_apps();

        let mut entries: Vec<Entry> = Vec::new();
        if query.is_empty() {
            entries = running;
        } else {
            let mut scored: Vec<(i32, Entry)> = Vec::new();
            let mut cache = self.ivars().installed_cache.borrow_mut();
            let installed = cache.get_or_insert_with(apps::scan_installed);
            let mut seen: Vec<String> = Vec::new();
            for mut entry in running {
                if let Some((s, positions)) = apps::match_positions(&query, &entry.name) {
                    seen.push(entry.name.to_lowercase());
                    entry.matched = positions;
                    scored.push((s + RUNNING_BONUS, entry));
                }
            }
            for app in installed.iter() {
                if seen.contains(&app.name.to_lowercase()) {
                    continue;
                }
                if let Some((s, positions)) = apps::match_positions(&query, &app.name) {
                    scored.push((
                        s,
                        Entry {
                            name: display_name(&app.name),
                            path: Some(app.path.clone()),
                            running: None,
                            matched: positions,
                            windows: 0,
                            stats: None,
                            command: None,
                        },
                    ));
                }
            }
            for (name, cmd) in &self.ivars().config.borrow().shortcuts {
                if seen.contains(&name.to_lowercase()) {
                    continue;
                }
                if let Some((s, positions)) = apps::match_positions(&query, name) {
                    scored.push((
                        s,
                        Entry {
                            name: display_name(name),
                            path: None,
                            running: None,
                            matched: positions,
                            windows: 0,
                            stats: None,
                            command: Some(cmd.clone()),
                        },
                    ));
                }
            }
            // Sort by score (running apps carry a soft RUNNING_BONUS baked in),
            // then alphabetically. Running is a preference, not an override: a
            // clearly stronger match on a cold app can outrank a warm one.
            scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.name.cmp(&b.1.name)));
            entries.extend(scored.into_iter().map(|(_, e)| e));
        }
        entries.truncate(MAX_ROWS);

        // Stats for the visible running apps (a handful of syscalls plus one
        // window-list snapshot — microseconds, fresh every time). CPU% needs
        // two samples: rows show "…" until the second sample lands, then a
        // one-shot refreshTick fills the number in. Never blocks.
        let window_counts = stats::window_counts();
        let procs = stats::ProcSnapshot::new();
        let mut cpu_pending = false;
        {
            let mut samples = self.ivars().cpu_samples.borrow_mut();
            samples.retain(|pid, _| procs.is_alive(*pid));
            let now = std::time::Instant::now();
            for entry in entries.iter_mut() {
                if let Some(app) = &entry.running {
                    let pid = unsafe { app.processIdentifier() };
                    entry.windows = window_counts.get(&pid).copied().unwrap_or(0);
                    // An app's real footprint is its whole process tree:
                    // browser renderers, GPU helpers, and XPC services are
                    // children of the root pid, not part of it.
                    let mut pct_sum = 0.0f64;
                    let mut root_seen = false;
                    let mut root_ready = false;
                    for tree_pid in procs.tree_pids(pid) {
                        let Some((_ram, cpu_secs)) = stats::proc_stats(tree_pid) else {
                            continue;
                        };
                        let pct = match samples.get(&tree_pid) {
                            Some(prev) => {
                                let dt = now.duration_since(prev.at).as_secs_f64();
                                if dt >= CPU_MIN_INTERVAL {
                                    let p =
                                        (cpu_secs - prev.cpu_secs).max(0.0) / dt * 100.0;
                                    samples.insert(
                                        tree_pid,
                                        CpuSample { cpu_secs, at: now, pct: Some(p) },
                                    );
                                    Some(p)
                                } else {
                                    prev.pct // too soon; reuse last good reading
                                }
                            }
                            None => {
                                samples.insert(
                                    tree_pid,
                                    CpuSample { cpu_secs, at: now, pct: None },
                                );
                                None
                            }
                        };
                        if tree_pid == pid {
                            root_seen = true;
                            root_ready = pct.is_some();
                        }
                        pct_sum += pct.unwrap_or(0.0);
                    }
                    if !root_seen {
                        continue;
                    }
                    if !root_ready {
                        cpu_pending = true;
                    }
                    entry.stats = Some(RowStats { cpu_pct: pct_sum });
                }
            }
        }
        if cpu_pending {
            unsafe {
                let nil = std::ptr::null::<objc2::runtime::AnyObject>();
                let _: () = msg_send![
                    objc2::class!(NSObject),
                    cancelPreviousPerformRequestsWithTarget: self,
                    selector: sel!(refreshTick),
                    object: nil
                ];
                let _: () = msg_send![
                    self,
                    performSelector: sel!(refreshTick),
                    withObject: nil,
                    afterDelay: 0.4f64
                ];
            }
        }

        let ivars = self.ivars();
        if ivars.selected.get() >= entries.len() {
            ivars.selected.set(entries.len().saturating_sub(1));
        }
        *ivars.entries.borrow_mut() = entries;
        self.relayout();
    }

    fn move_selection(&self, delta: isize) {
        let ivars = self.ivars();
        let len = ivars.entries.borrow().len();
        if len == 0 {
            return;
        }
        let current = ivars.selected.get() as isize;
        let next = (current + delta).rem_euclid(len as isize) as usize;
        ivars.selected.set(next);
        self.relayout();
    }

    /// Reposition everything for the current entry count and rebuild rows.
    fn relayout(&self) {
        unsafe { self.relayout_impl() }
    }

    unsafe fn relayout_impl(&self) {
        let mtm = MainThreadMarker::new().unwrap();
        let ivars = self.ivars();
        let (Some(panel), Some(field), Some(glyph), Some(rows_area)) = (
            ivars.panel.get(),
            ivars.field.get(),
            ivars.glyph.get(),
            ivars.rows_area.get(),
        ) else {
            return;
        };

        let entries = ivars.entries.borrow();
        let n = entries.len();
        let pad = ivars.config.borrow().style.panel_padding;
        let rows_h = if n > 0 {
            n as f64 * ROW_H + ROWS_PAD
        } else {
            0.0
        };
        // Padding wraps the content on both ends: `pad` above the input band
        // and `pad` below the last row.
        let h = pad + INPUT_H + rows_h + pad;

        let panel_w = ivars.config.borrow().style.width;
        let old = panel.frame();
        let top = ivars.top_y.get();
        panel.setFrame_display(
            NSRect::new(
                NSPoint::new(old.origin.x, top - h),
                NSSize::new(panel_w, h),
            ),
            true,
        );

        // Input band, inset from the top by `pad`. The extra 12px keeps the
        // search glyph aligned with the row glyphs (rows inset by 12px).
        let input_inset = pad + 12.0;
        let input_bottom = h - pad - INPUT_H;
        let glyph_size = glyph.frame().size;
        glyph.setFrameOrigin(NSPoint::new(
            input_inset,
            input_bottom + (INPUT_H - glyph_size.height) / 2.0,
        ));
        let field_x = input_inset + glyph_size.width + 14.0;
        let field_h = field.frame().size.height.max(30.0);
        field.setFrame(NSRect::new(
            NSPoint::new(field_x, input_bottom + (INPUT_H - field_h) / 2.0),
            NSSize::new(panel_w - field_x - input_inset, field_h),
        ));

        // Rows fill the space below the input; no footer.
        rows_area.setFrame(NSRect::new(
            NSPoint::new(0.0, pad),
            NSSize::new(panel_w, rows_h),
        ));
        for view in rows_area.subviews().iter() {
            unsafe { view.removeFromSuperview() };
        }
        let selected = ivars.selected.get();
        for (i, entry) in entries.iter().enumerate() {
            let y = rows_h - ROWS_PAD / 2.0 - (i as f64 + 1.0) * ROW_H;
            self.build_row(mtm, rows_area, y, i, entry, i == selected);
        }
    }

    /// Mouse selection; typing afterwards resets it (controlTextDidChange
    /// zeroes the selection), so the keyboard always wins.
    fn select_row(&self, index: usize) {
        let len = self.ivars().entries.borrow().len();
        if index < len && self.ivars().selected.get() != index {
            self.ivars().selected.set(index);
            self.relayout();
        }
    }

    unsafe fn build_row(
        &self,
        mtm: MainThreadMarker,
        parent: &NSView,
        y: f64,
        index: usize,
        entry: &Entry,
        selected: bool,
    ) {
        let cfg = self.ivars().config.borrow();
        let style = &cfg.style;
        let icons = &cfg.icons;
        let row_w = style.width - 2.0 * style.panel_padding;
        let row: Retained<RowView> = {
            let this = mtm.alloc::<RowView>().set_ivars(RowIvars {
                index: Cell::new(index),
                delegate: Cell::new(self as *const Delegate as usize),
            });
            unsafe {
                msg_send_id![
                    super(this),
                    initWithFrame: NSRect::new(
                        NSPoint::new(style.panel_padding, y),
                        NSSize::new(row_w, ROW_H),
                    ),
                ]
            }
        };
        if selected {
            row.setWantsLayer(true);
            if let Some(layer) = row.layer() {
                layer.setCornerRadius(style.selected_item_corner_radius);
                set_layer_bg(
                    &layer,
                    &rgba(style.selected_item_background, style.selected_item_opacity),
                );
            }
        }

        let is_running = entry.running.is_some();
        // All row content takes its hue from one foreground per state; the
        // original design's brightness steps carry over as alphas.
        let fg = if selected {
            style.selected_item_foreground
        } else {
            style.item_foreground
        };

        // Leading state glyph column; `[icons.apps]` overrides win.
        let entry_lower = entry.name.to_lowercase();
        let glyph_text = cfg
            .icon_overrides
            .iter()
            .find_map(|(n, g)| (*n == entry_lower).then_some(g.as_str()))
            .unwrap_or_else(|| state_glyph(entry, icons));
        let glyph_font = unsafe { NSFont::systemFontOfSize(GLYPH_PT) };
        // The icon carries visibility state through its brightness: an app
        // with on-screen windows is brightest, a backgrounded running app is
        // dimmer, a cold app is faintest.
        let glyph_alpha = if entry.windows > 0 {
            0.92
        } else if is_running {
            0.52
        } else if selected {
            0.42
        } else {
            0.30
        };
        let glyph = make_label(mtm, glyph_text, &glyph_font, &rgba(fg, glyph_alpha));
        let glyph_h = glyph.frame().size.height;
        glyph.setFrameOrigin(NSPoint::new(12.0, (ROW_H - glyph_h) / 2.0));
        row.addSubview(&glyph);

        let name_x = 12.0 + GLYPH_COL_W + 10.0;
        let name_font =
            resolve_font(&style.font_family, style.item_font_weight, style.item_font_size);
        // Row text is always full-strength; visibility state lives on the icon.
        let name_alpha = 1.0;
        let name = make_label(mtm, &entry.name, &name_font, &rgba(fg, name_alpha));
        // Matched characters render in the highlight color for the row state.
        if !entry.matched.is_empty() {
            let base = rgba(fg, name_alpha);
            let hi = rgba(
                if selected {
                    style.selected_item_foreground_highlight
                } else {
                    style.item_foreground_highlight
                },
                1.0,
            );
            let attr = attributed_name(&entry.name, &entry.matched, &name_font, &base, &hi);
            unsafe {
                name.setAttributedStringValue(&attr);
                name.sizeToFit();
            }
        }
        let name_h = name.frame().size.height;
        name.setFrameOrigin(NSPoint::new(name_x, (ROW_H - name_h) / 2.0));
        row.addSubview(&name);

        if let Some(rs) = &entry.stats {
            // Running apps: a CPU warning at the right edge once the tree
            // burns ≥ CPU_ALERT_PCT of a core, otherwise a plain presence dot.
            if rs.cpu_pct >= CPU_ALERT_PCT {
                self.build_cpu_warning(mtm, &row, row_w - 12.0, rs.cpu_pct, selected);
            } else {
                self.build_running_dot(mtm, &row, row_w - 12.0);
            }
        } else if let Some(path) = &entry.path {
            // Installed rows: location tag pill with symbol.
            let (label_text, symbol) = location_for(path, icons);
            self.build_tag_pill(mtm, &row, row_w, label_text, symbol, selected);
        } else if entry.command.is_some() {
            self.build_tag_pill(mtm, &row, row_w, "Shortcut", &icons.shortcut, selected);
        }

        parent.addSubview(&row);
    }

    /// Right-aligned outlined pill: SF Symbol icon + label. Location tag on
    /// installed rows, "Shortcut" tag on command rows.
    unsafe fn build_tag_pill(
        &self,
        mtm: MainThreadMarker,
        row: &NSView,
        row_w: f64,
        label_text: &str,
        symbol: &str,
        selected: bool,
    ) {
        let cfg = self.ivars().config.borrow();
        let style = &cfg.style;
        let fg = if selected {
            style.selected_item_foreground
        } else {
            style.item_foreground
        };
        let alpha = if selected { 0.80 } else { 0.55 };

        let font = unsafe { NSFont::systemFontOfSize(11.0) };
        let label = make_label(mtm, label_text, &font, &rgba(fg, alpha));
        let label_size = label.frame().size;

        let icon_d = 12.0;
        let gap = 5.0;
        let pad_h = 9.0;
        let pill_h = 22.0;
        let image = unsafe {
            objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(
                &NSString::from_str(symbol),
                None,
            )
        };
        let content_w =
            if image.is_some() { icon_d + gap } else { 0.0 } + label_size.width;
        let pill_w = content_w + 2.0 * pad_h;

        let pill = unsafe {
            NSView::initWithFrame(
                mtm.alloc(),
                NSRect::new(
                    NSPoint::new(row_w - 12.0 - pill_w, (ROW_H - pill_h) / 2.0),
                    NSSize::new(pill_w, pill_h),
                ),
            )
        };
        pill.setWantsLayer(true);
        if let Some(layer) = pill.layer() {
            layer.setCornerRadius(pill_h / 2.0);
            let border = rgba(fg, if selected { 0.35 } else { 0.20 });
            unsafe {
                let cg: *mut c_void = msg_send![&*border, CGColor];
                let _: () = msg_send![&*layer, setBorderColor: cg];
                let _: () = msg_send![&*layer, setBorderWidth: 1.0f64];
            }
        }

        let mut x = pad_h;
        if let Some(image) = image {
            let iv = unsafe {
                objc2_app_kit::NSImageView::initWithFrame(
                    mtm.alloc(),
                    NSRect::new(
                        NSPoint::new(x, (pill_h - icon_d) / 2.0),
                        NSSize::new(icon_d, icon_d),
                    ),
                )
            };
            unsafe {
                iv.setImage(Some(&image));
                iv.setImageScaling(
                    objc2_app_kit::NSImageScaling::NSImageScaleProportionallyUpOrDown,
                );
                iv.setContentTintColor(Some(&rgba(fg, alpha)));
            }
            pill.addSubview(&iv);
            x += icon_d + gap;
        }
        label.setFrameOrigin(NSPoint::new(x, (pill_h - label_size.height) / 2.0));
        pill.addSubview(&label);
        row.addSubview(&pill);
    }

    /// A small filled dot with its right edge at `right_x`, vertically
    /// centered — the presence marker for a running app that isn't busy.
    unsafe fn build_running_dot(&self, mtm: MainThreadMarker, row: &NSView, right_x: f64) {
        const DOT_D: f64 = 6.0;
        let color = self.ivars().config.borrow().style.running_dot;
        let dot = unsafe {
            NSView::initWithFrame(
                mtm.alloc(),
                NSRect::new(
                    NSPoint::new(right_x - DOT_D, (ROW_H - DOT_D) / 2.0),
                    NSSize::new(DOT_D, DOT_D),
                ),
            )
        };
        dot.setWantsLayer(true);
        if let Some(layer) = dot.layer() {
            layer.setCornerRadius(DOT_D / 2.0);
            set_layer_bg(&layer, &rgba(color, 1.0));
        }
        row.addSubview(&dot);
    }

    /// CPU alert at the row's right edge (right edge at `right_x`): a red
    /// warning symbol with "CPU 88%" to its left. Shown in place of the
    /// presence dot once the tree crosses CPU_ALERT_PCT.
    unsafe fn build_cpu_warning(
        &self,
        mtm: MainThreadMarker,
        row: &NSView,
        right_x: f64,
        pct: f64,
        selected: bool,
    ) {
        let color = self.ivars().config.borrow().style.cpu_alert;
        let alpha = if selected { 1.0 } else { 0.9 };

        // Warning glyph on the right.
        let icon_d = 13.0;
        let gap = 6.0;
        let image = unsafe {
            objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(
                &NSString::from_str("exclamationmark.triangle.fill"),
                None,
            )
        };
        let mut text_right = right_x;
        if let Some(image) = image {
            let iv = unsafe {
                objc2_app_kit::NSImageView::initWithFrame(
                    mtm.alloc(),
                    NSRect::new(
                        NSPoint::new(right_x - icon_d, (ROW_H - icon_d) / 2.0),
                        NSSize::new(icon_d, icon_d),
                    ),
                )
            };
            unsafe {
                iv.setImage(Some(&image));
                iv.setImageScaling(
                    objc2_app_kit::NSImageScaling::NSImageScaleProportionallyUpOrDown,
                );
                iv.setContentTintColor(Some(&rgba(color, 1.0)));
            }
            row.addSubview(&iv);
            text_right = right_x - icon_d - gap;
        }

        // "CPU 88%" to the left of the glyph; monospaced digits so the number
        // doesn't jitter as it updates.
        let font: Retained<NSFont> = unsafe {
            msg_send_id![
                NSFont::class(),
                monospacedDigitSystemFontOfSize: 11.0f64,
                weight: 0.3f64 // NSFontWeightSemibold
            ]
        };
        let text = format!("CPU {}%", pct.round() as i64);
        let label = make_label(mtm, &text, &font, &rgba(color, alpha));
        let size = label.frame().size;
        label.setFrameOrigin(NSPoint::new(
            text_right - size.width,
            (ROW_H - size.height) / 2.0,
        ));
        row.addSubview(&label);
    }

    /// Activate the selected running app, or launch it. `force_open` skips
    /// the activate path and sends a real open (reopen event) even when the
    /// app is already running.
    fn execute(&self, force_open: bool) {
        let entry_data = {
            let entries = self.ivars().entries.borrow();
            let Some(entry) = entries.get(self.ivars().selected.get()) else {
                return;
            };
            (entry.path.clone(), entry.running.clone(), entry.command.clone())
        };
        self.hide();

        let (path, running, command) = entry_data;
        if let Some(cmd) = command {
            // Fire-and-forget; the shell owns the child from here.
            let _ = std::process::Command::new("/bin/sh").arg("-c").arg(&cmd).spawn();
            return;
        }
        if !force_open {
            if let Some(app) = &running {
                unsafe {
                    app.activateWithOptions(
                        NSApplicationActivationOptions::NSApplicationActivateIgnoringOtherApps,
                    );
                }
                return;
            }
        }
        let launch_path = path.or_else(|| {
            running
                .as_ref()
                .and_then(|a| unsafe { a.bundleURL() })
                .and_then(|u| unsafe { u.path() }.map(|p| PathBuf::from(p.to_string())))
        });
        if let Some(p) = launch_path {
            if let Some(s) = p.to_str() {
                let url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(s)) };
                let config = unsafe { NSWorkspaceOpenConfiguration::configuration() };
                unsafe {
                    NSWorkspace::sharedWorkspace()
                        .openApplicationAtURL_configuration_completionHandler(&url, &config, None);
                }
            }
        } else if let Some(app) = &running {
            // No bundle path available; best effort.
            unsafe {
                app.activateWithOptions(
                    NSApplicationActivationOptions::NSApplicationActivateIgnoringOtherApps,
                );
            }
        }
    }

    /// Reveal the selected app's bundle in Finder (like `open --reveal`).
    fn reveal(&self) {
        let entry_data = {
            let entries = self.ivars().entries.borrow();
            let Some(entry) = entries.get(self.ivars().selected.get()) else {
                return;
            };
            (entry.path.clone(), entry.running.clone())
        };
        self.hide();

        let (path, running) = entry_data;
        let reveal_path = path.or_else(|| {
            running
                .as_ref()
                .and_then(|a| unsafe { a.bundleURL() })
                .and_then(|u| unsafe { u.path() }.map(|p| PathBuf::from(p.to_string())))
        });
        let Some(s) = reveal_path.as_ref().and_then(|p| p.to_str()) else {
            return;
        };
        unsafe {
            let url = NSURL::fileURLWithPath(&NSString::from_str(s));
            let urls = objc2_foundation::NSArray::from_vec(vec![url]);
            let ws = NSWorkspace::sharedWorkspace();
            let _: () = msg_send![&*ws, activateFileViewerSelectingURLs: &*urls];
        }
    }
}

/// Running apps with a Dock presence (Regular activation policy), current
/// process excluded.
fn running_apps() -> Vec<Entry> {
    unsafe { running_apps_impl() }
}

unsafe fn running_apps_impl() -> Vec<Entry> {
    let mut out = Vec::new();
    let ws = unsafe { NSWorkspace::sharedWorkspace() };
    let running = unsafe { ws.runningApplications() };
    let own_pid = std::process::id() as i32;
    for i in 0..running.len() {
        let app = running.objectAtIndex(i);
        if unsafe { app.activationPolicy() }
            != NSApplicationActivationPolicy::Regular
        {
            continue;
        }
        if unsafe { app.processIdentifier() } == own_pid {
            continue;
        }
        let Some(name) = (unsafe { app.localizedName() }) else {
            continue;
        };
        out.push(Entry {
            name: display_name(&name.to_string()),
            path: None,
            running: Some(app),
            matched: Vec::new(),
            windows: 0,
            stats: None,
            command: None,
        });
    }
    out
}

extern "C" {
    fn getxattr(
        path: *const std::ffi::c_char,
        name: *const std::ffi::c_char,
        value: *mut c_void,
        size: usize,
        position: u32,
        options: std::ffi::c_int,
    ) -> isize;
}

/// FinderInfo finderFlags bit 0x0400 = kHasCustomIcon. Cheap idempotence
/// check so the resource fork isn't rewritten on every launch.
fn file_has_custom_icon(path: &str) -> bool {
    let Ok(cpath) = std::ffi::CString::new(path) else {
        return false;
    };
    let mut info = [0u8; 32];
    let n = unsafe {
        getxattr(
            cpath.as_ptr(),
            b"com.apple.FinderInfo\0".as_ptr().cast(),
            info.as_mut_ptr().cast(),
            info.len(),
            0,
            0,
        )
    };
    n == 32 && u16::from_be_bytes([info[8], info[9]]) & 0x0400 != 0
}

extern "C" fn hotkey_pressed(_next: *mut c_void, event: *mut c_void, user: *mut c_void) -> i32 {
    // Carbon dispatches this on the main run loop.
    let delegate = unsafe { &*(user as *const Delegate) };
    let id = unsafe { hotkey::event_hotkey_id(event) }.unwrap_or(1);
    let mode = delegate
        .ivars()
        .config
        .borrow()
        .hotkeys
        .get(id.saturating_sub(1) as usize)
        .map(|(_, mode)| *mode)
        .unwrap_or(Mode::Launcher);
    match mode {
        Mode::Launcher => delegate.toggle(),
    }
    0
}

fn main() {
    let cfg = config::load();
    let mtm = MainThreadMarker::new().unwrap();
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    // Bare binary, no .app bundle: Activity Monitor and friends take the
    // process icon from the executable file's Finder icon, not from anything
    // set at runtime. Stamp the embedded icon onto our own file once —
    // resource fork + FinderInfo only, so the Mach-O data (and its code
    // signature) is untouched. setApplicationIconImage covers any transient
    // dock-tile contexts.
    let icon_data = NSData::with_bytes(include_bytes!("../assets/icon.png"));
    if let Some(icon) = objc2_app_kit::NSImage::initWithData(
        mtm.alloc::<objc2_app_kit::NSImage>(),
        &icon_data,
    ) {
        unsafe { app.setApplicationIconImage(Some(&icon)) };
        let exe = std::env::current_exe()
            .ok()
            .and_then(|p| p.to_str().map(String::from));
        if let Some(exe) = exe {
            if !file_has_custom_icon(&exe) {
                let ws = unsafe { NSWorkspace::sharedWorkspace() };
                unsafe {
                    ws.setIcon_forFile_options(
                        Some(&icon),
                        &NSString::from_str(&exe),
                        objc2_app_kit::NSWorkspaceIconCreationOptions(0),
                    )
                };
            }
        }
    }

    // (vk, mods) per configured trigger; ids follow list order (1-based).
    let mut triggers: Vec<(u32, u32)> = cfg
        .hotkeys
        .iter()
        .filter_map(|(chord, _)| {
            config::carbon_vk(chord.key).map(|vk| (vk, config::carbon_mods(chord)))
        })
        .collect();
    if triggers.is_empty() {
        eprintln!("motherfucker: no usable hotkeys in config; falling back to ⌥Space");
        triggers.push((hotkey::VK_SPACE, hotkey::MOD_OPTION));
    }

    let delegate = Delegate::new(mtm, cfg);
    app.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));

    unsafe {
        hotkey::register_all(
            &triggers,
            hotkey_pressed,
            Retained::as_ptr(&delegate) as *mut c_void,
        )
        .expect("failed to register global hotkey (is another instance running?)");
    }

    unsafe { app.run() };
}
