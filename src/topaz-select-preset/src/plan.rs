//! What a selection becomes: the filter chain, the preset name and the output
//! file — composed exactly as topaz-pick composes them, and named exactly as
//! topaz-encode names them, so an encode started here and one started from
//! topaz-pick find each other's partial files.

use crate::catalog::{Enhancement, Interpolation};
use crate::probe::ResOption;
use std::path::{Path, PathBuf};

/// Fill `@SCALE@` the way the mpv menu's enh_filter_for does — scale=1,
/// scale=2 or scale=0:w:h, the free-target form gaining a lanczos tail so the
/// encode lands on the exact size whatever the model produced — then add the
/// interpolation, which goes before that tail.
pub fn compose_filter(enh: &Enhancement, res: &ResOption, interp: Option<&Interpolation>) -> String {
    let has = |s: &str| enh.scales.iter().any(|x| x == s);
    let (w, h) = (res.w, res.h);
    let lanczos = format!("scale=w={w}:h={h}:flags=lanczos:threads=0");
    let upscaled = res.key != "orig" && w > 0;
    let mut tail = None;
    let mut parts: Vec<String> = Vec::new();
    if !enh.body.is_empty() {
        let clause = if enh.free_size && w > 0 {
            if upscaled {
                tail = Some(lanczos.clone());
            }
            format!("scale=0:w={w}:h={h}")
        } else if res.key == "2x" {
            "scale=2".to_string()
        } else if upscaled {
            if has("4k") {
                tail = Some(lanczos.clone());
                format!("scale=0:w={w}:h={h}")
            } else {
                "scale=2".to_string() // the 1080p case: 2x is the 4K target
            }
        } else {
            "scale=1".to_string()
        };
        parts.push(enh.body.replace("@SCALE@", &clause));
    } else if upscaled {
        tail = Some(lanczos); // Original upscaled: a plain lanczos, no AI pass
    }
    if let Some(i) = interp {
        parts.push(i.filter.clone());
    }
    parts.extend(tail);
    if parts.is_empty() { "null".into() } else { parts.join(",") }
}

/// "Proteus — Sharp → 4K + Apollo 60fps — best quality - HEVC variable bitrate",
/// as topaz-pick builds it.
pub fn preset_name(enh: &Enhancement, res: &ResOption, interp: Option<&Interpolation>, output: &str) -> String {
    let mut name = enh.display.clone();
    if res.key != "orig" {
        name.push_str(&format!(" → {}", res.key.to_uppercase()));
    }
    if let Some(i) = interp {
        name.push_str(&format!(" + {}", i.display));
    }
    name.push_str(&format!(" - {output}"));
    name
}

/// "<dir>/<stem minus trailing [tags]> [Topaz - <label>].<ext>", the name
/// topaz-encode gives a full encode (topaz_file_preset_label,
/// topaz_short_preset_label, topaz_strip_trailing_brackets, topaz_fit_filename).
pub fn output_path(input: &Path, preset_name: &str, ext: &str) -> PathBuf {
    let dir = input.parent().unwrap_or(Path::new("."));
    let stem = input.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let suffix = format!(" [Topaz - {}].{ext}", file_label(short_label(preset_name)));
    let mut stem = strip_trailing_brackets(&stem);
    // Keep the name comfortably under the 255-byte limit.
    while stem.len() + suffix.len() > 200 && stem.chars().count() > 1 {
        stem.pop();
    }
    dir.join(format!("{}{suffix}", stem.trim_end()))
}

/// The in-flight fragmented file topaz-encode writes for `output`.
pub fn frag_path(output: &Path) -> PathBuf {
    with_suffix(output, ".frag")
}

/// The tail a resumed encode writes before joining it onto the partial.
pub fn resume_tail_path(output: &Path) -> PathBuf {
    with_suffix(output, " [resume tail]")
}

fn with_suffix(output: &Path, suffix: &str) -> PathBuf {
    let ext = output.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_default();
    let stem = output.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    output.with_file_name(format!("{stem}{suffix}.{ext}"))
}

