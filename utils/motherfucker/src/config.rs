//! User configuration: `~/.config/motherfucker/config.toml`.
//!
//! Read once at process start (one small file read — restart the agent to
//! apply changes: `setup/install/motherfucker.sh` or `launchctl kickstart -k
//! gui/$UID/com.nom.motherfucker`). Parsed with a hand-rolled TOML-subset
//! parser — sections and `key = value` lines only — to keep the binary
//! dependency-free. Every field has a built-in default; a missing or
//! malformed file just means defaults, never a crash.

use crate::hotkey;

/// A key with no modifier semantics of its own.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    Char(char),
    Space,
    Enter,
    Escape,
    Tab,
    Up,
    Down,
    Left,
    Right,
    Backspace,
}

/// Modifier + key combination, usable both as a global hotkey (Carbon) and
/// an in-panel binding (NSEvent matching).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Chord {
    pub cmd: bool,
    pub ctrl: bool,
    pub opt: bool,
    pub shift: bool,
    pub key: Key,
}

impl Chord {
    const fn plain(key: Key) -> Self {
        Chord { cmd: false, ctrl: false, opt: false, shift: false, key }
    }
}

/// What a global trigger summons. One mode today; the config format already
/// carries a mode name per hotkey so new modes are additive.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Launcher,
}

/// In-panel actions bindable to chords.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Open,
    LaunchNew,
    Reveal,
    Clear,
    Dismiss,
    SelectAll,
    MoveUp,
    MoveDown,
    RefreshConfig,
}

#[derive(Clone)]
pub struct Style {
    pub width: f64,
    pub panel_background: (f64, f64, f64),
    pub panel_foreground: (f64, f64, f64),
    pub panel_opacity: f64,
    pub panel_padding: f64,
    pub panel_corner_radius: f64,
    pub item_foreground: (f64, f64, f64),
    pub item_font_size: f64,
    pub selected_item_background: (f64, f64, f64),
    pub selected_item_foreground: (f64, f64, f64),
    pub selected_item_opacity: f64,
    pub selected_item_corner_radius: f64,
    pub item_foreground_highlight: (f64, f64, f64),
    pub selected_item_foreground_highlight: (f64, f64, f64),
    pub input_font_size: f64,
    pub cpu_alert: (f64, f64, f64),
    pub running_dot: (f64, f64, f64),
    /// Font family for input + row text; empty = system font.
    pub font_family: String,
    /// NSFontWeight (-1.0..1.0) for row item text; ignored when a custom
    /// `font_family` is set (name a weighted variant instead).
    pub item_font_weight: f64,
    /// Glyph column + search-icon color; `None` = row/panel foreground.
    pub icon_foreground: Option<(f64, f64, f64)>,
    /// Stroke around the whole panel; width 0 (the default) = no border.
    pub border: (f64, f64, f64),
    pub border_width: f64,
    /// Tag-pill text ("Applications", "Utilities", "Shortcut");
    /// `None` = row foreground at the stock alpha.
    pub item_info_foreground: Option<(f64, f64, f64)>,
    /// Tag-pill fill; `None` = clear with the stock hairline outline.
    pub item_info_background: Option<(f64, f64, f64)>,
    /// Inset stroke on the selected row; width 0 (the default) = none.
    pub selected_item_border: (f64, f64, f64),
    pub selected_item_border_width: f64,
    /// Fill behind the CPU warning badge; `None` = `cpu_alert` at 0.16.
    pub cpu_alert_background: Option<(f64, f64, f64)>,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            width: 620.0,
            panel_background: (0.0, 0.0, 0.0),
            panel_foreground: (1.0, 1.0, 1.0),
            panel_opacity: 0.70,
            panel_padding: 10.0,
            panel_corner_radius: 18.0,
            item_foreground: (1.0, 1.0, 1.0),
            item_font_size: 13.5,
            selected_item_background: (0.60, 0.70, 0.90),
            selected_item_foreground: (1.0, 1.0, 1.0),
            selected_item_opacity: 0.085,
            selected_item_corner_radius: 8.0,
            item_foreground_highlight: (0.38, 0.75, 1.0),
            selected_item_foreground_highlight: (0.38, 0.75, 1.0),
            input_font_size: 24.0,
            cpu_alert: (0.94, 0.33, 0.31),
            running_dot: (0.30, 0.80, 0.39),
            font_family: String::new(),
            item_font_weight: 0.0,
            icon_foreground: None,
            border: (1.0, 1.0, 1.0),
            border_width: 0.0,
            item_info_foreground: None,
            item_info_background: None,
            selected_item_border: (1.0, 1.0, 1.0),
            selected_item_border_width: 0.0,
            cpu_alert_background: None,
        }
    }
}

