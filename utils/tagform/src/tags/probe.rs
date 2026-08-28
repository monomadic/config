//! Reading tags. Two readers, always (SPEC §4.1).
//!
//! ffprobe cannot see XMP — measured, docs/CONTAINER.md §2. A file carrying all
//! six of rename-footage's XMP fields reports exactly the same format_tags as
//! one carrying none. Reading with ffprobe alone would report every footage file
//! as having no people, no tags, no channel, no location and no rating, and the
//! form would then offer to write that emptiness back. exiftool is a hard
//! dependency of the read path, not an enhancement.

use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::model::value::Value;

#[derive(Debug, Clone)]
pub struct FileTags {
    pub path: PathBuf,
    /// Container tags as ffprobe sees them, keys lower-cased.
    pub atoms: BTreeMap<String, Value>,
    /// XMP tags as exiftool sees them, keyed `XMP-<ns>:<Tag>`.
    pub xmp: BTreeMap<String, Value>,
}

impl FileTags {
    /// Resolve one field, XMP first (SPEC §4.1) — that is where rename-footage
    /// puts authored data, and a remux-written atom may be staler.
    pub fn lookup(&self, f: &crate::model::schema::FieldDef) -> Option<Value> {
        for k in f.xmp {
            if let Some(v) = self.xmp.get(*k) {
                if !v.is_empty() {
                    return Some(v.clone());
                }
            }
        }
        for k in f.read {
            if let Some(v) = self.atoms.get(*k) {
                if !v.is_empty() {
                    return Some(v.clone());
                }
            }
        }
        None
    }

    /// Where the two readers disagree. Surfaced rather than silently resolved,
    /// because a disagreement usually means one writer clobbered the other.
    pub fn disputes(&self, f: &crate::model::schema::FieldDef) -> Option<(Value, Value)> {
        let x = f.xmp.iter().find_map(|k| self.xmp.get(*k)).filter(|v| !v.is_empty())?;
        let a = f.read.iter().find_map(|k| self.atoms.get(*k)).filter(|v| !v.is_empty())?;
        (x != a).then(|| (x.clone(), a.clone()))
    }
}

pub fn probe(path: &Path) -> Result<FileTags> {
    if !path.is_file() {
        bail!("not a file: {}", path.display());
    }
    Ok(FileTags {
        path: path.to_path_buf(),
        atoms: probe_atoms(path)?,
        xmp: probe_xmp(path)?,
    })
}

fn probe_atoms(path: &Path) -> Result<BTreeMap<String, Value>> {
    let out = Command::new("ffprobe")
        .args(["-v", "error", "-show_entries", "format_tags", "-of", "json"])
        .arg("--")
        .arg(path)
        .output()
        .context("running ffprobe (is it installed?)")?;
    if !out.status.success() {
        bail!("ffprobe failed on {}: {}", path.display(), String::from_utf8_lossy(&out.stderr).trim());
    }
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).context("parsing ffprobe json")?;
    let mut map = BTreeMap::new();
    if let Some(tags) = v.get("format").and_then(|f| f.get("tags")).and_then(|t| t.as_object()) {
        for (k, val) in tags {
            let key = k.to_ascii_lowercase();
            if crate::model::schema::JUNK_KEYS.contains(&key.as_str()) {
                continue;
            }
            if let Some(s) = json_scalar(val) {
                map.insert(key, Value::Text(s));
            }
        }
    }
    Ok(map)
}

fn probe_xmp(path: &Path) -> Result<BTreeMap<String, Value>> {
    let out = Command::new("exiftool")
        .args(["-j", "-G1", "-n", "-XMP:all"])
        .arg("--")
        .arg(path)
        .output()
        .context("running exiftool (is it installed?)")?;
    // exiftool exits non-zero for a file with no XMP at all; that is not an
    // error, it is the common case for a plain download.
    if out.stdout.is_empty() {
        return Ok(BTreeMap::new());
    }
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).context("parsing exiftool json")?;
    let mut map = BTreeMap::new();
    let Some(obj) = v.get(0).and_then(|o| o.as_object()) else {
        return Ok(map);
    };
    for (k, val) in obj {
        if k == "SourceFile" || k == "XMP-x:XMPToolkit" {
            continue;
        }
        // exiftool collapses a single-entry list to a scalar, so a list field
        // has to accept both shapes or a one-actor file reads as no list.
        let value = match val {
            serde_json::Value::Array(a) => {
                Value::List(a.iter().filter_map(json_scalar).collect())
            }
            other => match json_scalar(other) {
                Some(s) => Value::Text(s),
                None => continue,
            },
        };
        map.insert(k.clone(), value);
    }
    Ok(map)
}

fn json_scalar(v: &serde_json::Value) -> Option<String> {
    match v {
        serde_json::Value::String(s) => Some(s.clone()),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}
