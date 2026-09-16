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
mod modes;
mod stats;

use config::{Action, Config, Mode, SearchEngine, SigilKind};

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
    NSTrackingArea, NSTrackingAreaOptions,
    NSEventModifierFlags, NSFocusRingType, NSFont, NSFontAttributeName,
    NSForegroundColorAttributeName, NSPanel, NSRunningApplication, NSScreen, NSTextField,
    NSTextFieldDelegate, NSTextView, NSVisualEffectBlendingMode, NSVisualEffectMaterial,
    NSVisualEffectState, NSVisualEffectView, NSView, NSWindowAnimationBehavior,
    NSWindowCollectionBehavior,
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
// Soft ranking bonus for already-running apps. Worth just over one column of
// first-hit position (8 per column in apps::match_positions), so a running app
// wins near-ties but a clearly earlier match on a cold app still outranks it.
const RUNNING_BONUS: i32 = 12;
// Score for a `[search_engines]` shortcut typed whole ("yt"). Deliberately
// out of reach of any fuzzy name match: naming the shortcut is unambiguous,
// so the engine takes the top row rather than tying with an app that happens
// to contain the same two letters.
const SHORTCUT_SCORE: i32 = 10_000;
// CPU sampling: minimum interval for a trustworthy percentage.
const CPU_MIN_INTERVAL: f64 = 0.25;
// How long after the row window moves to re-sample stats for the rows that
// scrolled in. Short enough to read as instant, long enough that trackpad
// momentum coalesces into one pass instead of one per event.
const SCROLL_SAMPLE_DELAY: f64 = 0.06;
// Glide timer period (~120 Hz) and its ease constant: the time the offset
// takes to close 1 - 1/e of the distance left. Small enough that a keypress
// still feels instant, large enough to read as motion rather than a jump.
const SCROLL_FRAME: f64 = 1.0 / 120.0;
const SCROLL_EASE_TAU: f64 = 0.045;
// Whole-tree CPU (Activity Monitor scale: 100 = one full core) at which a
// row's gauge turns red.
const CPU_ALERT_PCT: f64 = 70.0;

// State glyph column metrics (glyph strings themselves live in
// config::Icons — SF Symbols as text, zero I/O).
const GLYPH_COL_W: f64 = 24.0;
const GLYPH_PT: f64 = 16.0;
// Glass rim clip: the material extends this far past its clipping wrapper so
// the built-in edge highlight is cut off (see setup_impl).
const RIM_CLIP: f64 = 2.0;

/// Rows the panel synthesizes itself (no app bundle behind them).
#[derive(Clone, PartialEq)]
enum Builtin {
    /// "Setting: Change Theme (…)" — enter the theme picker.
    ThemePicker,
    /// A row in the picker; `None` is the built-in base ("Black Glass" =
    /// the un-overlaid `[style]`).
    ApplyTheme(Option<String>),
    /// A synthesized result (math, currency, a search engine); Enter
    /// copies or opens per action.
    ModeRow(modes::ModeAction),
}

/// What the panel is listing right now.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum PanelMode {
    #[default]
    Launcher,
    /// Theme rows; moving the selection restyles the live panel.
    ThemePicker,
    /// Subcommand rows for one app/shortcut (`[commands.<Name>]`), entered
    /// via the `ShowCommands` chord (default Tab). The app name and its
    /// commands live in `State::command_context`.
    AppCommands,
    /// A `[search_engines]` item has the panel: the field holds search
    /// terms, the badge holds the engine's glyph, and the rows are the
    /// engines those terms can go to. The active one lives in
    /// `State::search_engine`.
    SearchEngine,
}

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
    /// Panel-internal rows (theme picker and its entry point).
    builtin: Option<Builtin>,
    /// Dim right-aligned text (math mode: alternate results).
    detail: Option<String>,
    /// Inline pill after the name (engines: the shortcut).
    tag: Option<String>,
    /// Glyph for the icon column, overriding both `[icons.apps]` and the
    /// state glyph. A `[search_engines]` item carries its own here — the
    /// whole point of the section is that an item's icon travels with it.
    icon: Option<String>,
    /// `[search_engines]` item: enter or tab hands the panel over to this
    /// engine for the search terms, instead of activating anything.
    engine: Option<SearchEngine>,
    /// `PanelMode::AppCommands` built-in row (Open/Focus/Reveal/Info/
    /// Close/Kill): takes priority over `command`/`builtin` on activation,
    /// so Enter always does what the row says regardless of the global
    /// Open binding. `path`/`running` above are the *target* app's, not
    /// this row's own (there is no bundle behind a command row).
    app_action: Option<AppRowAction>,
}

/// Built-in `PanelMode::AppCommands` row actions. Not shell commands —
/// each is a direct native call against the context app's `path`/`running`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AppRowAction {
    /// Launch an installed, non-running app (same as the default Enter
    /// behavior on a cold app).
    Open,
    /// Activate a running app (same as the default Enter behavior on a
    /// warm app) — labeled "Focus" instead of "Open" since it already is one.
    Focus,
    /// Reveal the bundle in Finder.
    Reveal,
    /// Finder's "Get Info" window.
    Info,
    /// Graceful quit (`NSRunningApplication.terminate`).
    Close,
    /// Force quit (`NSRunningApplication.forceTerminate`).
    Kill,
}

/// The built-in rows for one app, in display order — running apps get
/// Focus/Close/Kill instead of Open, everything else (Reveal, Info) is
/// shared. Any `[commands.<Name>]` entries are appended after these.
fn builtin_app_commands(running: bool) -> &'static [(&'static str, AppRowAction)] {
    if running {
        &[
            ("Focus", AppRowAction::Focus),
            ("Reveal", AppRowAction::Reveal),
            ("Info", AppRowAction::Info),
            ("Close", AppRowAction::Close),
            ("Kill", AppRowAction::Kill),
        ]
    } else {
        &[
            ("Open", AppRowAction::Open),
            ("Reveal", AppRowAction::Reveal),
            ("Info", AppRowAction::Info),
        ]
    }
}

/// A live "cmd+_" row-jump hint, computed fresh from whatever's on screen
/// (see `compute_row_hints`) — never stored in config, since which rows
/// exist changes every keystroke. Digits are always unique per row; a
/// letter can be shared by several running apps and cycles between them
/// (see `Delegate::try_activate_hint`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum RowHint {
    Digit(u8),
    Letter(char),
}

/// `cmd+<letter>`-only chords already claimed by a configured `[keys]`
/// bind (default or user override) — `compute_row_hints` skips these so a
/// row-jump hint can never shadow e.g. cmd+r (Reveal) or cmd+a (Select
/// All), even if a visible app happens to start with R or A.
fn claimed_letters(binds: &[(config::Chord, Action)]) -> std::collections::HashSet<char> {
    binds
        .iter()
        .filter_map(|(c, _)| {
            let config::Key::Char(ch) = c.key else { return None };
            (c.cmd && !c.ctrl && !c.opt && !c.shift && ch.is_ascii_alphabetic())
                .then(|| ch.to_ascii_uppercase())
        })
        .collect()
}

/// One hint per row, aligned by index with the current entry list: running
/// apps get the first letter of their name (skipping a non-letter lead
/// character, e.g. "1Password" → 'P'), unless that letter is already
/// claimed by a configured bind or another *earlier* running row already
/// took it as a digit fallback — everything else fills the remaining
/// cmd+1..cmd+9 slots in row order. Two+ running rows sharing a letter all
/// get that same letter; there's no upper bound on how many can share one.
fn compute_row_hints(entries: &[Entry], binds: &[(config::Chord, Action)]) -> Vec<Option<RowHint>> {
    let claimed = claimed_letters(binds);
    let mut hints: Vec<Option<RowHint>> = vec![None; entries.len()];
    for (i, e) in entries.iter().enumerate() {
        // Only a plain running-app row — never an AppCommands built-in row
        // (those carry `app_action` and copy the context app's `running`
        // onto Focus/Close/Kill too, which would otherwise also grab a
        // letter meant for the app itself).
        if e.running.is_none() || e.app_action.is_some() {
            continue;
        }
        let Some(letter) =
            e.name.chars().find(|c| c.is_ascii_alphabetic()).map(|c| c.to_ascii_uppercase())
        else {
            continue;
        };
        if claimed.contains(&letter) {
            continue;
        }
        hints[i] = Some(RowHint::Letter(letter));
    }
    let mut digit = 1u8;
    for hint in hints.iter_mut() {
        if hint.is_some() {
            continue;
        }
        if digit > 9 {
            break;
        }
        *hint = Some(RowHint::Digit(digit));
        digit += 1;
    }
    hints
}

/// The drawn row window for one scroll position. The list scrolls in
/// *points*, not rows — `px` is how far it has slid up past the top slot —
/// so `first` and the last drawn row are usually cut off by the rows-area
/// clip, and `frac` is how far up the whole stack is nudged to show it.
#[derive(Clone, Copy, PartialEq, Debug)]
struct RowWindow {
    /// Scroll position in points, clamped to the list.
    px: f64,
    /// First row to draw — partly above the top edge when `frac` > 0.
    first: usize,
    /// Rows to draw: `win`, plus the one peeking in at the bottom.
    drawn: usize,
    /// Rows that fit whole. The panel is this tall and stays this tall
    /// while scrolling.
    win: usize,
    /// How far the drawn rows are slid up, in `0.0..ROW_H`.
    frac: f64,
}

impl RowWindow {
    /// The rows entirely on screen — what the selection is allowed to be.
    /// Empty only in the degenerate `max_rows = 1` mid-scroll case, where
    /// the caller falls back to the row covering most of the window.
    fn full_rows(&self) -> std::ops::Range<usize> {
        if self.frac > 0.0 {
            (self.first + 1)..(self.first + self.win)
        } else {
            self.first..(self.first + self.win)
        }
    }
}

/// Resolve a scroll position against a list of `total` rows: clamp it to
/// what there is to scroll, then say which rows that draws.
fn row_window(total: usize, max_rows: usize, scroll_px: f64) -> RowWindow {
    if total == 0 {
        return RowWindow { px: 0.0, first: 0, drawn: 0, win: 0, frac: 0.0 };
    }
    let win = total.min(max_rows.max(1));
    let max_px = (total - win) as f64 * ROW_H;
    let px = scroll_px.clamp(0.0, max_px);
    let first = (px / ROW_H).floor() as usize;
    let frac = px - first as f64 * ROW_H;
    // The partial row at the bottom is only there when the stack is nudged
    // up, and only if the list actually has another row to show.
    let drawn = (win + usize::from(frac > 0.0)).min(total - first);
    RowWindow { px, first, drawn, win, frac }
}

/// The scroll position that puts `selected` fully on screen, moving as
/// little as possible from `scroll_px`. Row-aligned: the keyboard drives
/// this, and arrowing out of a half-scrolled list should tidy it up rather
/// than carry the offset along forever.
fn scroll_to_show(total: usize, max_rows: usize, selected: usize, scroll_px: f64) -> f64 {
    if total == 0 {
        return 0.0;
    }
    let win = total.min(max_rows.max(1));
    let max_px = (total - win) as f64 * ROW_H;
    let selected = selected.min(total - 1);
    let top = selected as f64 * ROW_H;
    let mut px = (scroll_px / ROW_H).round() * ROW_H;
    if top < px {
        px = top;
    } else if top + ROW_H > px + win as f64 * ROW_H {
        px = top + ROW_H - win as f64 * ROW_H;
    }
    px.clamp(0.0, max_px)
}

/// What `PanelMode::AppCommands` is listing: the context app's identity
/// (for `path`/`running` on built-in rows) plus its full row list — built-ins
/// first, then any `[commands.<Name>]` shell extras.
#[derive(Clone)]
struct AppCommandContext {
    path: Option<PathBuf>,
    running: Option<Retained<NSRunningApplication>>,
    commands: Vec<(String, AppCommand)>,
}

#[derive(Clone)]
enum AppCommand {
    Builtin(AppRowAction),
    Shell(String),
}

struct RowStats {
    /// Whole-tree CPU on the Activity Monitor scale (100 = one full core).
    cpu_pct: f64,
}

