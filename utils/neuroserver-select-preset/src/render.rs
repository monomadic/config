//! One preview render = one topaz-preview-frame run with --keep-window, in a
//! thread, reporting neuroserver's own progress (JSON lines in the log) back
//! to the UI and finally the frame paths parsed from --print-paths output.

use crate::catalog::Preset;
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone, Debug, Default)]
pub struct RenderResult {
    pub original: PathBuf,
    pub topaz: PathBuf,
    pub window_start: u64,
    pub target: usize,
    /// offset in window -> path, for the Topaz and the source frames.
    pub topaz_frames: BTreeMap<usize, PathBuf>,
    pub original_frames: BTreeMap<usize, PathBuf>,
    pub seconds: f64,
}

pub enum Event {
    Stage(String),
    Note(String),
    Done(Result<RenderResult>),
}

pub struct Job {
    pub rx: Receiver<Event>,
    pid: Arc<Mutex<Option<u32>>>,
    pub key: String,
}

impl Job {
    /// SIGTERM the wrapper; it forwards the signal to neuroserver / ffmpeg.
    pub fn cancel(&self) {
        if let Some(pid) = *self.pid.lock().unwrap() {
            let _ = Command::new("kill").arg("-TERM").arg(pid.to_string()).status();
        }
    }
}

pub struct Request {
    pub input: PathBuf,
    pub preset: Preset,
    pub preset_label: String,
    pub size: Option<(u32, u32)>,
    pub time: f64,
    pub key: String,
}

fn log_path(slug: &str) -> PathBuf {
    let dir = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".into());
    let safe: String = slug
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
        .collect();
    Path::new(&dir).join(format!("neuroserver-select-preset-{safe}.log"))
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
    let log = log_path(&req.preset.slug);
    let _ = fs::write(&log, "");
    let started = std::time::Instant::now();

    let mut cmd = Command::new(&script);
    cmd.arg("--input").arg(&req.input)
        .arg("--preset-name").arg(&req.preset_label)
        .arg("--preset-flag").arg("--filter_complex")
        .arg("--ns-model").arg(&req.preset.ns_model)
        .arg("--time").arg(format!("{:.3}", req.time))
        .arg("--no-open")
        .arg("--print-paths")
        .arg("--keep-window")
        .arg("--log-file").arg(&log);
    if !req.preset.ns_store.is_empty() {
        cmd.arg("--ns-store").arg(&req.preset.ns_store);
    }
    if let Some(p) = &req.preset.ns_params {
        cmd.arg("--ns-params").arg(p);
    }
    if let Some((w, h)) = req.size {
        cmd.arg("--ns-size").arg(format!("{w}x{h}"));
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

    let mut log_pos: u64 = 0;
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break Ok(st),
            Ok(None) => {}
            Err(e) => break Err(e),
        }
        tail_log(&log, &mut log_pos, &tx);
        thread::sleep(Duration::from_millis(250));
    };
    tail_log(&log, &mut log_pos, &tx);
    *pid.lock().unwrap() = None;

    let stdout = out_h.join().unwrap_or_default();
    let stderr = err_h.join().unwrap_or_default();
    let result = match status {
        Ok(st) if st.success() => parse_paths(&stdout).map(|mut r| {
            r.seconds = started.elapsed().as_secs_f64();
            r
        }),
        Ok(st) => Err(anyhow!(
            "{}",
            failure_reason(&log, &stderr, st.code())
        )),
        Err(e) => Err(anyhow!("waiting for render: {e}")),
    };
    let _ = tx.send(Event::Done(result));
}

fn tail_log(log: &Path, pos: &mut u64, tx: &Sender<Event>) {
    let Ok(mut f) = fs::File::open(log) else { return };
    let Ok(len) = f.metadata().map(|m| m.len()) else { return };
    if len <= *pos {
        return;
    }
    if f.seek(SeekFrom::Start(*pos)).is_err() {
        return;
    }
    let mut buf = String::new();
    if f.read_to_string(&mut buf).is_err() {
        return;
    }
    *pos = len;
    for line in buf.lines() {
        if let Some(stage) = line.strip_prefix("### ") {
            let _ = tx.send(Event::Stage(stage.trim().to_string()));
        } else if line.contains("\"progress\"") {
            // {"status": "RUNNING", "frame": 9, "progress": 99, "message": "Done"}
            let pct = field(line, "\"progress\":").and_then(|v| v.trim().trim_end_matches(',').parse::<u32>().ok());
            let msg = field(line, "\"message\":").map(|v| v.trim().trim_matches(|c| c == '"' || c == '}' || c == ',').to_string());
            let note = match (msg, pct) {
                (Some(m), Some(p)) => format!("{m} · {p}%"),
                (Some(m), None) => m,
                (None, Some(p)) => format!("{p}%"),
                _ => continue,
            };
            let _ = tx.send(Event::Note(note));
        } else if line.starts_with("frame=") {
            let _ = tx.send(Event::Note("extracting".into()));
        }
    }
}

fn field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let i = line.find(key)? + key.len();
    let rest = &line[i..];
    let end = rest.find(|c| c == ',' || c == '}').unwrap_or(rest.len());
    Some(&rest[..end])
}

fn failure_reason(log: &Path, stderr: &str, code: Option<i32>) -> String {
    let stderr = stderr.trim();
    if !stderr.is_empty() {
        return stderr.lines().last().unwrap_or(stderr).to_string();
    }
    if let Ok(text) = fs::read_to_string(log) {
        let hit = text
            .lines()
            .rev()
            .find(|l| {
                let l = l.to_ascii_lowercase();
                l.contains("error") || l.contains("traceback") || l.contains("failed")
            })
            .map(|l| l.trim().to_string());
        if let Some(h) = hit {
            return h;
        }
    }
    format!("render exited with status {:?} (log: {})", code, log.display())
}

fn parse_paths(stdout: &str) -> Result<RenderResult> {
    let mut r = RenderResult::default();
    for line in stdout.lines() {
        let Some((k, v)) = line.split_once('=') else { continue };
        match k {
            "TOPAZ_PREVIEW_ORIGINAL" => r.original = PathBuf::from(v),
            "TOPAZ_PREVIEW_TOPAZ" => r.topaz = PathBuf::from(v),
            "TOPAZ_PREVIEW_WINDOW_START" => r.window_start = v.parse().unwrap_or(0),
            "TOPAZ_PREVIEW_WINDOW_TARGET" => r.target = v.parse().unwrap_or(0),
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
    if r.topaz.as_os_str().is_empty() {
        return Err(anyhow!("render printed no preview path"));
    }
    if r.topaz_frames.is_empty() {
        r.topaz_frames.insert(r.target, r.topaz.clone());
    }
    if r.original_frames.is_empty() && !r.original.as_os_str().is_empty() {
        r.original_frames.insert(r.target, r.original.clone());
    }
    Ok(r)
}
