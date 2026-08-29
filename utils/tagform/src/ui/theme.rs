//! One place for every colour (SPEC §7).
//!
//! True colour throughout rather than the 16-colour palette. The old rendering
//! leaned on `DarkGray` for anything secondary, which on a dark terminal is
//! very nearly the background -- long custom keys like
//! `com.apple.quicktime.creationdate` rendered as blank labels beside floating
//! values. Everything here is an explicit RGB with a checked contrast against
//! the page, so "subdued" never means "invisible".

use ratatui::style::Color;
use std::sync::atomic::{AtomicUsize, Ordering};

/// A complete set of colours. Everything the UI draws comes from one of these,
/// so adding a scheme is adding a row here and nothing else.
pub struct Palette {
    pub name: &'static str,
    /// The terminal background this scheme assumes.
    ///
    /// Deliberately never painted: the terminal's own background shows through,
    /// which is what keeps a translucent terminal translucent. It exists so the
    /// contrast test knows what the text actually lands on, and it is why every
    /// scheme here is a dark one.
    #[allow(dead_code)]
    pub page: Color,

    pub header_bg: Color,
    pub badge_bg: Color,
    pub badge_fg: Color,
    pub header_fg: Color,

    /// The editable region of a field, in its four states.
    pub input_bg: Color,
    pub input_bg_focus: Color,
    pub input_bg_edit: Color,
    pub input_bg_readonly: Color,

    pub label: Color,
    pub label_focus: Color,
    /// A different *hue* from `label`, not a dimmer shade of it.
    pub label_custom: Color,

    pub value: Color,
    pub value_empty: Color,
    pub mixed: Color,

    pub accent: Color,
    pub staged: Color,
    pub warn: Color,
    pub error: Color,
    pub muted: Color,
    /// Rules and dividers. The one colour here that never draws text.
    pub rule: Color,
    pub star: Color,
    pub path: Color,
}

const fn c(r: u8, g: u8, b: u8) -> Color {
    Color::Rgb(r, g, b)
}