/// How a `[search_engines]` item ranks against a launcher query, and which
/// of its characters to highlight. Two ways to hit: the name, matched
/// fuzzily like anything else in the index, or the shortcut typed whole —
/// "yt" has to find YouTube even though those letters don't spell it. A
/// shortcut hit takes `SHORTCUT_SCORE` rather than the name's score, so
/// naming an engine outright beats apps that merely contain the letters;
/// nothing is highlighted in that case, because the match isn't in the name.
fn engine_match(query: &str, engine: &SearchEngine) -> Option<(i32, Vec<usize>)> {
    if !engine.shortcut.is_empty() && engine.shortcut.eq_ignore_ascii_case(query.trim()) {
        return Some((SHORTCUT_SCORE, Vec::new()));
    }
    apps::match_positions(query, &engine.name)
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
    /// aside — those are registered once at launch). `config.style` is the
    /// ACTIVE style — base plus the current theme overlay, if any.
    config: RefCell<Config>,
    /// The un-themed `[style]` from the config file; themes overlay this.
    base_style: RefCell<config::Style>,
    /// Active theme name. Seeded by `theme =` in `[style]`, updated by the
    /// picker. In-memory only — the picker never writes the config file, so
    /// a picked theme lasts until restart (or refresh-config removes it).
    session_theme: RefCell<Option<String>>,
    mode: Cell<PanelMode>,
    /// Active sigil mode (`=`, `!`, …), or `None` for the app launcher. The
    /// sigil is lifted out of the text field into the input badge, so the
    /// field only ever holds the query terms.
    sigil: Cell<Option<char>>,
    /// Sigil char for an autodetected mode (`auto_kind`), recomputed every
    /// refresh. Drives the same input badge as `sigil`, but transiently —
    /// the char stays in the field and backspace needs no special case.
    auto_sigil: Cell<Option<char>>,
    /// Style to restore when the picker is dismissed without committing.
    saved_style: RefCell<Option<config::Style>>,
    panel: OnceCell<Retained<Panel>>,
    field: OnceCell<Retained<NSTextField>>,
    glyph: OnceCell<Retained<NSTextField>>,
    /// The colored box shown in place of `glyph` while a sigil is active,
    /// and the label holding the sigil character inside it.
    sigil_box: OnceCell<Retained<NSView>>,
    sigil_label: OnceCell<Retained<NSTextField>>,
    rows_area: OnceCell<Retained<NSView>>,
    /// Chrome handles for live restyling: the view whose layer carries the
    /// panel's corner radius + border (glass wrapper or vibrancy effect),
    /// the glass view (radius follows the wrapper), and the wash layer that
    /// carries panel_background/panel_opacity on both material paths.
    chrome_view: OnceCell<Retained<NSView>>,
    glass_view: OnceCell<Retained<NSView>>,
    tint_view: OnceCell<Retained<NSView>>,
    /// Holds every content view; inset from the window edge by the outer
    /// border's width so that ring has somewhere to draw.
    container_view: OnceCell<Retained<NSView>>,
    /// Fills the whole window and strokes `outer_border` in the margin
    /// around the panel body — outside the material, unlike `chrome_view`'s
    /// inner `border`.
    outer_view: OnceCell<Retained<NSView>>,
    entries: RefCell<Vec<Entry>>,
    /// Set while `mode` is `AppCommands`: the context app and its row list,
    /// so `refresh` can list them and backspace-to-exit knows there's
    /// something to leave.
    command_context: RefCell<Option<AppCommandContext>>,
    /// Set while `mode` is `SearchEngine`: the engine that took the panel
    /// over. Its glyph drives the input badge and its row leads the list,
    /// the way `sigil` does for a sigil mode.
    search_engine: RefCell<Option<SearchEngine>>,
    /// Live Cmd-key state, tracked by a `flagsChanged` monitor — while true,
    /// rows show their row-jump hint (see `build_hint_badge`).
    cmd_held: Cell<bool>,
    /// This relayout's hints, aligned by index with `entries` — computed by
    /// `compute_row_hints` and read back by both rendering and
    /// `try_activate_hint`, so the two never disagree about what's live.
    row_hints: RefCell<Vec<Option<RowHint>>>,
    /// Per-letter round-robin cursor for rows that share a hint letter
    /// (`compute_row_hints`) — persists for the process lifetime, not just
    /// one summon, so repeated cmd+<letter> presses actually advance.
    letter_cycle: RefCell<std::collections::HashMap<char, usize>>,
    selected: Cell<usize>,
    /// Where the drawn window sits, in points down the list. The entry
    /// list is never truncated — `[style] max_rows` is how many rows fit on
    /// screen at once, and this is how far past them the list has slid
    /// (see `row_window`). Points, not rows, so a trackpad tracks the
    /// finger instead of stepping.
    scroll_px: Cell<f64>,
    /// Where `scroll_px` is heading when something moved the window in row
    /// steps (a key, a wheel notch) and `[animation] scroll` is on. Equal to
    /// `scroll_px` when nothing is animating.
    scroll_target: Cell<f64>,
    /// What the last relayout actually put on screen: first row, how many,
    /// and which one was highlighted. Sliding the rows without rebuilding
    /// them is only valid while all three still hold, so this is what the
    /// fast path checks — not the scroll position it came from.
    drawn_first: Cell<usize>,
    drawn_count: Cell<usize>,
    drawn_selected: Cell<usize>,
    /// Whether the running glide should pull the selection along with the
    /// window (a wheel notch) or leave it be (a keypress, where the
    /// selection is what the window is chasing).
    glide_drags_selection: Cell<bool>,
    /// The glide timer, alive only while `scroll_px != scroll_target`.
    scroll_timer: RefCell<Option<Retained<objc2::runtime::AnyObject>>>,
    /// Timestamp of the last glide step, for a frame-rate-independent ease.
    scroll_at: Cell<Option<std::time::Instant>>,
    /// Screen position the cursor was at when a row last claimed the
    /// selection by hover. Rows sliding under a still cursor fire
    /// `mouseEntered` exactly like a real hover does; comparing against
    /// this is what tells the two apart, so scrolling never yanks the
    /// selection out from under the keyboard.
    hover_at: Cell<Option<(f64, f64)>>,
    top_y: Cell<f64>,
    hiding: Cell<bool>,
    cpu_samples: RefCell<std::collections::HashMap<i32, CpuSample>>,
    /// Repeating stats-refresh timer, alive only while the panel is visible.
    stats_timer: RefCell<Option<Retained<objc2::runtime::AnyObject>>>,
    /// App-directory scan, cached for the lifetime of one open panel. Filled on
    /// the first keystroke after a summon and cleared on hide, so a freshly
    /// installed app still appears next summon without re-scanning per keystroke.
    installed_cache: RefCell<Option<Vec<apps::InstalledApp>>>,
    /// Running-app order, pinned for the lifetime of one open panel. The
    /// switcher list is `NSWorkspace`'s array, and that is explicitly
    /// unordered — it reshuffles as apps activate. Refresh used to run on a
    /// keystroke and the one-second tick, so the churn was rare; now that
    /// scrolling refreshes too, an unpinned order visibly reorders the list
    /// under the pointer. Cleared on hide, so a newly launched app still
    /// lands wherever the system puts it next summon.
    running_order: RefCell<Option<Vec<i32>>>,
    /// Exchange rates read from the disk cache, loaded once per summon and
    /// cleared on hide (so a completed background refresh is picked up next
    /// summon). `Some` with an empty map means "loaded, but no cache yet".
    rates_cache: RefCell<Option<Rates>>,
}

/// Exchange rates from the on-disk cache: code → units-per-USD, plus how
/// old the cache file is.
#[derive(Default)]
struct Rates {
    map: std::collections::HashMap<String, f64>,
    age_secs: Option<u64>,
}

impl Rates {
    /// Human freshness for the first conversion row ("just now", "2h ago").
    fn age_display(&self) -> String {
        match self.age_secs {
            None => "live".to_string(),
            Some(s) if s < 60 => "just now".to_string(),
            Some(s) if s < 3600 => format!("{}m ago", s / 60),
            Some(s) if s < 86_400 => format!("{}h ago", s / 3600),
            Some(s) => format!("{}d ago", s / 86_400),
        }
    }
}

/// Coinbase's keyless endpoint: USD → every fiat and crypto rate in one
/// document, so both `500,000 php` and `1.4btc` resolve from one fetch.
const RATE_URL: &str = "https://api.coinbase.com/v2/exchange-rates?currency=USD";
/// Refresh the cache in the background once it is older than this.
const RATE_TTL_SECS: u64 = 3600;
/// One background fetch at a time, across the whole process.
static RATE_FETCHING: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

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

impl RowView {
    fn delegate(&self) -> Option<&Delegate> {
        let ptr = self.ivars().delegate.get();
        (ptr != 0).then(|| unsafe { &*(ptr as *const Delegate) })
    }
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
            if let Some(delegate) = self.delegate() {
                delegate.select_row(self.ivars().index.get());
            }
        }

        /// Hovering a row selects it, so the pointer and the keyboard share
        /// one highlight. Guarded on the cursor having actually moved: rows
        /// scrolling under a still cursor fire this too, and letting that
        /// through would have the list fight the arrow keys.
        #[method(mouseEntered:)]
        fn mouse_entered(&self, _event: &NSEvent) {
            if let Some(delegate) = self.delegate() {
                delegate.hover_row(self.ivars().index.get());
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
            self.maybe_enter_sigil();
            self.ivars().selected.set(0);
            self.ivars().scroll_px.set(0.0);
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
                self.dismiss();
                true
            } else if command == sel!(deleteBackward:) {
                // Backspace on an empty field leaves the sigil mode, the
                // search engine, or the app-commands picker instead of doing
                // nothing — the badge "deletes" back to the launcher.
                if !self.query().is_empty() {
                    false
                } else if self.ivars().sigil.get().is_some() {
                    self.ivars().sigil.set(None);
                    self.ivars().selected.set(0);
                    self.ivars().scroll_px.set(0.0);
                    self.refresh();
                    true
                } else if self.ivars().mode.get() == PanelMode::SearchEngine {
                    self.exit_search_engine();
                    true
                } else if self.ivars().mode.get() == PanelMode::AppCommands {
                    self.exit_app_commands();
                    true
                } else {
                    false
                }
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

        /// One frame of the scroll glide (see `glide_scroll_to`).
        #[method(scrollTick)]
        fn scroll_tick(&self) {
            if self.ivars().panel.get().is_some_and(|p| p.isVisible()) {
                self.scroll_glide_step();
            } else {
                self.stop_scroll_glide();
            }
        }
    }
);

/// Configured `(r, g, b)` at the given alpha.
/// The margin the window carries outside the panel body, so the outer border
/// has somewhere to draw. Zero width = no ring and no margin, and the window
/// is exactly the panel again.
fn outer_inset(style: &config::Style) -> f64 {
    style.outer_border_width
}

/// Paint the outer ring into `view`'s layer: a stroke inside a layer that is
/// `o` larger than the body on every side, so it lands entirely in the margin
/// with its inner edge flush to the panel's rounded corner.
fn apply_outer_border(view: &NSView, style: &config::Style) {
    let Some(layer) = (unsafe { view.layer() }) else {
        return;
    };
    let o = outer_inset(style);
    layer.setCornerRadius(style.panel_corner_radius + o);
    let color = rgba(style.outer_border, style.outer_border_opacity);
    unsafe {
        let curve = NSString::from_str("continuous");
        let _: () = msg_send![&*layer, setCornerCurve: &*curve];
        let cg: *mut c_void = msg_send![&*color, CGColor];
        let _: () = msg_send![&*layer, setBorderColor: cg];
        let _: () = msg_send![&*layer, setBorderWidth: o];
    }
}

