//! A full-clip encode, run as a child neuroserver-encode while the TUI stays
//! up: progress from its log, and the newest frame of the half-written output.
//!
//! Starlight works in chunks of about 100 source frames, each going through
//! encode → upscale → decode → post-processing; neuroserver pipes a chunk's
//! frames to its ffmpeg only at that last step. So the output grows in bursts,
//! one chunk at a time (about 80 minutes apart at 4K on an M4 Pro), and the
//! live frame advances with it — not continuously.
//!
//! That last part only works because the output is written as *fragmented*
//! MP4/MOV (the flags the Topaz app itself uses for exports): a plain MP4 has
//! no index until it is closed and cannot be opened mid-write at all. The mux
//! pass in neuroserver-encode rewrites it as an ordinary file at the end.

use crate::logtail::{LogEvent, Tail};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub const FRAG_FLAGS: &str = "-movflags frag_keyframe+empty_moov+delay_moov";

pub enum Event {
    Stage(String),
    Progress { pct: Option<u32>, frame: Option<u64>, message: Option<String> },
    /// A freshly extracted JPEG of the newest frame in the partial output.
    LiveFrame(PathBuf),
    Done { ok: bool, detail: String },
}

pub struct Encode {
    pub rx: Receiver<Event>,
    pgid: Arc<Mutex<Option<i32>>>,
    pub output: PathBuf,
    pub started: Instant,
}

impl Encode {
    /// SIGTERM the whole process group: the script runs neuroserver in a
    /// pipeline subshell, so signalling only the script would orphan it.
    pub fn cancel(&self) {
        if let Some(pgid) = *self.pgid.lock().unwrap() {
            let _ = Command::new("kill").arg("-TERM").arg(format!("-{pgid}")).status();
        }
    }
}

pub struct Request {
    pub command: Vec<String>,
    pub output: PathBuf,
    pub partial: PathBuf,
    pub log: PathBuf,
    pub scratch: PathBuf,
}

pub fn spawn(req: Request) -> Encode {
    let (tx, rx) = mpsc::channel();
    let pgid = Arc::new(Mutex::new(None));
    let output = req.output.clone();
    let p2 = pgid.clone();
    thread::spawn(move || run(req, tx, p2));
    Encode { rx, pgid, output, started: Instant::now() }
}

fn run(req: Request, tx: Sender<Event>, pgid: Arc<Mutex<Option<i32>>>) {
    let mut cmd = Command::new(&req.command[0]);
    cmd.args(&req.command[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .process_group(0);
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(Event::Done { ok: false, detail: format!("cannot run {}: {e}", req.command[0]) });
            return;
        }
    };
    *pgid.lock().unwrap() = Some(child.id() as i32);
    let mut stderr = child.stderr.take().unwrap();
    let err_h = thread::spawn(move || {
        let mut s = String::new();
        let _ = std::io::Read::read_to_string(&mut stderr, &mut s);
        s
    });

    let _ = std::fs::create_dir_all(&req.scratch);
    let mut tail = Tail::default();
    let mut last_size: u64 = 0;
    let mut last_grab = Instant::now() - Duration::from_secs(60);
    let mut serial: u64 = 0;
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break Some(st),
            Ok(None) => {}
            Err(_) => break None,
        }
        for ev in tail.poll(&req.log) {
            let _ = tx.send(match ev {
                LogEvent::Stage(s) => Event::Stage(s),
                LogEvent::Progress { pct, frame, message } => Event::Progress { pct, frame, message },
            });
        }
        // Newest frame, whenever the partial file has grown — at most every 4s.
        let size = std::fs::metadata(&req.partial).map(|m| m.len()).unwrap_or(0);
        if size > last_size && last_grab.elapsed() >= Duration::from_secs(4) {
            last_size = size;
            last_grab = Instant::now();
            serial += 1;
            let jpg = req.scratch.join(format!("live-{serial:05}.jpg"));
            if grab_last_frame(&req.partial, &jpg) {
                if serial > 1 {
                    let _ = std::fs::remove_file(req.scratch.join(format!("live-{:05}.jpg", serial - 1)));
                }
                let _ = tx.send(Event::LiveFrame(jpg));
            }
        }
        thread::sleep(Duration::from_millis(400));
    };
    for ev in tail.poll(&req.log) {
        if let LogEvent::Progress { pct, frame, message } = ev {
            let _ = tx.send(Event::Progress { pct, frame, message });
        }
    }
    *pgid.lock().unwrap() = None;
    let stderr = err_h.join().unwrap_or_default();
    let ok = status.map(|s| s.success()).unwrap_or(false);
    let detail = if ok {
        req.output.display().to_string()
    } else {
        let last = stderr.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("").trim().to_string();
        if last.is_empty() { format!("encode failed — see {}", req.log.display()) } else { last }
    };
    let _ = tx.send(Event::Done { ok, detail });
}

/// The last ~2 seconds of the fragmented partial, decoded, keeping the final
/// frame. A shorter window fails: the tail fragment may not be complete yet.
fn grab_last_frame(partial: &Path, jpg: &Path) -> bool {
    let ffmpeg = crate::probe::ffprobe_path().with_file_name("ffmpeg");
    let ffmpeg = if ffmpeg.is_file() { ffmpeg } else { PathBuf::from("ffmpeg") };
    Command::new(ffmpeg)
        .args(["-v", "error", "-y", "-sseof", "-2", "-i"])
        .arg(partial)
        .args(["-update", "1", "-q:v", "3"])
        .arg(jpg)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success() && jpg.is_file())
        .unwrap_or(false)
}

/// "<dir>/<stem minus trailing [tags]> [Topaz - <preset>].<ext>" — the same
/// name neuroserver-encode would pick, computed here so the TUI knows where
/// the partial file will appear.
pub fn output_path(input: &Path, preset_name: &str, ext: &str) -> PathBuf {
    let stem = input.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let mut clean = stem.trim_end().to_string();
    while clean.ends_with(']') {
        match clean.rfind('[') {
            Some(i) => clean = clean[..i].trim_end().to_string(),
            None => break,
        }
    }
    if clean.trim().is_empty() {
        clean = stem;
    }
    let safe: String = preset_name.chars().map(|c| if c == '/' || c == ':' { '-' } else { c }).collect();
    input
        .parent()
        .unwrap_or(Path::new("."))
        .join(format!("{clean} [Topaz - {safe}].{ext}"))
}
