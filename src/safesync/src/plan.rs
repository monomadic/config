//! What a one-way sync would do, worked out from two indexes alone.
use crate::{
    drive::Extras,
    manifest::{Entry, Manifest},
};
use anyhow::Result;
use std::{
    collections::{BTreeMap, HashMap},
    path::PathBuf,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// On the source only.
    Copy { path: PathBuf, size: u64 },
    /// Both sides have the path with different content; the backup's version
    /// goes to history first.
    Replace { path: PathBuf, size: u64 },
    /// The backup already holds this content under the source's old name.
    Rename {
        from: PathBuf,
        to: PathBuf,
        size: u64,
    },
    /// On the backup only, and the sentinel says such files go to history.
    Retire { path: PathBuf, size: u64 },
}
impl Action {
    pub fn path(&self) -> &PathBuf {
        match self {
            Self::Copy { path, .. } | Self::Replace { path, .. } | Self::Retire { path, .. } => {
                path
            }
            Self::Rename { to, .. } => to,
        }
    }
    /// Bytes that have to cross the bus.
    pub fn transfer(&self) -> u64 {
        match self {
            Self::Copy { size, .. } | Self::Replace { size, .. } => *size,
            _ => 0,
        }
    }
}

#[derive(Debug, Default)]
pub struct Plan {
    pub actions: Vec<Action>,
    pub unchanged: usize,
    /// Backup-only files left alone because the sentinel says `extras = "keep"`.
    pub kept_extras: Vec<(PathBuf, u64)>,
}
impl Plan {
    pub fn transfer_bytes(&self) -> u64 {
        self.actions.iter().map(Action::transfer).sum()
    }
    pub fn renamed_bytes(&self) -> u64 {
        self.actions
            .iter()
            .map(|action| match action {
                Action::Rename { size, .. } => *size,
                _ => 0,
            })
            .sum()
    }
    pub fn count(&self, matches: impl Fn(&Action) -> bool) -> usize {
        self.actions.iter().filter(|action| matches(action)).count()
    }
}

// Fingerprints decide when both sides have one. Otherwise size and mtime do,
// as in rclone: copies keep their mtime, so an untouched pair agrees.
pub fn same_content(a: &Entry, b: &Entry) -> bool {
    if a.stamp.size != b.stamp.size {
        return false;
    }
    match (&a.sha256, &b.sha256) {
        (Some(a), Some(b)) => a == b,
        _ => {
            (a.stamp.mtime_seconds, a.stamp.mtime_nanos)
                == (b.stamp.mtime_seconds, b.stamp.mtime_nanos)
        }
    }
}

// What a moved file is recognised by. A multi-gigabyte video sharing both its
// size and its nanosecond mtime with a different video does not happen.
type Identity = (u64, i64, i64);
fn identity(entry: &Entry) -> Identity {
    (
        entry.stamp.size,
        entry.stamp.mtime_seconds,
        entry.stamp.mtime_nanos,
    )
}

pub fn plan(source: &Manifest, backup: &Manifest, extras: Extras) -> Result<Plan> {
    let mut there: BTreeMap<PathBuf, &Entry> = BTreeMap::new();
    for entry in &backup.entries {
        there.insert(entry.path()?, entry);
    }
    let mut plan = Plan::default();
    let mut missing = Vec::new();
    for entry in &source.entries {
        let path = entry.path()?;
        match there.remove(&path) {
            Some(existing) if same_content(entry, existing) => plan.unchanged += 1,
            Some(_) => plan.actions.push(Action::Replace {
                path,
                size: entry.stamp.size,
            }),
            None => missing.push((path, entry)),
        }
    }

    // `there` now holds backup-only files: candidates for a rename. Only an
    // unambiguous pairing counts — one missing file, one leftover, same identity.
    let mut leftovers: HashMap<Identity, Vec<PathBuf>> = HashMap::new();
    for (path, entry) in &there {
        leftovers
            .entry(identity(entry))
            .or_default()
            .push(path.clone());
    }
    let mut wanted: HashMap<Identity, usize> = HashMap::new();
    for (_, entry) in &missing {
        *wanted.entry(identity(entry)).or_default() += 1;
    }
    for (path, entry) in missing {
        let key = identity(entry);
        let size = entry.stamp.size;
        let from = match leftovers.get(&key) {
            Some(paths) if paths.len() == 1 && wanted[&key] == 1 && size > 0 => {
                Some(paths[0].clone())
            }
            _ => None,
        };
        match from {
            Some(from) if same_content(entry, there[&from]) => {
                there.remove(&from);
                plan.actions.push(Action::Rename {
                    from,
                    to: path,
                    size,
                });
            }
            _ => plan.actions.push(Action::Copy { path, size }),
        }
    }

    for (path, entry) in there {
        match extras {
            Extras::Keep => plan.kept_extras.push((path, entry.stamp.size)),
            Extras::History => plan.actions.push(Action::Retire {
                path,
                size: entry.stamp.size,
            }),
        }
    }
    // Renames and retirements first: they free names and space for the copies.
    plan.actions.sort_by_key(|action| match action {
        Action::Rename { .. } => 0,
        Action::Retire { .. } => 1,
        Action::Replace { .. } => 2,
        Action::Copy { .. } => 3,
    });
    Ok(plan)
}