/// An NSPanel left at `Default` animation behavior gets the window server's
/// utility-window fade on every `orderFront`/`orderOut` — nothing in this
/// process animates, so `None` is the only way to get an instant panel.
fn fade_behavior(fade: bool) -> NSWindowAnimationBehavior {
    if fade {
        NSWindowAnimationBehavior::Default
    } else {
        NSWindowAnimationBehavior::None
    }
}

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
    fn new(mtm: MainThreadMarker, mut config: Config) -> Retained<Self> {
        // `theme = "name"` in [style] seeds the session theme; the active
        // style becomes base + overlay, the base is kept for the picker.
        let base = config.style.clone();
        let mut session = config.theme.clone();
        if let Some(name) = &session {
            match config.themes.iter().find(|t| &t.name == name) {
                Some(theme) => {
                    let mut style = base.clone();
                    config::apply_theme(&mut style, theme);
                    config.style = style;
                }
                None => {
                    eprintln!("motherfucker: config: unknown theme `{name}`");
                    session = None;
                }
            }
        }
        let this = mtm.alloc::<Self>().set_ivars(State {
            config: RefCell::new(config),
            base_style: RefCell::new(base),
            session_theme: RefCell::new(session),
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
        // The window is the panel body plus the outer border's margin on
        // every side; `body` is the visible panel within it. With no outer
        // border the margin is 0 and the two are the same rect.
        let o = outer_inset(&cfg.style);
        let rect = NSRect::new(
            NSPoint::new(0.0, 0.0),
            NSSize::new(cfg.style.width + 2.0 * o, 200.0 + 2.0 * o),
        );
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
        unsafe { panel.setAnimationBehavior(fade_behavior(cfg.fade)) };
        unsafe {
            panel.setAppearance(NSAppearance::appearanceNamed(NSAppearanceNameVibrantDark).as_deref());
        }
        panel.setDelegate(Some(ProtocolObject::from_ref(self)));

        let content = panel.contentView().unwrap();
        let bounds = content.bounds();
        let resize_mask = NSAutoresizingMaskOptions::NSViewWidthSizable
            | NSAutoresizingMaskOptions::NSViewHeightSizable;
        // The panel body, inset from the window edge by the margin. Chrome
        // and content both live here; `body_bounds` is the same rect in the
        // body's own coordinates, for its subviews.
        let body = NSRect::new(
            NSPoint::new(o, o),
            NSSize::new(bounds.size.width - 2.0 * o, bounds.size.height - 2.0 * o),
        );
        let body_bounds = NSRect::new(NSPoint::new(0.0, 0.0), body.size);

        // Outer border: behind everything, filling the window, stroking the
        // margin. Added first so the material and content draw over it.
        let outer = unsafe { NSView::initWithFrame(mtm.alloc(), bounds) };
        outer.setAutoresizingMask(resize_mask);
        outer.setWantsLayer(true);
        apply_outer_border(&outer, &cfg.style);
        content.addSubview(&outer);

        // All content lives in one container; the material view hosts it.
        let container = unsafe { NSView::initWithFrame(mtm.alloc(), body) };
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

        let chrome_ref: Retained<NSView>;
        let mut tint_ref: Option<Retained<NSView>> = None;
        let mut glass_ref: Option<Retained<NSView>> = None;
        if let Some(glass) = &glass {
            // Rim removal: the glass sits inside a clipping wrapper and
            // extends RIM_CLIP px beyond it on every side, so the material's
            // built-in edge highlight falls outside the visible shape and is
            // cut off entirely.
            let wrapper = unsafe { NSView::initWithFrame(mtm.alloc(), body) };
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
                    body.size.width + 2.0 * RIM_CLIP,
                    body.size.height + 2.0 * RIM_CLIP,
                ),
            ));
            glass.setAutoresizingMask(resize_mask);
            unsafe {
                let radius = cfg.style.panel_corner_radius + RIM_CLIP;
                let _: () = msg_send![&**glass, setCornerRadius: radius];
                // The tint IS the wash: NSGlassEffectView renders an opaque
                // frost when untinted and shows the desktop through only once
                // tinted. panel_background at panel_opacity — the color's
                // alpha is honored, so panel_opacity drives translucency.
                let tint_color = rgba(cfg.style.panel_background, cfg.style.panel_opacity);
                let _: () = msg_send![&**glass, setTintColor: &*tint_color];
            }
            wrapper.addSubview(glass);
            content.addSubview(&wrapper);
            content.addSubview(&container);
            chrome_ref = wrapper;
            glass_ref = Some(glass.clone());
        } else {
            let effect = unsafe { NSVisualEffectView::initWithFrame(mtm.alloc(), body) };
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

            let tint = unsafe { NSView::initWithFrame(mtm.alloc(), body_bounds) };
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
            chrome_ref = unsafe { Retained::cast(effect) };
            tint_ref = Some(tint);
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

        // Sigil badge: a colored rounded box with the sigil char centered.
        // Hidden until a mode is active; positioned and styled in relayout.
        let zero = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(0.0, 0.0));
        let sigil_box = unsafe { NSView::initWithFrame(mtm.alloc(), zero) };
        sigil_box.setWantsLayer(true);
        unsafe {
            let _: () = msg_send![&*sigil_box, setHidden: true];
        }
        let sigil_label = make_label(mtm, "", &glyph_font, &rgba(cfg.style.panel_background, 1.0));
        sigil_box.addSubview(&sigil_label);
        container.addSubview(&sigil_box);

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

        // Results container. Layer-backed and clipping, because a row
        // scrolling in or out hangs off the top and bottom edges and must
        // not draw over the input band or past the panel's own padding.
        let rows_area = unsafe { NSView::initWithFrame(mtm.alloc(), body_bounds) };
        rows_area.setWantsLayer(true);
        if let Some(layer) = rows_area.layer() {
            layer.setMasksToBounds(true);
        }
        container.addSubview(&rows_area);

        let ivars = self.ivars();
        ivars.panel.set(panel).ok();
        ivars.field.set(field).ok();
        ivars.glyph.set(glyph).ok();
        ivars.sigil_box.set(sigil_box).ok();
        ivars.sigil_label.set(sigil_label).ok();
        ivars.rows_area.set(rows_area).ok();
        ivars.chrome_view.set(chrome_ref).ok();
        ivars.container_view.set(container.clone()).ok();
        ivars.outer_view.set(outer).ok();
        if let Some(v) = tint_ref {
            ivars.tint_view.set(v).ok();
        }
        if let Some(v) = glass_ref {
            ivars.glass_view.set(v).ok();
        }
        drop(cfg);
        // Applies the panel border (and any startup theme's chrome) that the
        // construction above doesn't know about.
        self.apply_live_style();

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

        // Second monitor, Cmd only: toggles the "⌘1".."⌘9" row-jump hints
        // (see `cmd_held`/`build_hint_badge`) live as the key goes up and
        // down — a plain keyDown/keyUp pair doesn't fire for modifier-only
        // presses, flagsChanged is the only event that does.
        let flags_ptr = self as *const Delegate as usize;
        let flags_block = block2::RcBlock::new(
            move |event: std::ptr::NonNull<NSEvent>| -> *mut NSEvent {
                let delegate = unsafe { &*(flags_ptr as *const Delegate) };
                let held = unsafe { event.as_ref().modifierFlags() }
                    .contains(NSEventModifierFlags::NSEventModifierFlagCommand);
                let changed = delegate.ivars().cmd_held.replace(held) != held;
                let visible = delegate.ivars().panel.get().is_some_and(|p| p.isVisible());
                if changed && visible {
                    delegate.relayout();
                }
                event.as_ptr()
            },
        );
        let flags_monitor = unsafe {
            NSEvent::addLocalMonitorForEventsMatchingMask_handler(
                NSEventMask::FlagsChanged,
                &flags_block,
            )
        };
        std::mem::forget(flags_block);
        std::mem::forget(flags_monitor);

        // Third monitor: the wheel/trackpad. The rows are plain views in a
        // fixed-height panel, not a scroll view — there is nothing for
        // AppKit to scroll on its own, so the window moves here.
        let scroll_ptr = self as *const Delegate as usize;
        let scroll_block = block2::RcBlock::new(
            move |event: std::ptr::NonNull<NSEvent>| -> *mut NSEvent {
                let delegate = unsafe { &*(scroll_ptr as *const Delegate) };
                if delegate.handle_scroll_event(unsafe { event.as_ref() }) {
                    std::ptr::null_mut()
                } else {
                    event.as_ptr()
                }
            },
        );
        let scroll_monitor = unsafe {
            NSEvent::addLocalMonitorForEventsMatchingMask_handler(
                NSEventMask::ScrollWheel,
                &scroll_block,
            )
        };
        std::mem::forget(scroll_block);
        std::mem::forget(scroll_monitor);
    }

    /// Wheel/trackpad scrolling over the panel. A trackpad reports precise
    /// deltas in points and the list follows them one for one — that is all
    /// "smooth scrolling" is, the offset tracking the fingers, momentum
    /// phase included, with no stepping in between. A mouse wheel has no
    /// finger to track: it reports whole lines, so a notch is a row and the
    /// glide covers the distance. Swallowed whenever the panel is up —
    /// nothing else in this process wants the event.
    fn handle_scroll_event(&self, event: &NSEvent) -> bool {
        let ivars = self.ivars();
        if !ivars.panel.get().is_some_and(|p| p.isVisible()) {
            return false;
        }
        let delta = unsafe { event.scrollingDeltaY() };
        if delta == 0.0 {
            return true;
        }
        // Positive deltaY means the content moves down, i.e. toward the top
        // of the list — the same sense NSScrollView gives it, so the
        // natural-scrolling preference is already baked into the sign.
        if unsafe { event.hasPreciseScrollingDeltas() } {
            self.stop_scroll_glide();
            self.scroll_with_selection(ivars.scroll_px.get() - delta);
        } else {
            let from = if ivars.scroll_timer.borrow().is_some() {
                ivars.scroll_target.get()
            } else {
                ivars.scroll_px.get()
            };
            self.glide_scroll_to(from - delta * ROW_H, true);
        }
        true
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
        if let Some(a) = action {
            self.perform(a);
            return true;
        }
        // No configured bind claimed this chord — try a live row-jump hint
        // (cmd+1.."cmd+9"/cmd+<letter>, see `compute_row_hints`). Configured
        // binds always win first, so cmd+r/cmd+a above can never be shadowed
        // by a row that happens to start with R or A.
        if cmd && !ctrl && !opt && !shift {
            if let Some(ch) = chars.chars().next() {
                return self.try_activate_hint(ch);
            }
        }
        false
    }

    /// Activate whichever row currently shows the hint for `ch` (a digit or
    /// a letter, already lowercased by the caller). Digits are always
    /// unique per row; a letter shared by several running apps (see
    /// `compute_row_hints`) cycles to the next one on each press, tracked
    /// in `letter_cycle` for as long as the process runs.
    fn try_activate_hint(&self, ch: char) -> bool {
        let ivars = self.ivars();
        let hints = ivars.row_hints.borrow().clone();
        let target = if let Some(d) = ch.to_digit(10) {
            (d != 0).then(|| RowHint::Digit(d as u8))
        } else if ch.is_ascii_alphabetic() {
            Some(RowHint::Letter(ch.to_ascii_uppercase()))
        } else {
            None
        };
        let Some(target) = target else {
            return false;
        };
        let matches: Vec<usize> = hints
            .iter()
            .enumerate()
            .filter(|(_, h)| **h == Some(target))
            .map(|(i, _)| i)
            .collect();
        if matches.is_empty() {
            return false;
        }
        let index = if let RowHint::Letter(c) = target {
            let mut cycle = ivars.letter_cycle.borrow_mut();
            let cursor = cycle.entry(c).or_insert(0);
            let index = matches[*cursor % matches.len()];
            *cursor = (*cursor + 1) % matches.len();
            index
        } else {
            matches[0]
        };
        ivars.selected.set(index);
        self.execute(false);
        true
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
                    ivars.scroll_px.set(0.0);
                    ivars.sigil.set(None);
                    ivars.mode.set(PanelMode::Launcher);
                    ivars.search_engine.borrow_mut().take();
                    self.refresh();
                }
            }
            Action::Dismiss => self.dismiss(),
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
            Action::ShowCommands => self.enter_app_commands(),
        }
    }

    /// Enter `PanelMode::AppCommands` for the currently selected row. Any
    /// real app — installed or running — is always tab-able: it gets
    /// Open/Reveal/Info (cold) or Focus/Reveal/Info/Close/Kill (running),
    /// plus any `[commands.<Name>]` extras appended by name (case-
    /// insensitive). A `[shortcuts]` row has no bundle to act on, so it's
    /// tab-able only when it has `[commands.<Name>]` extras of its own.
    /// Mode rows and panel built-ins (theme picker, …) never are. A
    /// `[search_engines]` row is its own thing entirely: tab is one of the
    /// two keys that hand the panel over to it, so it never reaches the
    /// subcommand path.
    fn enter_app_commands(&self) {
        let ivars = self.ivars();
        if ivars.mode.get() != PanelMode::Launcher {
            return;
        }
        if let Some(engine) = self.selected_engine() {
            self.enter_search_engine(engine);
            return;
        }
        let selected = {
            let entries = ivars.entries.borrow();
            entries.get(ivars.selected.get()).and_then(|e| {
                (e.builtin.is_none()).then(|| {
                    (e.name.clone(), e.path.clone(), e.running.clone(), e.command.is_some())
                })
            })
        };
        let Some((name, path, running, is_shortcut)) = selected else {
            return;
        };
        let name_lower = name.to_lowercase();
        let extras = ivars
            .config
            .borrow()
            .app_commands
            .iter()
            .find(|(n, _)| *n == name_lower)
            .map(|(_, cmds)| cmds.clone())
            .unwrap_or_default();

        let mut commands: Vec<(String, AppCommand)> = Vec::new();
        if !is_shortcut {
            for (label, action) in builtin_app_commands(running.is_some()) {
                commands.push((label.to_string(), AppCommand::Builtin(*action)));
            }
        }
        commands.extend(extras.into_iter().map(|(label, cmd)| (label, AppCommand::Shell(cmd))));
        if commands.is_empty() {
            return;
        }

        *ivars.command_context.borrow_mut() = Some(AppCommandContext { path, running, commands });
        ivars.mode.set(PanelMode::AppCommands);
        ivars.selected.set(0);
        ivars.scroll_px.set(0.0);
        self.set_field_text("");
        self.refresh();
    }

    /// Back out one level: Escape in the app-commands submenu returns to the
    /// launcher list rather than closing the panel. Everywhere else it still
    /// dismisses.
    fn dismiss(&self) {
        match self.ivars().mode.get() {
            PanelMode::AppCommands => self.exit_app_commands(),
            PanelMode::SearchEngine => self.exit_search_engine(),
            _ => self.hide(),
        }
    }

    /// Leave `PanelMode::AppCommands` back to the plain launcher, discarding
    /// the subcommand context (mirrors how sigil mode backs out on an empty
    /// field's backspace).
    fn exit_app_commands(&self) {
        let ivars = self.ivars();
        ivars.mode.set(PanelMode::Launcher);
        *ivars.command_context.borrow_mut() = None;
        ivars.selected.set(0);
        ivars.scroll_px.set(0.0);
        self.refresh();
    }

    /// The `[search_engines]` item under the selection, if that's what it is.
    fn selected_engine(&self) -> Option<SearchEngine> {
        let ivars = self.ivars();
        let entries = ivars.entries.borrow();
        entries.get(ivars.selected.get()).and_then(|e| e.engine.clone())
    }

    /// Hand the panel over to a search engine: the field is cleared for the
    /// terms and the engine's glyph takes the input badge, so the panel
    /// reads as "typing into Google" rather than "searching for Google".
    /// Both enter and tab land here — an engine is one item you step into,
    /// not a thing with separate activate and expand behaviors.
    fn enter_search_engine(&self, engine: SearchEngine) {
        let ivars = self.ivars();
        *ivars.search_engine.borrow_mut() = Some(engine);
        ivars.mode.set(PanelMode::SearchEngine);
        ivars.sigil.set(None);
        ivars.auto_sigil.set(None);
        ivars.selected.set(0);
        ivars.scroll_px.set(0.0);
        self.set_field_text("");
        self.refresh();
    }

    /// Back out to the launcher, dropping the engine (mirrors how a sigil
    /// mode leaves on an empty field's backspace).
    fn exit_search_engine(&self) {
        let ivars = self.ivars();
        ivars.mode.set(PanelMode::Launcher);
        *ivars.search_engine.borrow_mut() = None;
        ivars.selected.set(0);
        ivars.scroll_px.set(0.0);
        self.set_field_text("");
        self.refresh();
    }

    /// Re-read the config file (and themes dir) and re-apply it live. Global
    /// hotkeys are registered once at launch and still need a restart;
    /// everything else — layout, colors, chrome, fonts, icons, shortcuts,
    /// in-panel binds — updates immediately. An interactively picked theme
    /// survives the reload as long as its file still exists.
    fn reload_config(&self) {
        let ivars = self.ivars();
        // A reload while the picker is up abandons the preview, and one
        // while an engine holds the panel drops back to the launcher — the
        // engine it was pointing at may not survive the reload.
        ivars.mode.set(PanelMode::Launcher);
        *ivars.saved_style.borrow_mut() = None;
        *ivars.search_engine.borrow_mut() = None;

        let fresh = config::load();
        *ivars.base_style.borrow_mut() = fresh.style.clone();
        {
            let mut session = ivars.session_theme.borrow_mut();
            if let Some(name) = session.clone() {
                if !fresh.themes.iter().any(|t| t.name == name) {
                    eprintln!("motherfucker: config: theme `{name}` is gone; back to base");
                    *session = None;
                }
            } else {
                *session = fresh.theme.clone();
            }
        }
        *ivars.config.borrow_mut() = fresh;
        self.apply_session_theme();

        if let Some(glyph) = ivars.glyph.get() {
            unsafe {
                glyph.setStringValue(&NSString::from_str(
                    &ivars.config.borrow().icons.search,
                ));
                glyph.sizeToFit();
            }
        }
        self.apply_live_style();
        // Motion, not style: set straight from the reloaded config, never
        // from the theme-overlaid style.
        if let Some(panel) = ivars.panel.get() {
            unsafe { panel.setAnimationBehavior(fade_behavior(ivars.config.borrow().fade)) };
        }
        // Rebuild the results with the fresh style/layout.
        self.refresh();
    }

    /// Set the active style to base + the session theme's overlay (if any).
    fn apply_session_theme(&self) {
        let ivars = self.ivars();
        let overlay = {
            let cfg = ivars.config.borrow();
            ivars
                .session_theme
                .borrow()
                .as_ref()
                .and_then(|name| cfg.themes.iter().find(|t| &t.name == name).cloned())
        };
        let mut style = ivars.base_style.borrow().clone();
        if let Some(theme) = &overlay {
            config::apply_theme(&mut style, theme);
        }
        ivars.config.borrow_mut().style = style;
    }

    /// Push the active style onto everything that isn't rebuilt per-refresh:
    /// the input field, the search glyph, and the panel chrome (tint, corner
    /// radius, border). Rows pick the style up on the next relayout.
    fn apply_live_style(&self) {
        let ivars = self.ivars();
        let cfg = ivars.config.borrow();
        let style = &cfg.style;
        if let Some(field) = ivars.field.get() {
            unsafe {
                field.setFont(Some(&resolve_font(
                    &style.font_family,
                    0.0,
                    style.input_font_size,
                )));
                field.setTextColor(Some(&rgba(style.panel_foreground, 1.0)));
            }
        }
        if let Some(glyph) = ivars.glyph.get() {
            let color = match style.icon_foreground {
                Some(c) => rgba(c, 1.0),
                None => rgba(style.panel_foreground, 0.85),
            };
            unsafe { glyph.setTextColor(Some(&color)) };
        }
        if let Some(chrome) = ivars.chrome_view.get() {
            if let Some(layer) = unsafe { chrome.layer() } {
                layer.setCornerRadius(style.panel_corner_radius);
                let border = rgba(style.border, 1.0);
                unsafe {
                    let cg: *mut c_void = msg_send![&*border, CGColor];
                    let _: () = msg_send![&*layer, setBorderColor: cg];
                    let _: () = msg_send![&*layer, setBorderWidth: style.border_width];
                }
            }
        }
        if let Some(glass) = ivars.glass_view.get() {
            unsafe {
                let radius = style.panel_corner_radius + RIM_CLIP;
                let _: () = msg_send![&**glass, setCornerRadius: radius];
                let tint_color = rgba(style.panel_background, style.panel_opacity);
                let _: () = msg_send![&**glass, setTintColor: &*tint_color];
            }
        }
        if let Some(outer) = ivars.outer_view.get() {
            apply_outer_border(outer, style);
        }
        // Vibrancy fallback: a plain tint layer carries panel_background at
        // panel_opacity (the glass path uses the material tint above).
        if let Some(tint) = ivars.tint_view.get() {
            if let Some(layer) = unsafe { tint.layer() } {
                set_layer_bg(
                    &layer,
                    &rgba(style.panel_background, style.panel_opacity),
                );
            }
        }
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
        ivars.scroll_px.set(0.0);
        ivars.scroll_target.set(0.0);
        ivars.sigil.set(None);
        ivars.auto_sigil.set(None);
        ivars.mode.set(PanelMode::Launcher);
        ivars.command_context.borrow_mut().take();
        ivars.search_engine.borrow_mut().take();
        // Seed from the real current state: if the summon chord itself is
        // held (e.g. the default cmd+space), our flagsChanged monitor never
        // saw cmd go down — it wasn't key window yet — so without this the
        // "⌘N" hints would stay hidden until cmd is released and re-pressed.
        ivars.cmd_held.set(
            unsafe { NSEvent::modifierFlags_class() }
                .contains(NSEventModifierFlags::NSEventModifierFlagCommand),
        );

        let vf = screen.visibleFrame();
        // Rounded: a fractional anchor makes every window origin fractional,
        // and AppKit rounds those inconsistently — a setFrame that also
        // changes the height lands a point lower than one that doesn't, so
        // the panel dips on each resize and pops back on the next relayout.
        ivars.top_y.set((vf.origin.y + vf.size.height * 0.72).round());
        self.refresh();
        // `h` is the whole window, margin included; centre and pin by that,
        // which leaves the body itself centred and its top edge on `top_y`.
        let h = panel.frame().size.height;
        let o = outer_inset(&ivars.config.borrow().style);
        let win_w = ivars.config.borrow().style.width + 2.0 * o;
        let x = vf.origin.x + (vf.size.width - win_w) / 2.0;
        panel.setFrameOrigin(NSPoint::new(x, ivars.top_y.get() - h + o));

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
        // Dismissing the picker without committing reverts the preview.
        if ivars.mode.get() == PanelMode::ThemePicker {
            ivars.mode.set(PanelMode::Launcher);
            if let Some(style) = ivars.saved_style.borrow_mut().take() {
                ivars.config.borrow_mut().style = style;
                self.apply_live_style();
            }
        }
        if ivars.mode.get() == PanelMode::AppCommands {
            ivars.mode.set(PanelMode::Launcher);
            ivars.command_context.borrow_mut().take();
        }
        if ivars.mode.get() == PanelMode::SearchEngine {
            ivars.mode.set(PanelMode::Launcher);
            ivars.search_engine.borrow_mut().take();
        }
        if let Some(timer) = ivars.stats_timer.borrow_mut().take() {
            let _: () = unsafe { msg_send![&*timer, invalidate] };
        }
        self.stop_scroll_glide();
        // Drop the cached scan so the next summon re-reads the app directories,
        // and the rates so a completed background refresh is picked up.
        ivars.installed_cache.borrow_mut().take();
        ivars.rates_cache.borrow_mut().take();
        ivars.running_order.borrow_mut().take();
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
        if self.ivars().mode.get() == PanelMode::ThemePicker {
            self.refresh_theme_picker();
            return;
        }
        if self.ivars().mode.get() == PanelMode::AppCommands {
            self.refresh_app_commands();
            return;
        }
        if self.ivars().mode.get() == PanelMode::SearchEngine {
            self.refresh_search_engine();
            return;
        }
        let query = self.query();

        // Sigil modes: the active sigil (held in state, shown in the input
        // badge — never in the field) routes the query terms to a resolver
        // instead of the app matcher. Resolved before the app scan, so mode
        // keystrokes never touch the filesystem. One dispatch for every
        // sigil — a new mode plugs in via `SigilKind` and its resolver.
        // With no sigil, the bare query can still classify as math or
        // currency per keystroke (`auto_kind`). An autodetected mode shows
        // the same badge as its sigil (via `auto_sigil`), but the char
        // stays in the field and the state clears itself next refresh.
        let kind = {
            let cfg = self.ivars().config.borrow();
            let explicit = self.ivars().sigil.get().and_then(|c| cfg.sigil_kind(c));
            let auto = if explicit.is_none() {
                modes::auto_kind(
                    &query,
                    &cfg.currency_targets,
                    cfg.sigil_math.is_some(),
                    cfg.sigil_currency.is_some(),
                )
            } else {
                None
            };
            self.ivars().auto_sigil.set(match auto {
                Some(SigilKind::Math) => cfg.sigil_math,
                Some(SigilKind::Currency) => cfg.sigil_currency,
                None => None,
            });
            explicit.or(auto)
        };
        let sigil_rows = match kind {
            Some(SigilKind::Math) => Some(modes::math_rows(&query)),
            Some(SigilKind::Currency) => Some(self.currency_rows(&query)),
            None => None,
        };
        if let Some(rows) = sigil_rows {
            let ivars = self.ivars();
            let entries: Vec<Entry> = rows
                .into_iter()
                .map(|r| Entry {
                    name: r.name,
                    path: None,
                    running: None,
                    matched: Vec::new(),
                    windows: 0,
                    stats: None,
                    command: None,
                    builtin: Some(Builtin::ModeRow(r.action)),
                    detail: r.detail,
                    tag: r.tag,
                    icon: r.icon,
                    engine: None,
                    app_action: None,
                })
                .collect();
            let len = entries.len();
            *ivars.entries.borrow_mut() = entries;
            self.visible_range(len);
            self.relayout();
            return;
        }
        let running = self.running_apps_in_order();

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
                            builtin: None,
                            detail: None,
                            tag: None,
                            icon: None,
                            engine: None,
                            app_action: None,
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
                            builtin: None,
                            detail: None,
                            tag: None,
                            icon: None,
                            engine: None,
                            app_action: None,
                        },
                    ));
                }
            }
            // `[search_engines]` items, ranked by `engine_match`.
            let icons_engine = self.ivars().config.borrow().icons.engine.clone();
            for engine in &self.ivars().config.borrow().search_engines {
                let Some((score, matched)) = engine_match(&query, engine) else {
                    continue;
                };
                scored.push((
                    score,
                    Entry {
                        name: engine.name.clone(),
                        path: None,
                        running: None,
                        matched,
                        windows: 0,
                        stats: None,
                        command: None,
                        builtin: None,
                        detail: None,
                        tag: Some(modes::engine_pill(engine).to_string()),
                        icon: Some(engine.glyph(&icons_engine)),
                        engine: Some(engine.clone()),
                        app_action: None,
                    },
                ));
            }
            // Built-in settings rows, matched like everything else. The
            // theme-picker entry point names the active theme so the current
            // state is visible before you enter the picker.
            {
                let setting_name =
                    format!("Setting: Change Theme ({})", self.current_theme_display());
                if let Some((s, positions)) = apps::match_positions(&query, &setting_name) {
                    scored.push((
                        s,
                        Entry {
                            name: setting_name,
                            path: None,
                            running: None,
                            matched: positions,
                            windows: 0,
                            stats: None,
                            command: None,
                            builtin: Some(Builtin::ThemePicker),
                            detail: None,
                            tag: None,
                            icon: None,
                            engine: None,
                            app_action: None,
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
        // Stats for the visible running apps only (a handful of syscalls plus
        // one window-list snapshot — microseconds, fresh every time), so the
        // per-tick cost tracks the panel's height and not the match count.
        // Scrolling calls back through here, which is how a row that just
        // came into view gets its gauge. CPU% needs two samples: rows show
        // "…" until the second sample lands, then a one-shot refreshTick
        // fills the number in. Never blocks.
        let w = self.visible_range(entries.len());
        let (first, last) = (w.first, w.first + w.drawn);
        let window_counts = stats::window_counts();
        let procs = stats::ProcSnapshot::new();
        let mut cpu_pending = false;
        {
            let mut samples = self.ivars().cpu_samples.borrow_mut();
            samples.retain(|pid, _| procs.is_alive(*pid));
            let now = std::time::Instant::now();
            for entry in entries[first..last].iter_mut() {
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
            self.schedule_refresh(0.4);
        }

        *self.ivars().entries.borrow_mut() = entries;
        self.relayout();
    }

    /// One coalesced `refreshTick`, `delay` seconds out: a second sample for
    /// a CPU gauge that isn't ready yet, or fresh stats for rows that just
    /// scrolled into view. Coalesced because a refresh walks the process
    /// table and the callers can fire faster than that is worth doing —
    /// trackpad momentum lands dozens of scroll events a second.
    fn schedule_refresh(&self, delay: f64) {
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
                afterDelay: delay
            ];
        }
    }

    /// The running apps in a stable order: whatever the system handed back
    /// the first time this panel asked, with anything launched since
    /// appended. See `running_order` for why the raw order won't do.
    fn running_apps_in_order(&self) -> Vec<Entry> {
        let mut apps = running_apps();
        let ivars = self.ivars();
        let mut order = ivars.running_order.borrow_mut();
        let pinned = order.get_or_insert_with(|| apps.iter().map(entry_pid).collect());
        apps.sort_by_key(|e| {
            let pid = entry_pid(e);
            pinned.iter().position(|p| *p == pid).unwrap_or(usize::MAX)
        });
        // Remember the newcomers so they hold their place too.
        for e in apps.iter() {
            let pid = entry_pid(e);
            if !pinned.contains(&pid) {
                pinned.push(pid);
            }
        }
        apps
    }

    /// On the first keystroke of a launcher session, a reserved leading
    /// character enters its sigil mode: the sigil is stored in state and
    /// lifted out of the field, leaving only the query terms behind.
    /// What the input badge shows, if anything: the glyph of the engine
    /// holding the panel, otherwise the active or autodetected sigil. One
    /// slot, one answer — a search engine and a sigil mode are never both
    /// live, and both mean the same thing to the eye ("the field is no
    /// longer an app search").
    fn input_badge(&self) -> Option<String> {
        let ivars = self.ivars();
        if ivars.mode.get() == PanelMode::SearchEngine {
            let default_icon = &ivars.config.borrow().icons.engine;
            return ivars.search_engine.borrow().as_ref().map(|e| e.glyph(default_icon));
        }
        ivars.sigil.get().or(ivars.auto_sigil.get()).map(|c| c.to_string())
    }

    fn maybe_enter_sigil(&self) {
        let ivars = self.ivars();
        if ivars.mode.get() != PanelMode::Launcher || ivars.sigil.get().is_some() {
            return;
        }
        let q = self.query();
        let Some(c) = q.chars().next() else {
            return;
        };
        let is_sigil = ivars.config.borrow().sigil_kind(c).is_some();
        if is_sigil {
            ivars.sigil.set(Some(c));
            self.set_field_text(&q[c.len_utf8()..]);
        }
    }

    /// Set the field's text and move the caret to the end. Programmatic
    /// changes don't fire `controlTextDidChange`, so this never recurses.
    fn set_field_text(&self, text: &str) {
        let ivars = self.ivars();
        let (Some(panel), Some(field)) = (ivars.panel.get(), ivars.field.get()) else {
            return;
        };
        let ns = NSString::from_str(text);
        let len = ns.length();
        unsafe { field.setStringValue(&ns) };
        if let Some(editor) = unsafe { panel.fieldEditor_forObject(true, Some(field)) } {
            unsafe {
                let _: () = msg_send![&*editor, setSelectedRange: NSRange::new(len, 0)];
            }
        }
    }

    /// Currency rows for the current query. Loads the rate cache (spawning a
    /// background refresh if it's missing or stale) and never blocks: with no
    /// cache yet it returns a single "fetching" notice.
    fn currency_rows(&self, query: &str) -> Vec<modes::ModeRow> {
        self.ensure_rates();
        let ivars = self.ivars();
        let targets = ivars.config.borrow().currency_targets.clone();
        let cache = ivars.rates_cache.borrow();
        match &*cache {
            Some(r) if !r.map.is_empty() => {
                modes::currency_rows(query, &r.map, &targets, Some(r.age_display()))
            }
            _ => vec![modes::info_row("Fetching exchange rates…")],
        }
    }

    /// Populate `rates_cache` from disk once per summon; kick off a background
    /// `curl` refresh when the cache is absent or older than the TTL.
    fn ensure_rates(&self) {
        let ivars = self.ivars();
        if ivars.rates_cache.borrow().is_some() {
            return;
        }
        let path = rates_cache_path();
        let loaded = path.as_ref().and_then(|p| load_rates(p));
        let stale = loaded
            .as_ref()
            .map_or(true, |r| r.age_secs.map_or(true, |a| a > RATE_TTL_SECS));
        if stale {
            if let Some(p) = path {
                spawn_rate_fetch(p);
            }
        }
        *ivars.rates_cache.borrow_mut() = Some(loaded.unwrap_or_default());
    }

    /// Display name of the active theme ("Black Glass" = un-overlaid base).
    fn current_theme_display(&self) -> String {
        self.ivars()
            .session_theme
            .borrow()
            .as_deref()
            .map(config::theme_display_name)
            .unwrap_or_else(|| "Black Glass".to_string())
    }

    /// Populate the panel with one row per theme (base first, then
    /// `themes/*.toml` in name order), filtered by the query.
    fn refresh_theme_picker(&self) {
        let ivars = self.ivars();
        let query = self.query();
        let mut entries: Vec<Entry> = Vec::new();
        let mut names: Vec<Option<String>> = vec![None];
        names.extend(
            ivars.config.borrow().themes.iter().map(|t| Some(t.name.clone())),
        );
        for name in names {
            let display = name
                .as_deref()
                .map(config::theme_display_name)
                .unwrap_or_else(|| "Black Glass".to_string());
            let matched = if query.is_empty() {
                Vec::new()
            } else {
                match apps::match_positions(&query, &display) {
                    Some((_, positions)) => positions,
                    None => continue,
                }
            };
            entries.push(Entry {
                name: display,
                path: None,
                running: None,
                matched,
                windows: 0,
                stats: None,
                command: None,
                builtin: Some(Builtin::ApplyTheme(name)),
                detail: None,
                tag: None,
                icon: None,
                engine: None,
                app_action: None,
            });
        }
        let len = entries.len();
        *ivars.entries.borrow_mut() = entries;
        self.visible_range(len);
        // Typing moves the selection, and the selection IS the preview.
        self.preview_selected_theme();
        self.relayout();
    }

    /// Populate the panel with the active `command_context`'s rows, filtered
    /// like everything else. Insertion order is preserved (not re-sorted by
    /// score) so Open/Focus stays first — only the fuzzy filter thins the
    /// list. Built-in rows carry the context's `path`/`running` so `execute`
    /// can act on the target app directly; `[commands.<Name>]` extras carry
    /// a plain shell `command`, same as a `[shortcuts]` row.
    fn refresh_app_commands(&self) {
        let ivars = self.ivars();
        let query = self.query();
        let context = ivars.command_context.borrow().clone();
        let Some(context) = context else {
            // Context lost somehow (e.g. a config reload mid-session); don't
            // strand the panel in a mode with nothing to show.
            ivars.mode.set(PanelMode::Launcher);
            self.refresh();
            return;
        };
        let mut entries: Vec<Entry> = Vec::new();
        for (label, cmd) in &context.commands {
            let matched = if query.is_empty() {
                Vec::new()
            } else {
                match apps::match_positions(&query, label) {
                    Some((_, m)) => m,
                    None => continue,
                }
            };
            let (path, running, command, app_action) = match cmd {
                AppCommand::Builtin(action) => {
                    (context.path.clone(), context.running.clone(), None, Some(*action))
                }
                AppCommand::Shell(sh) => (None, None, Some(sh.clone()), None),
            };
            entries.push(Entry {
                name: label.clone(),
                path,
                running,
                matched,
                windows: 0,
                stats: None,
                command,
                builtin: None,
                detail: None,
                tag: None,
                icon: None,
                engine: None,
                app_action,
            });
        }
        let len = entries.len();
        *ivars.entries.borrow_mut() = entries;
        self.visible_range(len);
        self.relayout();
    }

    /// Rows while an engine holds the panel: the field is search terms, not
    /// a filter, so nothing is matched here — the whole query goes to every
    /// engine and the active one leads.
    fn refresh_search_engine(&self) {
        let ivars = self.ivars();
        let engine = ivars.search_engine.borrow().clone();
        let Some(engine) = engine else {
            // Context lost (e.g. a config reload mid-session); don't strand
            // the panel in a mode with nothing to search.
            ivars.mode.set(PanelMode::Launcher);
            self.refresh();
            return;
        };
        let terms = self.query();
        let cfg = ivars.config.borrow();
        let active = cfg
            .search_engines
            .iter()
            .position(|e| e.name == engine.name)
            .unwrap_or(usize::MAX);
        // A reload can drop the engine from the config; it still searches,
        // it just no longer leads a list it isn't in.
        let mut engines = cfg.search_engines.clone();
        let active = if active == usize::MAX {
            engines.insert(0, engine);
            0
        } else {
            active
        };
        let default_icon = cfg.icons.engine.clone();
        drop(cfg);
        let entries: Vec<Entry> = modes::engine_rows(&terms, &engines, active, &default_icon)
            .into_iter()
            .map(|r| Entry {
                name: r.name,
                path: None,
                running: None,
                matched: Vec::new(),
                windows: 0,
                stats: None,
                command: None,
                builtin: Some(Builtin::ModeRow(r.action)),
                detail: r.detail,
                tag: r.tag,
                icon: r.icon,
                engine: None,
                app_action: None,
            })
            .collect();
        let len = entries.len();
        *ivars.entries.borrow_mut() = entries;
        self.visible_range(len);
        self.relayout();
    }

    /// Switch the panel into theme-picker mode, with the active theme
    /// selected. The current style is saved for revert-on-dismiss.
    fn enter_theme_picker(&self) {
        let ivars = self.ivars();
        *ivars.saved_style.borrow_mut() = Some(ivars.config.borrow().style.clone());
        ivars.mode.set(PanelMode::ThemePicker);
        if let Some(field) = ivars.field.get() {
            unsafe { field.setStringValue(&NSString::from_str("")) };
        }
        // Base row is index 0; theme rows follow in sorted order.
        let index = match ivars.session_theme.borrow().as_ref() {
            Some(name) => ivars
                .config
                .borrow()
                .themes
                .iter()
                .position(|t| &t.name == name)
                .map(|i| i + 1)
                .unwrap_or(0),
            None => 0,
        };
        ivars.selected.set(index);
        ivars.scroll_px.set(0.0);
        self.refresh();
        // The active theme can sit well past the sixth row.
        self.scroll_to_selection(false);
    }

    /// Live preview: restyle the visible panel with the selected row's theme.
    fn preview_selected_theme(&self) {
        let ivars = self.ivars();
        if ivars.mode.get() != PanelMode::ThemePicker {
            return;
        }
        let name = {
            let entries = ivars.entries.borrow();
            match entries.get(ivars.selected.get()).map(|e| &e.builtin) {
                Some(Some(Builtin::ApplyTheme(name))) => name.clone(),
                _ => return,
            }
        };
        let overlay = {
            let cfg = ivars.config.borrow();
            name.as_ref()
                .and_then(|n| cfg.themes.iter().find(|t| &t.name == n).cloned())
        };
        let mut style = ivars.base_style.borrow().clone();
        if let Some(theme) = &overlay {
            config::apply_theme(&mut style, theme);
        }
        ivars.config.borrow_mut().style = style;
        self.apply_live_style();
    }

    /// Clamp the selection and the scroll position to a list of `total`
    /// rows and hand back the window that draws. Idempotent — every path
    /// that changes the list, the selection or the offset ends here.
    fn visible_range(&self, total: usize) -> RowWindow {
        let ivars = self.ivars();
        let max_rows = ivars.config.borrow().max_rows;
        let window = row_window(total, max_rows, ivars.scroll_px.get());
        ivars.scroll_px.set(window.px);
        ivars.selected.set(ivars.selected.get().min(total.saturating_sub(1)));
        window
    }

    /// Put the window at `px` and show it. Purely positional — it never
    /// touches the selection, because the keyboard path scrolls precisely
    /// *to* a selection that is off screen and clamping here would drag it
    /// back on every frame of the glide.
    ///
    /// While the same rows and the same highlight are on screen, sliding
    /// the rows-area bounds is the whole of the work: a 120 Hz trackpad
    /// drag rebuilds nothing until it crosses a row.
    fn scroll_to(&self, px: f64) {
        let ivars = self.ivars();
        let total = ivars.entries.borrow().len();
        let max_rows = ivars.config.borrow().max_rows;
        let after = row_window(total, max_rows, px);
        let same_rows = after.first == ivars.drawn_first.get()
            && after.drawn == ivars.drawn_count.get()
            && ivars.selected.get() == ivars.drawn_selected.get();
        if same_rows && (after.px - ivars.scroll_px.get()).abs() <= f64::EPSILON {
            return;
        }
        ivars.scroll_px.set(after.px);
        if same_rows {
            self.slide_rows(after.frac);
        } else {
            self.relayout();
        }
    }

    /// Scroll the way the pointer asked, dragging the selection along at
    /// whichever edge it would otherwise fall off — Enter must never fire
    /// on a row that has scrolled out of sight. Only the wheel and the
    /// trackpad come through here; under the keyboard the selection leads
    /// and the window follows it.
    fn scroll_with_selection(&self, px: f64) {
        let ivars = self.ivars();
        let total = ivars.entries.borrow().len();
        let max_rows = ivars.config.borrow().max_rows;
        let full = row_window(total, max_rows, px).full_rows();
        if !full.is_empty() {
            let selected = ivars.selected.get();
            let clamped = selected.clamp(full.start, full.end - 1);
            if clamped != selected {
                ivars.selected.set(clamped);
                self.preview_selected_theme();
            }
        }
        self.scroll_to(px);
        self.schedule_refresh(SCROLL_SAMPLE_DELAY);
    }

    /// Nudge the drawn rows up by `frac` points without rebuilding them.
    /// The rows-area bounds origin is the same lever `NSClipView` pulls —
    /// subviews move with it, and the area's own layer clips whatever hangs
    /// off either end.
    ///
    /// The sign: a subview's frame is read in its superview's *bounds*
    /// space, so it draws `frame.y - bounds.origin.y` up from the bottom
    /// edge. Rows have to travel UP as the list scrolls down into itself,
    /// which means the origin goes negative.
    fn slide_rows(&self, frac: f64) {
        if let Some(rows_area) = self.ivars().rows_area.get() {
            unsafe { rows_area.setBoundsOrigin(NSPoint::new(0.0, -frac)) };
        }
    }

    /// Head for `px` over the next few frames instead of jumping there.
    /// Used where the movement is in row steps — a key, a wheel notch —
    /// since those have no finger position to track. `[animation] scroll`
    /// off (or no distance worth easing) lands immediately.
    fn glide_scroll_to(&self, px: f64, drag_selection: bool) {
        let ivars = self.ivars();
        let smooth = ivars.config.borrow().scroll_animation;
        if !smooth || (px - ivars.scroll_px.get()).abs() < 1.0 {
            self.stop_scroll_glide();
            if drag_selection {
                self.scroll_with_selection(px);
            } else {
                self.scroll_to(px);
            }
            return;
        }
        ivars.glide_drags_selection.set(drag_selection);
        ivars.scroll_target.set(px);
        if ivars.scroll_timer.borrow().is_some() {
            return;
        }
        ivars.scroll_at.set(Some(std::time::Instant::now()));
        let nil = std::ptr::null::<objc2::runtime::AnyObject>();
        // Built unscheduled and added for the common modes: on the default
        // mode alone the glide stalls mid-flight whenever the run loop is
        // tracking something (the field editor, a menu).
        let timer: Retained<objc2::runtime::AnyObject> = unsafe {
            msg_send_id![
                objc2::class!(NSTimer),
                timerWithTimeInterval: SCROLL_FRAME,
                target: self,
                selector: sel!(scrollTick),
                userInfo: nil,
                repeats: true
            ]
        };
        unsafe {
            let loop_: Retained<objc2::runtime::AnyObject> =
                msg_send_id![objc2::class!(NSRunLoop), currentRunLoop];
            let mode = NSString::from_str("kCFRunLoopCommonModes");
            let _: () = msg_send![&*loop_, addTimer: &*timer, forMode: &*mode];
        }
        *ivars.scroll_timer.borrow_mut() = Some(timer);
    }

    /// One glide frame: ease toward the target by a fixed fraction of the
    /// remaining distance per unit time, so the speed is the same whatever
    /// the frame rate happens to be, and stop once there's nothing left.
    /// `drag_selection` is set for a wheel notch and clear for a keypress —
    /// see `scroll_with_selection`.
    fn scroll_glide_step(&self) {
        let ivars = self.ivars();
        let target = ivars.scroll_target.get();
        let now = std::time::Instant::now();
        let dt = ivars
            .scroll_at
            .replace(Some(now))
            .map_or(SCROLL_FRAME, |at| now.duration_since(at).as_secs_f64())
            .min(0.1);
        let px = ivars.scroll_px.get();
        let mut next = px + (target - px) * (1.0 - (-dt / SCROLL_EASE_TAU).exp());
        let arrived = (target - next).abs() < 0.5;
        if arrived {
            self.stop_scroll_glide();
            next = target;
        }
        if ivars.glide_drags_selection.get() {
            self.scroll_with_selection(next);
        } else {
            self.scroll_to(next);
        }
    }

    fn stop_scroll_glide(&self) {
        let ivars = self.ivars();
        if let Some(timer) = ivars.scroll_timer.borrow_mut().take() {
            let _: () = unsafe { msg_send![&*timer, invalidate] };
        }
        ivars.scroll_at.set(None);
        ivars.scroll_target.set(ivars.scroll_px.get());
    }

    /// Bring the selected row into view. `glide` eases it there (arrow
    /// keys, where the movement is the point); entering a mode with a row
    /// already picked out just puts the window where it belongs.
    fn scroll_to_selection(&self, glide: bool) {
        let ivars = self.ivars();
        let total = ivars.entries.borrow().len();
        let max_rows = ivars.config.borrow().max_rows;
        let target =
            scroll_to_show(total, max_rows, ivars.selected.get(), ivars.scroll_px.get());
        if (target - ivars.scroll_px.get()).abs() <= f64::EPSILON {
            return;
        }
        if glide {
            self.glide_scroll_to(target, false);
        } else {
            self.stop_scroll_glide();
            self.scroll_to(target);
        }

        // Rows that just came into view have never been sampled, so their
        // gauges land on a coalesced tick rather than walking the process
        // table once per keypress.
        self.schedule_refresh(SCROLL_SAMPLE_DELAY);
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
        self.preview_selected_theme();
        self.relayout();
        self.scroll_to_selection(true);
    }

    /// Reposition everything for the current entry count and rebuild rows.
    fn relayout(&self) {
        unsafe { self.relayout_impl() }
    }

    unsafe fn relayout_impl(&self) {
        let mtm = MainThreadMarker::new().unwrap();
        let ivars = self.ivars();
        let (
            Some(panel),
            Some(field),
            Some(glyph),
            Some(sigil_box),
            Some(sigil_label),
            Some(rows_area),
        ) = (
            ivars.panel.get(),
            ivars.field.get(),
            ivars.glyph.get(),
            ivars.sigil_box.get(),
            ivars.sigil_label.get(),
            ivars.rows_area.get(),
        ) else {
            return;
        };

        let entries = ivars.entries.borrow();
        let w = self.visible_range(entries.len());
        let (first, last) = (w.first, w.first + w.drawn);
        // Recomputed every relayout (cheap — at most a handful of rows), so
        // it's never stale relative to what's about to be drawn, and the
        // key handler reads the exact same values back. Only the drawn
        // window gets hints: a ⌘-jump has to point at something you can see,
        // and the digits restart from ⌘1 at the top of each window.
        {
            let mut hints: Vec<Option<RowHint>> = vec![None; entries.len()];
            let window = compute_row_hints(&entries[first..last], &ivars.config.borrow().binds);
            hints[first..last].copy_from_slice(&window);
            *ivars.row_hints.borrow_mut() = hints;
        }
        // Height follows the rows that fit, never the partial one scrolling
        // past — the panel must not breathe in and out as the list moves.
        let pad = ivars.config.borrow().style.panel_padding;
        let rows_h = if w.win > 0 {
            w.win as f64 * ROW_H + ROWS_PAD
        } else {
            0.0
        };
        // Padding wraps the content on both ends: `pad` above the input band
        // and `pad` below the last row.
        // Integral too, so `top - h` stays whole and the top edge is pinned
        // to the same device pixel at every height.
        let h = (pad + INPUT_H + rows_h + pad).round();

        let panel_w = ivars.config.borrow().style.width;
        let o = outer_inset(&ivars.config.borrow().style);
        let old = panel.frame();
        let top = ivars.top_y.get();
        // The window is `o` larger than the body on every side, and `top_y`
        // pins the *body's* top edge — so the window sits `o` higher than the
        // body wants to, and adding an outer border never shifts the panel.
        let want_y = (top - h - o).round();
        panel.setFrame_display(
            NSRect::new(
                NSPoint::new(old.origin.x, want_y),
                NSSize::new(panel_w + 2.0 * o, h + 2.0 * o),
            ),
            true,
        );
        // Set explicitly rather than leaning on autoresizing: the margin
        // itself changes when a theme changes `outer_border_width`, and
        // autoresizing only ever moves edges, never the inset.
        let body = NSRect::new(NSPoint::new(o, o), NSSize::new(panel_w, h));
        if let Some(chrome) = ivars.chrome_view.get() {
            chrome.setFrame(body);
        }
        if let Some(container) = ivars.container_view.get() {
            container.setFrame(body);
        }

        // Input band, inset from the top by `pad`. The extra 12px keeps the
        // search glyph aligned with the row glyphs (rows inset by 12px).
        let input_inset = pad + 12.0;
        let input_bottom = h - pad - INPUT_H;
        // Leading slot: the search glyph, or — in a mode — a colored box
        // holding the sigil character. Both are centered in a slot of the
        // same fixed width so the field starts at the same x either way:
        // autodetect swaps the glyph for a badge mid-query, and a slot sized
        // to whichever is showing would jump the text sideways as you type.
        let input_fs = ivars.config.borrow().style.input_font_size;
        let slot = (input_fs + 10.0).clamp(24.0, 52.0);
        if let Some(badge) = self.input_badge() {
            unsafe {
                let _: () = msg_send![&**glyph, setHidden: true];
                let _: () = msg_send![&**sigil_box, setHidden: false];
            }
            let (bg, fg) = {
                let cfg = ivars.config.borrow();
                let s = &cfg.style;
                (
                    s.sigil_background.unwrap_or(s.item_foreground_highlight),
                    s.sigil_foreground.unwrap_or(s.panel_background),
                )
            };
            sigil_box.setFrame(NSRect::new(
                NSPoint::new(input_inset, input_bottom + (INPUT_H - slot) / 2.0),
                NSSize::new(slot, slot),
            ));
            if let Some(layer) = sigil_box.layer() {
                layer.setCornerRadius((slot * 0.26).min(10.0));
                set_layer_bg(&layer, &rgba(bg, 1.0));
            }
            let font: Retained<NSFont> = unsafe {
                msg_send_id![NSFont::class(), systemFontOfSize: input_fs * 0.78, weight: 0.4f64]
            };
            unsafe {
                sigil_label.setStringValue(&NSString::from_str(&badge));
                sigil_label.setFont(Some(&font));
                sigil_label.setTextColor(Some(&rgba(fg, 1.0)));
                sigil_label.sizeToFit();
            }
            let ls = sigil_label.frame().size;
            sigil_label.setFrameOrigin(NSPoint::new(
                (slot - ls.width) / 2.0,
                (slot - ls.height) / 2.0,
            ));
        } else {
            unsafe {
                let _: () = msg_send![&**sigil_box, setHidden: true];
                let _: () = msg_send![&**glyph, setHidden: false];
            }
            let glyph_size = glyph.frame().size;
            glyph.setFrameOrigin(NSPoint::new(
                input_inset + (slot - glyph_size.width).max(0.0) / 2.0,
                input_bottom + (INPUT_H - glyph_size.height) / 2.0,
            ));
        }
        let field_x = input_inset + slot + 14.0;
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
        // Rows sit at whole-row positions and the bounds origin carries the
        // sub-row offset, so a scroll in flight is one origin change rather
        // than a rebuild (see `slide_rows`, which derives the sign).
        self.slide_rows(w.frac);
        for view in rows_area.subviews().iter() {
            unsafe { view.removeFromSuperview() };
        }
        // Where the cursor is as this frame goes up, for `hover_row` to
        // tell a real hover from rows moving underneath a still pointer.
        let at = unsafe { NSEvent::mouseLocation() };
        ivars.hover_at.set(Some((at.x, at.y)));
        let selected = ivars.selected.get();
        // What is on screen from here on, for `scroll_to`'s slide fast path.
        ivars.drawn_first.set(first);
        ivars.drawn_count.set(w.drawn);
        ivars.drawn_selected.set(selected);
        for (row, entry) in entries[first..last].iter().enumerate() {
            let index = first + row;
            let y = rows_h - ROWS_PAD / 2.0 - (row as f64 + 1.0) * ROW_H;
            self.build_row(mtm, rows_area, y, index, entry, index == selected);
        }
    }

    /// A row claiming the selection because the pointer came to rest on it.
    ///
    /// Only honoured if the cursor has moved since the panel last drew.
    /// `mouseEntered` fires just as readily when the rows move under a
    /// still pointer — every scroll step and every rebuild does it — and
    /// taking those at face value would have the list fight the arrow keys
    /// for the highlight. "Is the cursor where it was when we drew this?"
    /// is the whole test: if it is, this event is our own doing.
    fn hover_row(&self, index: usize) {
        let at = unsafe { NSEvent::mouseLocation() };
        let moved = self.ivars().hover_at.get().is_none_or(|(x, y)| {
            (x - at.x).abs() > 0.01 || (y - at.y).abs() > 0.01
        });
        if moved {
            self.select_row(index);
        }
    }

    /// Mouse selection; typing afterwards resets it (controlTextDidChange
    /// zeroes the selection), so the keyboard always wins.
    fn select_row(&self, index: usize) {
        let len = self.ivars().entries.borrow().len();
        if index < len && self.ivars().selected.get() != index {
            self.ivars().selected.set(index);
            self.preview_selected_theme();
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
                // Inset stroke: the border draws inside the row rect, so rows
                // never shift as the selection moves.
                if style.selected_item_border_width > 0.0 {
                    let border = rgba(style.selected_item_border, 1.0);
                    unsafe {
                        let cg: *mut c_void = msg_send![&*border, CGColor];
                        let _: () = msg_send![&*layer, setBorderColor: cg];
                        let _: () = msg_send![
                            &*layer,
                            setBorderWidth: style.selected_item_border_width
                        ];
                    }
                }
            }
        }

        // The glyph column dims for exactly one thing: a running app you
        // can't see — hidden, or with no on-screen windows. Everything else,
        // running or not, draws at full strength.
        let out_of_sight = match &entry.running {
            Some(app) => entry.windows == 0 || unsafe { app.isHidden() },
            None => false,
        };
        // All row content takes its hue from one foreground per state; the
        // original design's brightness steps carry over as alphas.
        let fg = if selected {
            style.selected_item_foreground
        } else {
            style.item_foreground
        };

        // Math rows present differently from apps: no leading icon at all
        // (the "= …" stands alone, name flush left). The state-brightness
        // dimming is likewise an app-only affordance, never a mode row's.
        let mode_math = matches!(
            &entry.builtin,
            Some(Builtin::ModeRow(modes::ModeAction::Copy(_)))
        );
        let name_x = if mode_math {
            12.0
        } else {
            // Leading state glyph column. An entry carrying its own icon
            // (a `[search_engines]` item) wins outright — that icon is part
            // of the item, not a per-machine override of it. Otherwise
            // `[icons.apps]` overrides apply (exact name match, or a
            // `*pattern*` match), then the running/installed state glyph.
            let entry_lower = entry.name.to_lowercase();
            let glyph_text = entry
                .icon
                .as_deref()
                .or_else(|| config::find_icon_override(&cfg.icon_overrides, &entry_lower))
                .unwrap_or_else(|| state_glyph(entry, icons));
            let glyph_font = unsafe { NSFont::systemFontOfSize(GLYPH_PT) };
            // The icon carries visibility state through its brightness: a
            // running app that is hidden or has no windows is dimmed, and
            // nothing else is.
            let glyph_alpha = if out_of_sight { 0.52 } else { 0.92 };
            // `icon_foreground` recolors the glyph column; the state-brightness
            // alphas carry over unchanged.
            let glyph_color = style.icon_foreground.unwrap_or(fg);
            let glyph =
                make_label(mtm, glyph_text, &glyph_font, &rgba(glyph_color, glyph_alpha));
            let glyph_h = glyph.frame().size.height;
            glyph.setFrameOrigin(NSPoint::new(12.0, (ROW_H - glyph_h) / 2.0));
            row.addSubview(&glyph);
            12.0 + GLYPH_COL_W + 10.0
        };

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

        // Tag pills sit inline, right after the name.
        let pill_x = name_x + name.frame().size.width + 10.0;
        if let Some(tag) = &entry.tag {
            // Search engines: the shortcut ("g", "yt") as a label-only pill.
            self.build_tag_pill(mtm, &row, pill_x, tag, "", selected);
        } else if let Some(rs) = &entry.stats {
            // Running apps: a CPU warning at the right edge once the tree
            // burns ≥ CPU_ALERT_PCT of a core. Below that the row's right
            // edge stays bare — the presence dot is off for now.
            if rs.cpu_pct >= CPU_ALERT_PCT {
                self.build_cpu_warning(mtm, &row, row_w - 12.0, rs.cpu_pct, selected);
            }
        } else if let Some(path) = &entry.path {
            // Installed rows: location tag pill with symbol.
            let (label_text, symbol) = location_for(path, icons);
            self.build_tag_pill(mtm, &row, pill_x, label_text, symbol, selected);
        } else if entry.command.is_some() {
            self.build_tag_pill(mtm, &row, pill_x, "Shortcut", &icons.shortcut, selected);
        } else {
            match &entry.builtin {
                Some(Builtin::ThemePicker) => {
                    self.build_tag_pill(mtm, &row, pill_x, "Setting", &icons.system, selected);
                }
                // The active theme's row carries the presence dot.
                Some(Builtin::ApplyTheme(name))
                    if *name == *self.ivars().session_theme.borrow() =>
                {
                    self.build_running_dot(mtm, &row, row_w - 12.0);
                }
                _ => {}
            }
        }

        // Dim right-aligned detail (mode rows: URLs, alternate results).
        if let Some(detail) = &entry.detail {
            let font = resolve_font(&style.font_family, 0.0, 11.0);
            let alpha = if selected { 0.65 } else { 0.45 };
            let label = make_label(mtm, detail, &font, &rgba(fg, alpha));
            let size = label.frame().size;
            label.setFrameOrigin(NSPoint::new(
                row_w - 12.0 - size.width,
                (ROW_H - size.height) / 2.0,
            ));
            row.addSubview(&label);
        }

        // Cmd-held row-jump hint ("⌘F" for a running app, "⌘N" otherwise —
        // see `compute_row_hints`/`try_activate_hint`). Added last so it
        // sits above any other right-edge content (running dot, CPU badge,
        // tag pill, detail) while it's showing, since it's a transient
        // overlay, not fixed row content.
        if self.ivars().cmd_held.get() {
            if let Some(hint) = self.ivars().row_hints.borrow().get(index).copied().flatten() {
                self.build_hint_badge(mtm, &row, row_w, hint);
            }
        }

        parent.addSubview(&row);

        // Hover tracking, over the part of the row that is actually on
        // screen: the first and last drawn rows hang off the ends of the
        // clip, and a tracking area doesn't care about clipping — without
        // the intersection, the row scrolled up behind the search field
        // would still claim the pointer. `ActiveAlways` because the panel
        // is non-activating: the app is never frontmost, and the default
        // active-app-only modes would never fire.
        // Worked out from the frames rather than asked of `visibleRect`,
        // which wants the view to be in an on-screen window — the first
        // draw of a summon happens before the panel is ordered front.
        let clip = parent.bounds();
        let frame = row.frame();
        let lo = frame.origin.y.max(clip.origin.y);
        let hi = (frame.origin.y + ROW_H).min(clip.origin.y + clip.size.height);
        if hi > lo {
            let area: Retained<NSTrackingArea> = unsafe {
                NSTrackingArea::initWithRect_options_owner_userInfo(
                    mtm.alloc(),
                    NSRect::new(
                        NSPoint::new(0.0, lo - frame.origin.y),
                        NSSize::new(row_w, hi - lo),
                    ),
                    NSTrackingAreaOptions::NSTrackingMouseEnteredAndExited
                        | NSTrackingAreaOptions::NSTrackingActiveAlways,
                    Some(&row),
                    None,
                )
            };
            unsafe { row.addTrackingArea(&area) };
        }
    }

    /// Inline pill right after the name: SF Symbol icon + small label.
    /// Location tag on installed rows, "Shortcut" on command rows, "Setting"
    /// on builtin rows. `item_info_background` fills it (no outline);
    /// otherwise it keeps the stock hairline outline. `item_info_foreground`
    /// recolors text + icon.
    unsafe fn build_tag_pill(
        &self,
        mtm: MainThreadMarker,
        row: &NSView,
        pill_x: f64,
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
        let content_color = match style.item_info_foreground {
            Some(c) => rgba(c, if selected { 1.0 } else { 0.9 }),
            None => rgba(fg, alpha),
        };

        let font = unsafe { NSFont::systemFontOfSize(10.0) };
        let label = make_label(mtm, label_text, &font, &content_color);
        let label_size = label.frame().size;

        let icon_d = 11.0;
        let gap = 4.0;
        let pad_h = 7.0;
        let pill_h = 18.0;
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
                    NSPoint::new(pill_x, (ROW_H - pill_h) / 2.0),
                    NSSize::new(pill_w, pill_h),
                ),
            )
        };
        pill.setWantsLayer(true);
        if let Some(layer) = pill.layer() {
            layer.setCornerRadius(pill_h / 2.0);
            match style.item_info_background {
                Some(c) => set_layer_bg(&layer, &rgba(c, 1.0)),
                None => {
                    let border = rgba(fg, if selected { 0.35 } else { 0.20 });
                    unsafe {
                        let cg: *mut c_void = msg_send![&*border, CGColor];
                        let _: () = msg_send![&*layer, setBorderColor: cg];
                        let _: () = msg_send![&*layer, setBorderWidth: 1.0f64];
                    }
                }
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
                iv.setContentTintColor(Some(&content_color));
            }
            pill.addSubview(&iv);
            x += icon_d + gap;
        }
        label.setFrameOrigin(NSPoint::new(x, (pill_h - label_size.height) / 2.0));
        pill.addSubview(&label);
        row.addSubview(&pill);
    }

    /// Right-edge "⌘F"/"⌘N" badge for one row, shown only while Cmd is
    /// held. Same colored-box treatment as the sigil badge
    /// (`sigil_background`/`sigil_foreground`, falling back to
    /// `item_foreground_highlight`/`panel_background`) so it reads as a key
    /// hint rather than another status pill.
    unsafe fn build_hint_badge(&self, mtm: MainThreadMarker, row: &NSView, row_w: f64, hint: RowHint) {
        let cfg = self.ivars().config.borrow();
        let style = &cfg.style;
        let bg = style.sigil_background.unwrap_or(style.item_foreground_highlight);
        let fg = style.sigil_foreground.unwrap_or(style.panel_background);

        let text = match hint {
            RowHint::Digit(n) => format!("⌘{n}"),
            RowHint::Letter(c) => format!("⌘{c}"),
        };
        let font = unsafe { NSFont::systemFontOfSize(10.5) };
        let label = make_label(mtm, &text, &font, &rgba(fg, 1.0));
        let label_size = label.frame().size;

        let pad_h = 6.0;
        let badge_h = 17.0;
        let badge_w = label_size.width + 2.0 * pad_h;
        let badge = unsafe {
            NSView::initWithFrame(
                mtm.alloc(),
                NSRect::new(
                    NSPoint::new(row_w - 12.0 - badge_w, (ROW_H - badge_h) / 2.0),
                    NSSize::new(badge_w, badge_h),
                ),
            )
        };
        badge.setWantsLayer(true);
        if let Some(layer) = badge.layer() {
            layer.setCornerRadius(badge_h / 2.0);
            set_layer_bg(&layer, &rgba(bg, 1.0));
        }
        label.setFrameOrigin(NSPoint::new(pad_h, (badge_h - label_size.height) / 2.0));
        badge.addSubview(&label);
        row.addSubview(&badge);
    }

    /// A small filled dot with its right edge at `right_x`, vertically
    /// centered — the active marker on the theme picker's current row.
    unsafe fn build_running_dot(&self, mtm: MainThreadMarker, row: &NSView, right_x: f64) {
        const DOT_D: f64 = 7.0;
        // Pull the dot a hair left of the right edge so it lines up under the
        // pill column, and cuff it in translucent black so it reads as a
        // defined, slightly thicker shape against any background.
        const NUDGE: f64 = 2.0;
        let color = self.ivars().config.borrow().style.running_dot;
        let dot = unsafe {
            NSView::initWithFrame(
                mtm.alloc(),
                NSRect::new(
                    NSPoint::new(right_x - DOT_D - NUDGE, (ROW_H - DOT_D) / 2.0),
                    NSSize::new(DOT_D, DOT_D),
                ),
            )
        };
        dot.setWantsLayer(true);
        if let Some(layer) = dot.layer() {
            layer.setCornerRadius(DOT_D / 2.0);
            set_layer_bg(&layer, &rgba(color, 1.0));
            let cuff = rgba((0.0, 0.0, 0.0), 0.35);
            unsafe {
                let cg: *mut c_void = msg_send![&*cuff, CGColor];
                let _: () = msg_send![&*layer, setBorderColor: cg];
                let _: () = msg_send![&*layer, setBorderWidth: 1.0];
            }
        }
        row.addSubview(&dot);
    }

    /// CPU alert badge at the row's right edge (right edge at `right_x`):
    /// a filled pill holding a warning glyph, a bold "CPU", and a small load
    /// meter. No number — only the meter's fill width changes between
    /// samples, so nothing textual repaints every second. The only thing a
    /// running row draws at its right edge, once the tree crosses
    /// CPU_ALERT_PCT.
    unsafe fn build_cpu_warning(
        &self,
        mtm: MainThreadMarker,
        row: &NSView,
        right_x: f64,
        pct: f64,
        selected: bool,
    ) {
        let cfg = self.ivars().config.borrow();
        let style = &cfg.style;
        let color = style.cpu_alert;
        let alpha = if selected { 1.0 } else { 0.9 };
        let content_color = rgba(color, alpha);

        let icon_d = 11.0;
        let gap = 5.0;
        let pad_h = 8.0;
        let pill_h = 20.0;
        let meter_w = 24.0;
        let meter_h = 4.0;

        // Bold per spec: the word is the warning, the meter is the reading.
        let font: Retained<NSFont> = unsafe {
            msg_send_id![
                NSFont::class(),
                systemFontOfSize: 10.0f64,
                weight: 0.4f64 // NSFontWeightBold
            ]
        };
        let label = make_label(mtm, "CPU", &font, &content_color);
        let label_size = label.frame().size;

        let image = unsafe {
            objc2_app_kit::NSImage::imageWithSystemSymbolName_accessibilityDescription(
                &NSString::from_str("exclamationmark.triangle.fill"),
                None,
            )
        };
        let icon_w = if image.is_some() { icon_d + gap } else { 0.0 };
        let pill_w = pad_h + icon_w + label_size.width + gap + meter_w + pad_h;

        let pill = unsafe {
            NSView::initWithFrame(
                mtm.alloc(),
                NSRect::new(
                    NSPoint::new(right_x - pill_w, (ROW_H - pill_h) / 2.0),
                    NSSize::new(pill_w, pill_h),
                ),
            )
        };
        pill.setWantsLayer(true);
        if let Some(layer) = pill.layer() {
            layer.setCornerRadius(pill_h / 2.0);
            match style.cpu_alert_background {
                Some(c) => set_layer_bg(&layer, &rgba(c, 1.0)),
                None => set_layer_bg(&layer, &rgba(color, 0.16)),
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
                iv.setContentTintColor(Some(&content_color));
            }
            pill.addSubview(&iv);
            x += icon_d + gap;
        }
        label.setFrameOrigin(NSPoint::new(x, (pill_h - label_size.height) / 2.0));
        pill.addSubview(&label);
        x += label_size.width + gap;

        // Load meter: faint track, solid fill. 100% of the track = one full
        // core (Activity Monitor scale), clamped — the badge already only
        // exists past CPU_ALERT_PCT, so the meter reads 70%..full.
        let meter_y = (pill_h - meter_h) / 2.0;
        let track = unsafe {
            NSView::initWithFrame(
                mtm.alloc(),
                NSRect::new(NSPoint::new(x, meter_y), NSSize::new(meter_w, meter_h)),
            )
        };
        track.setWantsLayer(true);
        if let Some(layer) = track.layer() {
            layer.setCornerRadius(meter_h / 2.0);
            set_layer_bg(&layer, &rgba(color, 0.25));
        }
        pill.addSubview(&track);
        let fill_w = meter_w * (pct / 100.0).clamp(0.0, 1.0);
        let fill = unsafe {
            NSView::initWithFrame(
                mtm.alloc(),
                NSRect::new(NSPoint::new(x, meter_y), NSSize::new(fill_w, meter_h)),
            )
        };
        fill.setWantsLayer(true);
        if let Some(layer) = fill.layer() {
            layer.setCornerRadius(meter_h / 2.0);
            set_layer_bg(&layer, &rgba(color, alpha));
        }
        pill.addSubview(&fill);

        row.addSubview(&pill);
    }

    /// Run a `PanelMode::AppCommands` built-in row against the context
    /// app's `path`/`running` — carried on the selected row exactly like a
    /// normal app row, so this reuses the same resolution as `execute`.
    fn perform_app_action(&self, action: AppRowAction) {
        let entry_data = {
            let entries = self.ivars().entries.borrow();
            let Some(entry) = entries.get(self.ivars().selected.get()) else {
                return;
            };
            (entry.path.clone(), entry.running.clone())
        };
        self.hide();
        let (path, running) = entry_data;
        match action {
            AppRowAction::Open => {
                if let Some(p) = resolved_bundle_path(&path, &running) {
                    open_app_at_path(&p);
                } else if let Some(app) = &running {
                    unsafe {
                        app.activateWithOptions(
                            NSApplicationActivationOptions::NSApplicationActivateIgnoringOtherApps,
                        );
                    }
                }
            }
            AppRowAction::Focus => {
                if let Some(app) = &running {
                    unsafe {
                        app.activateWithOptions(
                            NSApplicationActivationOptions::NSApplicationActivateIgnoringOtherApps,
                        );
                    }
                }
            }
            AppRowAction::Reveal => {
                if let Some(p) = resolved_bundle_path(&path, &running) {
                    reveal_bundle_in_finder(&p);
                }
            }
            AppRowAction::Info => {
                if let Some(p) = resolved_bundle_path(&path, &running) {
                    show_finder_info(&p);
                }
            }
            AppRowAction::Close => {
                if let Some(app) = &running {
                    let _: bool = unsafe { app.terminate() };
                }
            }
            AppRowAction::Kill => {
                if let Some(app) = &running {
                    let _: bool = unsafe { app.forceTerminate() };
                }
            }
        }
    }

    /// Activate the selected running app, or launch it. `force_open` skips
    /// the activate path and sends a real open (reopen event) even when the
    /// app is already running.
    fn execute(&self, force_open: bool) {
        // An engine is a destination, not a launch: enter steps into it for
        // the terms, exactly as tab does.
        if let Some(engine) = self.selected_engine() {
            self.enter_search_engine(engine);
            return;
        }
        let app_action = {
            let entries = self.ivars().entries.borrow();
            entries.get(self.ivars().selected.get()).and_then(|e| e.app_action)
        };
        if let Some(action) = app_action {
            self.perform_app_action(action);
            return;
        }
        let builtin = {
            let entries = self.ivars().entries.borrow();
            entries
                .get(self.ivars().selected.get())
                .and_then(|e| e.builtin.clone())
        };
        if let Some(builtin) = builtin {
            match builtin {
                Builtin::ThemePicker => self.enter_theme_picker(),
                Builtin::ApplyTheme(name) => {
                    // Commit: the previewed style stays, in memory only — the
                    // config file is never written, so this lasts until the
                    // process restarts.
                    let ivars = self.ivars();
                    *ivars.session_theme.borrow_mut() = name;
                    *ivars.saved_style.borrow_mut() = None;
                    ivars.mode.set(PanelMode::Launcher);
                    self.hide();
                }
                Builtin::ModeRow(action) => {
                    self.hide();
                    match action {
                        modes::ModeAction::Copy(text) => copy_to_clipboard(&text),
                        modes::ModeAction::OpenUrl(url) => open_url(&url),
                    }
                }
            }
            return;
        }
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
        let launch_path = resolved_bundle_path(&path, &running);
        if let Some(p) = launch_path {
            open_app_at_path(&p);
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
        if let Some(p) = resolved_bundle_path(&path, &running) {
            reveal_bundle_in_finder(&p);
        }
    }
}

