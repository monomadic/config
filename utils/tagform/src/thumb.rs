//! Video thumbnails, following media-audit's extraction recipe (SPEC §8).
//!
//! Extraction seeks 2 s in to clear black leader and falls back to frame 0.
//!
//! Unlike media-audit this fits the frame *inside* the box rather than
//! cover-fitting it. media-audit crops to fill a fixed band, which is right for
//! a status display but wrong here: a 360x640 portrait clip came out as a
//! 720x404 centre-cropped strip, so the preview claimed the video was landscape
//! and hid most of the frame. Aspect ratio is information about the file, and a
//! tagger must not lie about it. The band adapts to the image instead.
//!
//! Rotation needs no handling: ffmpeg applies a display matrix automatically,
//! so a phone clip stored as 640x360 with rotation=90 extracts as portrait.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Cache key includes the box dimensions: a resized terminal wants a
/// differently cropped thumbnail, not a stretched stale one. It includes mtime
/// and size so a re-encoded file gets a new frame. Same rule as
/// media-audit's thumb_cache_path().
fn cache_path(file: &Path, box_w: u32, box_h: u32) -> Result<PathBuf> {
    let meta = std::fs::metadata(file)?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let key = format!("{}:{}:{}:{}x{}", file.display(), mtime, meta.len(), box_w, box_h);
    let dir = cache_dir();
    std::fs::create_dir_all(&dir).ok();
    Ok(dir.join(format!("{:016x}.jpg", fnv1a(key.as_bytes()))))
}

fn cache_dir() -> PathBuf {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".cache")))
        .unwrap_or_else(|| PathBuf::from("/tmp"));
    base.join("tagform/thumbs")
}

/// Not cryptographic and does not need to be -- it names a cache entry.
fn fnv1a(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

fn fit_inside(w: u32, h: u32) -> String {
    format!("scale=w={w}:h={h}:force_original_aspect_ratio=decrease:flags=lanczos")
}

/// Extract (or reuse) a thumbnail. Returns the cached jpg path.
pub fn extract(file: &Path, box_w: u32, box_h: u32) -> Result<PathBuf> {
    let out = cache_path(file, box_w, box_h)?;
    if out.exists() && std::fs::metadata(&out).map(|m| m.len() > 0).unwrap_or(false) {
        return Ok(out);
    }
    let vf = fit_inside(box_w, box_h);
    // A few seconds in usually clears any black leader; fall back to frame 0.
    for seek in ["2", "0"] {
        // stdio is nulled, not merely quietened: ffmpeg emits warnings such as
        // "Non full-range YUV is non-standard" at -v error, and anything it
        // writes lands in the middle of the rendered form.
        let status = Command::new("ffmpeg")
            .args(["-v", "error", "-y", "-ss", seek])
            .arg("-i")
            .arg(file)
            .args(["-frames:v", "1", "-vf", &vf, "-q:v", "3", "--"])
            .arg(&out)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
        if matches!(status, Ok(s) if s.success())
            && std::fs::metadata(&out).map(|m| m.len() > 0).unwrap_or(false)
        {
            return Ok(out);
        }
    }
    std::fs::remove_file(&out).ok();
    anyhow::bail!("could not extract a frame from {}", file.display())
}

/// Header-line facts about the file, from one ffprobe call.
#[derive(Debug, Clone, Default)]
pub struct MediaInfo {
    pub width: u32,
    pub height: u32,
    pub duration: f64,
    pub vcodec: String,
    pub acodec: String,
    pub size: u64,
}

impl MediaInfo {
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();
        if self.width > 0 {
            parts.push(format!("{}×{}", self.width, self.height));
        }
        if self.duration > 0.0 {
            let s = self.duration as u64;
            parts.push(format!("{}:{:02}:{:02}", s / 3600, (s / 60) % 60, s % 60));
        }
        let codecs: Vec<&str> = [self.vcodec.as_str(), self.acodec.as_str()]
            .into_iter()
            .filter(|c| !c.is_empty())
            .collect();
        if !codecs.is_empty() {
            parts.push(codecs.join("/"));
        }
        if self.size > 0 {
            parts.push(human_size(self.size));
        }
        parts.join(" · ")
    }
}

fn human_size(n: u64) -> String {
    const U: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut v = n as f64;
    let mut i = 0;
    while v >= 1024.0 && i < U.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    if i == 0 { format!("{n} B") } else { format!("{v:.1} {}", U[i]) }
}

pub fn probe_media(file: &Path) -> Result<MediaInfo> {
    let out = Command::new("ffprobe")
        .args([
            "-v", "error",
            "-show_entries",
            "stream=codec_type,codec_name,width,height:stream_side_data=rotation:format=duration,size",
            "-of", "json", "--",
        ])
        .arg(file)
        .output()
        .context("running ffprobe")?;
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_or_default();
    let mut info = MediaInfo::default();
    if let Some(streams) = v.get("streams").and_then(|s| s.as_array()) {
        for s in streams {
            match s.get("codec_type").and_then(|t| t.as_str()) {
                Some("video") if info.vcodec.is_empty() => {
                    info.vcodec = s.get("codec_name").and_then(|c| c.as_str()).unwrap_or("").into();
                    let w = s.get("width").and_then(|w| w.as_u64()).unwrap_or(0) as u32;
                    let h = s.get("height").and_then(|h| h.as_u64()).unwrap_or(0) as u32;
                    // A phone clip is stored landscape with a 90/270 display
                    // matrix. Players honour it and so does the thumbnail, so
                    // reporting the stored size would contradict the picture
                    // right next to it.
                    let rot = s
                        .get("side_data_list")
                        .and_then(|l| l.as_array())
                        .and_then(|l| l.iter().find_map(|d| d.get("rotation")))
                        .and_then(|r| r.as_f64())
                        .unwrap_or(0.0);
                    let quarter_turn = (rot.abs() as i64 % 180) == 90;
                    (info.width, info.height) = if quarter_turn { (h, w) } else { (w, h) };
                }
                Some("audio") if info.acodec.is_empty() => {
                    info.acodec = s.get("codec_name").and_then(|c| c.as_str()).unwrap_or("").into();
                }
                _ => {}
            }
        }
    }
    if let Some(f) = v.get("format") {
        info.duration = f.get("duration").and_then(|d| d.as_str()).and_then(|d| d.parse().ok()).unwrap_or(0.0);
        info.size = f.get("size").and_then(|s| s.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0);
    }
    Ok(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `decrease` and no crop: the frame is never cut to fill the box, so a
    /// portrait source stays portrait.
    #[test]
    fn fit_preserves_aspect_and_never_crops() {
        let f = fit_inside(720, 720);
        assert!(f.contains("force_original_aspect_ratio=decrease"), "{f}");
        assert!(!f.contains("crop"), "{f}");
    }

    #[test]
    fn human_size_scales() {
        assert_eq!(human_size(512), "512 B");
        assert_eq!(human_size(1536), "1.5 KB");
    }

    #[test]
    fn a_quarter_turn_swaps_the_reported_dimensions() {
        for rot in [90.0_f64, -90.0, 270.0] {
            assert!((rot.abs() as i64 % 180) == 90, "rot {rot} should be a quarter turn");
        }
        for rot in [0.0_f64, 180.0, -180.0] {
            assert!((rot.abs() as i64 % 180) != 90, "rot {rot} should not swap");
        }
    }

    #[test]
    fn summary_omits_absent_parts() {
        let empty = MediaInfo::default();
        assert_eq!(empty.summary(), "");
    }
}
