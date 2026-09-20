//! A full-clip encode, run in a thread while the TUI stays up: progress from
//! the encoder (nsencode), and the newest frame of the half-written output.
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
//! pass rewrites it as an ordinary file at the end.

use crate::nsencode::{self, Cancel, Options};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub enum Event {
    Stage(String),
    Progress { pct: Option<u32>, frame: Option<u64>, message: Option<String> },
    /// A freshly extracted JPEG of the newest frame in the partial output.
    LiveFrame(PathBuf),
    Done { ok: bool, detail: String },
}

pub struct Encode {
    pub rx: Receiver<Event>,
    cancel: Cancel,
    pub output: PathBuf,
    pub started: Instant,
}

impl Encode {
    /// Stops neuroserver's whole process group; what it wrote stays on disk
    /// for a resume.
    pub fn cancel(&self) {
        self.cancel.cancel();
    }
}

pub struct Request {
    pub options: Options,
    pub output: PathBuf,
    pub partial: PathBuf,
    pub scratch: PathBuf,
}

pub fn spawn(req: Request) -> Encode {
    let (tx, rx) = mpsc::channel();
    let cancel = Cancel::isolated();
    let output = req.output.clone();
    let c2 = cancel.clone();
    thread::spawn(move || run(req, tx, c2));
    Encode { rx, cancel, output, started: Instant::now() }
}

fn run(req: Request, tx: Sender<Event>, cancel: Cancel) {
    let stop = Arc::new(AtomicBool::new(false));
    let live = {
        let (tx, stop) = (tx.clone(), stop.clone());
        let (partial, scratch) = (req.partial.clone(), req.scratch.clone());
        thread::spawn(move || watch_live(&partial, &scratch, &tx, &stop))
    };
    let result = nsencode::plan(&req.options).and_then(|plan| {
        nsencode::run(&plan, &cancel, &mut |ev| {
            let _ = tx.send(match ev {
                nsencode::Event::Stage(s) | nsencode::Event::Warn(s) => Event::Stage(s),
                nsencode::Event::Progress { pct, frame, message } => Event::Progress { pct, frame, message },
            });
        })
    });
    stop.store(true, Ordering::SeqCst);
    let _ = live.join();
    let _ = tx.send(match result {
        Ok(out) => Event::Done { ok: true, detail: out.display().to_string() },
        // One status row: fold the explanation onto it.
        Err(e) => Event::Done { ok: false, detail: format!("{e:#}").replace('\n', "  ") },
    });
}

/// Newest frame, whenever the partial file has grown — at most every 4s.
fn watch_live(partial: &Path, scratch: &Path, tx: &Sender<Event>, stop: &AtomicBool) {
    let _ = std::fs::create_dir_all(scratch);
    let mut last_size: u64 = 0;
    let mut last_grab = Instant::now() - Duration::from_secs(60);
    let mut serial: u64 = 0;
    while !stop.load(Ordering::SeqCst) {
        let size = std::fs::metadata(partial).map(|m| m.len()).unwrap_or(0);
        if size > last_size && last_grab.elapsed() >= Duration::from_secs(4) {
            last_size = size;
            last_grab = Instant::now();
            serial += 1;
            let jpg = scratch.join(format!("live-{serial:05}.jpg"));
            if grab_last_frame(partial, &jpg) {
                if serial > 1 {
                    let _ = std::fs::remove_file(scratch.join(format!("live-{:05}.jpg", serial - 1)));
                }
                let _ = tx.send(Event::LiveFrame(jpg));
            }
        }
        thread::sleep(Duration::from_millis(400));
    }
}

/// The last ~2 seconds of the fragmented partial, decoded, keeping the final
/// frame. A shorter window fails: the tail fragment may not be complete yet.
fn grab_last_frame(partial: &Path, jpg: &Path) -> bool {
    Command::new(crate::probe::ffmpeg_path())
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
