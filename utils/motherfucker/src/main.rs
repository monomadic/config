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
mod hotkey;
mod stats;

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
    MainThreadMarker, NSMutableAttributedString, NSNotification, NSObject, NSObjectProtocol,
    NSPoint, NSRange, NSRect, NSSize, NSString, NSURL,
};
use objc2_quartz_core::CALayer;

// ---- visual spec (black glass) ----
const PANEL_W: f64 = 620.0;
const RADIUS: f64 = 18.0;
const INPUT_H: f64 = 58.0;
const ROW_H: f64 = 32.0;
const ROWS_PAD: f64 = 12.0;
const HINTS_H: f64 = 44.0;
const MAX_ROWS: usize = 6;
// Tint over the glass: deep pure black.
const TINT_ALPHA: f64 = 0.50;
const TINT_RGB: (f64, f64, f64) = (0.0, 0.0, 0.0);
// Selection: slightly lighter than the panel.
const SEL_RGB: (f64, f64, f64) = (1.0, 1.0, 1.0);
const SEL_ALPHA: f64 = 0.10;
// Chips: solid black pill, outline on the key box only.
const KEYBOX_BORDER_ALPHA: f64 = 0.30;
// Matched query characters render in this accent (ice blue).
const HI_RGB: (f64, f64, f64) = (0.38, 0.75, 1.0);
// CPU sampling: minimum interval for a trustworthy percentage.
const CPU_MIN_INTERVAL: f64 = 0.25;

// ---- summon key: ⌥Space (change to MOD_CMD for ⌘Space once Spotlight is off)
const SUMMON_KEY: u32 = hotkey::VK_SPACE;
const SUMMON_MODS: u32 = hotkey::MOD_OPTION;

struct Entry {
    name: String,
    path: Option<PathBuf>,
    running: Option<Retained<NSRunningApplication>>,
    /// Char indices in `name` matched by the query (for highlighting).
    matched: Vec<usize>,
    /// Right-aligned stats for running apps: "1.2 GB  12%  􀏜 3".
    stats_line: Option<String>,
}

/// Last CPU-time sample per pid, for computing a percentage between looks.
struct CpuSample {
    cpu_secs: f64,
    at: std::time::Instant,
    pct: Option<f64>,
}

#[derive(Default)]
struct State {
    panel: OnceCell<Retained<Panel>>,
    field: OnceCell<Retained<NSTextField>>,
    glyph: OnceCell<Retained<NSTextField>>,
    rows_area: OnceCell<Retained<NSView>>,
    hints: OnceCell<Retained<NSView>>,
    entries: RefCell<Vec<Entry>>,
    selected: Cell<usize>,
    top_y: Cell<f64>,
    hiding: Cell<bool>,
    cpu_samples: RefCell<std::collections::HashMap<i32, CpuSample>>,
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
                self.execute();
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
        /// One-shot follow-up render once CPU has a second sample.
        #[method(refreshTick)]
        fn refresh_tick(&self) {
            if self.ivars().panel.get().is_some_and(|p| p.isVisible()) {
                self.refresh();
            }
        }
    }
);

fn white(alpha: f64) -> Retained<NSColor> {
    unsafe { NSColor::colorWithWhite_alpha(1.0, alpha) }
}

fn set_layer_bg(layer: &CALayer, color: &NSColor) {
    unsafe {
        let cg: *mut c_void = msg_send![color, CGColor];
        let _: () = msg_send![layer, setBackgroundColor: cg];
    }
}