/// An entry's bundle path, falling back to a running app's own `bundleURL`
/// when it has no `path` of its own (running-app entries never carry one —
/// see `running_apps_impl`). Shared by `execute`, `reveal`, and every
/// `AppRowAction` that needs a bundle to act on.
fn resolved_bundle_path(
    path: &Option<PathBuf>,
    running: &Option<Retained<NSRunningApplication>>,
) -> Option<PathBuf> {
    path.clone().or_else(|| {
        running
            .as_ref()
            .and_then(|a| unsafe { a.bundleURL() })
            .and_then(|u| unsafe { u.path() }.map(|p| PathBuf::from(p.to_string())))
    })
}

/// Launch (or bring forward) the app bundle at `path` via `NSWorkspace`.
fn open_app_at_path(path: &std::path::Path) {
    let Some(s) = path.to_str() else {
        return;
    };
    let url = unsafe { NSURL::fileURLWithPath(&NSString::from_str(s)) };
    let config = unsafe { NSWorkspaceOpenConfiguration::configuration() };
    unsafe {
        NSWorkspace::sharedWorkspace()
            .openApplicationAtURL_configuration_completionHandler(&url, &config, None);
    }
}

/// Reveal the bundle at `path` in Finder (like `open --reveal`).
fn reveal_bundle_in_finder(path: &std::path::Path) {
    let Some(s) = path.to_str() else {
        return;
    };
    unsafe {
        let url = NSURL::fileURLWithPath(&NSString::from_str(s));
        let urls = objc2_foundation::NSArray::from_vec(vec![url]);
        let ws = NSWorkspace::sharedWorkspace();
        let _: () = msg_send![&*ws, activateFileViewerSelectingURLs: &*urls];
    }
}