/// Map a named weight to its NSFontWeight value, or parse a raw number in
/// -1.0..1.0. Names match Apple's `NSFontWeight*` constants.
pub fn parse_weight(s: &str) -> Option<f64> {
    let w = match s.trim().to_lowercase().as_str() {
        "ultralight" => -0.8,
        "thin" => -0.6,
        "light" => -0.4,
        "regular" | "normal" => 0.0,
        "medium" => 0.23,
        "semibold" => 0.3,
        "bold" => 0.4,
        "heavy" => 0.56,
        "black" => 0.62,
        other => other.parse::<f64>().ok()?.clamp(-1.0, 1.0),
    };
    Some(w)
}

/// Customizable glyphs. `search` and the four row-state glyphs are literal
/// strings (SF Symbols pasted as text); the location entries are SF Symbol
/// *names* rendered through NSImage for the installed-row location tag.
pub struct Icons {
    pub search: String,
    pub running_many: String,
    pub running_one: String,
    pub running_none: String,
    pub installed: String,
    pub utilities: String,
    pub system: String,
    pub applications: String,
    pub shortcut: String,
}

impl Default for Icons {
    fn default() -> Self {
        Icons {
            search: "⌕".into(),
            running_many: "\u{10088C}".into(), // running, 2+ windows
            running_one: "\u{1003DC}".into(),  // running, one window
            running_none: "\u{100941}".into(), // running, no windows
            installed: "\u{100943}".into(),    // installed, launchable
            utilities: "wrench.and.screwdriver".into(),
            system: "gearshape".into(),
            applications: "app.fill".into(),
            shortcut: "terminal".into(),
        }
    }
}

/// A named `[style]` overlay loaded from `themes/<name>.toml`. Overrides are
/// kept as raw key/value pairs and replayed through `apply_style`, so a theme
/// file speaks exactly the `[style]` grammar — nothing new to parse.
#[derive(Clone)]
pub struct Theme {
    /// File stem, e.g. "ocean-breeze".
    pub name: String,
    pub overrides: Vec<(String, String)>,
}

