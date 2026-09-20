//! Framed copy-lifecycle journal. Records are evidence, never filesystem commands.
//! Recovery and executor integration deliberately remain separate.
use crate::manifest;
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{File, OpenOptions},
    io::{Read, Write},
    os::{
        fd::{AsRawFd, FromRawFd},
        unix::fs::{MetadataExt, OpenOptionsExt},
    },
    path::Path,
};

const MAGIC: &[u8; 17] = b"SAFESYNC-JNL-001\n";
const MAX_PAYLOAD: usize = 1024 * 1024;
const HEADER: usize = 76; // sequence, payload length, previous digest, header digest

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Copying,
    StagedVerified,
    OldVersionArchived,
    Installed,
    CatalogCommitted,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case", deny_unknown_fields)]
pub enum Event {
    Start {
        run_id: String,
        plan_sha256: String,
        operation_ids: Vec<String>,
    },
    Intent {
        operation_id: String,
        stage: Stage,
    },
    Completed {
        operation_id: String,
        stage: Stage,
    },
    RunCommitted,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct Operation {
    pub completed: Option<Stage>,
    pub pending: Option<Stage>,
}
#[derive(Clone, Debug, Default, Serialize)]
pub struct State {
    pub run_id: Option<String>,
    pub plan_sha256: Option<String>,
    pub operations: BTreeMap<String, Operation>,
    pub committed: bool,
}
impl State {
    fn apply(&mut self, event: &Event) -> Result<()> {
        ensure!(!self.committed, "Record after run commit");
        match event {
            Event::Start {
                run_id,
                plan_sha256,
                operation_ids,
            } => {
                ensure!(self.run_id.is_none(), "Duplicate run start");
                ensure!(!run_id.is_empty() && run_id.len() <= 256, "Invalid run ID");
                ensure!(
                    plan_sha256.len() == 64
                        && plan_sha256
                            .bytes()
                            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
                    "Invalid immutable plan digest"
                );
                let mut operations = BTreeMap::new();
                for id in operation_ids {
                    ensure!(
                        !id.is_empty()
                            && id.len() <= 256
                            && operations
                                .insert(id.clone(), Operation::default())
                                .is_none(),
                        "Invalid/duplicate operation ID"
                    );
                }
                self.run_id = Some(run_id.clone());
                self.plan_sha256 = Some(plan_sha256.clone());
                self.operations = operations;
            }
            Event::Intent {
                operation_id,
                stage,
            } => {
                ensure!(self.run_id.is_some(), "Missing run start");
                let op = self
                    .operations
                    .get_mut(operation_id)
                    .context("Unknown planned operation")?;
                ensure!(op.pending.is_none(), "Unresolved operation intent");
                let legal = matches!(
                    (op.completed, stage),
                    (None, Stage::Copying)
                        | (Some(Stage::Copying), Stage::StagedVerified)
                        | (
                            Some(Stage::StagedVerified),
                            Stage::OldVersionArchived | Stage::Installed
                        )
                        | (Some(Stage::OldVersionArchived), Stage::Installed)
                        | (Some(Stage::Installed), Stage::CatalogCommitted)
                );
                ensure!(legal, "Invalid operation stage transition");
                op.pending = Some(*stage);
            }
            Event::Completed {
                operation_id,
                stage,
            } => {
                let op = self
                    .operations
                    .get_mut(operation_id)
                    .context("Unknown planned operation")?;
                ensure!(
                    op.pending == Some(*stage),
                    "Completion lacks matching durable intent"
                );
                op.completed = Some(*stage);
                op.pending = None;
            }
            Event::RunCommitted => {
                ensure!(
                    self.run_id.is_some()
                        && self.operations.values().all(|op| op.pending.is_none()
                            && op.completed == Some(Stage::CatalogCommitted)),
                    "Cannot commit an incomplete run"
                );
                self.committed = true;
            }
        }
        Ok(())
    }
}
#[derive(Debug, Serialize)]
pub struct Inspection {
    pub records: u64,
    pub valid_bytes: u64,
    pub torn_tail: bool,
    pub recovery_required: bool,
    pub state: State,
}
fn frame(sequence: u64, previous: &[u8; 32], event: &Event) -> Result<(Vec<u8>, [u8; 32])> {
    let payload = serde_json::to_vec(event)?;
    ensure!(
        payload.len() <= MAX_PAYLOAD,
        "Journal event exceeds size limit"
    );
    let mut bytes = Vec::with_capacity(HEADER + payload.len() + 32);
    bytes.extend(sequence.to_le_bytes());
    bytes.extend((payload.len() as u32).to_le_bytes());
    bytes.extend(previous);
    bytes.extend(Sha256::digest(&bytes));
    bytes.extend(payload);
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    bytes.extend(digest);
    Ok((bytes, digest))
}
fn read_chunk(reader: &mut impl Read, count: usize) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take(count as u64).read_to_end(&mut bytes)?;
    Ok(bytes)
}
fn inspect_reader(mut reader: impl Read) -> Result<Inspection> {
    ensure!(
        read_chunk(&mut reader, MAGIC.len())? == MAGIC,
        "Missing or unsupported journal header"
    );
    let mut state = State::default();
    let mut records = 0u64;
    let mut valid_bytes = MAGIC.len() as u64;
    let mut previous = [0u8; 32];
    let mut torn_tail = false;
    loop {
        let header = read_chunk(&mut reader, HEADER)?;
        if header.is_empty() {
            break;
        }
        if header.len() != HEADER {
            torn_tail = true;
            break;
        }
        ensure!(
            Sha256::digest(&header[..44]).as_slice() == &header[44..],
            "Corrupt journal frame header"
        );
        let sequence = u64::from_le_bytes(header[..8].try_into()?);
        let length = u32::from_le_bytes(header[8..12].try_into()?) as usize;
        ensure!(
            sequence == records + 1 && header[12..44] == previous,
            "Journal sequence/hash-chain mismatch"
        );
        ensure!(length <= MAX_PAYLOAD, "Unsupported journal payload length");
        let body = read_chunk(&mut reader, length + 32)?;
        if body.len() != length + 32 {
            torn_tail = true;
            break;
        }
        let mut hash = Sha256::new();
        hash.update(&header);
        hash.update(&body[..length]);
        let digest: [u8; 32] = hash.finalize().into();
        ensure!(digest == body[length..], "Corrupt journal frame payload");
        let event: Event =
            serde_json::from_slice(&body[..length]).context("Invalid journal event")?;
        state.apply(&event)?;
        previous = digest;
        records += 1;
        valid_bytes += (HEADER + length + 32) as u64;
    }
    let recovery_required = torn_tail || !state.committed;
    Ok(Inspection {
        records,
        valid_bytes,
        torn_tail,
        recovery_required,
        state,
    })
}
fn lock(file: &File, kind: i32) -> Result<()> {
    ensure!(
        file.metadata()?.is_file() && file.metadata()?.nlink() == 1,
        "Unsafe journal file"
    );
    ensure!(
        unsafe { libc::flock(file.as_raw_fd(), kind | libc::LOCK_NB) } == 0,
        "Journal is busy"
    );
    Ok(())
}
pub fn inspect(path: &Path) -> Result<Inspection> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
        .open(path)?;
    inspect_file(file)
}

