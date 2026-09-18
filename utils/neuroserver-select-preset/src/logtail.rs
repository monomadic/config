//! Incremental reader for the log both topaz-preview-frame and
//! neuroserver-encode write: "### stage" markers from the wrapper scripts and
//! neuroserver's own JSON progress lines,
//!   {"status": "RUNNING", "frame": 9, "progress": 99, "message": "Done"}

use std::fs;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Clone, Debug)]
pub enum LogEvent {
    Stage(String),
    Progress { pct: Option<u32>, frame: Option<u64>, message: Option<String> },
}

#[derive(Default)]
pub struct Tail {
    pos: u64,
}

impl Tail {
    pub fn poll(&mut self, log: &Path) -> Vec<LogEvent> {
        let mut out = Vec::new();
        let Ok(mut f) = fs::File::open(log) else { return out };
        let Ok(len) = f.metadata().map(|m| m.len()) else { return out };
        if len < self.pos {
            self.pos = 0; // truncated and restarted
        }
        if len == self.pos || f.seek(SeekFrom::Start(self.pos)).is_err() {
            return out;
        }
        let mut bytes = Vec::new();
        if f.read_to_end(&mut bytes).is_err() {
            return out;
        }
        self.pos = len;
        for line in String::from_utf8_lossy(&bytes).lines() {
            if let Some(stage) = line.strip_prefix("### ") {
                out.push(LogEvent::Stage(stage.trim().to_string()));
            } else if line.contains("\"progress\"") {
                let pct = field(line, "\"progress\":").and_then(|v| v.trim().parse().ok());
                let frame = field(line, "\"frame\":").and_then(|v| v.trim().parse().ok());
                let message = field(line, "\"message\":")
                    .map(|v| v.trim().trim_matches(|c| c == '"' || c == '}').to_string())
                    .filter(|m| !m.is_empty());
                out.push(LogEvent::Progress { pct, frame, message });
            }
        }
        out
    }
}

fn field<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let i = line.rfind(key)? + key.len();
    let rest = &line[i..];
    let end = rest.find(|c| c == ',' || c == '}').unwrap_or(rest.len());
    Some(&rest[..end])
}