/// Open Finder's "Get Info" window for the bundle at `path`. AppKit has no
/// direct API for this — it's a Finder UI action — so it goes through
/// AppleScript, same as the `[shortcuts]` entries that already shell out to
/// `osascript` for System Settings panes and the like.
fn show_finder_info(path: &std::path::Path) {
    let Some(s) = path.to_str() else {
        return;
    };
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!(
        "tell application \"Finder\" to open information window of (POSIX file \"{escaped}\" as alias)"
    );
    let _ = std::process::Command::new("/usr/bin/osascript").arg("-e").arg(script).spawn();
}

/// Copy via pbcopy: keeps NSPasteboard out of the binding surface, and the
/// panel is already hidden by the time this runs, so the wait is invisible.
fn copy_to_clipboard(text: &str) {
    use std::io::Write;
    let child = std::process::Command::new("/usr/bin/pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn();
    if let Ok(mut child) = child {
        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(text.as_bytes());
        }
        let _ = child.wait();
    }
}

/// Open a URL in the default browser.
fn open_url(url: &str) {
    unsafe {
        if let Some(u) = NSURL::URLWithString(&NSString::from_str(url)) {
            NSWorkspace::sharedWorkspace().openURL(&u);
        }
    }
}

/// `$XDG_CACHE_HOME/motherfucker/rates.json` (or `~/.cache/…`).
fn rates_cache_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))?;
    Some(base.join("motherfucker").join("rates.json"))
}

