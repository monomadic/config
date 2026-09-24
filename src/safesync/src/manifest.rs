use crate::filesystem::{Stamp, Volume, hex};
use anyhow::{Context, Result, bail, ensure};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    os::{
        fd::AsRawFd,
        unix::{
            ffi::{OsStrExt, OsStringExt},
            fs::OpenOptionsExt,
        },
    },
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

pub const SCHEMA: u32 = 1;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Inventory,
    OfflineSnapshot,
}

/// Historical display metadata only. Write permissions always come from the
/// live, UUID-checked sentinel, never from an index.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordedDrive {
    pub role: crate::drive::Role,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_uuid: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Header {
    pub schema: u32,
    pub generation: String,
    pub role: Role,
    pub volume: Volume,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub drive: Option<RecordedDrive>,
    pub root_base64: String,
    pub root_file_id: u64,
    pub started_unix: u64,
    pub finished_unix: u64,
    pub hash_algorithm: String,
    pub content_hashed: bool,
    pub exclusions: Vec<String>,
    pub skipped_symlinks: u64,
    pub skipped_special: u64,
    pub skipped_mounts: u64,
    /// Fingerprints carried over from earlier scans instead of read this time.
    #[serde(default)]
    pub reused_hashes: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entry {
    pub path_base64: String,
    pub stamp: Stamp,
    pub sha256: Option<String>,
}
impl Entry {
    pub fn path(&self) -> Result<PathBuf> {
        decode_path(&self.path_base64)
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub header: Header,
    pub entries: Vec<Entry>,
}
/// The header, the file count and the byte total: everything the drives
/// screen shows about an index, read from the first and last lines only.
/// `bytes` is `None` for an index written before the footer recorded it.
#[derive(Clone, Debug)]
pub struct Summary {
    pub header: Header,
    pub files: usize,
    pub bytes: Option<u64>,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "record", rename_all = "snake_case")]
enum Record {
    Header(Header),
    File(Entry),
    End {
        files: usize,
        sha256: String,
        /// Sum of every entry's size, so a reader can show it without the entries.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bytes: Option<u64>,
    },
}

/// The longest line `summary` will read at either end of the file: the
/// header (with its exclusion list) and the footer are both far shorter.
const EDGE_LINE: u64 = 1024 * 1024;

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub fn generation() -> String {
    static SEQUENCE: AtomicU64 = AtomicU64::new(0);
    format!(
        "{}-{}-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        std::process::id(),
        SEQUENCE.fetch_add(1, Ordering::Relaxed)
    )
}
pub fn encode_path(path: &Path) -> String {
    STANDARD.encode(path.as_os_str().as_bytes())
}
pub fn decode_path(value: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(std::ffi::OsString::from_vec(
        STANDARD.decode(value)?,
    )))
}
fn validate(header: &Header, entries: &[Entry]) -> Result<()> {
    ensure!(
        header.schema == SCHEMA,
        "Unsupported manifest schema {}",
        header.schema
    );
    ensure!(
        header.hash_algorithm == "sha256",
        "Unsupported hash algorithm"
    );
    ensure!(
        !header.volume.uuid.is_empty(),
        "Manifest has no volume identity"
    );
    ensure!(
        decode_path(&header.root_base64)?.is_absolute(),
        "Manifest root is not absolute"
    );
    let mut paths = HashSet::new();
    for entry in entries {
        let path = entry.path()?;
        ensure!(
            !path.as_os_str().is_empty()
                && !path.as_os_str().as_bytes().contains(&0)
                && path
                    .components()
                    .collect::<PathBuf>()
                    .as_os_str()
                    .as_bytes()
                    == path.as_os_str().as_bytes()
                && path.components().all(|c| matches!(c, Component::Normal(_))),
            "Unsafe manifest path"
        );
        ensure!(paths.insert(path), "Duplicate manifest path");
        if let Some(hash) = &entry.sha256 {
            ensure!(
                hash.len() == 64
                    && hash
                        .bytes()
                        .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)),
                "Invalid content digest"
            );
        }
        ensure!(
            !header.content_hashed || entry.sha256.is_some(),
            "Content manifest contains an unhashed file"
        );
    }
    Ok(())
}

impl Manifest {
    pub fn validate(&self) -> Result<()> {
        validate(&self.header, &self.entries)
    }

    pub fn load(path: &Path) -> Result<Self> {
        let file = File::open(path).with_context(|| format!("Cannot read manifest {:?}", path))?;
        Self::read_from(file)
    }

