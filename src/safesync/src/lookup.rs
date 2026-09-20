use crate::{filesystem, manifest::Manifest};
use anyhow::Result;
use serde::Serialize;
use std::{ffi::OsStr, path::Path};

#[derive(Serialize)]
pub struct Match {
    pub volume_name: String,
    pub volume_uuid: String,
    pub generation: String,
    pub scanned_unix: u64,
    pub root_base64: String,
    pub path_base64: String,
    pub path_display: String,
    pub evidence: String,
}
#[derive(Serialize)]
pub struct Report {
    pub scope: &'static str,
    pub matches: Vec<Match>,
    pub unverified_candidates: usize,
    pub manifests_searched: usize,
    pub verdict: &'static str,
}
impl Report {
    pub fn exit_code(&self) -> i32 {
        if !self.matches.is_empty() {
            0
        } else if self.unverified_candidates > 0 {
            3
        } else {
            1
        }
    }
}
pub enum Query<'a> {
    Name(&'a OsStr),
    File(&'a Path),
}
pub fn lookup(manifests: &[Manifest], query: Query<'_>) -> Result<Report> {
    let fingerprint = match query {
        Query::File(path) => Some(filesystem::hash_path(path)?),
        _ => None,
    };
    let mut matches = Vec::new();
    let mut unknown = 0;
    for manifest in manifests {
        for entry in &manifest.entries {
            let path = entry.path()?;
            let evidence = match &query {
                Query::Name(name) => {
                    if path.file_name() != Some(*name) {
                        continue;
                    }
                    "exact_filename_only"
                }
                Query::File(_) => {
                    let (stamp, hash) = fingerprint.as_ref().expect("file query has fingerprint");
                    if entry.stamp.size != stamp.size {
                        continue;
                    }
                    match &entry.sha256 {
                        None => {
                            unknown += 1;
                            continue;
                        }
                        Some(saved) if saved != hash => continue,
                        Some(_) => "sha256_content_at_scan_time",
                    }
                }
            };
            matches.push(Match {
                volume_name: manifest.header.volume.name.clone(),
                volume_uuid: manifest.header.volume.uuid.clone(),
                generation: manifest.header.generation.clone(),
                scanned_unix: manifest.header.finished_unix,
                root_base64: manifest.header.root_base64.clone(),
                path_base64: entry.path_base64.clone(),
                path_display: path.to_string_lossy().into(),
                evidence: evidence.into(),
            });
        }
    }
    let verdict = if !matches.is_empty() {
        "recorded_match"
    } else if unknown > 0 {
        "unknown_missing_content_hashes"
    } else {
        "not_recorded_in_selected_manifests"
    };
    Ok(Report {
        scope: "historical_offline_inventory_not_live_presence",
        matches,
        unverified_candidates: unknown,
        manifests_searched: manifests.len(),
        verdict,
    })
}