/// Read and parse the rate cache, tagging it with the file's age. Absent or
/// unreadable file → `None` (caller treats it as "no cache yet").
fn load_rates(path: &std::path::Path) -> Option<Rates> {
    let meta = std::fs::metadata(path).ok()?;
    let age_secs = meta
        .modified()
        .ok()
        .and_then(|m| m.elapsed().ok())
        .map(|d| d.as_secs());
    let text = std::fs::read_to_string(path).ok()?;
    Some(Rates { map: modes::parse_rates(&text), age_secs })
}

/// Refresh the rate cache off the main thread via `curl`, writing atomically
/// (temp file + rename). At most one fetch runs at a time; failures leave any
/// existing cache untouched. Never blocks the panel.
fn spawn_rate_fetch(path: PathBuf) {
    use std::sync::atomic::Ordering;
    if RATE_FETCHING.swap(true, Ordering::SeqCst) {
        return;
    }
    std::thread::spawn(move || {
        if let Some(dir) = path.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let tmp = path.with_extension("json.tmp");
        let ok = std::process::Command::new("/usr/bin/curl")
            .args(["-sfL", "--max-time", "10", RATE_URL, "-o"])
            .arg(&tmp)
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if ok {
            let _ = std::fs::rename(&tmp, &path);
        } else {
            let _ = std::fs::remove_file(&tmp);
        }
        RATE_FETCHING.store(false, Ordering::SeqCst);
    });
}