pub static PALETTES: &[Palette] = &[
    Palette {
        name: "midnight",
        page: c(0x0d, 0x0f, 0x14),
        header_bg: c(0x22, 0x26, 0x33),
        badge_bg: c(0x7a, 0xa2, 0xf7),
        badge_fg: c(0x0f, 0x11, 0x17),
        header_fg: c(0xc0, 0xc7, 0xd6),
        input_bg: c(0x1a, 0x1d, 0x25),
        input_bg_focus: c(0x25, 0x2a, 0x38),
        input_bg_edit: c(0x2f, 0x36, 0x4a),
        input_bg_readonly: c(0x16, 0x18, 0x1e),
        label: c(0x8a, 0x93, 0xa8),
        label_focus: c(0xe0, 0xc9, 0x7a),
        label_custom: c(0x8f, 0x7b, 0xb0),
        value: c(0xc8, 0xcd, 0xd8),
        value_empty: c(0x64, 0x6c, 0x80),
        mixed: c(0x8b, 0x94, 0xad),
        accent: c(0x7a, 0xa2, 0xf7),
        staged: c(0x9e, 0xd9, 0x7a),
        warn: c(0xe0, 0xaf, 0x68),
        error: c(0xf7, 0x76, 0x8e),
        muted: c(0x62, 0x6b, 0x80),
        rule: c(0x2a, 0x2f, 0x3c),
        star: c(0xe0, 0xaf, 0x68),
        path: c(0x7d, 0x87, 0x9e),
    },
    // Matches the gruvbox leaf already ships a theme for.
    Palette {
        name: "gruvbox",
        page: c(0x1d, 0x20, 0x21),
        header_bg: c(0x3c, 0x38, 0x36),
        badge_bg: c(0xfa, 0xbd, 0x2f),
        badge_fg: c(0x1d, 0x20, 0x21),
        header_fg: c(0xeb, 0xdb, 0xb2),
        input_bg: c(0x32, 0x30, 0x2f),
        input_bg_focus: c(0x3c, 0x38, 0x36),
        input_bg_edit: c(0x50, 0x49, 0x45),
        input_bg_readonly: c(0x28, 0x28, 0x28),
        label: c(0xbd, 0xae, 0x93),
        label_focus: c(0xfa, 0xbd, 0x2f),
        label_custom: c(0xd3, 0x86, 0x9b),
        value: c(0xeb, 0xdb, 0xb2),
        value_empty: c(0x92, 0x83, 0x74),
        mixed: c(0xa8, 0x99, 0x84),
        accent: c(0x83, 0xa5, 0x98),
        staged: c(0xb8, 0xbb, 0x26),
        warn: c(0xfe, 0x80, 0x19),
        error: c(0xfb, 0x49, 0x34),
        muted: c(0x92, 0x83, 0x74),
        rule: c(0x50, 0x49, 0x45),
        star: c(0xfa, 0xbd, 0x2f),
        path: c(0xa8, 0x99, 0x84),
    },
    Palette {
        name: "nord",
        page: c(0x24, 0x29, 0x33),
        header_bg: c(0x3b, 0x42, 0x52),
        badge_bg: c(0x88, 0xc0, 0xd0),
        badge_fg: c(0x2e, 0x34, 0x40),
        header_fg: c(0xd8, 0xde, 0xe9),
        input_bg: c(0x2e, 0x34, 0x40),
        input_bg_focus: c(0x3b, 0x42, 0x52),
        input_bg_edit: c(0x43, 0x4c, 0x5e),
        input_bg_readonly: c(0x2b, 0x30, 0x3b),
        label: c(0xa8, 0xb2, 0xc6),
        label_focus: c(0xeb, 0xcb, 0x8b),
        label_custom: c(0xb4, 0x8e, 0xad),
        value: c(0xd8, 0xde, 0xe9),
        value_empty: c(0x7b, 0x86, 0x9c),
        mixed: c(0x9a, 0xa5, 0xb8),
        accent: c(0x88, 0xc0, 0xd0),
        staged: c(0xa3, 0xbe, 0x8c),
        warn: c(0xd0, 0x87, 0x70),
        error: c(0xbf, 0x61, 0x6a),
        muted: c(0x84, 0x8e, 0xa3),
        rule: c(0x43, 0x4c, 0x5e),
        star: c(0xeb, 0xcb, 0x8b),
        path: c(0x9a, 0xa5, 0xb8),
    },
    Palette {
        name: "rose-pine",
        page: c(0x19, 0x17, 0x24),
        header_bg: c(0x26, 0x23, 0x3a),
        badge_bg: c(0xc4, 0xa7, 0xe7),
        badge_fg: c(0x19, 0x17, 0x24),
        header_fg: c(0xe0, 0xde, 0xf4),
        input_bg: c(0x1f, 0x1d, 0x2e),
        input_bg_focus: c(0x26, 0x23, 0x3a),
        input_bg_edit: c(0x33, 0x2e, 0x4c),
        input_bg_readonly: c(0x1b, 0x19, 0x28),
        label: c(0x90, 0x8c, 0xaa),
        label_focus: c(0xf6, 0xc1, 0x77),
        label_custom: c(0xeb, 0xbc, 0xba),
        value: c(0xe0, 0xde, 0xf4),
        value_empty: c(0x6e, 0x6a, 0x86),
        mixed: c(0x9c, 0x97, 0xb8),
        accent: c(0x9c, 0xcf, 0xd8),
        staged: c(0xa3, 0xd2, 0xa5),
        warn: c(0xf6, 0xc1, 0x77),
        error: c(0xeb, 0x6f, 0x92),
        muted: c(0x82, 0x7d, 0x9c),
        rule: c(0x35, 0x31, 0x4d),
        star: c(0xf6, 0xc1, 0x77),
        path: c(0x9c, 0x97, 0xb8),
    },
];

/// Which scheme is live. An atomic rather than a parameter threaded through
/// every draw call: the palette is read in dozens of places per frame and
/// written once per keystroke, so the ergonomics matter more than the purity.
static ACTIVE: AtomicUsize = AtomicUsize::new(0);

pub fn active() -> &'static Palette {
    &PALETTES[ACTIVE.load(Ordering::Relaxed) % PALETTES.len()]
}

/// Select by name, for `--theme`. Unknown names leave the scheme alone and say
/// so, rather than falling back silently to something the user did not ask for.
pub fn set_by_name(name: &str) -> bool {
    match PALETTES.iter().position(|p| p.name.eq_ignore_ascii_case(name)) {
        Some(i) => {
            ACTIVE.store(i, Ordering::Relaxed);
            true
        }
        None => false,
    }
}

/// Step to the next scheme; returns the new one's name.
pub fn cycle() -> &'static str {
    let next = (ACTIVE.load(Ordering::Relaxed) + 1) % PALETTES.len();
    ACTIVE.store(next, Ordering::Relaxed);
    PALETTES[next].name
}

