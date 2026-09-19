//! Pure initial-adoption preview. No baseline, live validation, or write authority.
use crate::{
    compare::{self, Status},
    manifest::{Entry, Header, Manifest, encode_path},
};
use anyhow::{Context, Result};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    os::unix::ffi::OsStrExt,
    path::PathBuf,
};

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    KeepContent,
    Copy,
    ReplaceWithHistory,
    ReviewContent,
    ReviewAlternateContent,
    PreserveDestination,
}
#[derive(Debug, Serialize)]
pub struct Item {
    pub path_base64: String,
    pub path_display: String,
    pub proposed_action: Action,
    pub source: Option<Entry>,
    pub destination: Option<Entry>,
    pub alternate_content_paths_base64: Vec<String>,
}
#[derive(Debug, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Blocker {
    pub reason: String,
    pub paths_base64: Vec<String>,
}
#[derive(Debug, Serialize, Default)]
pub struct Summary {
    pub keep_content: usize,
    pub copies: usize,
    pub replacements: usize,
    pub review: usize,
    pub preserve_destination: usize,
    /// Logical incoming bytes for copy/replacement proposals only; not free-space requirements.
    pub proposed_transfer_bytes: u64,
    /// Logical predecessor sizes; APFS allocation, existing history and reserve are unknown.
    pub proposed_history_bytes: u64,
}
#[derive(Debug, Serialize)]
pub struct Preview {
    pub schema: u32,
    pub historical_only: bool,
    pub executable: bool,
    pub source: Header,
    pub destination: Header,
    pub blockers: Vec<Blocker>,
    pub warnings: Vec<String>,
    pub summary: Summary,
    pub items: Vec<Item>,
}
impl Preview {
    pub fn exit_code(&self) -> i32 {
        if self.blockers.is_empty() && self.summary.review == 0 {
            0
        } else {
            3
        }
    }
}

fn block(blockers: &mut BTreeSet<Blocker>, reason: &str, paths: impl IntoIterator<Item = PathBuf>) {
    let mut paths_base64: Vec<_> = paths.into_iter().map(|p| encode_path(&p)).collect();
    paths_base64.sort();
    paths_base64.dedup();
    blockers.insert(Blocker {
        reason: reason.into(),
        paths_base64,
    });
}

/// Conservative namespace checks: ASCII case folding always, non-ASCII held for review.
/// Scan v1 lacks destination collation and directory/special-file inventories.
fn namespace_blockers(
    source: &Manifest,
    destination: &Manifest,
    blockers: &mut BTreeSet<Blocker>,
) -> Result<()> {
    let mut names: BTreeMap<Vec<u8>, BTreeSet<PathBuf>> = BTreeMap::new();
    let mut files = BTreeSet::new();
    let mut directories = BTreeSet::new();
    for entry in source.entries.iter().chain(&destination.entries) {
        let path = entry.path()?;
        let raw = path.as_os_str().as_bytes();
        if !raw.is_ascii() {
            block(
                blockers,
                "Non-ASCII path requires destination Unicode/collation validation",
                [path.clone()],
            );
        }
        let mut prefix = PathBuf::new();
        for component in path.components() {
            prefix.push(component);
            let key = prefix.as_os_str().as_bytes().to_ascii_lowercase();
            names.entry(key.clone()).or_default().insert(prefix.clone());
            if prefix == path {
                files.insert(key);
            } else {
                directories.insert(key);
            }
            if component
                .as_os_str()
                .as_bytes()
                .eq_ignore_ascii_case(b".safesync")
            {
                block(
                    blockers,
                    "Reserved safesync namespace appears in media inventory",
                    [path.clone()],
                );
            }
        }
    }
    for (key, paths) in names {
        if paths.len() > 1 {
            block(
                blockers,
                "ASCII case collision in combined namespace",
                paths.iter().cloned(),
            );
        }
        if files.contains(&key) && directories.contains(&key) {
            block(
                blockers,
                "A regular-file path is also required as a directory",
                paths,
            );
        }
    }
    Ok(())
}

pub fn preview(source: &Manifest, destination: &Manifest) -> Result<Preview> {
    let comparison = compare::compare(source, destination)?;
    let mut blockers = BTreeSet::new();
    if source.header.volume.uuid == destination.header.volume.uuid {
        block(
            &mut blockers,
            "Source and destination snapshots identify the same volume",
            [],
        );
    }
    if source.header.exclusions.iter().collect::<BTreeSet<_>>()
        != destination
            .header
            .exclusions
            .iter()
            .collect::<BTreeSet<_>>()
    {
        block(&mut blockers, "Scan exclusion scopes differ", []);
    }
    for (label, header) in [
        ("Source", &source.header),
        ("Destination", &destination.header),
    ] {
        if header.skipped_symlinks != 0 || header.skipped_special != 0 || header.skipped_mounts != 0
        {
            block(
                &mut blockers,
                &format!(
                    "{label} scan omitted symlinks, special entries or mounted subtrees; their paths are unknown"
                ),
                [],
            );
        }
    }
    namespace_blockers(source, destination, &mut blockers)?;
    let mut items = Vec::new();
    let mut summary = Summary::default();
    for row in comparison.rows {
        let proposed_action = match row.status {
            Status::ContentMatch => {
                summary.keep_content += 1;
                Action::KeepContent
            }
            Status::ContentDiffers => {
                summary.replacements += 1;
                Action::ReplaceWithHistory
            }
            Status::ContentUnknown => {
                summary.review += 1;
                Action::ReviewContent
            }
            Status::SourceOnly if !row.content_candidates.is_empty() => {
                summary.review += 1;
                Action::ReviewAlternateContent
            }
            Status::SourceOnly => {
                summary.copies += 1;
                Action::Copy
            }
            Status::DestinationOnly => {
                summary.preserve_destination += 1;
                Action::PreserveDestination
            }
        };
        if matches!(proposed_action, Action::Copy | Action::ReplaceWithHistory) {
            summary.proposed_transfer_bytes = summary
                .proposed_transfer_bytes
                .checked_add(
                    row.source
                        .as_ref()
                        .context("Missing source entry")?
                        .stamp
                        .size,
                )
                .context("Proposed transfer size overflow")?;
        }
        if proposed_action == Action::ReplaceWithHistory {
            summary.proposed_history_bytes = summary
                .proposed_history_bytes
                .checked_add(
                    row.destination
                        .as_ref()
                        .context("Missing predecessor")?
                        .stamp
                        .size,
                )
                .context("Proposed history size overflow")?;
        }
        items.push(Item {
            path_base64: row.path_base64,
            path_display: row.path_display,
            proposed_action,
            source: row.source,
            destination: row.destination,
            alternate_content_paths_base64: row.content_candidates,
        });
    }
    Ok(Preview {
        schema: 1, historical_only: true, executable: false,
        source: source.header.clone(), destination: destination.header.clone(),
        blockers: blockers.into_iter().collect(), summary, items,
        warnings: vec![
            "Initial-adoption preview only: no relationship baseline, rename, deletion or application support.".into(),
            "Proposals are conditional; blockers prevent treating this preview as ready for further planning.".into(),
            "Historical fingerprints compare file contents only; current presence and metadata equality are unverified.".into(),
            "Destination collation is unknown: ASCII case checks are conservative, and non-ASCII paths block planning pending Unicode validation.".into(),
            "Empty directories, live attachment/role checks, filesystem verification, recovery and metadata policy remain unvalidated.".into(),
            "Byte totals cover logical proposed payloads only, not allocated space, existing history, journals or a safety reserve.".into(),
        ],
    })
}
