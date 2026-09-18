//! Source geometry, and the resolution rows derived from it — the same rows
//! and the same rules as the mpv menu's Output tab and topaz-pick.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Clone, Debug, Default)]
pub struct Profile {
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub duration: f64,
}

#[derive(Clone, Debug)]
pub struct ResOption {
    pub key: &'static str,
    pub label: String,
    pub w: u32,
    pub h: u32,
}

pub fn ffprobe_path() -> PathBuf {
    for app in ["/Applications/Topaz Video.app", "/Applications/Topaz Video AI.app"] {
        let p = Path::new(app).join("Contents/MacOS/ffprobe");
        if p.is_file() {
            return p;
        }
    }
    PathBuf::from("ffprobe")
}

pub fn probe(file: &Path) -> Result<Profile> {
    let out = Command::new(ffprobe_path())
        .args([
            "-v", "error", "-select_streams", "v:0",
            "-show_entries", "stream=width,height,r_frame_rate:format=duration",
            "-of", "default=noprint_wrappers=1", "--",
        ])
        .arg(file)
        .output()
        .context("running ffprobe")?;
    let mut p = Profile::default();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let Some((k, v)) = line.split_once('=') else { continue };
        match k {
            "width" => p.width = v.parse().unwrap_or(0),
            "height" => p.height = v.parse().unwrap_or(0),
            "duration" => p.duration = v.parse().unwrap_or(0.0),
            "r_frame_rate" => {
                p.fps = match v.split_once('/') {
                    Some((n, d)) => {
                        let n: f64 = n.parse().unwrap_or(0.0);
                        let d: f64 = d.parse().unwrap_or(1.0);
                        if d > 0.0 { n / d } else { 0.0 }
                    }
                    None => v.parse().unwrap_or(0.0),
                };
            }
            _ => {}
        }
    }
    if p.fps <= 0.0 {
        p.fps = 25.0;
    }
    Ok(p)
}

/// Orientation-aware 4K target: long edge 3840, aspect kept, even dimensions.
fn target_4k(w: u32, h: u32) -> (u32, u32) {
    let (tw, th) = if w >= h {
        (3840.0, h as f64 * 3840.0 / w as f64)
    } else {
        (w as f64 * 3840.0 / h as f64, 3840.0)
    };
    let even = |x: f64| ((x / 2.0).round() as u32) * 2;
    (even(tw), even(th))
}

pub struct ResSet {
    pub options: Vec<ResOption>,
    pub default: usize,
    pub two_x_is_4k: bool,
}

pub fn res_options(p: &Profile) -> ResSet {
    let (sw, sh) = (p.width, p.height);
    if sw == 0 || sh == 0 {
        return ResSet {
            options: vec![ResOption { key: "orig", label: "Original".into(), w: 0, h: 0 }],
            default: 0,
            two_x_is_4k: false,
        };
    }
    let (tw, th) = target_4k(sw, sh);
    let two_x_is_4k = sw * 2 == tw && sh * 2 == th;
    let mut options = vec![ResOption {
        key: "orig",
        label: format!("Original  {sw}×{sh}"),
        w: sw,
        h: sh,
    }];
    let is_4k = sw.max(sh) > 2560;
    if !is_4k {
        let mut ups: Vec<ResOption> = Vec::new();
        let mut add = |key: &'static str, name: &str, w: u32, h: u32| {
            if w.max(h) > 7680 || ups.iter().any(|o| o.w == w && o.h == h) {
                return;
            }
            ups.push(ResOption { key, label: format!("{name}  {w}×{h}"), w, h });
        };
        add("4k", "Upscale to 4K", tw, th);
        add("2x", "Upscale 2x", sw * 2, sh * 2);
        add("4x", "Upscale 4x", sw * 4, sh * 4);
        ups.sort_by_key(|o| o.w as u64 * o.h as u64);
        options.extend(ups);
    }
    let default = options.iter().position(|o| o.key == "4k").unwrap_or(0);
    ResSet { options, default, two_x_is_4k }
}

/// Mirrors res_available in the mpv menu.
pub fn res_available(scales: &[String], opt: &ResOption, two_x_is_4k: bool) -> bool {
    let has = |s: &str| scales.iter().any(|x| x == s);
    match opt.key {
        "orig" => has("1"),
        "2x" => has("2"),
        "4x" => has("4k"),
        _ => has("4k") || (two_x_is_4k && has("2")),
    }
}

/// Decodable video frames in `file` — what survives of a half-written,
/// fragmented output. Zero for a missing or unreadable file.
pub fn count_frames(file: &Path) -> u64 {
    Command::new(ffprobe_path())
        .args(["-v", "error", "-count_packets", "-select_streams", "v:0",
               "-show_entries", "stream=nb_read_packets", "-of", "csv=p=0", "--"])
        .arg(file)
        .output()
        .ok()
        .and_then(|o| String::from_utf8_lossy(&o.stdout).lines().next().and_then(|l| l.trim().parse().ok()))
        .unwrap_or(0)
}
