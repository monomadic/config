//! One preview render = one topaz-preview-frame run with --keep-window, in a
//! thread: the stage markers and ffmpeg's frame counter from its log go back to
//! the UI while it runs, the window's frame paths (from --print-paths) when it
//! is done.

use crate::logtail::{LogEvent, Tail};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone, Debug, Default)]
pub struct RenderResult {
    pub window_start: u64,
    /// Offset of the requested frame in the *source* window.
    pub target: usize,
    /// Topaz frames per source frame: 1, or more when interpolation is in.
    pub ratio: f64,
    /// offset in window -> path. Topaz offsets count Topaz frames, so with
    /// interpolation there are `ratio` of them per source offset.
    pub topaz_frames: BTreeMap<usize, PathBuf>,
    pub original_frames: BTreeMap<usize, PathBuf>,
    pub seconds: f64,
}

impl RenderResult {
    /// The Topaz frame that shows the requested source frame.
    pub fn topaz_target(&self) -> usize {
        (self.target as f64 * self.ratio).round() as usize
    }

    /// The source frame nearest a Topaz frame (an invented frame sits between
    /// two; this picks the closer).
    pub fn source_offset(&self, topaz: usize) -> usize {
        let src = (topaz as f64 / self.ratio.max(1e-6)).round() as usize;
        let last = self.original_frames.keys().next_back().copied().unwrap_or(0);
        src.min(last)
    }
}

pub enum Event {
    Stage(String),
    Frame(u64),
    Note(String),
    Done(Result<RenderResult>),
}

pub struct Job {
    pub rx: Receiver<Event>,
    pid: Arc<Mutex<Option<u32>>>,
    pub key: String,
}

impl Job {
    /// SIGTERM the wrapper; its trap forwards the signal to ffmpeg.
    pub fn cancel(&self) {
        if let Some(pid) = *self.pid.lock().unwrap() {
            let _ = Command::new("kill").arg("-TERM").arg(pid.to_string()).status();
        }
    }
}

pub struct Request {
    pub input: PathBuf,
    pub label: String,
    pub filter: String,
    pub time: f64,
    pub window: u32,
    pub keep_interpolation: bool,
    pub key: String,
}

fn log_path() -> PathBuf {
    std::env::temp_dir().join(format!("topaz-select-preset-{}.log", std::process::id()))
}

pub fn spawn(req: Request) -> Job {
    let (tx, rx) = mpsc::channel();
    let pid = Arc::new(Mutex::new(None));
    let pid2 = pid.clone();
    let key = req.key.clone();
    thread::spawn(move || run(req, tx, pid2));
    Job { rx, pid, key }
}

fn run(req: Request, tx: Sender<Event>, pid: Arc<Mutex<Option<u32>>>) {
    let script = crate::catalog::zsh_bin().join("topaz-preview-frame");
    let log = log_path();
    let _ = fs::write(&log, "");
    let started = std::time::Instant::now();

    let mut cmd = Command::new(&script);
    cmd.arg("--input").arg(&req.input)
        .arg("--preset-name").arg(&req.label)
        .arg("--preset-flag").arg("--filter_complex")
        .arg("--filter").arg(&req.filter)
        .arg("--time").arg(format!("{:.3}", req.time))
        .arg("--no-open")
        .arg("--print-paths")
        .arg("--keep-window")
        .arg("--window").arg(req.window.to_string())
        .arg("--log-file").arg(&log);
    if req.keep_interpolation {
        cmd.arg("--keep-interpolation");
    }
    cmd.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(Event::Done(Err(anyhow!("cannot run {}: {e}", script.display()))));
            return;
        }
    };
    *pid.lock().unwrap() = Some(child.id());

    // Drain stdout/stderr in threads (a full pipe would block the render), and
    // tail the log here for progress until the child exits.
    let mut stdout = child.stdout.take().unwrap();
    let mut stderr = child.stderr.take().unwrap();
    let out_h = thread::spawn(move || {
        let mut s = String::new();
        let _ = stdout.read_to_string(&mut s);
        s
    });
    let err_h = thread::spawn(move || {
        let mut s = String::new();
        let _ = stderr.read_to_string(&mut s);
        s
    });

    let mut tail = Tail::default();
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break Ok(st),
            Ok(None) => {}
            Err(e) => break Err(e),
        }
        forward(&mut tail, &log, &tx);
        thread::sleep(Duration::from_millis(200));
    };
    forward(&mut tail, &log, &tx);
    *pid.lock().unwrap() = None;

    let stdout = out_h.join().unwrap_or_default();
    let stderr = err_h.join().unwrap_or_default();
    let result = match status {
        Ok(st) if st.success() => parse_paths(&stdout).map(|mut r| {
            r.seconds = started.elapsed().as_secs_f64();
            r
        }),
        Ok(st) => Err(anyhow!("{}", failure_reason(&log, &stderr, st.code()))),
        Err(e) => Err(anyhow!("waiting for render: {e}")),
    };
    let _ = tx.send(Event::Done(result));
}