/// Display name for a theme: file stem with `-`/`_` as spaces, Title Case
/// ("ocean-breeze" → "Ocean Breeze").
pub fn theme_display_name(name: &str) -> String {
    name.split(['-', '_'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Overlay a theme onto a base style.
pub fn apply_theme(style: &mut Style, theme: &Theme) {
    for (key, val) in &theme.overrides {
        apply_style(style, key, val, key);
    }
}

pub struct Config {
    /// Global summon triggers, in file order. Hotkey id N (1-based) maps to
    /// index N-1 here.
    pub hotkeys: Vec<(Chord, Mode)>,
    pub binds: Vec<(Chord, Action)>,
    /// Base style from `[style]` — never includes a theme overlay; the
    /// active (possibly themed) style is owned by the UI layer.
    pub style: Style,
    /// Themes found in `themes/*.toml`, sorted by name.
    pub themes: Vec<Theme>,
    /// `theme = "name"` from `[style]`: overlay to apply at startup.
    pub theme: Option<String>,
    pub icons: Icons,
    /// Per-name glyph overrides from `[icons.apps]`: (lowercased entry
    /// name, glyph). Replaces the leading state glyph for matching rows.
    pub icon_overrides: Vec<(String, String)>,
    /// `[shortcuts]`: (display name, shell command). Matched like apps;
    /// selecting one runs the command via `sh -c`.
    pub shortcuts: Vec<(String, String)>,
    /// Seconds between stat refreshes while the panel is up.
    pub stats_interval: f64,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            hotkeys: vec![(
                Chord { opt: true, ..Chord::plain(Key::Space) },
                Mode::Launcher,
            )],
            binds: default_binds(),
            style: Style::default(),
            themes: Vec::new(),
            theme: None,
            icons: Icons::default(),
            icon_overrides: Vec::new(),
            shortcuts: Vec::new(),
            stats_interval: 1.0,
        }
    }
}

fn default_binds() -> Vec<(Chord, Action)> {
    let c = Chord::plain;
    vec![
        (c(Key::Enter), Action::Open),
        (Chord { cmd: true, ..c(Key::Enter) }, Action::LaunchNew),
        (Chord { cmd: true, ..c(Key::Char('r')) }, Action::Reveal),
        (Chord { ctrl: true, ..c(Key::Char('u')) }, Action::Clear),
        (Chord { ctrl: true, ..c(Key::Char('c')) }, Action::Dismiss),
        (Chord { cmd: true, ..c(Key::Char('a')) }, Action::SelectAll),
        (c(Key::Escape), Action::Dismiss),
        (c(Key::Up), Action::MoveUp),
        (c(Key::Down), Action::MoveDown),
        (
            Chord { cmd: true, shift: true, ..c(Key::Char('r')) },
            Action::RefreshConfig,
        ),
    ]
}

/// Carbon virtual keycode for a key (global hotkeys). ANSI layout.
pub fn carbon_vk(key: Key) -> Option<u32> {
    Some(match key {
        Key::Space => 0x31,
        Key::Enter => 0x24,
        Key::Escape => 0x35,
        Key::Tab => 0x30,
        Key::Up => 0x7E,
        Key::Down => 0x7D,
        Key::Left => 0x7B,
        Key::Right => 0x7C,
        Key::Backspace => 0x33,
        Key::Char(c) => match c {
            'a' => 0x00, 's' => 0x01, 'd' => 0x02, 'f' => 0x03, 'h' => 0x04,
            'g' => 0x05, 'z' => 0x06, 'x' => 0x07, 'c' => 0x08, 'v' => 0x09,
            'b' => 0x0B, 'q' => 0x0C, 'w' => 0x0D, 'e' => 0x0E, 'r' => 0x0F,
            'y' => 0x10, 't' => 0x11, '1' => 0x12, '2' => 0x13, '3' => 0x14,
            '4' => 0x15, '6' => 0x16, '5' => 0x17, '=' => 0x18, '9' => 0x19,
            '7' => 0x1A, '-' => 0x1B, '8' => 0x1C, '0' => 0x1D, ']' => 0x1E,
            'o' => 0x1F, 'u' => 0x20, '[' => 0x21, 'i' => 0x22, 'p' => 0x23,
            'l' => 0x25, 'j' => 0x26, '\'' => 0x27, 'k' => 0x28, ';' => 0x29,
            '\\' => 0x2A, ',' => 0x2B, '/' => 0x2C, 'n' => 0x2D, 'm' => 0x2E,
            '.' => 0x2F, '`' => 0x32,
            _ => return None,
        },
    })
}

pub fn carbon_mods(chord: &Chord) -> u32 {
    let mut m = 0;
    if chord.cmd {
        m |= hotkey::MOD_CMD;
    }
    if chord.shift {
        m |= hotkey::MOD_SHIFT;
    }
    if chord.opt {
        m |= hotkey::MOD_OPTION;
    }
    if chord.ctrl {
        m |= hotkey::MOD_CONTROL;
    }
    m
}

/// What `charactersIgnoringModifiers` reports for a key (in-panel matching).
/// Compared lowercased; shift is carried by the modifier flags.
pub fn event_chars(key: Key) -> String {
    match key {
        Key::Char(c) => return c.to_lowercase().to_string(),
        Key::Space => " ",
        Key::Enter => "\r",
        Key::Escape => "\u{1b}",
        Key::Tab => "\t",
        Key::Up => "\u{f700}",
        Key::Down => "\u{f701}",
        Key::Left => "\u{f702}",
        Key::Right => "\u{f703}",
        Key::Backspace => "\u{7f}",
    }
    .to_string()
}

pub fn load() -> Config {
    let mut cfg = Config::default();
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config")));
    let Some(base) = base else {
        return cfg;
    };
    let dir = base.join("motherfucker");
    if let Ok(text) = std::fs::read_to_string(dir.join("config.toml")) {
        parse_into(&mut cfg, &text);
    }
    cfg.themes = load_themes(&dir.join("themes"));
    cfg
}

/// `themes/*.toml`, each a `[style]` overlay; name = file stem. Read at
/// startup and on refresh-config only — never on the summon path.
fn load_themes(dir: &std::path::Path) -> Vec<Theme> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut themes: Vec<Theme> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
            continue;
        };
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        themes.push(Theme {
            name: name.to_string(),
            overrides: parse_theme(&text),
        });
    }
    themes.sort_by(|a, b| a.name.cmp(&b.name));
    themes
}

