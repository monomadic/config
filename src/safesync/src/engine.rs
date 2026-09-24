//! Runs `sync` and `fill` on a background thread. The interface — full-screen
//! or plain lines — only ever sees `Event`s and answers one question: go ahead?
use crate::{
    copy::{self, Copied},
    drive::{self, Drive, Role},
    filesystem::{Root, Stamp},
    manifest::{Entry, Manifest, encode_path, generation, now},
    plan::{self, Action},
    scan::{self, Hashing},
};
use anyhow::{Context, Result, ensure};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, Sender},
    },
    time::Instant,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Scanning,
    Review,
    Transfer,
    Indexing,
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Copy,
    Replace,
    Rename,
    Retire,
    Skip,
}

pub struct Item {
    pub kind: Kind,
    pub path: String,
    pub size: u64,
}

pub struct Overview {
    pub items: Vec<Item>,
    pub transfer_bytes: u64,
    pub notes: Vec<String>,
}

#[derive(Default, Clone)]
pub struct Summary {
    pub done: usize,
    pub failed: usize,
    pub bytes: u64,
    pub seconds: f64,
    pub cancelled: bool,
}

pub enum Event {
    /// Names for the header: the reading drives, then the one written to.
    Drives {
        sources: Vec<String>,
        destination: String,
        destination_path: PathBuf,
    },
    Phase(Phase),
    Scan {
        drive: usize,
        files: usize,
        bytes: u64,
        reused: u64,
        /// The previous index's file count: what the walk is probably heading for.
        expected: Option<usize>,
        hashing: Option<scan::HashProgress>,
    },
    Planned(Overview),
    Start {
        worker: usize,
        kind: Kind,
        path: String,
        size: u64,
    },
    Progress {
        worker: usize,
        bytes: u64,
    },
    Finish {
        worker: usize,
        error: Option<String>,
    },
    Log(String),
    Done(Summary),
    Failed(String),
}

pub struct Control {
    pub events: Sender<Event>,
    /// `true` starts the transfer, `false` (or a hang-up) abandons it.
    pub confirm: Mutex<Receiver<bool>>,
    pub cancel: Arc<AtomicBool>,
}
impl Control {
    fn send(&self, event: Event) {
        let _ = self.events.send(event);
    }
    fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::Relaxed)
    }
    fn approved(&self, overview: Overview) -> bool {
        self.send(Event::Planned(overview));
        self.send(Event::Phase(Phase::Review));
        let go = self
            .confirm
            .lock()
            .expect("confirm lock")
            .recv()
            .unwrap_or(false);
        go && !self.cancelled()
    }
}

pub struct SyncOptions {
    pub source: PathBuf,
    pub backup: PathBuf,
    pub hashing: Hashing,
    pub rehash: bool,
    pub verify: bool,
    pub exclude: Vec<PathBuf>,
}

pub struct FillOptions {
    pub from: Vec<PathBuf>,
    pub destination: PathBuf,
    /// Files or folders, relative to the library root or absolute beneath one.
    pub select: Vec<PathBuf>,
    pub verify: bool,
}

fn display(path: &Path) -> String {
    path.to_string_lossy()
        .chars()
        .map(|c| if c.is_control() { '?' } else { c })
        .collect()
}

pub fn scan_drive(
    drive: &Drive,
    index: usize,
    hashing: Hashing,
    rehash: bool,
    extra_excludes: &[PathBuf],
    control: &Control,
) -> Result<Manifest> {
    let cache = if rehash {
        scan::HashCache::new(&drive.volume)
    } else {
        drive.hash_cache()?
    };
    let mut exclude = drive.sentinel.exclude.clone();
    exclude.extend_from_slice(extra_excludes);
    let expected = drive.index_quiet().map(|m| m.entries.len());
    let events = control.events.clone();
    let mut last = Instant::now();
    scan::scan_with_reuse(
        &drive.root,
        drive.volume.clone(),
        hashing,
        &exclude,
        Some(&cache),
        |p| {
            if last.elapsed().as_millis() >= 100 {
                last = Instant::now();
                let _ = events.send(Event::Scan {
                    drive: index,
                    files: p.files,
                    bytes: p.bytes,
                    reused: p.reused,
                    expected,
                    hashing: p.hashing,
                });
            }
        },
    )
    .with_context(|| format!("Scanning {:?}", drive.sentinel.name))
}

