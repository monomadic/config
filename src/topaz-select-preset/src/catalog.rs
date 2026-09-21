//! The preset catalog, read straight from the TOML tree under
//! ~/.local/bin/lib/topaz-presets — the same files topaz-presets-emit.py renders
//! for the shell tools, so the tree stays the single source of truth. Only the
//! presets ffmpeg can run are kept: those with an `ns_model` belong to
//! neuroserver-select-preset.

use anyhow::{anyhow, Context, Result};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;


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
    PathBuf::from(home).join(".local/bin")
}

/// The preset tree, one `<type>/<slug>.toml` per preset. TOPAZ_PRESETS_DIR
/// overrides it (point it at the repo's bin/lib/topaz-presets to test edits
/// without deploying).
pub fn presets_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("TOPAZ_PRESETS_DIR") {
        return PathBuf::from(dir);
    }
    zsh_bin().join("lib/topaz-presets")
}

/// Every `*.toml` in one type's directory, as (slug, preset), sorted by `order`
/// then slug — the order topaz-presets-emit.py emits them in. The slug is the
/// filename stem.
fn load_dir<T: DeserializeOwned>(kind: &str) -> Result<Vec<(String, T)>> {
    let dir = presets_dir().join(kind);
    let entries = std::fs::read_dir(&dir)
        .with_context(|| format!("preset directory not found: {}", dir.display()))?;
    let mut items = Vec::new();
    for entry in entries {
        let path = entry?.path();
        let Some(slug) = path.file_stem().map(|s| s.to_string_lossy().into_owned()) else { continue };
        if slug.starts_with('.') || path.extension().is_none_or(|e| e != "toml") {
            continue;
        }
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let table: toml::Table = toml::from_str(&text)
            .with_context(|| format!("parsing {}", path.display()))?;
        let order = table.get("order").and_then(toml::Value::as_integer).unwrap_or(1_000_000);
        let preset: T = toml::Value::Table(table)
            .try_into()
            .with_context(|| format!("reading {}", path.display()))?;
        items.push((order, slug, preset));
    }
    items.sort_by(|a, b| (a.0, &a.1).cmp(&(b.0, &b.1)));
    Ok(items.into_iter().map(|(_, slug, preset)| (slug, preset)).collect())
}

#[derive(Deserialize)]
struct EnhancementToml {
    #[serde(default)]
    category: String,
    #[serde(default)]
    display: String,
    #[serde(default)]
    scales: Vec<String>,
    #[serde(default)]
    filter: String,
    #[serde(default)]
    blurb: String,
    #[serde(default)]
    metadata: String,
    #[serde(default)]
    ns_model: String,
    #[serde(default)]
    free_size: bool,
    /// Contributes only its [insight], no menu row (`__original__`).
    #[serde(default)]
    pseudo: bool,
    insight: Option<InsightToml>,
}

#[derive(Deserialize)]
struct InsightToml {
    strategy: Option<String>,
    #[serde(default)]
    notes: toml::Table,
    watch: Option<String>,
    vs: Option<String>,
    #[serde(default)]
    vs_note: String,
}

#[derive(Deserialize)]
struct InterpolationToml {
    display: String,
    filter: String,
    #[serde(default)]
    metadata: String,
}

#[derive(Deserialize)]
struct OutputToml {
    display: String,
    #[serde(default)]
    ext: String,
    video_args: String,
}

/// Original first, then every ffmpeg preset grouped by category (catalog order
/// within a group, as topaz-pick lists them).
pub fn enhancements() -> Result<Vec<Enhancement>> {
    let mut presets = Vec::new();
    for (slug, e) in load_dir::<EnhancementToml>("enhancement")? {
        if e.pseudo || !e.ns_model.is_empty() {
            continue; // ns_model: neuroserver only
        }
        if e.display.is_empty() {
            return Err(anyhow!("enhancement preset {slug} has no display name"));
        }
        presets.push(Enhancement {
            category: e.category,
            display: e.display,
            slug,
            scales: e.scales,
            body: e.filter,
            blurb: e.blurb,
            metadata: e.metadata,
            free_size: e.free_size,
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
    Ok(load_dir::<InterpolationToml>("interpolation")?
        .into_iter()
        .map(|(slug, i)| Interpolation { display: i.display, slug, filter: i.filter, metadata: i.metadata })
        .collect())
}

pub fn output_profiles() -> Result<Vec<OutputProfile>> {
    let profiles: Vec<OutputProfile> = load_dir::<OutputToml>("output")?
        .into_iter()
        .map(|(_, o)| OutputProfile { display: o.display, ext: o.ext, video_args: o.video_args })
        .collect();
    if profiles.is_empty() {
        return Err(anyhow!("no output profiles in the catalog"));
    }
    Ok(profiles)
}

/// Keyed by slug. Missing prose is not an error: the sheet just says less.
pub fn insights() -> HashMap<String, Insight> {
    let nonempty = |s: Option<String>| s.filter(|s| !s.is_empty());
    load_dir::<EnhancementToml>("enhancement")
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(slug, e)| {
            let ins = e.insight?;
            let notes = ins
                .notes
                .into_iter()
                .map(|(k, v)| {
                    let text = v.as_str().map(str::to_string).unwrap_or_else(|| v.to_string());
                    (k, text)
                })
                .collect();
            let insight = Insight {
                strategy: nonempty(ins.strategy),
                notes,
                watch: nonempty(ins.watch),
                vs: nonempty(ins.vs).map(|vs| (vs, ins.vs_note)),
            };
            Some((slug, insight))
        })
        .collect()
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