pub fn names() -> Vec<&'static str> {
    PALETTES.iter().map(|p| p.name).collect()
}

macro_rules! colour {
    ($($fn_name:ident),* $(,)?) => {
        $(pub fn $fn_name() -> Color { active().$fn_name })*
    };
}
colour!(
    header_bg, badge_bg, badge_fg, header_fg,
    input_bg, input_bg_focus, input_bg_edit, input_bg_readonly,
    label, label_focus, label_custom,
    value, value_empty, mixed,
    accent, staged, warn, error, muted, rule, star, path,
);

/// Fit a string to an exact display width, padding or truncating with an
/// ellipsis. Display width, not character count: a CJK title is twice as wide
/// per character, and padding by count would ragged the whole column.
pub fn fit(s: &str, width: usize) -> String {
    use unicode_width::UnicodeWidthStr;
    let w = s.width();
    if w == width {
        return s.to_string();
    }
    if w < width {
        return format!("{s}{}", " ".repeat(width - w));
    }
    if width == 0 {
        return String::new();
    }
    let mut out = String::new();
    let mut used = 0usize;
    for ch in s.chars() {
        let cw = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + cw > width.saturating_sub(1) {
            break;
        }
        out.push(ch);
        used += cw;
    }
    out.push('…');
    used += 1;
    if used < width {
        out.push_str(&" ".repeat(width - used));
    }
    out
}

/// Shorten a container key for display without making distinct keys look alike.
///
/// Reverse-DNS keys carry their meaning at the *end*:
/// `com.apple.quicktime.creationdate`, `.make`, `.model` and `.software` all
/// truncate to the same "com.apple.quic…" from the left, which is worse than
/// useless -- five different rows reading identically. Dropping the namespace
/// keeps them distinguishable and fits the column.
pub fn short_key(key: &str) -> String {
    let looks_reverse_dns = key.matches('.').count() >= 2
        && key.split('.').all(|seg| !seg.is_empty());
    if looks_reverse_dns {
        if let Some(last) = key.rsplit('.').next() {
            return last.to_string();
        }
    }
    key.to_string()
}

/// WCAG relative luminance, used only by the contrast test below.
#[cfg(test)]
fn luminance(c: Color) -> f64 {
    let (r, g, b) = match c {
        Color::Rgb(r, g, b) => (r, g, b),
        _ => panic!("theme colours must be true colour"),
    };
    let lin = |v: u8| {
        let v = v as f64 / 255.0;
        if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
    };
    0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)
}

#[cfg(test)]
fn contrast(a: Color, b: Color) -> f64 {
    let (la, lb) = (luminance(a), luminance(b));
    let (hi, lo) = if la > lb { (la, lb) } else { (lb, la) };
    (hi + 0.05) / (lo + 0.05)
}

