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

/// Strict subsequence match; returns (score, matched char indices) or None.
///
/// Query chars must match in order, and each hit must land on a "word start"
/// (start of name, after a non-letter, or an uppercase letter) unless it
/// directly continues the previous hit (a linear run). Mid-word lowercase
/// letters are otherwise unmatchable, so "sig" no longer scatters into
/// "System Settings". Earlier first hits rank higher. Case-insensitive.
/// The indices are char positions in `candidate`, for UI highlighting.
pub fn match_positions(query: &str, candidate: &str) -> Option<(i32, Vec<usize>)> {
    if query.is_empty() {
        return Some((0, Vec::new()));
    }
    let q: Vec<char> = query.chars().map(lower).collect();
    let c_orig: Vec<char> = candidate.chars().collect();
    if q.len() > c_orig.len() {
        return None;
    }
    let c_low: Vec<char> = c_orig.iter().map(|&ch| lower(ch)).collect();

    // A mid-word char is a lowercase letter preceded by another letter; it can
    // only be hit as a continuation. Everything else can start a new run.
    let word_start: Vec<bool> = c_orig
        .iter()
        .enumerate()
        .map(|(i, &ch)| {
            i == 0 || !(ch.is_alphabetic() && ch.is_lowercase() && c_orig[i - 1].is_alphabetic())
        })
        .collect();

    // Greedy leftmost placement can dead-end where a later start succeeds
    // ("di" must skip DaisyDisk's D-a and match D-i of "Disk"), so backtrack.
    // Trying candidate positions left to right yields the earliest valid
    // placement, which is also the highest-ranked one.
    fn dfs(
        qi: usize,
        from: usize,
        q: &[char],
        c_low: &[char],
        word_start: &[bool],
        positions: &mut Vec<usize>,
    ) -> bool {
        if qi == q.len() {
            return true;
        }
        for p in from..c_low.len() {
            if c_low[p] != q[qi] {
                continue;
            }
            let continues = p > 0 && positions.last() == Some(&(p - 1));
            if !word_start[p] && !continues {
                continue;
            }
            positions.push(p);
            if dfs(qi + 1, p + 1, q, c_low, word_start, positions) {
                return true;
            }
            positions.pop();
        }
        false
    }

    let mut positions = Vec::with_capacity(q.len());
    if !dfs(0, 0, &q, &c_low, &word_start, &mut positions) {
        return None;
    }

    // Earlier first hit ranks higher; one column costs 8, so the soft
    // RUNNING_BONUS (12) in main.rs can only leapfrog a one-column difference.
    let mut total: i32 = 64 - positions[0] as i32 * 8;
    // Tie-breakers at the same column: longer linear runs, then shorter names.
    for w in positions.windows(2) {
        if w[1] == w[0] + 1 {
            total += 1;
        }
    }
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

    // Mid-word lowercase letters are unmatchable unless they continue a run,
    // so "sig" can no longer scatter into "System Settings" at all.
    #[test]
    fn mid_word_scatter_rejected() {
        assert!(score("sig", "Signal").is_some());
        assert!(score("sig", "System Settings").is_none());
    }

    // Screenshot 1: "daid" hits Da(isy)D(isk) via the camelCase D, but the
    // scattered mid-word hits in the other results are now rejected.
    #[test]
    fn daid_matches_only_daisydisk() {
        assert!(score("daid", "DaisyDisk").is_some());
        assert!(score("daid", "Video Eraser & Retouch: VidFix").is_none());
        assert!(score("daid", "Soundcraft USB Firmware Update Utility").is_none());
    }

    // Screenshot 2: "sysr" hits Sys(tem:) r(estart); the r's in the other
    // results are all mid-word.
    #[test]
    fn sysr_matches_only_restart() {
        assert!(score("sysr", "System: restart").is_some());
        assert!(score("sysr", "System Information").is_none());
        assert!(score("sysr", "System Settings: keyboard").is_none());
        assert!(score("sysr", "System Settings: privacy").is_none());
    }

    // A camelCase capital or a symbol both start a new word, but a mid-word
    // lowercase 'd' does not: DaisyDisk and daisy-disk match, Daisydisk can't.
    #[test]
    fn word_starts_capitals_and_symbols() {
        assert!(score("daid", "daisy-disk").is_some());
        assert!(score("daid", "Daisydisk").is_none());
    }

    // Greedy leftmost placement dead-ends here: "di" must skip DaisyDisk's
    // leading D (whose next char is 'a') and match the D-i of "Disk".
    #[test]
    fn backtracks_past_dead_end_start() {
        let (_, positions) = match_positions("di", "DaisyDisk").unwrap();
        assert_eq!(positions, vec![5, 6]);
    }

    // Rule 3: earlier first hit outranks a later one.
    #[test]
    fn earlier_match_ranks_higher() {
        assert!(score("set", "Settings Helper").unwrap() > score("set", "System Settings").unwrap());
    }
}
