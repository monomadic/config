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
}

pub struct Style {
    pub width: f64,
    pub tint: (f64, f64, f64),
    pub tint_alpha: f64,
    pub selection: (f64, f64, f64),
    pub selection_alpha: f64,
    pub accent: (f64, f64, f64),
    pub input_font_size: f64,
    pub row_font_size: f64,
}

impl Default for Style {
    fn default() -> Self {
        Style {
            width: 620.0,
            tint: (0.0, 0.0, 0.0),
            tint_alpha: 0.70,
            selection: (0.60, 0.70, 0.90),
            selection_alpha: 0.085,
            accent: (0.38, 0.75, 1.0),
            input_font_size: 24.0,
            row_font_size: 13.5,
        }
    }
}

pub struct Config {
    /// Global summon triggers, in file order. Hotkey id N (1-based) maps to
    /// index N-1 here.
    pub hotkeys: Vec<(Chord, Mode)>,
    pub binds: Vec<(Chord, Action)>,
    pub style: Style,
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
    let path = base.join("motherfucker/config.toml");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return cfg; // no file: defaults
    };
    parse_into(&mut cfg, &text);
    cfg
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
            "style" => apply_style(&mut cfg.style, &key, &val, &line),
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
        "tint_alpha" => match num() {
            Ok(v) => style.tint_alpha = v.clamp(0.0, 1.0),
            Err(_) => warn(line, "expected a number"),
        },
        "selection_alpha" => match num() {
            Ok(v) => style.selection_alpha = v.clamp(0.0, 1.0),
            Err(_) => warn(line, "expected a number"),
        },
        "input_font_size" => match num() {
            Ok(v) => style.input_font_size = v.clamp(9.0, 64.0),
            Err(_) => warn(line, "expected a number"),
        },
        "row_font_size" => match num() {
            Ok(v) => style.row_font_size = v.clamp(8.0, 32.0),
            Err(_) => warn(line, "expected a number"),
        },
        "tint" => match parse_color(val) {
            Some(c) => style.tint = c,
            None => warn(line, "expected \"#rrggbb\""),
        },
        "selection" => match parse_color(val) {
            Some(c) => style.selection = c,
            None => warn(line, "expected \"#rrggbb\""),
        },
        "accent" => match parse_color(val) {
            Some(c) => style.accent = c,
            None => warn(line, "expected \"#rrggbb\""),
        },
        _ => warn(line, "unknown style key"),
    }
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
accent = "#ff8800"
tint_alpha = 0.5
width = 700

[stats]
interval = 2.0
"##,
        );
        assert!((cfg.style.accent.0 - 1.0).abs() < 1e-9);
        assert!((cfg.style.tint_alpha - 0.5).abs() < 1e-9);
        assert!((cfg.style.width - 700.0).abs() < 1e-9);
        assert!((cfg.stats_interval - 2.0).abs() < 1e-9);
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