/// Running apps with a Dock presence (Regular activation policy), current
/// process excluded.
/// A running row's pid, or 0 for a row with no process behind it.
fn entry_pid(entry: &Entry) -> i32 {
    entry
        .running
        .as_ref()
        .map(|app| unsafe { app.processIdentifier() })
        .unwrap_or(0)
}

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
            builtin: None,
            detail: None,
            tag: None,
            icon: None,
            engine: None,
            app_action: None,
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

#[cfg(test)]
mod tests {
    use super::{engine_match, row_window, scroll_to_show, SearchEngine, ROW_H, SHORTCUT_SCORE};

    fn engine(name: &str, shortcut: &str) -> SearchEngine {
        SearchEngine {
            name: name.into(),
            query: "https://x.test/?q={q}".into(),
            icon: String::new(),
            shortcut: shortcut.into(),
        }
    }

    #[test]
    fn an_engine_is_found_by_name_or_by_its_whole_shortcut() {
        let yt = engine("YouTube", "yt");
        // The shortcut typed whole wins outright, and highlights nothing —
        // "yt" is not where the match is.
        let (score, matched) = engine_match("yt", &yt).unwrap();
        assert_eq!(score, SHORTCUT_SCORE);
        assert!(matched.is_empty());
        assert_eq!(engine_match("YT", &yt).unwrap().0, SHORTCUT_SCORE); // case-insensitive
        // A partial shortcut is not a shortcut hit; it falls back to the
        // name, which "y" does match.
        let (score, matched) = engine_match("y", &yt).unwrap();
        assert!(score < SHORTCUT_SCORE);
        assert_eq!(matched, vec![0]);
        // Name matching is the ordinary fuzzy kind.
        assert!(engine_match("tube", &yt).is_some());
        assert!(engine_match("zzz", &yt).is_none());
        // An engine with no shortcut can only be found by name — an empty
        // shortcut must never match an empty-ish query.
        let w = engine("Wikipedia", "");
        assert_eq!(engine_match("wiki", &w).unwrap().1, vec![0, 1, 2, 3]);
        assert!(engine_match("q", &w).is_none());
    }