/// `[style]` key/value pairs from a theme file. Keys are validated lazily —
/// bad ones warn when the theme is applied, exactly like config.toml lines.
fn parse_theme(text: &str) -> Vec<(String, String)> {
    let mut section = String::new();
    let mut overrides = Vec::new();
    for raw in text.lines() {
        let line = strip_comment(raw).trim().to_string();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_lowercase();
            continue;
        }
        if section != "style" {
            continue;
        }
        if let Some((lhs, rhs)) = line.split_once('=') {
            overrides.push((unquote(lhs.trim()), unquote(rhs.trim())));
        }
    }
    overrides
}

fn warn(line: &str, what: &str) {
    eprintln!("motherfucker: config: {what}: `{line}`");
}

fn parse_into(cfg: &mut Config, text: &str) {
    let mut section = String::new();
    let mut hotkeys: Vec<(Chord, Mode)> = Vec::new();
    for raw in text.lines() {
        let line = strip_comment(raw).trim().to_string();
        if line.is_empty() {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = line[1..line.len() - 1].trim().to_lowercase();
            continue;
        }
        let Some((lhs, rhs)) = line.split_once('=') else {
            warn(&line, "expected `key = value`");
            continue;
        };
        let key = unquote(lhs.trim());
        let val = unquote(rhs.trim());
        match section.as_str() {
            "hotkeys" => {
                let Some(chord) = parse_chord(&key) else {
                    warn(&line, "unrecognized hotkey chord");
                    continue;
                };
                if carbon_vk(chord.key).is_none() {
                    warn(&line, "key has no global-hotkey keycode");
                    continue;
                }
                let Some(mode) = parse_mode(&val) else {
                    warn(&line, "unknown mode");
                    continue;
                };
                hotkeys.push((chord, mode));
            }
            "keys" => {
                let Some(chord) = parse_chord(&key) else {
                    warn(&line, "unrecognized chord");
                    continue;
                };
                // Later entries (user config) override defaults for the
                // same chord; "none" unbinds it.
                cfg.binds.retain(|(c, _)| *c != chord);
                if val.eq_ignore_ascii_case("none") {
                    continue;
                }
                let Some(action) = parse_action(&val) else {
                    warn(&line, "unknown action");
                    continue;
                };
                cfg.binds.push((chord, action));
            }
            "style" if key.replace('-', "_") == "theme" => {
                cfg.theme = (!val.is_empty()).then(|| val.clone());
            }
            "style" => apply_style(&mut cfg.style, &key, &val, &line),
            "icons" => apply_icon(&mut cfg.icons, &key, &val, &line),
            "icons.apps" => {
                let name = key.to_lowercase();
                cfg.icon_overrides.retain(|(n, _)| *n != name);
                cfg.icon_overrides.push((name, val));
            }
            "shortcuts" => {
                cfg.shortcuts.retain(|(n, _)| !n.eq_ignore_ascii_case(&key));
                cfg.shortcuts.push((key, val));
            }
            "stats" => {
                if key.replace('-', "_") == "interval" {
                    match val.parse::<f64>() {
                        Ok(v) => cfg.stats_interval = v.clamp(0.3, 60.0),
                        Err(_) => warn(&line, "expected a number of seconds"),
                    }
                } else {
                    warn(&line, "unknown stats key");
                }
            }
            _ => warn(&line, "unknown section"),
        }
    }
    if !hotkeys.is_empty() {
        cfg.hotkeys = hotkeys;
    }
}