fn scanned(control: &Control, drive: usize, manifest: &Manifest) {
    control.send(Event::Scan {
        drive,
        files: manifest.entries.len(),
        bytes: manifest.entries.iter().map(|e| e.stamp.size).sum(),
        reused: manifest.header.reused_hashes,
        expected: None,
        hashing: None,
    });
}

// Indexes are edited in memory as files land, so a sync ends with a current
// index on both drives without walking either of them again.
struct Index {
    manifest: Manifest,
    entries: BTreeMap<PathBuf, Entry>,
}
impl Index {
    fn new(mut manifest: Manifest) -> Result<Self> {
        let mut entries = BTreeMap::new();
        for entry in manifest.entries.drain(..) {
            entries.insert(entry.path()?, entry);
        }
        Ok(Self { manifest, entries })
    }
    fn finish(mut self) -> Manifest {
        self.manifest.entries = self.entries.into_values().collect();
        self.manifest
            .entries
            .sort_by(|a, b| a.path_base64.cmp(&b.path_base64));
        self.manifest.header.generation = generation();
        self.manifest.header.finished_unix = now();
        self.manifest.header.content_hashed =
            self.manifest.entries.iter().all(|e| e.sha256.is_some());
        self.manifest
    }
}

/// Index one drive and publish. No review step: nothing but metadata is written.
pub fn index_drive(root: PathBuf, hashing: Hashing, rehash: bool, control: Control) {
    let result = (|| -> Result<()> {
        let drive = Drive::open(&root)?;
        control.send(Event::Drives {
            sources: vec![drive.sentinel.name.clone()],
            destination: "index".into(),
            destination_path: drive.metadata_dir(),
        });
        control.send(Event::Phase(Phase::Scanning));
        let started = Instant::now();
        let mut manifest = scan_drive(&drive, 0, hashing, rehash, &[], &control)?;
        scanned(&control, 0, &manifest);
        control.send(Event::Phase(Phase::Indexing));
        let path = drive.publish(&mut manifest)?;
        control.send(Event::Log(format!(
            "{} files indexed, {} fingerprints reused → {}",
            manifest.entries.len(),
            manifest.header.reused_hashes,
            display(&path)
        )));
        control.send(Event::Done(Summary {
            done: manifest.entries.len(),
            bytes: 0,
            seconds: started.elapsed().as_secs_f64(),
            ..Summary::default()
        }));
        Ok(())
    })();
    if let Err(error) = result {
        control.send(Event::Failed(format!("{error:#}")));
    }
}

pub fn sync(options: SyncOptions, control: Control) {
    if let Err(error) = run_sync(options, &control) {
        control.send(Event::Failed(format!("{error:#}")));
    }
}