    /// `(first, drawn, win, frac)` — the shape of the drawn window, without
    /// the clamped `px` (which the position tests below cover).
    fn shape(total: usize, max_rows: usize, px: f64) -> (usize, usize, usize, f64) {
        let w = row_window(total, max_rows, px);
        (w.first, w.drawn, w.win, w.frac)
    }

    #[test]
    fn window_is_aligned_when_the_offset_is() {
        // Shorter than the window: everything is drawn, nothing to scroll.
        assert_eq!(shape(3, 6, 0.0), (0, 3, 3, 0.0));
        // Longer: six rows, no partial one, until the offset moves.
        assert_eq!(shape(20, 6, 0.0), (0, 6, 6, 0.0));
        assert_eq!(shape(20, 6, 2.0 * ROW_H), (2, 6, 6, 0.0));
        // Scrolled to the very end — the last row sits in the last slot.
        assert_eq!(shape(20, 6, 14.0 * ROW_H), (14, 6, 6, 0.0));
    }

    #[test]
    fn a_partial_row_is_drawn_mid_scroll() {
        // Half a row down: the first row is cut at the top and a seventh
        // row is drawn peeking in at the bottom.
        assert_eq!(shape(20, 6, 0.5 * ROW_H), (0, 7, 6, ROW_H / 2.0));
        // A seven-row list has a seventh row to show, so it is drawn.
        assert_eq!(shape(7, 6, 0.5 * ROW_H), (0, 7, 6, ROW_H / 2.0));
        // Scrolled to the end there is nothing left to peek in with.
        assert_eq!(shape(7, 6, 999.0), (1, 6, 6, 0.0));
    }

    #[test]
    fn offset_is_clamped_to_the_list() {
        assert_eq!(row_window(20, 6, -80.0).px, 0.0);
        assert_eq!(row_window(20, 6, 9_999.0).px, 14.0 * ROW_H);
        // Nothing to scroll when everything fits.
        assert_eq!(row_window(4, 6, 500.0).px, 0.0);
        assert_eq!(shape(0, 6, 40.0), (0, 0, 0, 0.0));
    }

    #[test]
    fn full_rows_excludes_the_clipped_ones() {
        assert_eq!(row_window(20, 6, 2.0 * ROW_H).full_rows(), 2..8);
        // Mid-scroll the top row is cut off, so only five are whole.
        assert_eq!(row_window(20, 6, 2.5 * ROW_H).full_rows(), 3..8);
        // max_rows = 1 mid-scroll: no row is whole (callers fall back).
        assert!(row_window(20, 1, 0.5 * ROW_H).full_rows().is_empty());
    }

    #[test]
    fn scrolling_to_a_row_moves_as_little_as_possible() {
        // Already on screen: stay put.
        assert_eq!(scroll_to_show(20, 6, 3, 0.0), 0.0);
        // Off the bottom by one row -> exactly one row of travel.
        assert_eq!(scroll_to_show(20, 6, 6, 0.0), ROW_H);
        // Off the top -> the row becomes the first one.
        assert_eq!(scroll_to_show(20, 6, 2, 5.0 * ROW_H), 2.0 * ROW_H);
        // Wrapping either way parks the window at that end of the list.
        assert_eq!(scroll_to_show(20, 6, 19, 0.0), 14.0 * ROW_H);
        assert_eq!(scroll_to_show(20, 6, 0, 14.0 * ROW_H), 0.0);
        // Nothing to scroll when everything fits.
        assert_eq!(scroll_to_show(4, 6, 3, 0.0), 0.0);
        assert_eq!(scroll_to_show(0, 6, 0, 0.0), 0.0);
    }

    #[test]
    fn keyboard_tidies_up_a_half_scrolled_list() {
        // The selection is already visible, but the list is mid-row: the
        // keyboard snaps it back onto the grid rather than carrying the
        // offset along forever.
        assert_eq!(scroll_to_show(20, 6, 4, 2.4 * ROW_H), 2.0 * ROW_H);
        assert_eq!(scroll_to_show(20, 6, 4, 2.6 * ROW_H), 3.0 * ROW_H);
        // Snapping must not push the selection off: row 8 stays visible.
        assert_eq!(scroll_to_show(20, 6, 8, 2.6 * ROW_H), 3.0 * ROW_H);
        assert_eq!(scroll_to_show(20, 6, 9, 2.6 * ROW_H), 4.0 * ROW_H);
    }
}