fn short_label(name: &str) -> &str {
    name.split(" + ").next().unwrap_or(name)
}

fn file_label(label: &str) -> String {
    let mut l = label.replace('/', " - ").replace("->", "to");
    if !l.contains(' ') {
        l = l.replace('-', " ");
    }
    l = l.replace("  ", " ");
    let l = l.strip_prefix(' ').unwrap_or(&l);
    let l = l.strip_suffix(' ').unwrap_or(l);
    l.to_string()
}

fn strip_trailing_brackets(stem: &str) -> String {
    let mut s = stem.trim_end().to_string();
    while s.ends_with(']') {
        match s.rfind('[') {
            Some(i) => s = s[..i].trim_end().to_string(),
            None => break,
        }
    }
    if s.trim().is_empty() { stem.to_string() } else { s }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enh(body: &str, scales: &[&str], free_size: bool) -> Enhancement {
        Enhancement {
            category: "polish".into(),
            display: "Proteus — Sharp".into(),
            slug: "proteus-sharp".into(),
            scales: scales.iter().map(|s| s.to_string()).collect(),
            body: body.into(),
            blurb: String::new(),
            metadata: String::new(),
            free_size,
        }
    }

    fn res(key: &'static str, w: u32, h: u32) -> ResOption {
        ResOption { key, label: String::new(), w, h }
    }

    #[test]
    fn scale_clauses_follow_topaz_pick() {
        let e = enh("tvai_up=model=prob-4:@SCALE@:x=1", &["1", "2", "4k"], false);
        assert_eq!(compose_filter(&e, &res("orig", 1920, 1080), None), "tvai_up=model=prob-4:scale=1:x=1");
        assert_eq!(compose_filter(&e, &res("2x", 1280, 720), None), "tvai_up=model=prob-4:scale=2:x=1");
        assert_eq!(
            compose_filter(&e, &res("4k", 3840, 2160), None),
            "tvai_up=model=prob-4:scale=0:w=3840:h=2160:x=1,scale=w=3840:h=2160:flags=lanczos:threads=0"
        );
        // A scale=2-only model reaching 4K from exactly 1080p.
        let two = enh("m:@SCALE@", &["2"], false);
        assert_eq!(compose_filter(&two, &res("4k", 3840, 2160), None), "m:scale=2");
        // free_size: explicit size even at the original resolution, no tail there.
        let mini = enh("slm:@SCALE@", &["1", "2", "4k"], true);
        assert_eq!(compose_filter(&mini, &res("orig", 1280, 720), None), "slm:scale=0:w=1280:h=720");
    }

    #[test]
    fn interpolation_goes_before_the_tail() {
        let e = enh("m:@SCALE@", &["4k"], false);
        let i = Interpolation {
            display: "Apollo".into(),
            slug: "apollo".into(),
            filter: "tvai_fi=model=apo-8:slowmo=2:fps=60".into(),
            metadata: String::new(),
        };
        assert_eq!(
            compose_filter(&e, &res("4k", 3840, 2160), Some(&i)),
            "m:scale=0:w=3840:h=2160,tvai_fi=model=apo-8:slowmo=2:fps=60,scale=w=3840:h=2160:flags=lanczos:threads=0"
        );
        assert_eq!(i.rate(), (Some(60.0), 2.0));
        let orig = Enhancement { body: String::new(), ..e };
        assert_eq!(compose_filter(&orig, &res("orig", 1920, 1080), None), "null");
    }

    #[test]
    fn output_names_match_topaz_encode() {
        let p = output_path(Path::new("/v/clip [1080p 30fps].mp4"), "Proteus — Sharp → 4K + Apollo - HEVC", "mp4");
        assert_eq!(p, PathBuf::from("/v/clip [Topaz - Proteus — Sharp → 4K].mp4"));
        assert_eq!(frag_path(&p), PathBuf::from("/v/clip [Topaz - Proteus — Sharp → 4K].frag.mp4"));
        let p = output_path(Path::new("/v/a.mov"), "slug-only", "mov");
        assert_eq!(p, PathBuf::from("/v/a [Topaz - slug only].mov"));
    }
}
