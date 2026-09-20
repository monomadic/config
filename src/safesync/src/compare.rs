//! Historical comparison only. These observations never authorize mutations.
use crate::manifest::{Entry, Header, Manifest};
use anyhow::Result;
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Status {
    ContentMatch,
    ContentDiffers,
    ContentUnknown,
    SourceOnly,
    DestinationOnly,
}
#[derive(Debug, Serialize)]
pub struct Row {
    pub path_base64: String,
    pub path_display: String,
    pub status: Status,
    pub source: Option<Entry>,
    pub destination: Option<Entry>,
    /// Equal full-content fingerprints elsewhere, not proposed rename operations.
    pub content_candidates: Vec<String>,
}
#[derive(Debug, Serialize)]
pub struct Report {
    pub schema: u32,
    pub historical_only: bool,
    pub executable: bool,
    pub source: Header,
    pub destination: Header,
    pub warnings: Vec<String>,
    pub rows: Vec<Row>,
}

pub fn compare(source: &Manifest, destination: &Manifest) -> Result<Report> {
    source.validate()?;
    destination.validate()?;
    let mut paths: BTreeMap<PathBuf, (Option<&Entry>, Option<&Entry>)> = BTreeMap::new();
    let mut fingerprints: BTreeMap<(u64, &str), Vec<String>> = BTreeMap::new();
    for entry in &source.entries {
        paths.entry(entry.path()?).or_default().0 = Some(entry);
    }
    for entry in &destination.entries {
        paths.entry(entry.path()?).or_default().1 = Some(entry);
        if let Some(hash) = &entry.sha256 {
            fingerprints
                .entry((entry.stamp.size, hash))
                .or_default()
                .push(entry.path_base64.clone());
        }
    }
    for candidates in fingerprints.values_mut() {
        candidates.sort();
    }
    let mut rows = Vec::with_capacity(paths.len());
    for (path, (src, dst)) in paths {
        let status = match (src, dst) {
            (Some(a), Some(b)) if a.stamp.size != b.stamp.size => Status::ContentDiffers,
            (Some(a), Some(b)) => match (&a.sha256, &b.sha256) {
                (Some(a), Some(b)) if a == b => Status::ContentMatch,
                (Some(_), Some(_)) => Status::ContentDiffers,
                _ => Status::ContentUnknown,
            },
            (Some(_), None) => Status::SourceOnly,
            (None, Some(_)) => Status::DestinationOnly,
            (None, None) => unreachable!(),
        };
        let entry = src.or(dst).expect("path has an entry");
        let content_candidates = src
            .and_then(|e| {
                e.sha256
                    .as_ref()
                    .and_then(|hash| fingerprints.get(&(e.stamp.size, hash.as_str())))
            })
            .map(|matches| {
                matches
                    .iter()
                    .filter(|p| *p != &entry.path_base64)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default();
        rows.push(Row {
            path_base64: entry.path_base64.clone(),
            path_display: path.to_string_lossy().into_owned(),
            status,
            source: src.cloned(),
            destination: dst.cloned(),
            content_candidates,
        });
    }
    let mut warnings = vec![
        "Historical regular-file observations only; current presence is not checked.".into(),
        "No relationship baseline: destination-only entries do not authorize deletion; content candidates do not authorize renames.".into(),
        "Case/Unicode collisions, directories, metadata preservation, roles and free space are not validated. This is not an executable plan.".into(),
    ];
    if source.header.exclusions != destination.header.exclusions {
        warnings.push(
            "Exclusion policies differ; missing paths may be outside the other scan's scope."
                .into(),
        );
    }
    if source.header.volume.uuid == destination.header.volume.uuid {
        warnings.push(
            "Both snapshots identify the same volume; they are not independent drive replicas."
                .into(),
        );
    }
    Ok(Report {
        schema: 1,
        historical_only: true,
        executable: false,
        source: source.header.clone(),
        destination: destination.header.clone(),
        warnings,
        rows,
    })
}
