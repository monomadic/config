//! The preset catalog, read through the same zsh functions every other Topaz
//! tool uses, so the TOML tree under ~/.zsh/bin/lib/topaz-presets stays the
//! single source of truth. Only the presets ffmpeg can run are kept: those with
//! an `ns_model` belong to neuroserver-select-preset.

use anyhow::{anyhow, Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

pub const ORIGINAL_SLUG: &str = "__original__";

#[derive(Clone, Debug)]
pub struct Enhancement {
    pub category: String,
    pub display: String,
    pub slug: String,
    pub scales: Vec<String>,
    /// The filter with its `@SCALE@` placeholder; empty for Original.
    pub body: String,
    pub blurb: String,
    pub metadata: String,
    /// Always rendered at an explicit size (scale=0:w:h), never scale=1/2.
    pub free_size: bool,
}

impl Enhancement {
    /// The no-model row topaz-pick offers first: re-encode, interpolate or
    /// plain-lanczos upscale only.
    fn original() -> Self {
        Enhancement {
            category: String::new(),
            display: "Original (no enhancement)".into(),
            slug: ORIGINAL_SLUG.into(),
            scales: vec!["1".into(), "2".into(), "4k".into()],
            body: String::new(),
            blurb: "Skip enhance — interpolate / re-encode only".into(),
            metadata: String::new(),
            free_size: false,
        }
    }

    pub fn is_original(&self) -> bool {
        self.slug == ORIGINAL_SLUG
    }
}

#[derive(Clone, Debug)]
pub struct Interpolation {
    pub display: String,
    pub slug: String,
    pub filter: String,
    pub metadata: String,
}

impl Interpolation {
    /// The tvai_fi stage's `fps=` (the output rate; None keeps the source's)
    /// and `slowmo=` (how many times longer the output runs).
    pub fn rate(&self) -> (Option<f64>, f64) {
        let param = |key: &str| {
            self.filter
                .split([':', ','])
                .find_map(|kv| kv.strip_prefix(key))
                .and_then(|v| v.parse::<f64>().ok())
                .filter(|v| *v > 0.0)
        };
        (param("fps="), param("slowmo=").unwrap_or(1.0))
    }
}

#[derive(Clone, Debug)]
pub struct OutputProfile {
    pub display: String,
    pub ext: String,
    pub video_args: String,
}

/// Prose for the details sheet, from each enhancement TOML's [insight] table.
#[derive(Clone, Debug, Default)]
pub struct Insight {
    pub strategy: Option<String>,
    pub notes: Vec<(String, String)>,
    pub watch: Option<String>,
    pub vs: Option<(String, String)>,
}

/// Menu groups, by what is wrong with the source — the keys and order of
/// CATEGORY_ORDER in the mpv menu and of topaz-pick.
const CATEGORIES: &[(&str, &str)] = &[
    ("polish", "Decent Source · Polish"),
    ("focus-fix", "Soft · Out of Focus"),
    ("lowlight", "Dark · Noisy"),
    ("compressed", "Compressed · Old Codecs"),
    ("interlaced", "Interlaced · Broadcast / DV"),
    ("stylized", "Stylized · Texture"),
    ("hdr", "SDR → HDR"),
];

pub fn category_label(key: &str) -> String {
    CATEGORIES
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, l)| l.to_string())
        .unwrap_or_else(|| key.to_string())
}

fn category_rank(key: &str) -> usize {
    CATEGORIES.iter().position(|(k, _)| *k == key).unwrap_or(CATEGORIES.len())
}

/// Where the deployed zsh tools live. The symlinked path is stable wherever the
/// repo checkout is, which is why it is used rather than any repo path.
pub fn zsh_bin() -> PathBuf {
    if let Ok(dir) = std::env::var("TOPAZ_ZSH_BIN") {
        return PathBuf::from(dir);
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
    PathBuf::from(home).join(".zsh/bin")
}

fn catalog_rows(function: &str) -> Result<Vec<Vec<String>>> {
    let catalog = zsh_bin().join("lib/topaz-preset-catalog.zsh");
    if !catalog.is_file() {
        return Err(anyhow!("preset catalog not found: {}", catalog.display()));
    }
    let script = format!("source {}; {}", shell_quote(&catalog.to_string_lossy()), function);
    let out = Command::new("zsh")
        .arg("-c")
        .arg(&script)
        .output()
        .context("running zsh for the preset catalog")?;
    if !out.status.success() {
        return Err(anyhow!(
            "{} failed: {}",
            function,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split('\t').map(str::to_string).collect())
        .collect())
}

/// Original first, then every ffmpeg preset grouped by category (catalog order
/// within a group, as topaz-pick lists them).
pub fn enhancements() -> Result<Vec<Enhancement>> {
    let mut presets = Vec::new();
    for f in catalog_rows("topaz_enhancement_preset_rows")? {
        let col = |i: usize| f.get(i).cloned().unwrap_or_default();
        if !col(7).is_empty() {
            continue; // ns_model: neuroserver only
        }
        presets.push(Enhancement {
            category: col(0),
            display: col(1),
            slug: col(2),
            scales: col(3).split(',').filter(|s| !s.is_empty()).map(str::to_string).collect(),
            body: col(4),
            blurb: col(5),
            metadata: col(6),
            free_size: col(10) == "1",
        });
    }
    if presets.is_empty() {
        return Err(anyhow!("no ffmpeg enhancement presets in the catalog"));
    }
    presets.sort_by_key(|p| category_rank(&p.category));
    presets.insert(0, Enhancement::original());
    Ok(presets)
}

pub fn interpolations() -> Result<Vec<Interpolation>> {
    Ok(catalog_rows("topaz_interpolation_preset_rows")?
        .into_iter()
        .filter(|f| f.len() >= 3)
        .map(|f| Interpolation {
            display: f[0].clone(),
            slug: f[1].clone(),
            filter: f[2].clone(),
            metadata: f.get(3).cloned().unwrap_or_default(),
        })
        .collect())
}

pub fn output_profiles() -> Result<Vec<OutputProfile>> {
    let profiles: Vec<OutputProfile> = catalog_rows("topaz_output_profile_rows")?
        .into_iter()
        .filter(|f| f.len() >= 4)
        .map(|f| OutputProfile { display: f[0].clone(), ext: f[2].clone(), video_args: f[3].clone() })
        .collect();
    if profiles.is_empty() {
        return Err(anyhow!("no output profiles in the catalog"));
    }
    Ok(profiles)
}

/// Keyed by slug. Missing prose is not an error: the sheet just says less.
pub fn insights() -> HashMap<String, Insight> {
    let mut map: HashMap<String, Insight> = HashMap::new();
    for f in catalog_rows("topaz_preset_insights").unwrap_or_default() {
        let col = |i: usize| f.get(i).cloned().unwrap_or_default();
        let entry = map.entry(col(0)).or_default();
        match col(1).as_str() {
            "strategy" => entry.strategy = Some(col(2)),
            "note" => entry.notes.push((col(2), col(3))),
            "watch" => entry.watch = Some(col(2)),
            "vs" => entry.vs = Some((col(2), col(3))),
            _ => {}
        }
    }
    map
}

pub fn shell_quote(s: &str) -> String {
    if !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || "-_./=:@%+,".contains(c))
    {
        s.to_string()
    } else {
        format!("'{}'", s.replace('\'', "'\\''"))
    }
}