    /// Header and footer only: O(1) in the number of entries, so the drives
    /// screen can open on a half-gigabyte index without parsing it. Checks the
    /// header and that a committed footer exists, but not the checksum or the
    /// entries; `load` does that when the entries are wanted.
    pub fn summary(path: &Path) -> Result<Summary> {
        let mut file =
            File::open(path).with_context(|| format!("Cannot read manifest {:?}", path))?;
        let mut first = Vec::new();
        BufReader::new(&mut file)
            .take(EDGE_LINE)
            .read_until(b'\n', &mut first)?;
        ensure!(
            first.last() == Some(&b'\n'),
            "Truncated or oversized manifest header"
        );
        let Record::Header(header) =
            serde_json::from_slice(&first).context("Invalid manifest header")?
        else {
            bail!("Missing manifest header");
        };
        validate(&header, &[])?;

        let len = file.metadata()?.len();
        let tail_start = len.saturating_sub(EDGE_LINE);
        file.seek(SeekFrom::Start(tail_start))?;
        let mut tail = Vec::new();
        file.read_to_end(&mut tail)?;
        ensure!(
            tail.last() == Some(&b'\n'),
            "Incomplete manifest: missing committed footer"
        );
        let body = &tail[..tail.len() - 1];
        let last = match body.iter().rposition(|b| *b == b'\n') {
            Some(i) => &body[i + 1..],
            // The whole tail is one line: only acceptable if it is the whole file.
            None if tail_start == 0 => body,
            None => bail!("Oversized manifest footer"),
        };
        let Record::End { files, bytes, .. } =
            serde_json::from_slice(last).context("Invalid manifest footer")?
        else {
            bail!("Incomplete manifest: missing committed footer");
        };
        Ok(Summary {
            header,
            files,
            bytes,
        })
    }

    pub(crate) fn read_from(file: File) -> Result<Self> {
        let mut reader = BufReader::new(file);
        let mut hash = Sha256::new();
        let mut header = None;
        let mut entries = Vec::new();
        let mut ended = false;
        loop {
            let mut line = Vec::new();
            let n = reader
                .by_ref()
                .take(16 * 1024 * 1024)
                .read_until(b'\n', &mut line)?;
            if n == 0 {
                break;
            }
            ensure!(!ended, "Data after manifest footer");
            ensure!(
                line.last() == Some(&b'\n'),
                "Truncated or oversized manifest record"
            );
            let record: Record =
                serde_json::from_slice(&line).context("Invalid manifest record")?;
            match record {
                Record::Header(h) => {
                    ensure!(
                        header.is_none() && entries.is_empty(),
                        "Duplicate/misplaced manifest header"
                    );
                    header = Some(h);
                }
                Record::File(e) => {
                    ensure!(header.is_some(), "Missing manifest header");
                    entries.push(e);
                }
                Record::End {
                    files,
                    sha256,
                    bytes,
                } => {
                    ensure!(
                        files == entries.len() && sha256 == hex(&hash.clone().finalize()),
                        "Manifest checksum/count mismatch"
                    );
                    ensure!(
                        bytes.is_none_or(|b| b == total_bytes(&entries)),
                        "Manifest byte total mismatch"
                    );
                    ended = true;
                    continue;
                }
            }
            hash.update(&line);
        }
        ensure!(ended, "Incomplete manifest: missing committed footer");
        let header = header.context("Missing manifest header")?;
        validate(&header, &entries)?;
        Ok(Self { header, entries })
    }

    // Publish a complete immutable snapshot without ever overwriting an existing
    // file. Atomic hard-link publication avoids rename's overwrite behavior.
    pub fn save_new(&self, output: &Path) -> Result<()> {
        validate(&self.header, &self.entries)?;
        let parent = output
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        ensure!(
            parent.is_dir(),
            "Manifest parent directory does not exist: {:?}",
            parent
        );
        let temp = parent.join(format!(".safesync-{}.partial", generation()));
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temp)?;
        let result = (|| -> Result<()> {
            let mut writer = BufWriter::new(file);
            self.write_to(&mut writer)?;
            full_sync(writer.get_ref())?;
            fs::hard_link(&temp, output)
                .context("Cannot publish manifest (existing outputs are never overwritten)")?;
            File::open(parent)?.sync_all()?;
            Ok(())
        })();
        let _ = fs::remove_file(&temp);
        result
    }
    pub(crate) fn write_to(&self, mut writer: impl Write) -> Result<()> {
        self.validate()?;
        let mut digest = Sha256::new();
        let mut write_record = |record: Record| -> Result<()> {
            let mut bytes = serde_json::to_vec(&record)?;
            bytes.push(b'\n');
            digest.update(&bytes);
            writer.write_all(&bytes)?;
            Ok(())
        };
        write_record(Record::Header(self.header.clone()))?;
        for entry in &self.entries {
            write_record(Record::File(entry.clone()))?;
        }
        let end = Record::End {
            files: self.entries.len(),
            sha256: hex(&digest.finalize()),
            bytes: Some(total_bytes(&self.entries)),
        };
        serde_json::to_writer(&mut writer, &end)?;
        writer.write_all(b"\n")?;
        writer.flush()?;
        Ok(())
    }
    pub fn export(&self, output: &Path) -> Result<()> {
        let mut snapshot = self.clone();
        snapshot.header.role = Role::OfflineSnapshot;
        snapshot.save_new(output)
    }
}
pub fn total_bytes(entries: &[Entry]) -> u64 {
    entries.iter().map(|e| e.stamp.size).sum()
}
pub(crate) fn full_sync(file: &File) -> Result<()> {
    file.sync_all()?;
    // SAFETY: file owns a valid fd; F_FULLFSYNC has no pointer argument.
    if unsafe { libc::fcntl(file.as_raw_fd(), libc::F_FULLFSYNC) } == -1 {
        bail!(
            "Full durability flush failed: {}",
            std::io::Error::last_os_error()
        );
    }
    Ok(())
}
