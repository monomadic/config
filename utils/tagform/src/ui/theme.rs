//! One place for every colour (SPEC §7).
//!
//! True colour throughout rather than the 16-colour palette. The old rendering
//! leaned on `DarkGray` for anything secondary, which on a dark terminal is
//! very nearly the background -- long custom keys like
//! `com.apple.quicktime.creationdate` rendered as blank labels beside floating
//! values. Everything here is an explicit RGB with a checked contrast against
//! the page, so "subdued" never means "invisible".

use ratatui::style::Color;

/// Header band and its badge.
pub const HEADER_BG: Color = Color::Rgb(0x22, 0x26, 0x33);
pub const BADGE_BG: Color = Color::Rgb(0x7a, 0xa2, 0xf7);
pub const BADGE_FG: Color = Color::Rgb(0x0f, 0x11, 0x17);
pub const HEADER_FG: Color = Color::Rgb(0xc0, 0xc7, 0xd6);

/// The editable region of a field. Every control paints one, so the form reads
/// as a form: you can see where a value can be typed before you focus it.
pub const INPUT_BG: Color = Color::Rgb(0x1a, 0x1d, 0x25);
pub const INPUT_BG_FOCUS: Color = Color::Rgb(0x25, 0x2a, 0x38);
pub const INPUT_BG_EDIT: Color = Color::Rgb(0x2f, 0x36, 0x4a);
pub const INPUT_BG_READONLY: Color = Color::Rgb(0x16, 0x18, 0x1e);

pub const LABEL: Color = Color::Rgb(0x8a, 0x93, 0xa8);
pub const LABEL_FOCUS: Color = Color::Rgb(0xe0, 0xc9, 0x7a);
/// Still clearly a label, still clearly not a schema field.
pub const LABEL_CUSTOM: Color = Color::Rgb(0x71, 0x7c, 0x94);

pub const VALUE: Color = Color::Rgb(0xc8, 0xcd, 0xd8);
/// An absent value: readable as "nothing here", not as a rendering failure.
pub const VALUE_EMPTY: Color = Color::Rgb(0x59, 0x60, 0x72);
pub const MIXED: Color = Color::Rgb(0x8b, 0x94, 0xad);

pub const ACCENT: Color = Color::Rgb(0x7a, 0xa2, 0xf7);
pub const STAGED: Color = Color::Rgb(0x9e, 0xd9, 0x7a);
pub const WARN: Color = Color::Rgb(0xe0, 0xaf, 0x68);
pub const ERROR: Color = Color::Rgb(0xf7, 0x76, 0x8e);
pub const MUTED: Color = Color::Rgb(0x62, 0x6b, 0x80);
pub const RULE: Color = Color::Rgb(0x2a, 0x2f, 0x3c);
pub const STAR: Color = Color::Rgb(0xe0, 0xaf, 0x68);

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

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_width::UnicodeWidthStr;

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