pub(crate) fn inspect_file(file: File) -> Result<Inspection> {
    lock(&file, libc::LOCK_SH)?;
    inspect_reader(file)
}

/// A new journal writer only. Existing/torn journals must go through future recovery.
/// The executor must publish/validate the bound immutable plan before creating this.
pub struct Journal {
    file: File,
    stamp: crate::filesystem::Stamp,
    state: State,
    previous: [u8; 32],
    records: u64,
    bytes: u64,
    poisoned: bool,
}
impl Journal {
    pub fn create(path: &Path, start: Event) -> Result<Self> {
        ensure!(
            matches!(start, Event::Start { .. }),
            "First event must bind a run to its plan"
        );
        let mut state = State::default();
        state.apply(&start)?;
        // Validate size before creating anything.
        frame(1, &[0; 32], &start)?;
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path)?;
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        Self::initialize(file, &File::open(parent)?, start)
    }

    pub(crate) fn create_in(directory: &File, start: Event) -> Result<Self> {
        ensure!(
            matches!(start, Event::Start { .. }),
            "First event must bind a run to its plan"
        );
        State::default().apply(&start)?;
        frame(1, &[0; 32], &start)?;
        let fd = unsafe {
            libc::openat(
                directory.as_raw_fd(),
                c"journal".as_ptr(),
                libc::O_RDWR | libc::O_CREAT | libc::O_EXCL | libc::O_NOFOLLOW | libc::O_CLOEXEC,
                0o600,
            )
        };
        ensure!(
            fd >= 0,
            "Cannot create run journal: {}",
            std::io::Error::last_os_error()
        );
        Self::initialize(unsafe { File::from_raw_fd(fd) }, directory, start)
    }

    fn initialize(mut file: File, directory: &File, start: Event) -> Result<Self> {
        lock(&file, libc::LOCK_EX)?;
        file.write_all(MAGIC)?;
        let mut journal = Self {
            stamp: crate::filesystem::Stamp::of(&file.metadata()?),
            file,
            state: State::default(),
            previous: [0; 32],
            records: 0,
            bytes: MAGIC.len() as u64,
            poisoned: false,
        };
        journal.append(start)?;
        directory.sync_all()?;
        manifest::full_sync(&journal.file)?;
        Ok(journal)
    }
    pub fn append(&mut self, event: Event) -> Result<()> {
        ensure!(
            !self.poisoned,
            "Journal write outcome is uncertain; recovery required"
        );
        let mut next = self.state.clone();
        next.apply(&event)?;
        let (bytes, digest) = frame(self.records + 1, &self.previous, &event)?;
        self.poisoned = true;
        ensure!(
            self.file.metadata()?.len() == self.bytes
                && crate::filesystem::Stamp::of(&self.file.metadata()?) == self.stamp,
            "Journal changed outside its writer"
        );
        self.file.write_all(&bytes)?;
        manifest::full_sync(&self.file)?;
        self.stamp = crate::filesystem::Stamp::of(&self.file.metadata()?);
        self.state = next;
        self.previous = digest;
        self.records += 1;
        self.bytes += bytes.len() as u64;
        self.poisoned = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn start() -> Event {
        Event::Start {
            run_id: "test-run".into(),
            plan_sha256: "a".repeat(64),
            operation_ids: vec!["copy-1".into()],
        }
    }
    fn intent(stage: Stage) -> Event {
        Event::Intent {
            operation_id: "copy-1".into(),
            stage,
        }
    }
    fn completed(stage: Stage) -> Event {
        Event::Completed {
            operation_id: "copy-1".into(),
            stage,
        }
    }
    fn encode(events: &[Event]) -> Vec<u8> {
        let mut bytes = MAGIC.to_vec();
        let mut previous = [0; 32];
        for (i, event) in events.iter().enumerate() {
            let (record, digest) = frame(i as u64 + 1, &previous, event).unwrap();
            bytes.extend(record);
            previous = digest;
        }
        bytes
    }
    struct Fixture(std::path::PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("safesync-journal-{}", manifest::generation()));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn path(&self) -> std::path::PathBuf {
            self.0.join("run.journal")
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn durable_lifecycle_requires_every_completion_and_excludes_competitors() {
        let f = Fixture::new();
        let mut writer = Journal::create(&f.path(), start()).unwrap();
        assert!(inspect(&f.path()).is_err());
        assert!(Journal::create(&f.path(), start()).is_err());
        assert!(writer.append(Event::RunCommitted).is_err());
        assert!(writer.append(completed(Stage::Copying)).is_err());
        for stage in [
            Stage::Copying,
            Stage::StagedVerified,
            Stage::OldVersionArchived,
            Stage::Installed,
            Stage::CatalogCommitted,
        ] {
            writer.append(intent(stage)).unwrap();
            writer.append(completed(stage)).unwrap();
        }
        writer.append(Event::RunCommitted).unwrap();
        assert!(writer.append(Event::RunCommitted).is_err());
        drop(writer);
        let report = inspect(&f.path()).unwrap();
        assert_eq!(report.records, 12);
        assert!(!report.recovery_required);
        assert!(report.state.committed);
    }
    #[test]
    fn every_truncated_final_frame_is_recovery_required() {
        let prefix = encode(&[start()]);
        let all = encode(&[start(), intent(Stage::Copying)]);
        for cut in prefix.len()..all.len() {
            let report = inspect_reader(&all[..cut]).unwrap();
            assert_eq!(report.records, 1);
            assert_eq!(report.valid_bytes, prefix.len() as u64);
            assert!(report.recovery_required);
            assert_eq!(report.torn_tail, cut != prefix.len());
        }
        let report = inspect_reader(all.as_slice()).unwrap();
        assert!(!report.torn_tail);
        assert_eq!(
            report.state.operations["copy-1"].pending,
            Some(Stage::Copying)
        );
        assert!(report.recovery_required);
    }
    #[test]
    fn corrupt_headers_payloads_chains_and_committed_suffixes_fail() {
        let original = encode(&[start(), intent(Stage::Copying), completed(Stage::Copying)]);
        for offset in [
            MAGIC.len(),
            MAGIC.len() + 8,
            MAGIC.len() + 12,
            MAGIC.len() + 44,
            MAGIC.len() + HEADER + 5,
            original.len() - 1,
        ] {
            let mut bytes = original.clone();
            bytes[offset] ^= 1;
            assert!(inspect_reader(bytes.as_slice()).is_err(), "offset {offset}");
        }
        let (record, _) = frame(3, &[0; 32], &intent(Stage::Copying)).unwrap();
        let mut bytes = encode(&[start()]);
        bytes.extend(record);
        assert!(inspect_reader(bytes.as_slice()).is_err());
        let empty_start = Event::Start {
            run_id: "empty".into(),
            plan_sha256: "0".repeat(64),
            operation_ids: vec![],
        };
        let bytes = encode(&[empty_start, Event::RunCommitted, Event::RunCommitted]);
        assert!(inspect_reader(bytes.as_slice()).is_err());
    }
    #[test]
    fn invalid_stage_transitions_and_unplanned_operations_fail() {
        for event in [
            intent(Stage::Installed),
            completed(Stage::Copying),
            Event::RunCommitted,
            Event::Intent {
                operation_id: "unplanned".into(),
                stage: Stage::Copying,
            },
        ] {
            assert!(inspect_reader(encode(&[start(), event]).as_slice()).is_err());
        }
        let events = [start(), intent(Stage::Copying), intent(Stage::Copying)];
        assert!(inspect_reader(encode(&events).as_slice()).is_err());
        let events = [start(), intent(Stage::Copying), completed(Stage::Installed)];
        assert!(inspect_reader(encode(&events).as_slice()).is_err());
    }
    #[test]
    fn external_append_poisons_writer_instead_of_overwriting() {
        let f = Fixture::new();
        let mut writer = Journal::create(&f.path(), start()).unwrap();
        let mut other = OpenOptions::new().append(true).open(f.path()).unwrap();
        other.write_all(b"external").unwrap();
        assert!(writer.append(intent(Stage::Copying)).is_err());
        let size = other.metadata().unwrap().len();
        assert!(writer.append(intent(Stage::Copying)).is_err());
        assert_eq!(other.metadata().unwrap().len(), size);
    }
    #[test]
    fn invalid_start_does_not_create_a_file_and_links_are_refused() {
        let f = Fixture::new();
        assert!(Journal::create(&f.path(), intent(Stage::Copying)).is_err());
        assert!(!f.path().exists());
        drop(Journal::create(&f.path(), start()).unwrap());
        let link = f.0.join("link");
        std::os::unix::fs::symlink(f.path(), &link).unwrap();
        assert!(inspect(&link).is_err());
        std::fs::remove_file(&link).unwrap();
        std::fs::hard_link(f.path(), &link).unwrap();
        assert!(inspect(&f.path()).is_err());
    }
}
