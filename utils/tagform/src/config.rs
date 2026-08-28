//! Enum sources (SPEC §3.5).
//!
//! Genre and Type are not invented here. They are exactly the aliases in
//! `config/yt-dlp/config`, which write `meta_genre` and `meta_type`:
//!
//! ```text
//! --alias footage '--embed-metadata --parse-metadata "Camera Footage:%(meta_genre)s"'
//! --alias clip    '--embed-metadata --parse-metadata "Clip:%(meta_type)s"'
//! ```
//!
//! Hard-coding the values would guarantee drift the first time an alias is
//! added, so the config is parsed instead. Failure is not fatal: the defaults
//! below stand in, because a tagger that will not start because a config file
//! moved is worse than one with a slightly stale dropdown.

use std::path::PathBuf;

pub const DEFAULT_GENRES: &[&str] = &["Media", "Footage", "Karaoke", "VJ Clip"];
pub const DEFAULT_TYPES: &[&str] = &["Clip", "Master", "Original"];

/// The `stik` media kind: a closed set the Apple ecosystem actually reads.
pub const KINDS: &[(&str, &str)] = &[
    ("0", "Home Video"),
    ("1", "Normal"),
    ("2", "Audiobook"),
    ("6", "Music Video"),
    ("9", "Movie"),
    ("10", "TV Show"),
    ("21", "Podcast"),
];

pub struct Enums {
    pub genre: Vec<String>,
    pub type_: Vec<String>,
}

impl Default for Enums {
    fn default() -> Self {
        Self {
            genre: DEFAULT_GENRES.iter().map(|s| s.to_string()).collect(),
            type_: DEFAULT_TYPES.iter().map(|s| s.to_string()).collect(),
        }
    }
}

impl Enums {
    pub fn load() -> Self {
        match ytdlp_config_path().and_then(|p| std::fs::read_to_string(p).ok()) {
            Some(text) => Self::from_ytdlp_config(&text),
            None => Self::default(),
        }
    }

    pub fn from_ytdlp_config(text: &str) -> Self {
        let mut me = Self {
            genre: parse_alias_values(text, "meta_genre"),
            type_: parse_alias_values(text, "meta_type"),
        };
        if me.genre.is_empty() {
            me.genre = DEFAULT_GENRES.iter().map(|s| s.to_string()).collect();
        }
        if me.type_.is_empty() {
            me.type_ = DEFAULT_TYPES.iter().map(|s| s.to_string()).collect();
        }
        me
    }
}

fn ytdlp_config_path() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    let p = base.join("yt-dlp/config");
    p.exists().then_some(p)
}

/// Pull every `"<VALUE>:%(<field>)s"` literal out of the alias lines.
fn parse_alias_values(text: &str, field: &str) -> Vec<String> {
    let needle = format!(":%({field})s");
    let mut out: Vec<String> = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with("--alias") {
            continue;
        }
        for (idx, _) in line.match_indices(&needle) {
            // Walk back from the marker to the quote that opens the literal.
            let before = &line[..idx];
            let Some(start) = before.rfind('"') else { continue };
            let value = normalize(&before[start + 1..]);
            if !value.is_empty() && !out.iter().any(|v| v.eq_ignore_ascii_case(&value)) {
                out.push(value);
            }
        }
    }
    out
}

/// Canonical spelling for values stored under an older name. "Footage" reads
/// better than "Camera Footage" and is what the form shows; files already
/// tagged the long way round display and re-save as the short one, so no
/// migration pass is needed.
pub fn normalize(value: &str) -> String {
    let v = value.trim();
    match v.to_ascii_lowercase().as_str() {
        "camera footage" => "Footage".to_string(),
        _ => v.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
# genre
--alias media '--embed-metadata --parse-metadata "Media:%(meta_genre)s"'
--alias footage '--embed-metadata --parse-metadata "Camera Footage:%(meta_genre)s"'
--alias karaoke '--embed-metadata --parse-metadata "Karaoke:%(meta_genre)s"'
--alias vj '--parse-metadata "VJ Clip:%(meta_genre)s"'

# type
--alias clip '--embed-metadata --parse-metadata "Clip:%(meta_type)s"'
--alias master '--embed-metadata --parse-metadata "Master:%(meta_type)s"'
--alias original '--embed-metadata --parse-metadata "Original:%(meta_type)s"'
"#;

    #[test]
    fn genres_come_from_the_aliases() {
        let e = Enums::from_ytdlp_config(SAMPLE);
        assert_eq!(e.genre, vec!["Media", "Footage", "Karaoke", "VJ Clip"]);
    }

    #[test]
    fn types_come_from_the_aliases() {
        let e = Enums::from_ytdlp_config(SAMPLE);
        assert_eq!(e.type_, vec!["Clip", "Master", "Original"]);
    }

    /// The alias literal is "Camera Footage"; the form says "Footage".
    #[test]
    fn camera_footage_normalizes() {
        assert_eq!(normalize("Camera Footage"), "Footage");
        assert_eq!(normalize("camera footage"), "Footage");
        assert_eq!(normalize(" Karaoke "), "Karaoke");
    }

    /// A missing or unreadable config must not empty the dropdowns.
    #[test]
    fn empty_config_falls_back_to_defaults() {
        let e = Enums::from_ytdlp_config("");
        assert_eq!(e.genre, DEFAULT_GENRES);
        assert_eq!(e.type_, DEFAULT_TYPES);
    }

    #[test]
    fn non_alias_lines_are_ignored() {
        let e = Enums::from_ytdlp_config("--parse-metadata \"Bogus:%(meta_genre)s\"");
        assert_eq!(e.genre, DEFAULT_GENRES);
    }
}
