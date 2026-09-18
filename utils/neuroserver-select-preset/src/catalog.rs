//! The preset catalog, read through the same zsh functions every other Topaz
//! tool uses, so the TOML tree under ~/.zsh/bin/lib/topaz-presets stays the
//! single source of truth. Only neuroserver presets (those with an `ns_model`)
//! are kept: this tool exists for the models ffmpeg cannot reach.

use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
use std::process::Command;

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

pub fn neuroserver_presets() -> Result<Vec<Preset>> {
    let mut presets = Vec::new();
    for f in catalog_rows("topaz_enhancement_preset_rows")? {
        let col = |i: usize| f.get(i).cloned().unwrap_or_default();
        let ns_model = col(7);
        if ns_model.is_empty() {
            continue;
        }
        let ns_params = col(9);
        presets.push(Preset {
            display: col(1),
            slug: col(2),
            scales: col(3)
                .split(',')
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect(),
            blurb: col(5),
            metadata: col(6),
            ns_model,
            ns_store: col(8),
            ns_params: if ns_params.is_empty() { None } else { Some(ns_params) },
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
    let rows = catalog_rows("topaz_output_profile_rows")?;
    let profiles: Vec<OutputProfile> = rows
        .into_iter()
        .filter(|f| f.len() >= 4)
        .map(|f| OutputProfile { display: f[0].clone(), slug: f[1].clone() })
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
