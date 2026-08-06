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

/// Where a jobs folder normally is when nobody has said otherwise: the local
/// queue on the machine that runs it, and the usual mount point for one shared
/// from elsewhere. Both are offered because the honest answer to "which folder
/// did you mean" depends on which machine you are sitting at, and a default
/// that names only the remote one is wrong on the server itself.
const DEFAULT_ROOTS: [&str; 2] = ["~/jobs", "/Volumes/Jobs"];

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

    // Only the ones that exist: an unconfigured monitor should show the queue
    // it can see, not a permanent "not mounted" for a folder this machine was
    // never going to have.
    let present: Vec<Root> = DEFAULT_ROOTS
        .iter()
        .filter_map(|raw| expand(raw))
        .filter(|path| path.is_dir())
        .map(Root::new)
        .collect();
    if present.is_empty() {
        // Nothing to show yet — name the mount point, so the menu can say it
        // isn't there rather than saying nothing at all.
        return vec![Root::new(expand(DEFAULT_ROOTS[1]).unwrap_or_default())];
    }
    present
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The default has to be right on both kinds of machine: the one running
    /// the queue, and one watching it over a share. Naming only the mount
    /// point made the monitor say "not mounted" forever on the server itself.
    #[test]
    fn defaults_name_only_folders_that_exist() {
        let home = home();
        let local = home.join("jobs");
        let roots = configured();

        assert!(!roots.is_empty());
        if local.is_dir() {
            assert!(
                roots.iter().any(|root| root.path() == local),
                "the local queue should be watched when it is there"
            );
        }
        for root in &roots {
            // Either it exists, or it is the mount point we name so the menu
            // has something to report as missing.
            assert!(
                root.path().is_dir() || root.path() == std::path::Path::new("/Volumes/Jobs"),
                "offered a root that is neither present nor the mount point: {}",
                root.path().display()
            );
        }
    }
}
