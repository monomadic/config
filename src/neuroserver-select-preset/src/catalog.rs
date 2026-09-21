//! The preset catalog, read straight from the TOML tree under
//! ~/.local/bin/lib/topaz-presets — the same files topaz-presets-emit.py renders
//! for the shell tools, so the tree stays the single source of truth. Only
//! neuroserver presets (those with an `ns_model`) are kept: this tool exists for
//! the models ffmpeg cannot reach.

use anyhow::{anyhow, Context, Result};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Preset {
    pub display: String,
    pub slug: String,
    pub scales: Vec<String>,
    pub blurb: String,
    pub metadata: String,
    pub ns_model: String,
    pub ns_store: String,
    pub ns_params: Option<String>,
}

#[derive(Clone, Debug)]
pub struct OutputProfile {
    pub display: String,
    pub slug: String,
    pub ext: String,
    pub video_args: String,
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
    display: String,
    #[serde(default)]
    scales: Vec<String>,
    #[serde(default)]
    blurb: String,
    #[serde(default)]
    metadata: String,
    #[serde(default)]
    ns_model: String,
    #[serde(default)]
    ns_store: String,
    /// Extra neuroserver --filters keys, handed on as a JSON object.
    ns_params: Option<toml::Table>,
    #[serde(default)]
    pseudo: bool,
}

#[derive(Deserialize)]
struct OutputToml {
    display: String,
    #[serde(default)]
    ext: String,
    video_args: String,
}

pub fn neuroserver_presets() -> Result<Vec<Preset>> {
    let mut presets = Vec::new();
    for (slug, e) in load_dir::<EnhancementToml>("enhancement")? {
        if e.pseudo || e.ns_model.is_empty() {
            continue;
        }
        if e.display.is_empty() {
            return Err(anyhow!("enhancement preset {slug} has no display name"));
        }
        let ns_params = match e.ns_params.filter(|p| !p.is_empty()) {
            Some(p) => Some(
                serde_json::to_string(&p).with_context(|| format!("ns_params of preset {slug}"))?,
            ),
            None => None,
        };
        presets.push(Preset {
            display: e.display,
            slug,
            scales: e.scales,
            blurb: e.blurb,
            metadata: e.metadata,
            ns_model: e.ns_model,
            ns_store: e.ns_store,
            ns_params,
        });
    }
    if presets.is_empty() {
        return Err(anyhow!(
            "no neuroserver presets in the catalog (none declares ns_model)"
        ));
    }
    Ok(presets)
}

pub fn output_profiles() -> Result<Vec<OutputProfile>> {
    let profiles: Vec<OutputProfile> = load_dir::<OutputToml>("output")?
        .into_iter()
        .map(|(slug, o)| OutputProfile { display: o.display, slug, ext: o.ext, video_args: o.video_args })
        .collect();
    if profiles.is_empty() {
        return Err(anyhow!("no output profiles in the catalog"));
    }
    Ok(profiles)
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