fn apply_style(style: &mut Style, key: &str, val: &str, line: &str) {
    let num = || val.parse::<f64>();
    match key.replace('-', "_").as_str() {
        "width" => match num() {
            Ok(v) => style.width = v.clamp(320.0, 1600.0),
            Err(_) => warn(line, "expected a number"),
        },
        "panel_opacity" => match num() {
            Ok(v) => style.panel_opacity = v.clamp(0.0, 1.0),
            Err(_) => warn(line, "expected a number"),
        },
        "panel_padding" => match num() {
            Ok(v) => style.panel_padding = v.clamp(0.0, 100.0),
            Err(_) => warn(line, "expected a number"),
        },
        "panel_corner_radius" => match num() {
            Ok(v) => style.panel_corner_radius = v.clamp(0.0, 60.0),
            Err(_) => warn(line, "expected a number"),
        },
        "selected_item_opacity" => match num() {
            Ok(v) => style.selected_item_opacity = v.clamp(0.0, 1.0),
            Err(_) => warn(line, "expected a number"),
        },
        "selected_item_corner_radius" => match num() {
            Ok(v) => style.selected_item_corner_radius = v.clamp(0.0, 40.0),
            Err(_) => warn(line, "expected a number"),
        },
        "input_font_size" => match num() {
            Ok(v) => style.input_font_size = v.clamp(9.0, 64.0),
            Err(_) => warn(line, "expected a number"),
        },
        "item_font_size" => match num() {
            Ok(v) => style.item_font_size = v.clamp(8.0, 32.0),
            Err(_) => warn(line, "expected a number"),
        },
        "panel_background" => match parse_color(val) {
            Some(c) => style.panel_background = c,
            None => warn(line, "expected \"#rrggbb\""),
        },
        "panel_foreground" => match parse_color(val) {
            Some(c) => style.panel_foreground = c,
            None => warn(line, "expected \"#rrggbb\""),
        },
        "item_foreground" => match parse_color(val) {
            Some(c) => style.item_foreground = c,
            None => warn(line, "expected \"#rrggbb\""),
        },
        "cpu_alert" => match parse_color(val) {
            Some(c) => style.cpu_alert = c,
            None => warn(line, "expected \"#rrggbb\""),
        },
        "running_dot" => match parse_color(val) {
            Some(c) => style.running_dot = c,
            None => warn(line, "expected \"#rrggbb\""),
        },
        "font_family" => style.font_family = val.to_string(),
        "item_font_weight" => match parse_weight(val) {
            Some(w) => style.item_font_weight = w,
            None => warn(line, "expected a weight name or -1.0..1.0"),
        },
        "selected_item_background" => match parse_color(val) {
            Some(c) => style.selected_item_background = c,
            None => warn(line, "expected \"#rrggbb\""),
        },
        "selected_item_foreground" => match parse_color(val) {
            Some(c) => style.selected_item_foreground = c,
            None => warn(line, "expected \"#rrggbb\""),
        },
        "item_foreground_highlight" => match parse_color(val) {
            Some(c) => style.item_foreground_highlight = c,
            None => warn(line, "expected \"#rrggbb\""),
        },
        "selected_item_foreground_highlight" => match parse_color(val) {
            Some(c) => style.selected_item_foreground_highlight = c,
            None => warn(line, "expected \"#rrggbb\""),
        },
        "icon_foreground" => match parse_color(val) {
            Some(c) => style.icon_foreground = Some(c),
            None => warn(line, "expected \"#rrggbb\""),
        },
        "border" => match parse_color(val) {
            Some(c) => style.border = c,
            None => warn(line, "expected \"#rrggbb\""),
        },
        "border_width" => match num() {
            Ok(v) => style.border_width = v.clamp(0.0, 12.0),
            Err(_) => warn(line, "expected a number"),
        },
        "item_info_foreground" => match parse_color(val) {
            Some(c) => style.item_info_foreground = Some(c),
            None => warn(line, "expected \"#rrggbb\""),
        },
        "item_info_background" => match parse_color(val) {
            Some(c) => style.item_info_background = Some(c),
            None => warn(line, "expected \"#rrggbb\""),
        },
        "selected_item_border" => match parse_color(val) {
            Some(c) => style.selected_item_border = c,
            None => warn(line, "expected \"#rrggbb\""),
        },
        "selected_item_border_width" => match num() {
            Ok(v) => style.selected_item_border_width = v.clamp(0.0, 6.0),
            Err(_) => warn(line, "expected a number"),
        },
        "cpu_alert_background" => match parse_color(val) {
            Some(c) => style.cpu_alert_background = Some(c),
            None => warn(line, "expected \"#rrggbb\""),
        },
        _ => warn(line, "unknown style key"),
    }
}