fn run_sync(options: SyncOptions, control: &Control) -> Result<()> {
    let source = Drive::open(&options.source)?;
    let backup = Drive::open(&options.backup)?;
    drive::check_sync(&source, &backup)?;
    let source_root = Root::open(&source.root)?;
    let backup_root = Root::open(&backup.root)?;
    control.send(Event::Drives {
        sources: vec![source.sentinel.name.clone()],
        destination: backup.sentinel.name.clone(),
        destination_path: backup.root.clone(),
    });
    control.send(Event::Phase(Phase::Scanning));
    // Both drives must agree on what is out of scope, or the backup's copy of
    // an excluded folder would look like an extra.
    let mut exclude = options.exclude.clone();
    exclude.extend(source.sentinel.exclude.iter().cloned());
    exclude.extend(backup.sentinel.exclude.iter().cloned());
    let (source_manifest, backup_manifest) = std::thread::scope(|scope| {
        let theirs = scope.spawn(|| {
            scan_drive(
                &backup,
                1,
                options.hashing,
                options.rehash,
                &exclude,
                control,
            )
        });
        let ours = scan_drive(
            &source,
            0,
            options.hashing,
            options.rehash,
            &exclude,
            control,
        );
        (ours, theirs.join().expect("scan thread panicked"))
    });
    let (source_manifest, backup_manifest) = (source_manifest?, backup_manifest?);
    scanned(control, 0, &source_manifest);
    scanned(control, 1, &backup_manifest);

    let plan = plan::plan(&source_manifest, &backup_manifest, backup.sentinel.extras)?;
    let (free, _) = copy::space(&backup.root)?;
    let mut notes = vec![format!("{} files already in sync", plan.unchanged)];
    if !plan.kept_extras.is_empty() {
        notes.push(format!(
            "{} files only on {:?} are left alone (extras = \"keep\")",
            plan.kept_extras.len(),
            backup.sentinel.name
        ));
    }
    if plan.renamed_bytes() > 0 {
        notes.push(format!(
            "Renames save copying {}",
            human(plan.renamed_bytes())
        ));
    }
    let transfer_bytes = plan.transfer_bytes();
    let overview = Overview {
        items: plan
            .actions
            .iter()
            .map(|action| Item {
                kind: match action {
                    Action::Copy { .. } => Kind::Copy,
                    Action::Replace { .. } => Kind::Replace,
                    Action::Rename { .. } => Kind::Rename,
                    Action::Retire { .. } => Kind::Retire,
                },
                path: match action {
                    Action::Rename { from, to, .. } => {
                        format!("{} → {}", display(from), display(to))
                    }
                    other => display(other.path()),
                },
                size: match action {
                    Action::Rename { size, .. } | Action::Retire { size, .. } => *size,
                    other => other.transfer(),
                },
            })
            .collect(),
        transfer_bytes,
        notes,
    };
    ensure!(
        transfer_bytes <= free,
        "{:?} has {} free but this sync needs {}",
        backup.sentinel.name,
        human(free),
        human(transfer_bytes)
    );

    let mut source_index = Index::new(source_manifest)?;
    let mut backup_index = Index::new(backup_manifest)?;
    let mut summary = Summary::default();
    let started = Instant::now();
    if plan.actions.is_empty() {
        control.send(Event::Planned(overview));
    } else if !control.approved(overview) {
        summary.cancelled = true;
        control.send(Event::Done(summary));
        return Ok(());
    } else {
        control.send(Event::Phase(Phase::Transfer));
        let history = PathBuf::from(drive::METADATA_DIR)
            .join("history")
            .join(&source_index.manifest.header.generation);
        for action in &plan.actions {
            if control.cancelled() {
                summary.cancelled = true;
                break;
            }
            let item_kind = match action {
                Action::Copy { .. } => Kind::Copy,
                Action::Replace { .. } => Kind::Replace,
                Action::Rename { .. } => Kind::Rename,
                Action::Retire { .. } => Kind::Retire,
            };
            control.send(Event::Start {
                worker: 0,
                kind: item_kind,
                path: display(action.path()),
                size: action.transfer(),
            });
            let result = apply(
                action,
                &source_root,
                &backup_root,
                &history,
                &mut source_index,
                &mut backup_index,
                options.verify,
                control,
            );
            match &result {
                Ok(()) => {
                    summary.done += 1;
                    summary.bytes += action.transfer();
                }
                Err(_) => summary.failed += 1,
            }
            control.send(Event::Finish {
                worker: 0,
                error: result.err().map(|e| format!("{e:#}")),
            });
        }
    }

    control.send(Event::Phase(Phase::Indexing));
    source.publish(&mut source_index.finish())?;
    backup.publish(&mut backup_index.finish())?;
    summary.seconds = started.elapsed().as_secs_f64();
    control.send(Event::Done(summary));
    Ok(())
}

fn checked_file(root: &Root, path: &Path, entry: &Entry) -> Result<std::fs::File> {
    let file = root.file(path, false)?.open()?;
    ensure!(
        Stamp::of(&file.metadata()?) == entry.stamp,
        "File changed since scan: {path:?}; scan again"
    );
    Ok(file)
}

