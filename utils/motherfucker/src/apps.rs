//! Cache-free app discovery and fuzzy matching.
//!
//! Discovery is a readdir over the standard app directories on every summon —
//! a few milliseconds cold, microseconds warm. Names come from bundle
//! filenames; file contents (Info.plist, icons) are never read, so there is
//! nothing to cache and nothing to go stale.

use std::path::PathBuf;

const APP_DIRS: &[&str] = &[
    "/Applications",
    "/Applications/Utilities",
    "/System/Applications",
    "/System/Applications/Utilities",
];

pub struct InstalledApp {
    pub name: String,
    pub path: PathBuf,
}

pub fn scan_installed() -> Vec<InstalledApp> {
    let mut dirs: Vec<PathBuf> = APP_DIRS.iter().map(PathBuf::from).collect();
    if let Some(home) = std::env::var_os("HOME") {
        dirs.push(PathBuf::from(home).join("Applications"));
    }

    let mut apps = Vec::with_capacity(256);
    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "app") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    apps.push(InstalledApp {
                        name: stem.to_string(),
                        path,
                    });
                }
            }
        }
    }
    apps.sort_by(|a, b| a.name.cmp(&b.name));
    apps.dedup_by(|a, b| a.name == b.name);
    apps
}

fn lower(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

/// Subsequence fuzzy match; returns (score, matched char indices) or None.
/// Higher score is better: bonuses for prefix matches, word starts, and
/// consecutive runs; mild penalty for gaps. Case-insensitive. The indices
/// are char positions in `candidate`, used to highlight matches in the UI.
pub fn match_positions(query: &str, candidate: &str) -> Option<(i32, Vec<usize>)> {
    if query.is_empty() {
        return Some((0, Vec::new()));
    }
    let q: Vec<char> = query.chars().map(lower).collect();
    let c_orig: Vec<char> = candidate.chars().collect();
    if q.len() > c_orig.len() {
        return None;
    }

    let mut total: i32 = 0;
    let mut positions = Vec::with_capacity(q.len());
    let mut ci = 0usize;
    let mut prev_hit: Option<usize> = None;

    for &qc in &q {
        let mut found = None;
        while ci < c_orig.len() {
            if lower(c_orig[ci]) == qc {
                found = Some(ci);
                break;
            }
            ci += 1;
        }
        let hit = found?;

        let mut s = 1;
        if hit == 0 {
            s += 12; // matches the very start
        } else {
            let prev = c_orig[hit - 1];
            if prev == ' ' || prev == '-' || prev == '_' || prev == '.' {
                s += 8; // word boundary
            } else if c_orig[hit].is_uppercase() {
                s += 6; // camelCase boundary
            }
        }
        if let Some(p) = prev_hit {
            if hit == p + 1 {
                s += 6; // consecutive run
            } else {
                s -= ((hit - p - 1).min(8)) as i32; // gap penalty
            }
        }
        total += s;
        positions.push(hit);
        prev_hit = Some(hit);
        ci = hit + 1;
    }

    // Prefer shorter names when scores tie.
    total += 8i32.saturating_sub(c_orig.len() as i32 / 4);
    Some((total, positions))
}

#[cfg(test)]
pub fn score(query: &str, candidate: &str) -> Option<i32> {
    match_positions(query, candidate).map(|(s, _)| s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_beats_scattered() {
        assert!(score("saf", "Safari").unwrap() > score("saf", "Site Analysis Fixer").unwrap());
    }

    #[test]
    fn no_match_is_none() {
        assert!(score("xyz", "Safari").is_none());
    }

    #[test]
    fn empty_query_matches_all() {
        assert_eq!(score("", "Anything"), Some(0));
    }

    #[test]
    fn word_boundaries_score() {
        assert!(score("al", "Ableton Live 12 Suite").is_some());
    }

    // "sig" is a clean prefix of Signal but only a gap-penalized scatter in
    // "System Settings". The gap must exceed main::RUNNING_BONUS (12) so a
    // running System Settings can't leapfrog a cold Signal on that soft bonus.
    #[test]
    fn strong_prefix_beats_running_scatter() {
        let signal = score("sig", "Signal").unwrap();
        let settings = score("sig", "System Settings").unwrap();
        assert!(signal - settings > 12, "signal={signal} settings={settings}");
    }
}