fn apply_icon(icons: &mut Icons, key: &str, val: &str, line: &str) {
    let slot = match key.replace('-', "_").as_str() {
        "search" => &mut icons.search,
        "running_many" => &mut icons.running_many,
        "running_one" => &mut icons.running_one,
        "running_none" => &mut icons.running_none,
        "installed" => &mut icons.installed,
        "utilities" => &mut icons.utilities,
        "system" => &mut icons.system,
        "applications" => &mut icons.applications,
        "shortcut" => &mut icons.shortcut,
        _ => {
            warn(line, "unknown icon key");
            return;
        }
    };
    *slot = val.to_string();
}

/// Drop a trailing `# comment`, respecting double-quoted strings (colors
/// like "#aabbcc" live inside quotes).
fn strip_comment(line: &str) -> &str {
    let mut in_str = false;
    for (i, ch) in line.char_indices() {
        match ch {
            '"' => in_str = !in_str,
            '#' if !in_str => return &line[..i],
            _ => {}
        }
    }
    line
}

fn unquote(s: &str) -> String {
    let s = s.trim();
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

/// "cmd+shift+r", "opt+space", "enter" → Chord. Case-insensitive.
fn parse_chord(s: &str) -> Option<Chord> {
    let mut chord = Chord::plain(Key::Space);
    let mut key: Option<Key> = None;
    for part in s.split('+') {
        let p = part.trim().to_lowercase();
        match p.as_str() {
            "cmd" | "command" | "super" => chord.cmd = true,
            "ctrl" | "control" => chord.ctrl = true,
            "opt" | "option" | "alt" => chord.opt = true,
            "shift" => chord.shift = true,
            "space" => key = Some(Key::Space),
            "enter" | "return" => key = Some(Key::Enter),
            "escape" | "esc" => key = Some(Key::Escape),
            "tab" => key = Some(Key::Tab),
            "up" => key = Some(Key::Up),
            "down" => key = Some(Key::Down),
            "left" => key = Some(Key::Left),
            "right" => key = Some(Key::Right),
            "backspace" | "delete" => key = Some(Key::Backspace),
            _ => {
                let mut chars = p.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => key = Some(Key::Char(c)),
                    _ => return None,
                }
            }
        }
    }
    chord.key = key?;
    Some(chord)
}

fn parse_mode(s: &str) -> Option<Mode> {
    match s.to_lowercase().as_str() {
        "launcher" => Some(Mode::Launcher),
        _ => None,
    }
}

fn parse_action(s: &str) -> Option<Action> {
    match s.to_lowercase().replace('_', "-").as_str() {
        "open" => Some(Action::Open),
        "launch-new" | "open-new" | "force-open" => Some(Action::LaunchNew),
        "reveal" => Some(Action::Reveal),
        "clear" => Some(Action::Clear),
        "dismiss" | "close" | "hide" => Some(Action::Dismiss),
        "select-all" => Some(Action::SelectAll),
        "move-up" => Some(Action::MoveUp),
        "move-down" => Some(Action::MoveDown),
        "refresh-config" | "reload-config" => Some(Action::RefreshConfig),
        _ => None,
    }
}

