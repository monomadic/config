//! Container structure: where `moov` sits relative to `mdat` (SPEC §9.2).
//!
//! A Rust port of mp4doctor's `atom_state()`. tagform does not repair
//! containers -- that stays mp4doctor's job -- but it must be able to *verify*
//! that a remux it asked to be faststart actually came out faststart, and to
//! know whether a file already is before deciding it needs a remux at all.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    /// `moov` precedes `mdat`: playable before the whole file arrives.
    FastStart,
    /// `mdat` precedes `moov`: the classic "moov at end".
    MoovAtEnd,
    /// Any `moof` at all. Repairable, but not by this tool.
    Fragmented,
    /// Truncated, not an ISO container, or an atom chain that does not add up.
    Inconclusive,
}

impl Layout {
    pub fn is_faststart(self) -> bool {
        self == Layout::FastStart
    }
}

/// Walk the top-level atom chain. Any `moof` means fragmented; a real `mdat`
/// settles the verdict by whether a `moov` came first.
pub fn layout(path: &Path) -> Layout {
    match scan(path) {
        Ok(l) => l,
        Err(_) => Layout::Inconclusive,
    }
}

fn scan(path: &Path) -> std::io::Result<Layout> {
    let mut f = File::open(path)?;
    let size = f.metadata()?.len();
    let mut pos: u64 = 0;
    let mut seen_moov = false;

    // Bounded so a hostile or corrupt file cannot spin here.
    for _ in 0..100_000 {
        if pos + 8 > size {
            return Ok(Layout::Inconclusive);
        }
        f.seek(SeekFrom::Start(pos))?;
        let mut hdr = [0u8; 8];
        if f.read_exact(&mut hdr).is_err() {
            return Ok(Layout::Inconclusive);
        }
        let mut atom_size = u32::from_be_bytes([hdr[0], hdr[1], hdr[2], hdr[3]]) as u64;
        let kind = &hdr[4..8];
        let mut header_len: u64 = 8;

        if atom_size == 1 {
            // 64-bit extended size.
            let mut ext = [0u8; 8];
            if f.read_exact(&mut ext).is_err() {
                return Ok(Layout::Inconclusive);
            }
            atom_size = u64::from_be_bytes(ext);
            header_len = 16;
        } else if atom_size == 0 {
            // Runs to end of file.
            atom_size = size - pos;
        }
        if atom_size < header_len {
            return Ok(Layout::Inconclusive);
        }

        match kind {
            b"moof" => return Ok(Layout::Fragmented),
            b"mdat" => {
                return Ok(if seen_moov { Layout::FastStart } else { Layout::MoovAtEnd })
            }
            b"moov" => seen_moov = true,
            _ => {}
        }
        pos = match pos.checked_add(atom_size) {
            Some(p) => p,
            None => return Ok(Layout::Inconclusive),
        };
    }
    Ok(Layout::Inconclusive)
}

/// Free bytes on the volume holding `dir`.
///
/// The remux is atomic-by-copy: it needs room for a second full-size file. A
/// 10 GB file on a volume with 4 GB free fails with nothing wrong with the
/// container at all, and reporting that as "could not write tags" sends you
/// looking for corruption that is not there -- so it is checked up front and
/// reported as itself. Same reasoning as mp4doctor's free_bytes_for.
pub fn free_bytes(dir: &Path) -> Option<u64> {
    let out = std::process::Command::new("df")
        .arg("-k")
        .arg(dir)
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let last = text.lines().last()?;
    let avail_kb: u64 = last.split_whitespace().nth(3)?.parse().ok()?;
    Some(avail_kb * 1024)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_file_is_inconclusive_not_a_panic() {
        assert_eq!(layout(Path::new("/nonexistent/nope.mp4")), Layout::Inconclusive);
    }

    #[test]
    fn garbage_is_inconclusive() {
        let p = std::env::temp_dir().join("tagform-atoms-garbage.bin");
        std::fs::write(&p, b"this is not an mp4 at all, not even close").unwrap();
        assert_eq!(layout(&p), Layout::Inconclusive);
        std::fs::remove_file(&p).ok();
    }

    /// A hand-built chain: ftyp, then moov, then mdat -> faststart.
    #[test]
    fn moov_before_mdat_is_faststart() {
        let p = std::env::temp_dir().join("tagform-atoms-fast.bin");
        std::fs::write(&p, chain(&[(b"ftyp", 8), (b"moov", 16), (b"mdat", 32)])).unwrap();
        assert_eq!(layout(&p), Layout::FastStart);
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn mdat_before_moov_is_moov_at_end() {
        let p = std::env::temp_dir().join("tagform-atoms-slow.bin");
        std::fs::write(&p, chain(&[(b"ftyp", 8), (b"mdat", 32), (b"moov", 16)])).unwrap();
        assert_eq!(layout(&p), Layout::MoovAtEnd);
        std::fs::remove_file(&p).ok();
    }

    #[test]
    fn any_moof_is_fragmented() {
        let p = std::env::temp_dir().join("tagform-atoms-frag.bin");
        std::fs::write(&p, chain(&[(b"ftyp", 8), (b"moov", 16), (b"moof", 16), (b"mdat", 32)]))
            .unwrap();
        assert_eq!(layout(&p), Layout::Fragmented);
        std::fs::remove_file(&p).ok();
    }

    /// An atom claiming to be smaller than its own header must not loop.
    #[test]
    fn a_nonsense_size_terminates() {
        let p = std::env::temp_dir().join("tagform-atoms-zero.bin");
        let mut v = Vec::new();
        v.extend_from_slice(&3u32.to_be_bytes());
        v.extend_from_slice(b"ftyp");
        v.extend_from_slice(&[0; 16]);
        std::fs::write(&p, v).unwrap();
        assert_eq!(layout(&p), Layout::Inconclusive);
        std::fs::remove_file(&p).ok();
    }

    fn chain(atoms: &[(&[u8; 4], u32)]) -> Vec<u8> {
        let mut v = Vec::new();
        for (kind, size) in atoms {
            v.extend_from_slice(&size.to_be_bytes());
            v.extend_from_slice(*kind);
            v.extend(std::iter::repeat(0).take(*size as usize - 8));
        }
        v
    }
}