/// CIE76 colour difference, for the "is this actually a different colour"
/// test. Lightness alone is not enough: two greys three shades apart measure
/// far closer than they look like they should.
#[cfg(test)]
fn delta_e(a: Color, b: Color) -> f64 {
    fn lab(c: Color) -> (f64, f64, f64) {
        let (r, g, b) = match c {
            Color::Rgb(r, g, b) => (r, g, b),
            _ => panic!("theme colours must be true colour"),
        };
        let lin = |v: u8| {
            let v = v as f64 / 255.0;
            if v <= 0.03928 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) }
        };
        let (r, g, b) = (lin(r), lin(g), lin(b));
        let x = (r * 0.4124 + g * 0.3576 + b * 0.1805) / 0.95047;
        let y = r * 0.2126 + g * 0.7152 + b * 0.0722;
        let z = (r * 0.0193 + g * 0.1192 + b * 0.9505) / 1.08883;
        let f = |t: f64| if t > 0.008856 { t.cbrt() } else { 7.787 * t + 16.0 / 116.0 };
        let (fx, fy, fz) = (f(x), f(y), f(z));
        (116.0 * fy - 16.0, 500.0 * (fx - fy), 200.0 * (fy - fz))
    }
    let (l1, a1, b1) = lab(a);
    let (l2, a2, b2) = lab(b);
    ((l1 - l2).powi(2) + (a1 - a2).powi(2) + (b1 - b2).powi(2)).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_width::UnicodeWidthStr;

    /// Every colour used for *text*, in *every* scheme, must clear 3:1 against
    /// the surfaces it lands on. Two bugs came from ignoring this: custom-key
    /// labels drawn in 16-colour DarkGray, and the file path drawn in a divider
    /// colour at 1.4:1. Running it across all palettes means adding a scheme
    /// cannot quietly reintroduce either.
    #[test]
    fn every_text_colour_in_every_scheme_is_readable() {
        for p in PALETTES {
            let text = [
                ("header_fg", p.header_fg), ("value", p.value), ("value_empty", p.value_empty),
                ("label", p.label), ("label_focus", p.label_focus),
                ("label_custom", p.label_custom), ("mixed", p.mixed), ("muted", p.muted),
                ("path", p.path), ("accent", p.accent), ("staged", p.staged),
                ("warn", p.warn), ("error", p.error), ("star", p.star),
            ];
            for (name, colour) in text {
                for (surface_name, surface) in
                    [("page", p.page), ("field", p.input_bg_readonly), ("input", p.input_bg)]
                {
                    let ratio = contrast(colour, surface);
                    assert!(
                        ratio >= 3.0,
                        "{}: {name} is {ratio:.2}:1 against the {surface_name}; below 3:1 it stops reading as text",
                        p.name
                    );
                }
            }
        }
    }

    /// The badge is light-on-dark inverted, so it needs checking the other way.
    #[test]
    fn every_badge_and_header_is_legible() {
        for p in PALETTES {
            assert!(contrast(p.badge_fg, p.badge_bg) >= 4.5, "{}: badge", p.name);
            assert!(contrast(p.header_fg, p.header_bg) >= 4.5, "{}: header", p.name);
        }
    }

    /// A custom-key label must be a different *hue* from an ordinary one, not a
    /// dimmer shade: the first attempt sat at ΔE 9, which does not read as a
    /// distinction at label size.
    #[test]
    fn custom_labels_are_distinguishable_in_every_scheme() {
        for p in PALETTES {
            let d = delta_e(p.label, p.label_custom);
            assert!(d >= 15.0, "{}: custom label is only ΔE {d:.1} from the normal one", p.name);
        }
    }

    #[test]
    fn schemes_have_unique_names_and_cycle_back_around() {
        let mut names = names();
        let n = names.len();
        assert!(n >= 2, "cycling needs at least two schemes");
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), n, "scheme names must be unique");

        let first = active().name;
        for _ in 0..n {
            cycle();
        }
        assert_eq!(active().name, first, "a full cycle must return to where it started");
    }

    #[test]
    fn an_unknown_scheme_name_is_refused_not_silently_ignored() {
        let before = active().name;
        assert!(!set_by_name("no-such-scheme"));
        assert_eq!(active().name, before);
        assert!(set_by_name("gruvbox"));
        assert_eq!(active().name, "gruvbox");
        set_by_name(before);
    }

    #[test]
    fn short_strings_are_padded_to_width() {
        assert_eq!(fit("Title", 10), "Title     ");
    }

    /// The bug from the screenshot: a long custom key must not push the value
    /// column out of alignment.
    #[test]
    fn long_labels_are_truncated_not_allowed_to_overflow() {
        let got = fit("com.apple.quicktime.creationdate", 14);
        assert_eq!(got.width(), 14);
        assert!(got.ends_with('…'));
    }

    #[test]
    fn exact_width_is_left_alone() {
        assert_eq!(fit("abcde", 5), "abcde");
    }

    #[test]
    fn wide_characters_count_double() {
        let got = fit("日本語のタイトル", 10);
        assert_eq!(got.width(), 10);
    }

    /// The screenshot bug: five Apple keys all rendering as "com.apple.quic…".
    #[test]
    fn reverse_dns_keys_keep_their_distinctive_tail() {
        for (key, want) in [
            ("com.apple.quicktime.creationdate", "creationdate"),
            ("com.apple.quicktime.make", "make"),
            ("com.apple.quicktime.model", "model"),
            ("com.apple.quicktime.software", "software"),
            ("com.apple.quicktime.location.ISO6709", "ISO6709"),
        ] {
            assert_eq!(short_key(key), want);
        }
    }

    #[test]
    fn ordinary_keys_are_left_alone() {
        assert_eq!(short_key("yt_dlp_id"), "yt_dlp_id");
        assert_eq!(short_key("title"), "title");
        assert_eq!(short_key("a.b"), "a.b");
    }

    #[test]
    fn zero_width_is_empty_not_a_panic() {
        assert_eq!(fit("anything", 0), "");
    }
}
