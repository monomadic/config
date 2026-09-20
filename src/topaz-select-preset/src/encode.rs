//! A full-clip encode, run as a child topaz-encode while the TUI stays up.
//!
//! topaz-encode does the real work — the fragmented in-flight file, resume,
//! the faststart finalize, metadata and creation date — and this only watches:
//! its transcript on stdout (for the phase), the raw ffmpeg -progress stream it
//! copies to --progress-file (for per-frame progress), and the newest frame of
//! the in-flight file.
//!
//! That last part works because topaz-encode always writes *fragmented* MP4/MOV
//! (NAME.frag.EXT, or NAME [resume tail].EXT when resuming): a fragment is
//! closed at every keyframe, so the file can be decoded mid-write and trails
//! the encoder by at most one GOP — about a second with the HEVC profiles, a
//! single frame with ProRes, where every frame is a keyframe.

use std::fs;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

/// The latest block of ffmpeg's -progress stream, plus topaz-encode's header.
#[derive(Clone, Debug, Default)]
pub struct Progress {
    pub frame: u64,
    pub fps: f64,
    /// Output seconds written by this run.
    pub out_time: f64,
    pub speed: f64,
    /// Output seconds this run will write (topaz_total_seconds).
    pub total: Option<f64>,
    /// Output seconds a resumed partial already holds (topaz_resume_from).
    pub resume_from: f64,
    pub ended: bool,
}

pub enum Event {
    /// A line of topaz-encode's transcript, timestamp stripped.
    Line(String),
    Progress(Progress),
    /// A freshly extracted JPEG of the newest frame in the in-flight file.
    LiveFrame(PathBuf),
    Done { ok: bool },
}

pub struct Encode {
    pub rx: Receiver<Event>,
    pgid: Arc<Mutex<Option<i32>>>,
    pub started: Instant,
}

impl Encode {
    fn signal(&self, sig: &str) {
        if let Some(pgid) = *self.pgid.lock().unwrap() {
            let _ = Command::new("kill").arg(format!("-{sig}")).arg(format!("-{pgid}")).status();
        }
    }

    /// Stop the whole process group — topaz-encode, its progress reader and
    /// the Topaz ffmpeg — in place. Nothing is lost: SIGCONT carries on from
    /// the same process state, as topaz-encode's own `p` key does.
    pub fn pause(&self) {
        self.signal("STOP");
    }

    pub fn resume(&self) {
        self.signal("CONT");
    }

    /// SIGTERM the whole group: ffmpeg closes its last fragment cleanly, and
    /// the in-flight file stays behind for the next run to resume. SIGCONT
    /// follows so a paused encode can act on it.
    pub fn cancel(&self) {
        self.signal("TERM");
        self.signal("CONT");
    }
}

pub struct Request {
    pub command: Vec<String>,
    pub progress_file: PathBuf,
    /// Files the encode may be writing, fragmented: NAME.frag.EXT, or the
    /// resume tail. Whichever grew most recently is the one shown.
    pub watch: Vec<PathBuf>,
    pub scratch: PathBuf,
}

pub fn spawn(req: Request) -> Encode {
    let (tx, rx) = mpsc::channel();
    let pgid = Arc::new(Mutex::new(None));
    let p2 = pgid.clone();
    thread::spawn(move || run(req, tx, p2));
    Encode { rx, pgid, started: Instant::now() }
}

fn run(req: Request, tx: Sender<Event>, pgid: Arc<Mutex<Option<i32>>>) {
    let _ = fs::create_dir_all(&req.scratch);
    let _ = fs::write(&req.progress_file, "");
    let mut cmd = Command::new(&req.command[0]);
    cmd.args(&req.command[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .process_group(0);
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(Event::Line(format!("✗ cannot run {}: {e}", req.command[0])));
            let _ = tx.send(Event::Done { ok: false });
            return;
        }
    };
    *pgid.lock().unwrap() = Some(child.id() as i32);
    let readers: Vec<_> = [
        child.stdout.take().map(|s| Box::new(s) as Box<dyn Read + Send>),
        child.stderr.take().map(|s| Box::new(s) as Box<dyn Read + Send>),
    ]
    .into_iter()
    .flatten()
    .map(|stream| {
        let tx = tx.clone();
        thread::spawn(move || {
            for line in BufReader::new(stream).lines().map_while(Result::ok) {
                let line = strip_stamp(&line);
                if !line.is_empty() {
                    let _ = tx.send(Event::Line(line.to_string()));
                }
            }
        })
    })
    .collect();

    let mut progress = ProgressTail::default();
    let mut last_grab = Instant::now() - Duration::from_secs(60);
    let mut grabbed: Option<(PathBuf, u64)> = None;
    let mut serial: u64 = 0;
    let status = loop {
        match child.try_wait() {
            Ok(Some(st)) => break Some(st),
            Ok(None) => {}
            Err(_) => break None,
        }
        for p in progress.poll(&req.progress_file) {
            let _ = tx.send(Event::Progress(p));
        }
        // Newest frame, whenever the in-flight file has grown — at most every 4s.
        if last_grab.elapsed() >= Duration::from_secs(4) {
            if let Some((file, size)) = newest(&req.watch) {
                if grabbed.as_ref() != Some(&(file.clone(), size)) {
                    last_grab = Instant::now();
                    serial += 1;
                    let jpg = req.scratch.join(format!("live-{serial:05}.jpg"));
                    if grab_last_frame(&file, &jpg) {
                        let _ = fs::remove_file(req.scratch.join(format!("live-{:05}.jpg", serial - 1)));
                        let _ = tx.send(Event::LiveFrame(jpg));
                    }
                    grabbed = Some((file, size));
                }
            }
        }
        thread::sleep(Duration::from_millis(300));
    };
    for p in progress.poll(&req.progress_file) {
        let _ = tx.send(Event::Progress(p));
    }
    *pgid.lock().unwrap() = None;
    for r in readers {
        let _ = r.join();
    }
    let _ = tx.send(Event::Done { ok: status.map(|s| s.success()).unwrap_or(false) });
}