fn set_layer_border(layer: &CALayer, color: &NSColor, width: f64) {
    unsafe {
        let cg: *mut c_void = msg_send![color, CGColor];
        let _: () = msg_send![layer, setBorderColor: cg];
    }
    layer.setBorderWidth(width);
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
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        let this = mtm.alloc::<Self>().set_ivars(State::default());
        unsafe { msg_send_id![super(this), init] }
    }

    fn setup(&self) {
        unsafe { self.setup_impl() }
    }

    unsafe fn setup_impl(&self) {
        let mtm = MainThreadMarker::new().unwrap();
        let style = NSWindowStyleMask::Borderless | NSWindowStyleMask::NonactivatingPanel;
        let rect = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(PANEL_W, 200.0));
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
                layer.setCornerRadius(RADIUS);
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
                let _: () = msg_send![&**glass, setCornerRadius: RADIUS + RIM_CLIP];
                let (r, g, b) = TINT_RGB;
                let tint_color =
                    NSColor::colorWithSRGBRed_green_blue_alpha(r, g, b, TINT_ALPHA);
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
                layer.setCornerRadius(RADIUS);
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
                let (r, g, b) = TINT_RGB;
                set_layer_bg(&layer, unsafe {
                    &NSColor::colorWithSRGBRed_green_blue_alpha(r, g, b, TINT_ALPHA)
                });
            }
            effect.addSubview(&tint);
            content.addSubview(&container);
        }

        // Input: glyph + borderless field.
        let glyph_font = unsafe { NSFont::systemFontOfSize(34.0) };
        let glyph = make_label(mtm, "⌕", &glyph_font, &white(0.85));
        container.addSubview(&glyph);

        let field = unsafe { NSTextField::new(mtm) };
        unsafe {
            field.setBezeled(false);
            field.setBordered(false);
            field.setDrawsBackground(false);
            field.setFont(Some(&NSFont::systemFontOfSize(24.0)));
            field.setTextColor(Some(&NSColor::whiteColor()));
            field.setFocusRingType(NSFocusRingType::None);
            field.setDelegate(Some(ProtocolObject::from_ref(self)));
        }
        unsafe { field.sizeToFit() };
        container.addSubview(&field);

        // Results container.
        let rows_area = unsafe { NSView::initWithFrame(mtm.alloc(), bounds) };
        container.addSubview(&rows_area);

        // Hint chips.
        let hints = unsafe { NSView::initWithFrame(mtm.alloc(), bounds) };
        let mut x = 20.0;
        for (key, action) in [("↩", "switch"), ("⌘↩", "open"), ("⎋", "dismiss")] {
            x += self.build_chip(mtm, &hints, x, key, action) + 10.0;
        }
        container.addSubview(&hints);

        let ivars = self.ivars();
        ivars.panel.set(panel).ok();
        ivars.field.set(field).ok();
        ivars.glyph.set(glyph).ok();
        ivars.rows_area.set(rows_area).ok();
        ivars.hints.set(hints).ok();

        // Key monitor for shortcuts the text system doesn't route reliably in
        // a borderless, menu-less panel: ⌃U (clear), ⌃C (dismiss), ⌘A
        // (select all). Local monitor = our process only, our key window only.
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

    /// Returns true if the event was handled and should be swallowed.
    fn handle_key_event(&self, event: &NSEvent) -> bool {
        let ivars = self.ivars();
        let (Some(panel), Some(field)) = (ivars.panel.get(), ivars.field.get()) else {
            return false;
        };
        if !panel.isVisible() || !panel.isKeyWindow() {
            return false;
        }
        let flags = unsafe { event.modifierFlags() };
        let ctrl = flags.contains(NSEventModifierFlags::NSEventModifierFlagControl);
        let cmd = flags.contains(NSEventModifierFlags::NSEventModifierFlagCommand);
        let chars = unsafe { event.charactersIgnoringModifiers() }
            .map(|s| s.to_string())
            .unwrap_or_default();

        if ctrl && chars == "u" {
            unsafe { field.setStringValue(&NSString::from_str("")) };
            ivars.selected.set(0);
            self.refresh();
            return true;
        }
        if ctrl && chars == "c" {
            self.hide();
            return true;
        }
        if cmd && chars == "a" {
            unsafe {
                if let Some(editor) = panel.fieldEditor_forObject(true, Some(field)) {
                    let _: () = msg_send![
                        &*editor,
                        selectAll: std::ptr::null::<objc2::runtime::AnyObject>()
                    ];
                }
            }
            return true;
        }
        false
    }

    /// Builds one bordered rounded chip; returns its width.
    unsafe fn build_chip(
        &self,
        mtm: MainThreadMarker,
        parent: &NSView,
        x: f64,
        key: &str,
        action: &str,
    ) -> f64 {
        let key_font = unsafe { NSFont::monospacedSystemFontOfSize_weight(12.0, 0.0) };
        let action_font = unsafe { NSFont::systemFontOfSize(12.5) };
        let key_label = make_label(mtm, key, &key_font, &white(0.90));
        let action_label = make_label(mtm, action, &action_font, &white(0.75));

        let key_w = key_label.frame().size.width;
        let key_h = key_label.frame().size.height;
        let action_w = action_label.frame().size.width;
        let action_h = action_label.frame().size.height;

        let chip_h = 26.0;
        let box_h = 19.0;
        let box_w = key_w + 12.0;
        let pad = 5.0; // chip padding around the key box
        let gap = 8.0;
        let end_pad = 12.0;
        let chip_w = pad + box_w + gap + action_w + end_pad;

        let chip = unsafe {
            NSView::initWithFrame(
                mtm.alloc(),
                NSRect::new(
                    NSPoint::new(x, (HINTS_H - chip_h) / 2.0),
                    NSSize::new(chip_w, chip_h),
                ),
            )
        };
        // Solid black pill, no outline.
        chip.setWantsLayer(true);
        if let Some(layer) = chip.layer() {
            layer.setCornerRadius(chip_h / 2.0);
            set_layer_bg(&layer, unsafe {
                &NSColor::colorWithSRGBRed_green_blue_alpha(0.0, 0.0, 0.0, 1.0)
            });
        }

        // Key shortcut gets the outline instead.
        let key_box = unsafe {
            NSView::initWithFrame(
                mtm.alloc(),
                NSRect::new(
                    NSPoint::new(pad, (chip_h - box_h) / 2.0),
                    NSSize::new(box_w, box_h),
                ),
            )
        };
        key_box.setWantsLayer(true);
        if let Some(layer) = key_box.layer() {
            layer.setCornerRadius(6.0);
            set_layer_border(&layer, &white(KEYBOX_BORDER_ALPHA), 1.0);
        }
        key_label.setFrameOrigin(NSPoint::new(
            (box_w - key_w) / 2.0,
            (box_h - key_h) / 2.0,
        ));
        key_box.addSubview(&key_label);

        action_label.setFrameOrigin(NSPoint::new(
            pad + box_w + gap,
            (chip_h - action_h) / 2.0,
        ));
        chip.addSubview(&key_box);
        chip.addSubview(&action_label);
        parent.addSubview(&chip);
        chip_w
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
        let x = vf.origin.x + (vf.size.width - PANEL_W) / 2.0;
        panel.setFrameOrigin(NSPoint::new(x, ivars.top_y.get() - h));

        panel.makeKeyAndOrderFront(None);
        panel.makeFirstResponder(Some(field));
        // White caret to match the design.
        if let Some(editor) = panel.fieldEditor_forObject(true, Some(field)) {
            let text_view: Retained<NSTextView> = unsafe { Retained::cast(editor) };
            unsafe { text_view.setInsertionPointColor(Some(&NSColor::whiteColor())) };
        }
    }

    fn hide(&self) {
        let ivars = self.ivars();
        if ivars.hiding.replace(true) {
            return;
        }
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

    /// Recompute results for the current query. Runs the readdir scan fresh
    /// every time — that is the cache-free contract, and it is microseconds
    /// warm.
    fn refresh(&self) {
        let query = self.query();
        let running = running_apps();

        let mut entries: Vec<Entry> = Vec::new();
        if query.is_empty() {
            entries = running;
        } else {
            let mut scored: Vec<(i32, Entry)> = Vec::new();
            let installed = apps::scan_installed();
            let mut seen: Vec<String> = Vec::new();
            for mut entry in running {
                if let Some((s, positions)) = apps::match_positions(&query, &entry.name) {
                    seen.push(entry.name.to_lowercase());
                    entry.matched = positions;
                    scored.push((s + 15, entry)); // running apps rank first
                }
            }
            for app in installed {
                if seen.contains(&app.name.to_lowercase()) {
                    continue;
                }
                if let Some((s, positions)) = apps::match_positions(&query, &app.name) {
                    scored.push((
                        s,
                        Entry {
                            name: display_name(&app.name),
                            path: Some(app.path),
                            running: None,
                            matched: positions,
                            stats_line: None,
                        },
                    ));
                }
            }
            scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.name.cmp(&b.1.name)));
            entries.extend(scored.into_iter().map(|(_, e)| e));
        }
        entries.truncate(MAX_ROWS);

        // Stats for the visible running apps (a handful of syscalls plus one
        // window-list snapshot — microseconds, fresh every time). CPU% needs
        // two samples: rows show "…" until the second sample lands, then a
        // one-shot refreshTick fills the number in. Never blocks.
        let window_counts = stats::window_counts();
        let mut cpu_pending = false;
        {
            let mut samples = self.ivars().cpu_samples.borrow_mut();
            let now = std::time::Instant::now();
            for entry in entries.iter_mut() {
                if let Some(app) = &entry.running {
                    let pid = unsafe { app.processIdentifier() };
                    let mut parts: Vec<String> = Vec::new();
                    if let Some((ram, cpu_secs)) = stats::proc_stats(pid) {
                        parts.push(stats::fmt_ram(ram));
                        let pct = match samples.get(&pid) {
                            Some(prev) => {
                                let dt = now.duration_since(prev.at).as_secs_f64();
                                if dt >= CPU_MIN_INTERVAL {
                                    let p = ((cpu_secs - prev.cpu_secs).max(0.0) / dt
                                        * 100.0)
                                        .round();
                                    samples.insert(
                                        pid,
                                        CpuSample { cpu_secs, at: now, pct: Some(p) },
                                    );
                                    Some(p)
                                } else {
                                    prev.pct // too soon; reuse last good reading
                                }
                            }
                            None => {
                                samples.insert(
                                    pid,
                                    CpuSample { cpu_secs, at: now, pct: None },
                                );
                                None
                            }
                        };
                        match pct {
                            Some(p) => parts.push(format!("{p:.0}%")),
                            None => {
                                parts.push("…".to_string());
                                cpu_pending = true;
                            }
                        }
                    }
                    let windows = window_counts.get(&pid).copied().unwrap_or(0);
                    parts.push(format!("\u{1003DC} {windows}")); // SF Symbol: macwindow
                    entry.stats_line = Some(parts.join("   "));
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
        let (Some(panel), Some(field), Some(glyph), Some(rows_area), Some(hints)) = (
            ivars.panel.get(),
            ivars.field.get(),
            ivars.glyph.get(),
            ivars.rows_area.get(),
            ivars.hints.get(),
        ) else {
            return;
        };

        let entries = ivars.entries.borrow();
        let n = entries.len();
        let rows_h = if n > 0 {
            n as f64 * ROW_H + ROWS_PAD
        } else {
            0.0
        };
        let h = INPUT_H + rows_h + HINTS_H;

        let old = panel.frame();
        let top = ivars.top_y.get();
        panel.setFrame_display(
            NSRect::new(
                NSPoint::new(old.origin.x, top - h),
                NSSize::new(PANEL_W, h),
            ),
            true,
        );

        // Input band at the top.
        let glyph_size = glyph.frame().size;
        glyph.setFrameOrigin(NSPoint::new(
            22.0,
            h - INPUT_H + (INPUT_H - glyph_size.height) / 2.0,
        ));
        let field_x = 22.0 + glyph_size.width + 14.0;
        let field_h = field.frame().size.height.max(30.0);
        field.setFrame(NSRect::new(
            NSPoint::new(field_x, h - INPUT_H + (INPUT_H - field_h) / 2.0),
            NSSize::new(PANEL_W - field_x - 22.0, field_h),
        ));

        // Hints pinned to the bottom.
        hints.setFrame(NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(PANEL_W, HINTS_H),
        ));

        // Rows between them.
        rows_area.setFrame(NSRect::new(
            NSPoint::new(0.0, HINTS_H),
            NSSize::new(PANEL_W, rows_h),
        ));
        for view in rows_area.subviews().iter() {
            unsafe { view.removeFromSuperview() };
        }
        let selected = ivars.selected.get();
        for (i, entry) in entries.iter().enumerate() {
            let y = rows_h - ROWS_PAD / 2.0 - (i as f64 + 1.0) * ROW_H;
            self.build_row(mtm, rows_area, y, entry, i == selected);
        }
    }

    unsafe fn build_row(
        &self,
        mtm: MainThreadMarker,
        parent: &NSView,
        y: f64,
        entry: &Entry,
        selected: bool,
    ) {
        let row_w = PANEL_W - 20.0;
        let row = unsafe {
            NSView::initWithFrame(
                mtm.alloc(),
                NSRect::new(NSPoint::new(10.0, y), NSSize::new(row_w, ROW_H)),
            )
        };
        if selected {
            row.setWantsLayer(true);
            if let Some(layer) = row.layer() {
                layer.setCornerRadius(8.0);
                let (r, g, b) = SEL_RGB;
                set_layer_bg(&layer, unsafe {
                    &NSColor::colorWithSRGBRed_green_blue_alpha(r, g, b, SEL_ALPHA)
                });
            }
        }

        let name_font = unsafe { NSFont::systemFontOfSize(13.5) };
        let name_color = if selected { white(1.0) } else { white(0.68) };
        let name = make_label(mtm, &entry.name, &name_font, &name_color);
        // Matched characters render in the accent color.
        if !entry.matched.is_empty() {
            let base = if selected { white(0.85) } else { white(0.55) };
            let (hr, hg, hb) = HI_RGB;
            let hi = unsafe { NSColor::colorWithSRGBRed_green_blue_alpha(hr, hg, hb, 1.0) };
            let attr = attributed_name(&entry.name, &entry.matched, &name_font, &base, &hi);
            unsafe {
                name.setAttributedStringValue(&attr);
                name.sizeToFit();
            }
        }
        let name_h = name.frame().size.height;
        name.setFrameOrigin(NSPoint::new(13.0, (ROW_H - name_h) / 2.0));
        row.addSubview(&name);

        // Stats sit on the right of the same line.
        if let Some(line) = &entry.stats_line {
            let stats_font = unsafe { NSFont::systemFontOfSize(11.0) };
            let stats_color = if selected { white(0.60) } else { white(0.38) };
            let stats = make_label(mtm, line, &stats_font, &stats_color);
            let stats_size = stats.frame().size;
            stats.setFrameOrigin(NSPoint::new(
                row_w - 13.0 - stats_size.width,
                (ROW_H - stats_size.height) / 2.0,
            ));
            row.addSubview(&stats);
        }

        parent.addSubview(&row);
    }

    fn execute(&self) {
        let force_open = unsafe {
            NSApplication::sharedApplication(MainThreadMarker::new().unwrap())
                .currentEvent()
                .is_some_and(|e| {
                    e.modifierFlags()
                        .contains(objc2_app_kit::NSEventModifierFlags::NSEventModifierFlagCommand)
                })
        };
        let entry_data = {
            let entries = self.ivars().entries.borrow();
            let Some(entry) = entries.get(self.ivars().selected.get()) else {
                return;
            };
            (entry.path.clone(), entry.running.clone())
        };
        self.hide();

        let (path, running) = entry_data;
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
            stats_line: None,
        });
    }
    out
}

extern "C" fn hotkey_pressed(_next: *mut c_void, _event: *mut c_void, user: *mut c_void) -> i32 {
    // Carbon dispatches this on the main run loop.
    let delegate = unsafe { &*(user as *const Delegate) };
    delegate.toggle();
    0
}

fn main() {
    let mtm = MainThreadMarker::new().unwrap();
    let app = NSApplication::sharedApplication(mtm);
    app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);

    let delegate = Delegate::new(mtm);
    app.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));

    unsafe {
        hotkey::register(
            SUMMON_KEY,
            SUMMON_MODS,
            hotkey_pressed,
            Retained::as_ptr(&delegate) as *mut c_void,
        )
        .expect("failed to register global hotkey (is another instance running?)");
    }

    unsafe { app.run() };
}
