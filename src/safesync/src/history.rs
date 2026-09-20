//! Historical rename analysis. Caller-supplied prior snapshots are observations,
//! not a committed destination-owned relationship or authority to mutate files.
use crate::{
    manifest::{Entry, Header, Manifest},
    plan::{self, Preview},
};
use anyhow::{Result, ensure};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Evidence {
    FullContentAtScanTime,
    IdentityAndMetadataOnly,
    ReviewRequired,
}
#[derive(Debug, Serialize)]
pub struct RenameObservation {
    pub previous_source: Entry,
    pub current_source: Option<Entry>,
    pub previous_destination: Option<Entry>,
    pub current_destination: Option<Entry>,
    pub evidence: Evidence,
    pub reason: String,
}
#[derive(Debug, Serialize)]
pub struct HistoryPreview {
    pub schema: u32,
    pub historical_only: bool,
    pub executable: bool,
    pub committed_relationship: bool,
    pub previous_source: Header,
    pub previous_destination: Header,
    pub previous_blockers: Vec<plan::Blocker>,
    pub current: Preview,
    pub observations: Vec<RenameObservation>,
    pub warnings: Vec<String>,
}
impl HistoryPreview {
    pub fn exit_code(&self) -> i32 {
        // Any proposed move still needs relationship enrollment and live review.
        if self.observations.is_empty() && self.previous_blockers.is_empty() {
            self.current.exit_code()
        } else {
            3
        }
    }
}
fn ids(manifest: &Manifest) -> BTreeMap<u64, Vec<&Entry>> {
    let mut result: BTreeMap<u64, Vec<&Entry>> = BTreeMap::new();
    for entry in &manifest.entries {
        result.entry(entry.stamp.file_id).or_default().push(entry);
    }
    result
}
fn paths(manifest: &Manifest) -> BTreeMap<&str, &Entry> {
    manifest
        .entries
        .iter()
        .map(|e| (e.path_base64.as_str(), e))
        .collect()
}
fn same_version_metadata(a: &Entry, b: &Entry) -> bool {
    a.stamp.file_id == b.stamp.file_id
        && a.stamp.size == b.stamp.size
        && a.stamp.mtime_seconds == b.stamp.mtime_seconds
        && a.stamp.mtime_nanos == b.stamp.mtime_nanos
}
fn content_equal(a: &Entry, b: &Entry) -> bool {
    a.stamp.size == b.stamp.size && a.sha256.is_some() && a.sha256 == b.sha256
}
fn content_contradicts(a: &Entry, b: &Entry) -> bool {
    a.stamp.size != b.stamp.size
        || (a.sha256.is_some() && b.sha256.is_some() && a.sha256 != b.sha256)
}
fn validate_history(previous: &Manifest, current: &Manifest) -> Result<()> {
    previous.validate()?;
    current.validate()?;
    ensure!(
        previous.header.volume.uuid == current.header.volume.uuid,
        "Previous/current snapshots identify different volumes"
    );
    ensure!(
        previous.header.root_file_id == current.header.root_file_id,
        "Previous/current snapshots identify different scan roots"
    );
    ensure!(
        previous.header.exclusions.iter().collect::<BTreeSet<_>>()
            == current.header.exclusions.iter().collect::<BTreeSet<_>>(),
        "Previous/current exclusion scopes differ"
    );
    // Mount paths may change between Macs. UUID + root file ID establish scope.
    Ok(())
}

