//! Sentinels, planning and the copy engine, on real APFS ram disks so volume
//! identities, preallocation and exclusive renames behave as they will on Tower.
use safesync::{
    copy,
    drive::{self, Drive, Extras, Role},
    engine::{self, Control, Event, FillOptions, SyncOptions},
    manifest::Manifest,
    plan::{Action, plan},
    scan::Hashing,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, Mutex, atomic::AtomicBool, mpsc},
};

struct RamDisk {
    device: String,
    pub root: PathBuf,
}
impl RamDisk {
    fn new(name: &str) -> Self {
        let name = format!("st-{name}-{}", safesync::manifest::generation());
        let out = Command::new("hdiutil")
            .args(["attach", "-nomount", "ram://65536"])
            .output()
            .expect("hdiutil");
        assert!(out.status.success(), "{out:?}");
        let device = String::from_utf8(out.stdout).unwrap().trim().to_string();
        let ok = Command::new("diskutil")
            .args(["erasevolume", "APFS", &name, &device])
            .output()
            .expect("diskutil");
        assert!(ok.status.success(), "{ok:?}");
        Self {
            device,
            root: PathBuf::from("/Volumes").join(&name),
        }
    }
}
impl Drop for RamDisk {
    fn drop(&mut self) {
        let _ = Command::new("diskutil")
            .args(["eject", &self.device])
            .output();
    }
}

fn write(root: &Path, relative: &str, content: &[u8]) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn home() -> PathBuf {
    let home = std::env::temp_dir().join(format!(
        "safesync-home-{}",
        safesync::manifest::generation()
    ));
    fs::create_dir_all(&home).unwrap();
    home
}

struct Harness {
    events: Vec<Event>,
}
fn drive_work(work: impl FnOnce(Control) + Send + 'static, approve: bool) -> Harness {
    let (events_tx, events_rx) = mpsc::channel();
    let (confirm_tx, confirm_rx) = mpsc::channel();
    let control = Control {
        events: events_tx,
        confirm: Mutex::new(confirm_rx),
        cancel: Arc::new(AtomicBool::new(false)),
    };
    let handle = std::thread::spawn(move || work(control));
    let mut events = Vec::new();
    for event in events_rx {
        if let Event::Phase(engine::Phase::Review) = event {
            let _ = confirm_tx.send(approve);
        }
        events.push(event);
    }
    handle.join().unwrap();
    Harness { events }
}
impl Harness {
    fn summary(&self) -> &engine::Summary {
        self.events
            .iter()
            .find_map(|e| match e {
                Event::Done(s) => Some(s),
                _ => None,
            })
            .expect("run did not finish")
    }
    fn failure(&self) -> Option<&str> {
        self.events.iter().find_map(|e| match e {
            Event::Failed(m) => Some(m.as_str()),
            _ => None,
        })
    }
    fn errors(&self) -> Vec<&str> {
        self.events
            .iter()
            .filter_map(|e| match e {
                Event::Finish { error: Some(m), .. } => Some(m.as_str()),
                _ => None,
            })
            .collect()
    }
}

fn sync(source: &Path, backup: &Path, approve: bool) -> Harness {
    let options = SyncOptions {
        source: source.into(),
        backup: backup.into(),
        hashing: Hashing::Known,
        rehash: false,
        verify: true,
        exclude: vec![],
    };
    drive_work(move |c| engine::sync(options, c), approve)
}

fn scan(root: &Path) {
    let root = root.to_path_buf();
    let h = drive_work(
        move |c| engine::index_drive(root, Hashing::Missing, false, c),
        true,
    );
    assert!(h.failure().is_none(), "{:?}", h.failure());
}

