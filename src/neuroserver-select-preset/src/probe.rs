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
    /// The container's frame count, when it declares one.
    pub nb_frames: Option<u64>,
}

#[derive(Clone, Debug)]
pub struct ResOption {
    pub key: &'static str,
    pub label: String,
    pub w: u32,
    pub h: u32,
}

/// The Topaz app bundle: `TOPAZ_APP` if set, else the first known install —
/// the same rule as topaz_resolve_app in lib/topaz-app.zsh.
pub fn topaz_app() -> Option<PathBuf> {
    if let Ok(app) = std::env::var("TOPAZ_APP") {
        let app = PathBuf::from(app.trim_end_matches('/'));
        return app.is_dir().then_some(app);
    }
    ["/Applications/Topaz Video.app", "/Applications/Topaz Video AI.app"]
        .into_iter()
        .map(PathBuf::from)
        .find(|p| p.is_dir())
}

/// A binary from the app's Contents/MacOS, falling back to PATH.
pub fn topaz_tool(name: &str) -> PathBuf {
    topaz_app()
        .map(|app| app.join("Contents/MacOS").join(name))
        .filter(|p| p.is_file())
        .unwrap_or_else(|| PathBuf::from(name))
}

pub fn ffprobe_path() -> PathBuf {
    topaz_tool("ffprobe")
}

pub fn ffmpeg_path() -> PathBuf {
    topaz_tool("ffmpeg")
}

pub fn probe(file: &Path) -> Result<Profile> {
    let out = Command::new(ffprobe_path())
        .args([
            "-v", "error", "-select_streams", "v:0",
            "-show_entries", "stream=width,height,nb_frames,r_frame_rate:format=duration",
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
            "nb_frames" => p.nb_frames = v.parse().ok().filter(|n| *n > 0),
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

impl Profile {
    /// Frames in the clip: the declared count, else duration × rate.
    pub fn total_frames(&self) -> u64 {
        self.nb_frames.unwrap_or_else(|| (self.duration * self.fps).round().max(0.0) as u64)
    }
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

/// ffprobe over the first video stream, stdout lines.
fn ffprobe_lines(file: &Path, args: &[&str]) -> Vec<String> {
    Command::new(ffprobe_path())
        .args(["-v", "error", "-select_streams", "v:0"])
        .args(args)
        .arg("--")
        .arg(file)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().map(|l| l.trim().to_string()).collect())
        .unwrap_or_default()
}

fn ffprobe_number(file: &Path, args: &[&str]) -> Option<u64> {
    ffprobe_lines(file, args).first().and_then(|l| l.parse().ok())
}

/// Video packets in `file` — what survives of a half-written, fragmented
/// output. Zero for a missing or unreadable file.
pub fn count_frames(file: &Path) -> u64 {
    ffprobe_number(file, &["-count_packets", "-show_entries", "stream=nb_read_packets", "-of", "csv=p=0"]).unwrap_or(0)
}

/// Frames that actually decode — fewer than the packets when a kill tore the last one.
pub fn count_decoded_frames(file: &Path) -> Option<u64> {
    ffprobe_number(file, &["-count_frames", "-show_entries", "stream=nb_read_frames", "-of", "csv=p=0"])
}

pub fn has_b_frames(file: &Path) -> Option<u64> {
    ffprobe_number(file, &["-show_entries", "stream=has_b_frames", "-of", "csv=p=0"])
}

/// Index of the last keyframe packet (0 when there is none).
pub fn last_keyframe(file: &Path) -> u64 {
    ffprobe_lines(file, &["-show_entries", "packet=flags", "-of", "csv=p=0"])
        .iter()
        .rposition(|flags| flags.contains('K'))
        .unwrap_or(0) as u64
}