// A move changes ctime but must not change the file object, bytes or mtime.
fn unchanged_by_move(before: &Stamp, after: &Stamp) -> bool {
    (
        before.device,
        before.file_id,
        before.size,
        before.mtime_seconds,
        before.mtime_nanos,
    ) == (
        after.device,
        after.file_id,
        after.size,
        after.mtime_seconds,
        after.mtime_nanos,
    )
}

#[allow(clippy::too_many_arguments)]
fn apply(
    action: &Action,
    source: &Root,
    backup: &Root,
    history: &Path,
    source_index: &mut Index,
    backup_index: &mut Index,
    verify: bool,
    control: &Control,
) -> Result<()> {
    match action {
        Action::Rename { from, to, .. } => {
            let expected_source = source_index
                .entries
                .get(to)
                .context("Missing source entry")?;
            let source_file = checked_file(source, to, expected_source)?;
            let mut entry = backup_index
                .entries
                .get(from)
                .context("Missing backup entry")?
                .clone();
            checked_file(backup, from, &entry)?;
            let origin = backup.file(from, false)?;
            let target = backup.file(to, true)?;
            // Recheck after preparing the destination, immediately before moving.
            checked_file(source, to, expected_source)?;
            checked_file(backup, from, &entry)?;
            origin.rename_to(&target)?;
            backup_index.entries.remove(from);
            entry.path_base64 = encode_path(to);
            // Never associate an old fingerprint with changed content metadata.
            let after = Stamp::of(&target.open()?.metadata()?);
            let unchanged = unchanged_by_move(&entry.stamp, &after);
            entry.stamp = after;
            if !unchanged {
                entry.sha256 = None;
            }
            backup_index.entries.insert(to.clone(), entry);
            backup.remove_empty_parents(from);
            ensure!(unchanged, "Backup changed during rename; scan again");
            ensure!(
                Stamp::of(&source_file.metadata()?) == expected_source.stamp,
                "Source changed during rename; scan again"
            );
        }
        Action::Retire { path, .. } => {
            let entry = backup_index
                .entries
                .get(path)
                .context("Missing backup entry")?;
            checked_file(backup, path, entry)?;
            backup
                .file(path, false)?
                .rename_to(&backup.file(&history.join(path), true)?)?;
            backup.remove_empty_parents(path);
            backup_index.entries.remove(path);
        }
        Action::Copy { path, .. } | Action::Replace { path, .. } => {
            let target = backup.file(path, true)?;
            let previous = if matches!(action, Action::Replace { .. }) {
                let entry = backup_index
                    .entries
                    .get(path)
                    .context("Missing backup entry")?
                    .clone();
                checked_file(backup, path, &entry)?;
                let saved = backup.file(&history.join(path), true)?;
                target.rename_to(&saved)?;
                backup_index.entries.remove(path);
                Some((saved, entry))
            } else {
                None
            };
            let expected = source_index.entries.get(path).cloned();
            // Include opening the source in rollback handling too.
            let copied = (|| {
                copy::copy_file(
                    &source.file(path, false)?,
                    &target,
                    expected.as_ref(),
                    verify,
                    &control.cancel,
                    |bytes| control.send(Event::Progress { worker: 0, bytes }),
                )
            })();
            let Copied { sha256, stamp } = match copied {
                Ok(copied) => copied,
                Err(error) => {
                    if let Some((saved, mut entry)) = previous {
                        saved
                            .rename_to(&target)
                            .context("and the previous version is still in history")?;
                        let stamp = Stamp::of(&target.open()?.metadata()?);
                        if !unchanged_by_move(&entry.stamp, &stamp) {
                            entry.sha256 = None;
                        }
                        entry.stamp = stamp;
                        backup_index.entries.insert(path.clone(), entry);
                    }
                    return Err(error);
                }
            };
            if let Some(entry) = source_index.entries.get_mut(path) {
                entry.sha256 = Some(sha256.clone());
            }
            backup_index.entries.insert(
                path.clone(),
                Entry {
                    path_base64: encode_path(path),
                    stamp,
                    sha256: Some(sha256),
                },
            );
        }
    }
    Ok(())
}