/// "#rrggbb" → sRGB floats.
fn parse_color(s: &str) -> Option<(f64, f64, f64)> {
    let hex = s.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some((
        byte(0)? as f64 / 255.0,
        byte(2)? as f64 / 255.0,
        byte(4)? as f64 / 255.0,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_survive_missing_sections() {
        let mut cfg = Config::default();
        parse_into(&mut cfg, "");
        assert_eq!(cfg.hotkeys.len(), 1);
        assert!(!cfg.binds.is_empty());
    }

    #[test]
    fn parses_hotkeys_and_keys() {
        let mut cfg = Config::default();
        parse_into(
            &mut cfg,
            r#"
[hotkeys]
"cmd+space" = "launcher"
"opt+space" = "launcher"

[keys]
"cmd+r" = "reveal"   # comment
"ctrl+c" = "none"
"#,
        );
        assert_eq!(cfg.hotkeys.len(), 2);
        assert!(cfg.hotkeys[0].0.cmd && !cfg.hotkeys[0].0.opt);
        // ctrl+c unbound; cmd+r still bound exactly once
        let ctrl_c = Chord { ctrl: true, ..Chord::plain(Key::Char('c')) };
        assert!(!cfg.binds.iter().any(|(c, _)| *c == ctrl_c));
        let cmd_r = Chord { cmd: true, ..Chord::plain(Key::Char('r')) };
        assert_eq!(
            cfg.binds.iter().filter(|(c, _)| *c == cmd_r).count(),
            1
        );
    }

    #[test]
    fn parses_style_and_stats() {
        let mut cfg = Config::default();
        parse_into(
            &mut cfg,
            r##"
[style]
item_foreground_highlight = "#ff8800"
panel_opacity = 0.5
width = 700
cpu_alert = "#ff0000"
running_dot = "#00ff00"
item_font_weight = "semibold"

[icons]
search = "*"
running_many = "M"

[icons.apps]
"Forklift" = "F"

[shortcuts]
"Movies" = "open -R ~/Movies"

[stats]
interval = 2.0
"##,
        );
        assert!((cfg.style.item_foreground_highlight.0 - 1.0).abs() < 1e-9);
        assert_eq!(cfg.style.cpu_alert, (1.0, 0.0, 0.0));
        assert_eq!(cfg.style.running_dot, (0.0, 1.0, 0.0));
        assert!((cfg.style.item_font_weight - 0.3).abs() < 1e-9);
        assert!((cfg.style.panel_opacity - 0.5).abs() < 1e-9);
        assert!((cfg.style.width - 700.0).abs() < 1e-9);
        assert!((cfg.stats_interval - 2.0).abs() < 1e-9);
        assert_eq!(cfg.icons.search, "*");
        assert_eq!(cfg.icons.running_many, "M");
        assert_eq!(cfg.icons.installed, "\u{100943}"); // untouched default
        assert_eq!(cfg.icon_overrides, vec![("forklift".to_string(), "F".to_string())]);
        assert_eq!(
            cfg.shortcuts,
            vec![("Movies".to_string(), "open -R ~/Movies".to_string())]
        );
    }

    #[test]
    fn parses_theme_keys_and_overlay() {
        let mut cfg = Config::default();
        parse_into(
            &mut cfg,
            r##"
[style]
theme = "ocean-breeze"
icon_foreground = "#59c8e8"
border = "#102030"
border_width = 1
item_info_foreground = "#a8dff2"
item_info_background = "#0b2e52"
selected_item_border = "#1e4e74"
selected_item_border_width = 1
cpu_alert_background = "#401010"
"##,
        );
        assert_eq!(cfg.theme.as_deref(), Some("ocean-breeze"));
        assert!(cfg.style.icon_foreground.is_some());
        assert!((cfg.style.border_width - 1.0).abs() < 1e-9);
        assert!(cfg.style.item_info_background.is_some());
        assert!((cfg.style.selected_item_border_width - 1.0).abs() < 1e-9);
        assert!(cfg.style.cpu_alert_background.is_some());

        // A theme overlays the base style; untouched keys survive.
        let theme = Theme {
            name: "test".into(),
            overrides: parse_theme(
                "# comment\n[style]\npanel_background = \"#041028\"\nborder_width = 2\n",
            ),
        };
        let mut style = cfg.style.clone();
        apply_theme(&mut style, &theme);
        assert_eq!(
            style.panel_background,
            (4.0 / 255.0, 16.0 / 255.0, 40.0 / 255.0)
        );
        assert!((style.border_width - 2.0).abs() < 1e-9);
        assert!(style.item_info_background.is_some()); // untouched
    }

    #[test]
    fn theme_display_names() {
        assert_eq!(theme_display_name("ocean-breeze"), "Ocean Breeze");
        assert_eq!(theme_display_name("vivid_nightfall"), "Vivid Nightfall");
        assert_eq!(theme_display_name("candy"), "Candy");
    }

    #[test]
    fn color_in_quotes_is_not_a_comment() {
        assert_eq!(
            strip_comment(r##"accent = "#aabbcc" # note"##),
            r##"accent = "#aabbcc" "##
        );
    }

    #[test]
    fn chord_roundtrip() {
        let c = parse_chord("cmd+shift+enter").unwrap();
        assert!(c.cmd && c.shift && c.key == Key::Enter);
        assert!(parse_chord("cmd+").is_none());
        assert!(parse_chord("meta+x").is_none());
    }
}