#[test]
fn sentinel_pins_role_and_volume() {
    let _guard = HOME.lock().unwrap_or_else(|p| p.into_inner());
    let a = RamDisk::new("a");
    let b = RamDisk::new("b");
    let source = Drive::init(&a.root, Role::Source, None).unwrap();
    assert!(
        Drive::init(&a.root, Role::Backup, None).is_err(),
        "already has a sentinel"
    );
    assert!(
        Drive::init(&b.root, Role::Backup, None).is_err(),
        "backup needs a source"
    );
    let backup = Drive::init(&b.root, Role::Backup, Some(&source)).unwrap();
    assert_eq!(
        backup.sentinel.source_uuid.as_deref(),
        Some(source.volume.uuid.as_str())
    );
    drive::check_sync(&source, &backup).unwrap();
    // Wrong direction and the wrong pairing are refused.
    assert!(drive::check_sync(&backup, &source).is_err());
    let c = RamDisk::new("c");
    let other = Drive::init(&c.root, Role::Source, None).unwrap();
    assert!(drive::check_sync(&other, &backup).is_err());
    // A sentinel copied to another disk is refused.
    fs::create_dir_all(c.root.join(".safesync")).unwrap();
    fs::copy(
        a.root.join(".safesync/drive.toml"),
        c.root.join(".safesync/drive.toml"),
    )
    .unwrap();
    assert!(
        Drive::open(&c.root)
            .unwrap_err()
            .to_string()
            .contains("copied")
    );
    // fill refuses to write onto a source or a backup, anywhere beneath the root.
    fs::create_dir_all(a.root.join("sub")).unwrap();
    assert!(drive::check_fill_destination(&a.root.join("sub")).is_err());
    assert!(drive::check_fill_destination(&b.root).is_err());
    let d = RamDisk::new("d");
    Drive::init(&d.root, Role::Scratch, None).unwrap();
    drive::check_fill_destination(&d.root.join("new")).unwrap_err(); // does not exist yet
    fs::create_dir_all(d.root.join("new")).unwrap();
    drive::check_fill_destination(&d.root.join("new")).unwrap();
}

static HOME: Mutex<()> = Mutex::new(());