pub fn preview(
    previous_source: &Manifest,
    previous_destination: &Manifest,
    source: &Manifest,
    destination: &Manifest,
) -> Result<HistoryPreview> {
    validate_history(previous_source, source)?;
    validate_history(previous_destination, destination)?;
    ensure!(
        source.header.volume.uuid != destination.header.volume.uuid,
        "Source/destination must be distinct volumes"
    );
    ensure!(
        previous_source
            .header
            .exclusions
            .iter()
            .collect::<BTreeSet<_>>()
            == previous_destination
                .header
                .exclusions
                .iter()
                .collect::<BTreeSet<_>>(),
        "Prior source/destination exclusion scopes differ"
    );
    let current = plan::preview(source, destination)?;
    let prior_plan = plan::preview(previous_source, previous_destination)?;
    let old_source_ids = ids(previous_source);
    let source_ids = ids(source);
    let old_destination_ids = ids(previous_destination);
    let destination_ids = ids(destination);
    let source_paths = paths(source);
    let destination_paths = paths(destination);
    let old_destination_paths = paths(previous_destination);
    let mut old_entries: Vec<_> = previous_source.entries.iter().collect();
    old_entries.sort_by(|a, b| a.path_base64.cmp(&b.path_base64));
    let mut observations = Vec::new();
    for old in old_entries {
        let now = source_ids.get(&old.stamp.file_id);
        // Unchanged paths with unique identities are handled by the ordinary plan.
        if old_source_ids[&old.stamp.file_id].len() == 1
            && now.is_some_and(|entries| {
                entries.len() == 1 && entries[0].path_base64 == old.path_base64
            })
        {
            continue;
        }
        let src = now
            .filter(|entries| entries.len() == 1)
            .map(|entries| entries[0]);
        let old_dst = old_destination_paths.get(old.path_base64.as_str()).copied();
        let dst = old_dst.and_then(|e| destination_paths.get(e.path_base64.as_str()).copied());
        let mut observation = RenameObservation {
            previous_source: old.clone(),
            current_source: src.cloned(),
            previous_destination: old_dst.cloned(),
            current_destination: dst.cloned(),
            evidence: Evidence::ReviewRequired,
            reason: String::new(),
        };
        let reason = if !current.blockers.is_empty() || !prior_plan.blockers.is_empty() {
            "Namespace or scan-coverage blockers prevent rename planning"
        } else if old_source_ids[&old.stamp.file_id].len() != 1
            || now.is_some_and(|entries| entries.len() != 1)
        {
            "Ambiguous source file identity (hard links or identity reuse); no move inferred"
        } else if let Some(src) = src {
            if !same_version_metadata(old, src) || content_contradicts(old, src) {
                "Source content/version changed; identity alone cannot authorize a rename"
            } else if source_paths.contains_key(old.path_base64.as_str()) {
                "Previous source path is reused; dependent replacements or rename cycles need review"
            } else if destination_paths.contains_key(src.path_base64.as_str()) {
                "Destination target is occupied; rename cycles, swaps and replacements need review"
            } else if let (Some(old_dst), Some(dst)) = (old_dst, dst) {
                if !content_equal(old, old_dst) {
                    "Prior same-path files lack matching complete-content hashes; no replica relationship inferred"
                } else if old_destination_ids[&old_dst.stamp.file_id].len() != 1
                    || destination_ids
                        .get(&old_dst.stamp.file_id)
                        .is_none_or(|entries| entries.len() != 1)
                {
                    "Ambiguous or missing destination identity; no move inferred"
                } else if !same_version_metadata(old_dst, dst) || content_contradicts(old_dst, dst)
                {
                    "Destination predecessor changed; preserve it for conflict review"
                } else if content_equal(src, dst) {
                    observation.evidence = Evidence::FullContentAtScanTime;
                    "Possible destination rename; current complete-content fingerprints match"
                } else {
                    observation.evidence = Evidence::IdentityAndMetadataOnly;
                    "Possible destination rename from identities and size/mtime; current content equality remains unverified"
                }
            } else {
                "Previous destination counterpart is missing; preserve other files and review"
            }
        } else {
            "Source identity is absent from the current scan; no deletion or archive is authorized"
        };
        observation.reason = reason.into();
        observations.push(observation);
    }
    Ok(HistoryPreview {
        schema: 1, historical_only: true, executable: false, committed_relationship: false,
        previous_source: previous_source.header.clone(), previous_destination: previous_destination.header.clone(), current, observations,
        previous_blockers: prior_plan.blockers,
        warnings: vec![
            "Prior snapshots are caller-selected history, not a committed relationship baseline. No rename, deletion or archive is authorized.".into(),
            "Identity matching is volume-scoped; device numbers and inode numbers across different volumes are never compared.".into(),
            "Size/mtime continuity cannot rule out inode reuse or same-size edits with preserved timestamps. ctime changes alone do not disprove a rename.".into(),
            "The initial-adoption plan remains unchanged; rename observations are alternatives for review, not additional executable operations.".into(),
        ],
    })
}
