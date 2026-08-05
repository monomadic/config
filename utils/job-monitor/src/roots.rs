//! Which folders to watch.
//!
//! Precedence: `$JOB_MONITOR_ROOTS` (colon-separated, handy for a one-off run),
//! then `~/.config/job-monitor/roots` (one path per line, `#` comments), then
//! the default mount point. Deliberately not a config format — a list of paths
//! is the whole configuration, and a parser would only be something else to
//! get wrong.

use std::fs;
use std::path::PathBuf;

use job_core::observe::Root;

const DEFAULT_ROOT: &str = "/Volumes/Jobs";

pub fn config_dir() -> PathBuf {
    home().join(".config/job-monitor")
}

fn home() -> PathBuf {
    std::env::var_os("HOME").map(PathBuf::from).unwrap_or_default()
}

fn expand(raw: &str) -> Option<PathBuf> {
    let trimmed = raw.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    Some(match trimmed.strip_prefix("~/") {
        Some(rest) => home().join(rest),
        None => PathBuf::from(trimmed),
    })
}

pub fn configured() -> Vec<Root> {
    if let Some(raw) = std::env::var_os("JOB_MONITOR_ROOTS") {
        let roots: Vec<Root> = raw
            .to_string_lossy()
            .split(':')
            .filter_map(expand)
            .map(Root::new)
            .collect();
        if !roots.is_empty() {
            return roots;
        }
    }

    if let Ok(text) = fs::read_to_string(config_dir().join("roots")) {
        let roots: Vec<Root> = text.lines().filter_map(expand).map(Root::new).collect();
        if !roots.is_empty() {
            return roots;
        }
    }

    vec![Root::new(DEFAULT_ROOT)]
}
