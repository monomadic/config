//! Incremental reader for the log topaz-preview-frame writes with --log-file:
//! "### stage" markers from the script, and the Topaz ffmpeg's own output at
//! info level — including its -stats lines ("frame=    3 fps=0.4 ..."), which
//! are separated by carriage returns rather than newlines.

use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Clone, Debug)]
pub enum LogEvent {
    Stage(String),
    Frame(u64),
    Note(String),
}

#[derive(Default)]
pub struct Tail {
    pos: u64,
    partial: String,
}

impl Tail {
    pub fn poll(&mut self, log: &Path) -> Vec<LogEvent> {
        let mut out = Vec::new();
        let Ok(mut f) = fs::File::open(log) else { return out };
        let Ok(len) = f.metadata().map(|m| m.len()) else { return out };
        if len < self.pos {
            self.pos = 0; // truncated and restarted
            self.partial.clear();
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
        // Everything up to the last separator is complete; keep the rest.
        let Some(cut) = self.partial.rfind(['\n', '\r']) else { return out };
        let rest = self.partial.split_off(cut + 1);
        for line in self.partial.split(['\n', '\r']) {
            if let Some(ev) = classify(line) {
                out.push(ev);
            }
        }
        self.partial = rest;
        out
    }
}

fn classify(line: &str) -> Option<LogEvent> {
    let line = line.trim();
    if let Some(stage) = line.strip_prefix("### ") {
        return Some(LogEvent::Stage(stage.trim().to_string()));
    }
    if let Some(rest) = line.strip_prefix("frame=") {
        let n = rest.split_whitespace().next()?;
        return n.parse().ok().map(LogEvent::Frame);
    }
    // Model downloads and failures are worth a status line; the rest of the
    // info-level chatter (stream mapping, encoder settings) is not.
    let lower = line.to_ascii_lowercase();
    if lower.contains("download") || lower.contains("error") || lower.contains("failed") {
        let mut s: String = line.chars().take(140).collect();
        if s.len() < line.len() {
            s.push('…');
        }
        return Some(LogEvent::Note(s));
    }
    None
}