fn forward(tail: &mut Tail, log: &Path, tx: &Sender<Event>) {
    for ev in tail.poll(log) {
        let _ = tx.send(match ev {
            LogEvent::Stage(s) => Event::Stage(s),
            LogEvent::Frame(n) => Event::Frame(n),
            LogEvent::Note(n) => Event::Note(n),
        });
    }
}

fn failure_reason(log: &Path, stderr: &str, code: Option<i32>) -> String {
    // The script's own refusals (a model with no weights, which would hang
    // rather than fail) come first on stderr and say what to do; keep them whole.
    let stderr = stderr.trim();
    if !stderr.is_empty() {
        return stderr.lines().filter(|l| !l.trim().is_empty()).take(3).collect::<Vec<_>>().join("  ");
    }
    if let Ok(text) = fs::read_to_string(log) {
        let hit = text
            .split(['\n', '\r'])
            .rev()
            .find(|l| {
                let l = l.to_ascii_lowercase();
                l.contains("error") || l.contains("failed") || l.contains("invalid")
            })
            .map(|l| l.trim().to_string());
        if let Some(h) = hit {
            return h;
        }
    }
    format!("render exited with status {:?} (log: {})", code, log.display())
}

fn parse_paths(stdout: &str) -> Result<RenderResult> {
    let mut r = RenderResult { ratio: 1.0, ..Default::default() };
    let mut topaz = PathBuf::new();
    let mut original = PathBuf::new();
    for line in stdout.lines() {
        let Some((k, v)) = line.split_once('=') else { continue };
        match k {
            "TOPAZ_PREVIEW_ORIGINAL" => original = PathBuf::from(v),
            "TOPAZ_PREVIEW_TOPAZ" => topaz = PathBuf::from(v),
            "TOPAZ_PREVIEW_WINDOW_START" => r.window_start = v.parse().unwrap_or(0),
            "TOPAZ_PREVIEW_WINDOW_TARGET" => r.target = v.parse().unwrap_or(0),
            "TOPAZ_PREVIEW_WINDOW_RATIO" => r.ratio = v.parse().ok().filter(|x: &f64| *x > 0.0).unwrap_or(1.0),
            "TOPAZ_PREVIEW_WINDOW_TOPAZ" | "TOPAZ_PREVIEW_WINDOW_ORIGINAL" => {
                let Some((idx, path)) = v.split_once('\t') else { continue };
                let Ok(idx) = idx.parse::<usize>() else { continue };
                let map = if k == "TOPAZ_PREVIEW_WINDOW_TOPAZ" {
                    &mut r.topaz_frames
                } else {
                    &mut r.original_frames
                };
                map.insert(idx, PathBuf::from(path));
            }
            _ => {}
        }
    }
    if topaz.as_os_str().is_empty() {
        return Err(anyhow!("render printed no preview path"));
    }
    if r.topaz_frames.is_empty() {
        r.topaz_frames.insert(r.topaz_target(), topaz);
    }
    if r.original_frames.is_empty() && !original.as_os_str().is_empty() {
        r.original_frames.insert(r.target, original);
    }
    Ok(r)
}