#[test]
fn plan_detects_renames_replacements_and_extras() {
    let tmp = std::env::temp_dir().join(format!(
        "safesync-plan-{}",
        safesync::manifest::generation()
    ));
    let (s, b) = (tmp.join("s"), tmp.join("b"));
    write(&s, "kept.mov", b"same");
    fs::create_dir_all(&b).unwrap();
    fs::copy(s.join("kept.mov"), b.join("kept.mov")).unwrap();
    let mtime = fs::metadata(s.join("kept.mov"))
        .unwrap()
        .modified()
        .unwrap();
    fs::File::open(b.join("kept.mov"))
        .unwrap()
        .set_modified(mtime)
        .unwrap();
    write(&s, "new/one.mov", b"new content");
    write(&s, "changed.mov", b"v2 longer");
    write(&b, "changed.mov", b"v1");
    write(&s, "moved/here.mov", b"moved video");
    fs::copy(s.join("moved/here.mov"), b.join("there.mov")).unwrap();
    // Copies keep mtime on APFS with fs::copy; a rename is size+mtime identity.
    let mtime = fs::metadata(s.join("moved/here.mov"))
        .unwrap()
        .modified()
        .unwrap();
    fs::File::open(b.join("there.mov"))
        .unwrap()
        .set_modified(mtime)
        .unwrap();
    write(&b, "extra.mov", b"only on backup");
    let volume = |name: &str| safesync::filesystem::Volume {
        uuid: name.into(),
        name: name.into(),
        filesystem: "apfs".into(),
    };
    let sm = safesync::scan::scan(&s, volume("s"), false, |_| {}).unwrap();
    let bm = safesync::scan::scan(&b, volume("b"), false, |_| {}).unwrap();
    let p = plan(&sm, &bm, Extras::Keep).unwrap();
    assert_eq!(p.unchanged, 1);
    assert_eq!(p.kept_extras.len(), 1);
    assert_eq!(
        p.actions,
        vec![
            Action::Rename {
                from: "there.mov".into(),
                to: "moved/here.mov".into(),
                size: 11
            },
            Action::Replace {
                path: "changed.mov".into(),
                size: 9
            },
            Action::Copy {
                path: "new/one.mov".into(),
                size: 11
            },
        ]
    );
    let p = plan(&sm, &bm, Extras::History).unwrap();
    assert!(p.actions.contains(&Action::Retire {
        path: "extra.mov".into(),
        size: 14
    }));
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn copy_never_overwrites_and_aborts_on_a_changed_source() {
    let tmp = std::env::temp_dir().join(format!(
        "safesync-copy-{}",
        safesync::manifest::generation()
    ));
    write(&tmp, "src.bin", &vec![7u8; 3 << 20]);
    let cancel = AtomicBool::new(false);
    let copied = copy::copy_file(
        &tmp.join("src.bin"),
        &tmp.join("out/dst.bin"),
        None,
        true,
        &cancel,
        |_| {},
    )
    .unwrap();
    assert_eq!(copied.stamp.size, 3 << 20);
    assert_eq!(
        fs::read(tmp.join("out/dst.bin")).unwrap(),
        fs::read(tmp.join("src.bin")).unwrap()
    );
    assert_eq!(
        fs::metadata(tmp.join("out/dst.bin"))
            .unwrap()
            .modified()
            .unwrap(),
        fs::metadata(tmp.join("src.bin"))
            .unwrap()
            .modified()
            .unwrap(),
        "mtime is preserved"
    );
    let again = copy::copy_file(
        &tmp.join("src.bin"),
        &tmp.join("out/dst.bin"),
        None,
        false,
        &cancel,
        |_| {},
    );
    assert!(again.unwrap_err().to_string().contains("already exists"));
    // A stale index entry stops the copy before any byte is written.
    let mut entry = safesync::manifest::Entry {
        path_base64: safesync::manifest::encode_path(Path::new("src.bin")),
        stamp: copied.stamp.clone(),
        sha256: Some(copied.sha256.clone()),
    };
    entry.stamp.size += 1;
    let stale = copy::copy_file(
        &tmp.join("src.bin"),
        &tmp.join("out/two.bin"),
        Some(&entry),
        false,
        &cancel,
        |_| {},
    );
    assert!(stale.unwrap_err().to_string().contains("Changed since"));
    // A wrong recorded fingerprint means the source no longer reads as indexed.
    entry.stamp.size -= 1;
    entry.sha256 = Some("0".repeat(64));
    let bad = copy::copy_file(
        &tmp.join("src.bin"),
        &tmp.join("out/three.bin"),
        Some(&entry),
        false,
        &cancel,
        |_| {},
    );
    assert!(bad.unwrap_err().to_string().contains("fingerprint"));
    assert!(!tmp.join("out/three.bin").exists());
    assert!(fs::read_dir(tmp.join("out")).unwrap().all(|e| {
        !e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".safesync-part")
    }));
    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn sync_copies_renames_and_updates_both_indexes() {
    let _guard = HOME.lock().unwrap_or_else(|p| p.into_inner());
    let home = home();
    // SAFETY: tests holding HOME serialise on the mutex above.
    unsafe { std::env::set_var("HOME", &home) };
    let a = RamDisk::new("src");
    let b = RamDisk::new("bak");
    let source = Drive::init(&a.root, Role::Source, None).unwrap();
    Drive::init(&b.root, Role::Backup, Some(&source)).unwrap();
    write(&a.root, "clips/one.mov", &vec![1u8; 2 << 20]);
    write(&a.root, "clips/two.mov", &vec![2u8; 1 << 20]);
    write(&a.root, "old-name.mov", &vec![3u8; 1 << 20]);
    write(&b.root, "leftover.mov", b"backup only");

    // Declined review copies nothing.
    let declined = sync(&a.root, &b.root, false);
    assert!(declined.summary().cancelled);
    assert!(!b.root.join("clips/one.mov").exists());

    let first = sync(&a.root, &b.root, true);
    assert_eq!(first.failure(), None);
    assert_eq!(first.summary().done, 3);
    assert_eq!(
        fs::read(b.root.join("clips/one.mov")).unwrap().len(),
        2 << 20
    );
    assert!(
        b.root.join("leftover.mov").exists(),
        "extras are kept by default"
    );
    let index = Drive::open(&b.root).unwrap().index().unwrap();
    assert_eq!(index.entries.len(), 4);
    let hashed = index.entries.iter().filter(|e| e.sha256.is_some()).count();
    assert_eq!(
        hashed, 3,
        "every copied file has a fingerprint; the leftover was never read"
    );
    let sindex = Drive::open(&a.root).unwrap().index().unwrap();
    assert!(
        sindex.header.content_hashed,
        "the source learnt fingerprints from the copy"
    );
    // Local copies for offline lookup exist for both drives.
    assert_eq!(
        fs::read_dir(home.join("Library/Application Support/safesync/manifests"))
            .unwrap()
            .count(),
        2
    );

    // A rename on the source becomes a rename on the backup: no bytes copied.
    fs::rename(
        a.root.join("old-name.mov"),
        a.root.join("clips/new-name.mov"),
    )
    .unwrap();
    let second = sync(&a.root, &b.root, true);
    assert_eq!(second.summary().done, 1);
    assert_eq!(second.summary().bytes, 0);
    assert!(b.root.join("clips/new-name.mov").exists());
    assert!(!b.root.join("old-name.mov").exists());

    // Nothing to do next time.
    let third = sync(&a.root, &b.root, true);
    assert_eq!(third.summary().done, 0);

    // Source was never written: only its .safesync changed.
    let mut names: Vec<_> = fs::read_dir(&a.root)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .filter(|name| name != ".fseventsd") // macOS's own, not ours
        .collect();
    names.sort();
    assert_eq!(names, [".safesync", "clips"].map(std::ffi::OsString::from));
    let _ = fs::remove_dir_all(&home);
}

#[test]
fn fill_reads_from_both_drives_and_skips_what_is_there() {
    let _guard = HOME.lock().unwrap_or_else(|p| p.into_inner());
    let home = home();
    unsafe { std::env::set_var("HOME", &home) };
    let a = RamDisk::new("lib");
    let b = RamDisk::new("mirror");
    let c = RamDisk::new("ssd");
    let source = Drive::init(&a.root, Role::Source, None).unwrap();
    Drive::init(&b.root, Role::Backup, Some(&source)).unwrap();
    Drive::init(&c.root, Role::Scratch, None).unwrap();
    write(&a.root, "set/x.mov", &vec![1u8; 2 << 20]);
    write(&a.root, "set/y.mov", &vec![2u8; 2 << 20]);
    write(&a.root, "other.mov", b"not selected");
    scan(&a.root);
    let synced = sync(&a.root, &b.root, true);
    assert_eq!(synced.summary().done, 3);
    write(&c.root, "set/x.mov", &vec![1u8; 2 << 20]);
    let mtime = fs::metadata(a.root.join("set/x.mov"))
        .unwrap()
        .modified()
        .unwrap();
    fs::File::open(c.root.join("set/x.mov"))
        .unwrap()
        .set_modified(mtime)
        .unwrap();

    let options = FillOptions {
        from: vec![a.root.clone(), b.root.clone()],
        destination: c.root.join("dump"),
        select: vec![PathBuf::from("set")],
        verify: true,
    };
    let filled = drive_work(move |ctl| engine::fill(options, ctl), true);
    assert_eq!(filled.failure(), None, "{:?}", filled.errors());
    assert_eq!(filled.summary().done, 2);
    assert!(c.root.join("dump/set/x.mov").exists());
    assert!(c.root.join("dump/set/y.mov").exists());
    assert!(!c.root.join("dump/other.mov").exists());
    let workers: std::collections::HashSet<_> = filled
        .events
        .iter()
        .filter_map(|e| match e {
            Event::Start { worker, .. } => Some(*worker),
            _ => None,
        })
        .collect();
    assert_eq!(workers.len(), 2, "both drives took a file");

    // Refused onto a backup.
    let options = FillOptions {
        from: vec![a.root.clone()],
        destination: b.root.join("dump"),
        select: vec![],
        verify: false,
    };
    let refused = drive_work(move |ctl| engine::fill(options, ctl), true);
    assert!(refused.failure().unwrap().contains("scratch"));
    let _ = fs::remove_dir_all(&home);
}

#[test]
fn manifests_written_by_sync_load_as_ordinary_indexes() {
    let tmp =
        std::env::temp_dir().join(format!("safesync-idx-{}", safesync::manifest::generation()));
    write(&tmp, "a.mov", b"x");
    let volume = safesync::filesystem::Volume {
        uuid: "u".into(),
        name: "n".into(),
        filesystem: "apfs".into(),
    };
    let m = safesync::scan::scan(&tmp, volume, true, |_| {}).unwrap();
    let path = tmp.join("m.jsonl");
    m.save_new(&path).unwrap();
    assert_eq!(Manifest::load(&path).unwrap().entries.len(), 1);
    let _ = fs::remove_dir_all(&tmp);
}