struct Job {
    path: PathBuf,
    entry: Entry,
    /// Indexes into the drive list: every drive holding this exact content.
    holders: Vec<usize>,
    claimed: bool,
}

pub fn fill(options: FillOptions, control: Control) {
    if let Err(error) = run_fill(options, &control) {
        control.send(Event::Failed(format!("{error:#}")));
    }
}

fn run_fill(options: FillOptions, control: &Control) -> Result<()> {
    ensure!(
        !options.from.is_empty(),
        "fill needs at least one --from drive"
    );
    let mut drives = Vec::new();
    for root in &options.from {
        let drive = Drive::open(root)?;
        ensure!(
            drive.sentinel.role != Role::Scratch,
            "{:?} is a scratch disk, not a library",
            drive.sentinel.name
        );
        drives.push(drive);
    }
    // The source is the authority when the drives disagree about a path.
    drives.sort_by_key(|drive| drive.sentinel.role != Role::Source);
    fs::create_dir_all(&options.destination)?;
    drive::check_fill_destination(&options.destination)?;
    let destination = options.destination.canonicalize()?;
    let destination_root = Root::open(&destination)?;
    let source_roots = drives
        .iter()
        .map(|d| Root::open(&d.root))
        .collect::<Result<Vec<_>>>()?;
    control.send(Event::Drives {
        sources: drives.iter().map(|d| d.sentinel.name.clone()).collect(),
        destination: display(&destination),
        destination_path: destination.clone(),
    });
    control.send(Event::Phase(Phase::Scanning));

    // No walking: the indexes on the drives say what is where.
    let mut indexes = Vec::new();
    for (number, drive) in drives.iter().enumerate() {
        let manifest = drive.index()?;
        scanned(control, number, &manifest);
        indexes.push(Index::new(manifest)?.entries);
    }
    let mut wanted = Vec::new();
    for selected in &options.select {
        let relative = drives
            .iter()
            .find_map(|drive| selected.strip_prefix(&drive.root).ok())
            .unwrap_or(selected);
        ensure!(
            relative.is_relative(),
            "{selected:?} is not inside any --from drive"
        );
        wanted.push(relative.to_path_buf());
    }
    let selected = |path: &Path| wanted.is_empty() || wanted.iter().any(|w| path.starts_with(w));

    let mut jobs: BTreeMap<PathBuf, Job> = BTreeMap::new();
    for (number, index) in indexes.iter().enumerate() {
        for (path, entry) in index.iter().filter(|(path, _)| selected(path)) {
            match jobs.get_mut(path) {
                Some(job) if plan::same_content(&job.entry, entry) => job.holders.push(number),
                Some(_) => {}
                None => {
                    jobs.insert(
                        path.clone(),
                        Job {
                            path: path.clone(),
                            entry: entry.clone(),
                            holders: vec![number],
                            claimed: false,
                        },
                    );
                }
            }
        }
    }
    ensure!(
        !jobs.is_empty(),
        "Nothing in the indexes matches that selection"
    );

    let mut items = Vec::new();
    let mut already = 0;
    let mut queue = Vec::new();
    for job in jobs.into_values() {
        match fs::symlink_metadata(destination.join(&job.path)) {
            Ok(existing) => {
                use std::os::unix::fs::MetadataExt;
                if existing.len() == job.entry.stamp.size
                    && existing.mtime() == job.entry.stamp.mtime_seconds
                {
                    already += 1;
                } else {
                    items.push(Item {
                        kind: Kind::Skip,
                        path: format!("{} (a different file is already there)", display(&job.path)),
                        size: 0,
                    });
                }
            }
            Err(_) => {
                items.push(Item {
                    kind: Kind::Copy,
                    path: display(&job.path),
                    size: job.entry.stamp.size,
                });
                queue.push(job);
            }
        }
    }
    // Largest first keeps both drives busy to the end instead of leaving one
    // long file for a single drive to finish alone.
    queue.sort_by_key(|job| std::cmp::Reverse(job.entry.stamp.size));
    let transfer_bytes = queue.iter().map(|job| job.entry.stamp.size).sum();
    let shared = queue.iter().filter(|job| job.holders.len() > 1).count();
    let mut notes = vec![format!(
        "{shared} of {} files are on more than one drive and will be read from whichever is free",
        queue.len()
    )];
    if already > 0 {
        notes.push(format!("{already} files are already at the destination"));
    }
    let (free, _) = copy::space(&destination)?;
    ensure!(
        transfer_bytes <= free,
        "The destination has {} free but this needs {}",
        human(free),
        human(transfer_bytes)
    );
    let mut summary = Summary::default();
    let started = Instant::now();
    let overview = Overview {
        items,
        transfer_bytes,
        notes,
    };
    if queue.is_empty() {
        control.send(Event::Planned(overview));
        control.send(Event::Done(summary));
        return Ok(());
    }
    if !control.approved(overview) {
        summary.cancelled = true;
        control.send(Event::Done(summary));
        return Ok(());
    }
    control.send(Event::Phase(Phase::Transfer));

    let queue = Mutex::new(queue);
    let totals = Mutex::new(&mut summary);
    std::thread::scope(|scope| {
        for (worker, source_root) in source_roots.iter().enumerate() {
            let (queue, totals, destination_root) = (&queue, &totals, &destination_root);
            scope.spawn(move || {
                loop {
                    if control.cancelled() {
                        break;
                    }
                    // Files only this drive holds come first, so the shared
                    // ones stay available to whichever drive runs dry.
                    let job = {
                        let mut queue = queue.lock().expect("queue lock");
                        let mine = |job: &&mut Job| !job.claimed && job.holders.contains(&worker);
                        let exclusive = queue
                            .iter_mut()
                            .filter(mine)
                            .find(|job| job.holders.len() == 1)
                            .map(|job| job.path.clone());
                        let choice = exclusive
                            .or_else(|| queue.iter_mut().find(mine).map(|job| job.path.clone()));
                        choice.and_then(|path| {
                            let job = queue.iter_mut().find(|job| job.path == path)?;
                            job.claimed = true;
                            Some((job.path.clone(), job.entry.clone()))
                        })
                    };
                    let Some((path, entry)) = job else { break };
                    control.send(Event::Start {
                        worker,
                        kind: Kind::Copy,
                        path: display(&path),
                        size: entry.stamp.size,
                    });
                    let result = (|| {
                        copy::copy_file(
                            &source_root.file(&path, false)?,
                            &destination_root.file(&path, true)?,
                            Some(&entry),
                            options.verify,
                            &control.cancel,
                            |bytes| control.send(Event::Progress { worker, bytes }),
                        )
                    })();
                    let mut totals = totals.lock().expect("summary lock");
                    match &result {
                        Ok(_) => {
                            totals.done += 1;
                            totals.bytes += entry.stamp.size;
                        }
                        Err(_) => totals.failed += 1,
                    }
                    control.send(Event::Finish {
                        worker,
                        error: result.err().map(|e| format!("{e:#}")),
                    });
                }
            });
        }
    });
    summary.cancelled = control.cancelled();
    summary.seconds = started.elapsed().as_secs_f64();
    control.send(Event::Done(summary));
    Ok(())
}

pub fn human(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1000.0 && unit < UNITS.len() - 1 {
        value /= 1000.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// Used by `fill` callers that read the selection from stdin.
pub fn parse_selection(input: &[u8]) -> Vec<PathBuf> {
    use std::os::unix::ffi::OsStrExt;
    let separator = if input.contains(&0) { 0 } else { b'\n' };
    input
        .split(|byte| *byte == separator)
        .filter(|line| !line.is_empty())
        .map(|line| PathBuf::from(std::ffi::OsStr::from_bytes(line)))
        .collect()
}