/// "[2026-09-19 06:34:02]   ↻ finalizing …" → "↻ finalizing …"
fn strip_stamp(line: &str) -> &str {
    let l = line.trim();
    if l.starts_with('[') && l.as_bytes().get(20) == Some(&b']') {
        l[21..].trim()
    } else {
        l
    }
}

/// The watched file that changed last, with its size — if it has any frames.
fn newest(watch: &[PathBuf]) -> Option<(PathBuf, u64)> {
    watch
        .iter()
        .filter_map(|p| {
            let m = fs::metadata(p).ok()?;
            (m.len() > 0).then(|| (p.clone(), m.len(), m.modified().unwrap_or(SystemTime::UNIX_EPOCH)))
        })
        .max_by_key(|(_, _, t)| *t)
        .map(|(p, len, _)| (p, len))
}

/// The last ~2 seconds of the fragmented file, decoded, keeping the final
/// frame. A shorter window can fail: the tail fragment may still be open.
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

/// Incremental reader for the --progress-file: key=value lines, a block per
/// ffmpeg update, each block closed by progress=continue|end.
#[derive(Default)]
struct ProgressTail {
    pos: u64,
    partial: String,
    cur: Progress,
}

impl ProgressTail {
    fn poll(&mut self, file: &Path) -> Vec<Progress> {
        let mut out = Vec::new();
        let Ok(mut f) = fs::File::open(file) else { return out };
        let len = f.metadata().map(|m| m.len()).unwrap_or(0);
        if len < self.pos {
            // topaz-encode truncates it when the encode proper starts.
            *self = ProgressTail::default();
        }
        if len == self.pos || f.seek(SeekFrom::Start(self.pos)).is_err() {
            return out;
        }
        let mut bytes = Vec::new();
        if f.read_to_end(&mut bytes).is_err() {
            return out;
        }
        self.pos += bytes.len() as u64;
        self.partial.push_str(&String::from_utf8_lossy(&bytes));
        let Some(cut) = self.partial.rfind('\n') else { return out };
        let rest = self.partial.split_off(cut + 1);
        for line in self.partial.lines() {
            let Some((k, v)) = line.split_once('=') else { continue };
            let num = || v.trim().trim_end_matches('x').parse::<f64>().ok();
            match k.trim() {
                "topaz_total_seconds" => self.cur.total = num(),
                "topaz_resume_from" => self.cur.resume_from = num().unwrap_or(0.0),
                "frame" => self.cur.frame = v.trim().parse().unwrap_or(self.cur.frame),
                "fps" => self.cur.fps = num().unwrap_or(self.cur.fps),
                "out_time_us" => {
                    if let Some(us) = num() {
                        self.cur.out_time = us / 1_000_000.0;
                    }
                }
                "speed" => self.cur.speed = num().unwrap_or(self.cur.speed),
                "progress" => {
                    self.cur.ended = v.trim() == "end";
                    out.push(self.cur.clone());
                }
                _ => {}
            }
        }
        self.partial = rest;
        out
    }
}

/// Move a file to the Trash — `trash` where it exists (macOS 15 ships one),
/// Finder otherwise. Either way it lands in the Trash of the file's own volume.
pub fn trash(path: &Path) -> Result<(), String> {
    if Command::new("trash").arg(path).stdout(Stdio::null()).stderr(Stdio::null()).status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        return Ok(());
    }
    let p = path.to_string_lossy().replace('\\', "\\\\").replace('"', "\\\"");
    let script = format!("tell application \"Finder\" to delete (POSIX file \"{p}\")");
    match Command::new("osascript").arg("-e").arg(script).stdout(Stdio::null()).stderr(Stdio::null()).status() {
        Ok(s) if s.success() => Ok(()),
        _ => Err(format!("could not move {} to the Trash", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_blocks_and_header() {
        let dir = std::env::temp_dir().join(format!("tsp-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let f = dir.join("p");
        fs::write(&f, "topaz_total_seconds=12.5\ntopaz_resume_from=3\nframe=10\nfps=2.5\nout_time_us=333333\nspeed=0.25x\nprogress=cont").unwrap();
        let mut t = ProgressTail::default();
        assert!(t.poll(&f).is_empty()); // the block is not closed yet
        fs::write(&f, "topaz_total_seconds=12.5\ntopaz_resume_from=3\nframe=10\nfps=2.5\nout_time_us=333333\nspeed=0.25x\nprogress=continue\n").unwrap();
        let p = t.poll(&f);
        assert_eq!(p.len(), 1);
        assert_eq!(p[0].frame, 10);
        assert_eq!(p[0].total, Some(12.5));
        assert_eq!(p[0].resume_from, 3.0);
        assert!((p[0].out_time - 0.333333).abs() < 1e-6);
        assert_eq!(p[0].speed, 0.25);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn stamps_are_stripped() {
        assert_eq!(strip_stamp("[2026-09-19 06:34:02]   ↻ finalizing"), "↻ finalizing");
        assert_eq!(strip_stamp("    topaz-encode --nice"), "topaz-encode --nice");
    }
}
