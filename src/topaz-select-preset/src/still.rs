//! The unrendered source frame at the chosen time, shown before any preview
//! render exists. One ffmpeg grab per time, in a thread; the frame is the one
//! topaz-preview-frame targets for the same time, round(t × fps).

use anyhow::{anyhow, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;

pub struct Job {
    pub frame: u64,
    pub rx: Receiver<Result<PathBuf>>,
}

pub fn frame_at(time: f64, fps: f64) -> u64 {
    (time * fps + 0.5).max(0.0) as u64
}

pub fn spawn(input: &Path, frame: u64, fps: f64, scratch: &Path) -> Job {
    let (tx, rx) = mpsc::channel();
    let input = input.to_path_buf();
    let out = scratch.join(format!("source-{frame:07}.jpg"));
    thread::spawn(move || {
        let _ = tx.send(grab(&input, frame, fps, out));
    });
    Job { frame, rx }
}

fn grab(input: &Path, frame: u64, fps: f64, out: PathBuf) -> Result<PathBuf> {
    // Half a frame early, so the accurate seek lands on `frame` itself.
    let seek = ((frame as f64 - 0.5) / fps.max(1e-6)).max(0.0);
    let status = Command::new(crate::probe::ffmpeg_path())
        .args(["-hide_banner", "-nostdin", "-y", "-loglevel", "error"])
        .arg("-ss").arg(format!("{seek:.6}"))
        .arg("-i").arg(input)
        .args(["-frames:v", "1", "-q:v", "2"])
        .arg(&out)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|e| anyhow!("cannot run ffmpeg: {e}"))?;
    if !status.success() || !out.is_file() {
        return Err(anyhow!("ffmpeg could not read source frame {frame}"));
    }
    Ok(out)
}
