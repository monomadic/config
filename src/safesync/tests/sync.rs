//! Sentinels, planning and the copy engine, on real APFS ram disks so volume
//! identities, preallocation and exclusive renames behave as they will on Tower.
use safesync::{
    copy,
    drive::{self, Drive, Extras, Role},
    drives::{self, Marking, Relation},
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
    let root = safesync::filesystem::Root::open(&tmp).unwrap();
    let copied = copy::copy_file(
        &root.file(Path::new("src.bin"), true).unwrap(),
        &root.file(Path::new("out/dst.bin"), true).unwrap(),
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
        &root.file(Path::new("src.bin"), true).unwrap(),
        &root.file(Path::new("out/dst.bin"), true).unwrap(),
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
        &root.file(Path::new("src.bin"), true).unwrap(),
        &root.file(Path::new("out/two.bin"), true).unwrap(),
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
        &root.file(Path::new("src.bin"), true).unwrap(),
        &root.file(Path::new("out/three.bin"), true).unwrap(),
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

#[test]
fn drives_inventory_lists_mounted_volumes_with_roles_and_index_state() {
    let _guard = HOME.lock().unwrap_or_else(|p| p.into_inner());
    let home = home();
    // SAFETY: tests holding HOME serialise on the mutex above.
    unsafe { std::env::set_var("HOME", &home) };
    let a = RamDisk::new("inv-src");
    let b = RamDisk::new("inv-bak");
    let c = RamDisk::new("inv-plain");
    write(&a.root, "clips/one.mov", b"one");
    write(&a.root, "clips/two.mov", b"two");
    let source = Drive::init(&a.root, Role::Source, None).unwrap();
    let backup_uuid = Drive::init(&b.root, Role::Backup, Some(&source))
        .unwrap()
        .volume
        .uuid;
    scan(&a.root);
    scan(&b.root);

    let inventory = drives::Inventory::load();
    let row = |root: &Path| {
        inventory
            .rows
            .iter()
            .find(|r| r.path.as_deref() == Some(root))
            .unwrap_or_else(|| panic!("{root:?} missing from {:?}", inventory.rows))
    };
    let boot = inventory
        .rows
        .iter()
        .find(|r| r.path.as_deref() == Some(Path::new("/")))
        .expect("boot volume listed");
    assert!(matches!(boot.marking, Marking::Boot));
    assert!(boot.assign_refusal().is_some());

    let src = row(&a.root);
    assert_eq!(src.role(), Some(Role::Source));
    assert!(src.total > 0 && src.free > 0);
    let index = src.index.as_ref().expect("source has an index");
    assert_eq!(index.files, 2);
    assert_eq!(index.bytes, 6);
    assert_eq!(index.generations, 1);
    assert!(index.saved_locally, "a copy landed under $HOME");
    assert!(src.scan_refusal().is_none());
    assert!(
        src.assign_refusal().is_some(),
        "roles are never changed from the screen"
    );
    assert!(
        matches!(&src.relation, Relation::Source { backups } if backups == &[b.root.file_name().unwrap().to_string_lossy().to_string()])
    );

    let bak = row(&b.root);
    assert_eq!(bak.role(), Some(Role::Backup));
    match &bak.relation {
        Relation::Backup {
            source_name,
            source_online,
            behind,
            ..
        } => {
            assert_eq!(source_name.as_deref(), Some(src.name.as_str()));
            assert!(source_online);
            assert_eq!(*behind, Some(2), "nothing has been synced yet");
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(
        bak.state(inventory.loaded_unix).0,
        "2 missing or size-changed files (saved indexes)"
    );

    let plain = row(&c.root);
    assert!(matches!(plain.marking, Marking::Unmarked));
    assert!(plain.assign_refusal().is_none());
    assert!(plain.scan_refusal().is_some());
    assert!(plain.index.is_none());
    let source_row = inventory
        .rows
        .iter()
        .position(|r| r.path.as_deref() == Some(a.root.as_path()))
        .unwrap();
    assert!(inventory.sources().contains(&source_row));
    let plain_row = inventory
        .rows
        .iter()
        .position(|r| r.path.as_deref() == Some(c.root.as_path()))
        .unwrap();
    assert!(!inventory.sources().contains(&plain_row));

    // The catalog covers every indexed file; search finds them by any part of the path.
    let mine = |inventory: &drives::Inventory, query: &str| {
        inventory
            .catalog
            .search(query, 100_000)
            .into_iter()
            .filter(|r| {
                let uuid = inventory.rows[r.row].uuid.as_deref();
                uuid == Some(source.volume.uuid.as_str()) || uuid == Some(backup_uuid.as_str())
            })
            .count()
    };
    assert_eq!(mine(&inventory, "clips/"), 2);

    // A copied sentinel is shown as refused, never as a role.
    fs::create_dir_all(c.root.join(".safesync")).unwrap();
    fs::copy(
        a.root.join(".safesync/drive.toml"),
        c.root.join(".safesync/drive.toml"),
    )
    .unwrap();
    let inventory = drives::Inventory::load();
    let copied = inventory
        .rows
        .iter()
        .find(|r| r.path.as_deref() == Some(c.root.as_path()))
        .unwrap();
    assert!(matches!(copied.marking, Marking::Copied(_)));
    assert_eq!(copied.role(), None);
    assert!(copied.assign_refusal().is_some());

    // Unmounted: the saved index stands in for the drive.
    let uuid = source.volume.uuid.clone();
    drop(a);
    let inventory = drives::Inventory::load();
    let offline = inventory
        .rows
        .iter()
        .find(|r| r.uuid.as_deref() == Some(uuid.as_str()))
        .expect("offline drive listed from its saved index");
    assert!(!offline.online());
    assert!(matches!(offline.marking, Marking::Offline));
    assert_eq!(offline.role(), Some(Role::Source));
    assert!(offline.scan_refusal().is_some());
    assert_eq!(offline.index.as_ref().unwrap().files, 2);
    // The backup was never synced, so only the ejected source's saved index holds the file.
    assert_eq!(
        mine(&inventory, "two.mov"),
        1,
        "an unplugged drive is still searchable"
    );
    let hit = inventory.catalog.search("clips/two.mov", 100_000);
    assert!(hit.iter().any(|r| !inventory.rows[r.row].online()));
    let bak = inventory
        .rows
        .iter()
        .find(|r| r.path.as_deref() == Some(b.root.as_path()))
        .unwrap();
    assert!(matches!(
        &bak.relation,
        Relation::Backup {
            source_online: false,
            behind: Some(2),
            ..
        }
    ));
    // Both ends stay grouped after unplugging, using metadata in their saved
    // indexes. It remains historical data, not a live sentinel.
    drop(b);
    let inventory = drives::Inventory::load();
    let section = inventory
        .sections()
        .into_iter()
        .find(|s| s.key == format!("source:{uuid}"))
        .unwrap();
    assert_eq!(section.rows.len(), 2);
    assert_eq!(
        inventory.rows[section.rows[0]].uuid.as_deref(),
        Some(uuid.as_str())
    );
    let backup = &inventory.rows[section.rows[1]];
    assert_eq!(backup.uuid.as_deref(), Some(backup_uuid.as_str()));
    assert_eq!(backup.role(), Some(Role::Backup));
    assert_eq!(backup.source_uuid(), Some(uuid.as_str()));
    assert!(backup.sentinel().is_none());
    assert!(backup.scan_refusal().is_some());
    assert!(matches!(
        backup.relation,
        Relation::Backup {
            behind: Some(2),
            source_online: false,
            ..
        }
    ));
    let _ = fs::remove_dir_all(&home);
}

fn review_sync(source: &Path, backup: &Path, mutate: impl FnOnce()) -> Harness {
    let options = SyncOptions {
        source: source.into(),
        backup: backup.into(),
        hashing: Hashing::Known,
        rehash: false,
        verify: true,
        exclude: vec![],
    };
    let (events_tx, events_rx) = mpsc::channel();
    let (confirm_tx, confirm_rx) = mpsc::channel();
    let control = Control {
        events: events_tx,
        confirm: Mutex::new(confirm_rx),
        cancel: Arc::new(AtomicBool::new(false)),
    };
    let handle = std::thread::spawn(move || engine::sync(options, control));
    let mut mutate = Some(mutate);
    let mut events = Vec::new();
    for event in events_rx {
        if let Event::Phase(engine::Phase::Review) = event {
            mutate.take().unwrap()();
            confirm_tx.send(true).unwrap();
        }
        events.push(event);
    }
    handle.join().unwrap();
    Harness { events }
}

#[test]
fn sync_and_fill_reject_symlinked_destination_parents() {
    let _guard = HOME.lock().unwrap_or_else(|p| p.into_inner());
    let home = home();
    unsafe { std::env::set_var("HOME", &home) };
    let a = RamDisk::new("review-src");
    let b = RamDisk::new("review-bak");
    let source = Drive::init(&a.root, Role::Source, None).unwrap();
    Drive::init(&b.root, Role::Backup, Some(&source)).unwrap();
    write(&a.root, "clips/one.mov", b"source media");
    fs::create_dir_all(a.root.join("unrelated")).unwrap();
    std::os::unix::fs::symlink(a.root.join("unrelated"), b.root.join("clips")).unwrap();
    let result = sync(&a.root, &b.root, true);
    assert_eq!(result.failure(), None);
    assert_eq!(result.summary().failed, 1, "{:?}", result.errors());
    assert!(!a.root.join("unrelated/one.mov").exists());
    let destination = home.join("dump");
    fs::create_dir_all(&destination).unwrap();
    std::os::unix::fs::symlink(a.root.join("unrelated"), destination.join("clips")).unwrap();
    let options = FillOptions {
        from: vec![a.root.clone()],
        destination,
        select: vec![],
        verify: true,
    };
    let filled = drive_work(move |c| engine::fill(options, c), true);
    assert_eq!(filled.failure(), None);
    assert_eq!(filled.summary().failed, 1);
    assert!(!a.root.join("unrelated/one.mov").exists());
    // The rename optimization must reject the same redirected destination.
    write(&b.root, "old.mov", b"source media");
    let mtime = fs::metadata(a.root.join("clips/one.mov"))
        .unwrap()
        .modified()
        .unwrap();
    fs::File::open(b.root.join("old.mov"))
        .unwrap()
        .set_modified(mtime)
        .unwrap();
    let renamed = sync(&a.root, &b.root, true);
    assert_eq!(renamed.failure(), None);
    assert_eq!(renamed.summary().failed, 1);
    assert!(b.root.join("old.mov").exists());
    assert!(!a.root.join("unrelated/one.mov").exists());
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn failed_replacement_restores_file_and_index_entry() {
    let _guard = HOME.lock().unwrap_or_else(|p| p.into_inner());
    let home = home();
    unsafe { std::env::set_var("HOME", &home) };
    let a = RamDisk::new("review-src");
    let b = RamDisk::new("review-bak");
    let source = Drive::init(&a.root, Role::Source, None).unwrap();
    Drive::init(&b.root, Role::Backup, Some(&source)).unwrap();
    write(&a.root, "clip.mov", b"new version");
    write(&b.root, "clip.mov", b"old");
    let result = review_sync(&a.root, &b.root, || {
        write(&a.root, "clip.mov", b"changed after scan")
    });
    assert_eq!(result.failure(), None);
    assert_eq!(result.summary().failed, 1);
    assert_eq!(fs::read(b.root.join("clip.mov")).unwrap(), b"old");
    let index = Drive::open(&b.root).unwrap().index().unwrap();
    assert_eq!(index.entries.len(), 1);
    assert_eq!(index.entries[0].path().unwrap(), Path::new("clip.mov"));
    assert_eq!(
        index.entries[0].stamp,
        safesync::filesystem::Stamp::of(&fs::metadata(b.root.join("clip.mov")).unwrap())
    );
    // fill relies on the published index, so the restored old version remains usable.
    let destination = home.join("restored");
    let options = FillOptions {
        from: vec![b.root.clone()],
        destination: destination.clone(),
        select: vec![],
        verify: true,
    };
    let filled = drive_work(move |c| engine::fill(options, c), true);
    assert_eq!(filled.failure(), None);
    assert_eq!(filled.summary().done, 1);
    assert_eq!(fs::read(destination.join("clip.mov")).unwrap(), b"old");
    // A later successful replacement keeps the old bytes in history.
    let replaced = sync(&a.root, &b.root, true);
    assert_eq!(replaced.failure(), None);
    assert_eq!(replaced.summary().failed, 0);
    assert_eq!(replaced.summary().done, 1);
    assert_eq!(
        fs::read(b.root.join("clip.mov")).unwrap(),
        b"changed after scan"
    );
    assert!(
        fs::read_dir(b.root.join(".safesync/history"))
            .unwrap()
            .any(|e| {
                fs::read(e.unwrap().path().join("clip.mov")).is_ok_and(|bytes| bytes == b"old")
            })
    );
    assert_eq!(sync(&a.root, &b.root, true).summary().done, 0);
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn rename_rejects_changes_during_review_without_poisoning_hash_cache() {
    let _guard = HOME.lock().unwrap_or_else(|p| p.into_inner());
    let home = home();
    unsafe { std::env::set_var("HOME", &home) };
    let a = RamDisk::new("review-src");
    let b = RamDisk::new("review-bak");
    let source = Drive::init(&a.root, Role::Source, None).unwrap();
    Drive::init(&b.root, Role::Backup, Some(&source)).unwrap();
    write(&a.root, "new.mov", b"original");
    write(&b.root, "old.mov", b"original");
    let mtime = fs::metadata(a.root.join("new.mov"))
        .unwrap()
        .modified()
        .unwrap();
    fs::File::open(b.root.join("old.mov"))
        .unwrap()
        .set_modified(mtime)
        .unwrap();
    scan(&a.root);
    scan(&b.root);
    let result = review_sync(&a.root, &b.root, || write(&b.root, "old.mov", b"modified"));
    assert_eq!(result.failure(), None);
    assert_eq!(result.summary().failed, 1);
    assert_eq!(result.summary().done, 0);
    assert!(!b.root.join("new.mov").exists());
    assert_eq!(fs::read(b.root.join("old.mov")).unwrap(), b"modified");
    let subsequent = sync(&a.root, &b.root, true);
    assert_eq!(subsequent.summary().failed, 0);
    assert_eq!(subsequent.summary().done, 1);
    assert_eq!(fs::read(b.root.join("new.mov")).unwrap(), b"original");
    // The source half of a planned rename must also still match the scan.
    fs::rename(a.root.join("new.mov"), a.root.join("next.mov")).unwrap();
    let changed_source = review_sync(&a.root, &b.root, || {
        write(&a.root, "next.mov", b"different")
    });
    assert_eq!(changed_source.failure(), None);
    assert_eq!(changed_source.summary().failed, 1);
    assert!(b.root.join("new.mov").exists());
    assert!(!b.root.join("next.mov").exists());
    fs::remove_dir_all(home).unwrap();
}

#[test]
fn copy_and_cleanup_stay_anchored_when_parent_is_replaced() {
    use safesync::filesystem::Root;
    use std::sync::atomic::Ordering;
    let tmp = home();
    write(&tmp, "source.bin", &vec![7; 20 << 20]);
    let root = Root::open(&tmp).unwrap();
    for cancel_copy in [false, true] {
        let base = if cancel_copy {
            "cancelled"
        } else {
            "completed"
        };
        let parent = tmp.join(base);
        let parked = tmp.join(format!("{base}-parked"));
        let outside = tmp.join(format!("{base}-outside"));
        fs::create_dir_all(&outside).unwrap();
        let target = root.file(&Path::new(base).join("file.bin"), true).unwrap();
        let cancel = AtomicBool::new(false);
        let mut switched = false;
        let result = copy::copy_file(
            &root.file(Path::new("source.bin"), false).unwrap(),
            &target,
            None,
            true,
            &cancel,
            |_| {
                if !switched {
                    fs::rename(&parent, &parked).unwrap();
                    std::os::unix::fs::symlink(&outside, &parent).unwrap();
                    cancel.store(cancel_copy, Ordering::Relaxed);
                    switched = true;
                }
            },
        );
        assert!(switched);
        assert_eq!(result.is_err(), cancel_copy);
        assert_eq!(fs::read_dir(&outside).unwrap().count(), 0);
        assert!(fs::read_dir(&parked).unwrap().all(|e| {
            !e.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with(copy::PARTIAL_PREFIX)
        }));
        if !cancel_copy {
            assert_eq!(
                fs::read(parked.join("file.bin")).unwrap(),
                fs::read(tmp.join("source.bin")).unwrap()
            );
        }
        // A newly resolved path must refuse the replacement symlink.
        assert!(
            root.file(&Path::new(base).join("another.bin"), true)
                .is_err()
        );
    }
    assert!(root.file(Path::new("../escape.bin"), true).is_err());
    fs::remove_dir_all(tmp).unwrap();
}
