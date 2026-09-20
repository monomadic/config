use safesync::{
    filesystem::{self, Volume},
    lookup::{self, Query},
    manifest::{Manifest, Role, generation},
    scan,
};
use std::{
    ffi::{OsStr, OsString},
    fs,
    os::unix::{ffi::OsStringExt, fs::symlink},
    path::{Path, PathBuf},
    process::Command,
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("safesync-test-{}", generation()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn root(&self) -> PathBuf {
        let path = self.0.join("source");
        fs::create_dir_all(&path).unwrap();
        path
    }
    fn volume() -> Volume {
        Volume {
            uuid: "test-volume-uuid".into(),
            name: "Test drive".into(),
            filesystem: "apfs".into(),
        }
    }
    fn scan(&self, hash: bool) -> Manifest {
        scan::scan(&self.root(), Self::volume(), hash, |_| {}).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn sha256_matches_known_full_content_digest() {
    let f = Fixture::new();
    let p = f.root().join("abc");
    fs::write(&p, b"abc").unwrap();
    let (_, hash) = filesystem::hash_path(&p).unwrap();
    assert_eq!(
        hash,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}
#[test]
fn content_lookup_matches_renamed_file_not_equal_size_impostor() {
    let f = Fixture::new();
    fs::write(f.root().join("old.mov"), b"original").unwrap();
    let manifest = f.scan(true);
    let query = f.0.join("renamed.mov");
    fs::write(&query, b"original").unwrap();
    let found = lookup::lookup(std::slice::from_ref(&manifest), Query::File(&query)).unwrap();
    assert_eq!(found.matches.len(), 1);
    assert_eq!(found.matches[0].path_display, "old.mov");
    fs::write(&query, b"impostor").unwrap();
    let not_found = lookup::lookup(&[manifest], Query::File(&query)).unwrap();
    assert_eq!(not_found.exit_code(), 1);
}
#[test]
fn metadata_only_content_lookup_is_unknown_not_negative() {
    let f = Fixture::new();
    fs::write(f.root().join("a"), b"same").unwrap();
    let report = lookup::lookup(&[f.scan(false)], Query::File(&f.root().join("a"))).unwrap();
    assert_eq!(report.exit_code(), 3);
    assert_eq!(report.unverified_candidates, 1);
}
#[test]
fn filename_query_does_not_claim_content_match() {
    let f = Fixture::new();
    fs::write(f.root().join("Film.mov"), b"anything").unwrap();
    let manifest = f.scan(false);
    let report = lookup::lookup(
        std::slice::from_ref(&manifest),
        Query::Name(OsStr::new("Film.mov")),
    )
    .unwrap();
    assert_eq!(report.matches[0].evidence, "exact_filename_only");
    assert_eq!(
        lookup::lookup(&[manifest], Query::Name(OsStr::new("film.mov")))
            .unwrap()
            .exit_code(),
        1
    );
}
#[test]
fn offline_snapshot_works_after_source_disappears() {
    let f = Fixture::new();
    fs::write(f.root().join("clip"), b"video").unwrap();
    let query = f.0.join("query");
    fs::write(&query, b"video").unwrap();
    let path = f.0.join("offline.jsonl");
    f.scan(true).export(&path).unwrap();
    fs::rename(f.root(), f.0.join("disconnected")).unwrap();
    let snapshot = Manifest::load(&path).unwrap();
    assert!(matches!(snapshot.header.role, Role::OfflineSnapshot));
    let result = lookup::lookup(&[snapshot], Query::File(&query)).unwrap();
    assert_eq!(result.exit_code(), 0);
    assert!(result.scope.contains("historical"));
}
#[test]
fn manifest_detects_truncation_and_valid_json_tampering() {
    let f = Fixture::new();
    fs::write(f.root().join("video"), b"abc").unwrap();
    let path = f.0.join("full.jsonl");
    f.scan(true).save_new(&path).unwrap();
    let original = fs::read(&path).unwrap();
    let short = f.0.join("short.jsonl");
    fs::write(&short, &original[..original.len() - 10]).unwrap();
    assert!(Manifest::load(&short).is_err());
    let bad = f.0.join("tampered.jsonl");
    fs::write(
        &bad,
        String::from_utf8(original)
            .unwrap()
            .replace("Test drive", "Fake drive"),
    )
    .unwrap();
    assert!(Manifest::load(&bad).is_err());
}
#[test]
fn publication_never_overwrites_an_existing_file() {
    let f = Fixture::new();
    let path = f.0.join("keep.mov");
    fs::write(&path, b"irreplaceable").unwrap();
    assert!(f.scan(false).save_new(&path).is_err());
    assert_eq!(fs::read(&path).unwrap(), b"irreplaceable");
    assert!(!fs::read_dir(&f.0).unwrap().any(|e| {
        e.unwrap()
            .path()
            .extension()
            .is_some_and(|x| x == "partial")
    }));
}
#[test]
fn scan_excludes_own_metadata_and_does_not_follow_links() {
    let f = Fixture::new();
    let root = f.root();
    fs::write(root.join("a"), b"data").unwrap();
    fs::create_dir(root.join(".safesync")).unwrap();
    fs::write(root.join(".safesync/private"), b"ignore").unwrap();
    symlink(f.0.join("not-found"), root.join("broken-link")).unwrap();
    symlink(&f.0, root.join("outside-link")).unwrap();
    let manifest = f.scan(true);
    assert_eq!(manifest.entries.len(), 1);
    assert_eq!(manifest.header.skipped_symlinks, 2);
}
#[test]
fn scan_aborts_when_file_changes_after_observation() {
    let f = Fixture::new();
    let root = f.root();
    let path = root.join("a");
    fs::write(&path, b"old").unwrap();
    let result = scan::scan(&root, Fixture::volume(), true, |_| {
        fs::write(&path, b"replacement").unwrap();
    });
    assert!(result.is_err());
}
#[test]
fn scan_aborts_when_directory_changes_mid_scan() {
    let f = Fixture::new();
    let root = f.root();
    fs::write(root.join("a"), b"old").unwrap();
    let result = scan::scan(&root, Fixture::volume(), false, |_| {
        fs::write(root.join("new"), b"new").unwrap();
    });
    assert!(result.is_err());
}
#[test]
fn raw_non_unicode_and_newline_names_roundtrip() {
    let f = Fixture::new();
    let name = OsString::from_vec(b"clip\n\xff.mov".to_vec());
    // APFS need not accept arbitrary byte sequences as real names. Test the
    // manifest's lossless encoding independently of that filesystem constraint.
    fs::write(f.root().join("clip\n.mov"), b"video").unwrap();
    let path = f.0.join("raw.jsonl");
    let mut scanned = f.scan(true);
    scanned.entries[0].path_base64 = safesync::manifest::encode_path(Path::new(&name));
    scanned.save_new(&path).unwrap();
    let manifest = Manifest::load(&path).unwrap();
    assert_eq!(manifest.entries[0].path().unwrap(), Path::new(&name));
    assert_eq!(
        lookup::lookup(&[manifest], Query::Name(&name))
            .unwrap()
            .matches
            .len(),
        1
    );
}
#[test]
fn anchored_file_open_rejects_symlink_and_path_escape() {
    let f = Fixture::new();
    let root = f.root();
    symlink(&f.0, root.join("escape")).unwrap();
    let directory = fs::File::open(&root).unwrap();
    use std::os::unix::fs::MetadataExt;
    let device = directory.metadata().unwrap().dev();
    assert!(filesystem::open_relative(&directory, Path::new("escape/any"), device).is_err());
    assert!(filesystem::open_relative(&directory, Path::new("../any"), device).is_err());
}
#[test]
fn cli_exports_and_searches_local_library_without_source_disk() {
    let f = Fixture::new();
    fs::write(f.root().join("movie.mov"), b"video").unwrap();
    let manifest = f.0.join("drive.jsonl");
    f.scan(true).save_new(&manifest).unwrap();
    let home = f.0.join("home");
    fs::create_dir(&home).unwrap();
    let cli = env!("CARGO_BIN_EXE_safesync");
    let exported = Command::new(cli)
        .env("HOME", &home)
        .arg("export")
        .arg(&manifest)
        .output()
        .unwrap();
    assert!(
        exported.status.success(),
        "{}",
        String::from_utf8_lossy(&exported.stderr)
    );
    fs::rename(f.root(), f.0.join("unplugged")).unwrap();
    let found = Command::new(cli)
        .env("HOME", &home)
        .args(["lookup", "--name", "movie.mov", "--json"])
        .output()
        .unwrap();
    assert!(
        found.status.success(),
        "{}",
        String::from_utf8_lossy(&found.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&found.stdout).unwrap();
    assert_eq!(json["matches"][0]["evidence"], "exact_filename_only");
    let empty = Command::new(cli)
        .env("HOME", &home)
        .args(["lookup", "--name", "absent"])
        .status()
        .unwrap();
    assert_eq!(empty.code(), Some(1));
}

#[test]
fn generation_ids_are_unique_even_with_concurrent_calls() {
    let results: Vec<_> = (0..8)
        .map(|_| std::thread::spawn(|| (0..100).map(|_| generation()).collect::<Vec<_>>()))
        .collect();
    let all: Vec<_> = results
        .into_iter()
        .flat_map(|t| t.join().unwrap())
        .collect();
    let unique: std::collections::HashSet<_> = all.iter().collect();
    assert_eq!(unique.len(), 800);
}

#[test]
fn explicit_exclusion_is_literal_and_recorded() {
    let f = Fixture::new();
    let root = f.root();
    fs::create_dir(root.join("omit")).unwrap();
    fs::write(root.join("omit/a"), b"omit").unwrap();
    fs::write(root.join("omit-other"), b"keep").unwrap();
    let manifest =
        scan::scan_with_exclusions(&root, Fixture::volume(), false, &["omit".into()], |_| {})
            .unwrap();
    assert_eq!(manifest.entries.len(), 1);
    assert!(
        manifest
            .header
            .exclusions
            .iter()
            .any(|s| s.contains("omit"))
    );
    assert!(
        scan::scan_with_exclusions(
            &root,
            Fixture::volume(),
            false,
            &["../escape".into()],
            |_| {}
        )
        .is_err()
    );
}

#[test]
fn comparison_distinguishes_content_from_metadata_and_retains_ambiguity() {
    use safesync::compare::{Status, compare};
    let f = Fixture::new();
    for (name, data) in [
        ("same", "aaaa"),
        ("changed", "bbbb"),
        ("unknown", "cccc"),
        ("new", "dddd"),
    ] {
        fs::write(f.root().join(name), data).unwrap();
    }
    let source = f.scan(true);
    fs::write(f.root().join("changed"), "zzzz").unwrap();
    fs::rename(f.root().join("new"), f.root().join("renamed")).unwrap();
    fs::write(f.root().join("duplicate"), "dddd").unwrap();
    let mut destination = f.scan(true);
    destination.header.volume.uuid = "other-volume".into();
    destination.header.content_hashed = false;
    destination
        .entries
        .iter_mut()
        .find(|e| e.path().unwrap() == Path::new("unknown"))
        .unwrap()
        .sha256 = None;
    let report = compare(&source, &destination).unwrap();
    let row = |name: &str| report.rows.iter().find(|r| r.path_display == name).unwrap();
    assert_eq!(row("same").status, Status::ContentMatch);
    assert_eq!(row("changed").status, Status::ContentDiffers);
    assert_eq!(row("unknown").status, Status::ContentUnknown);
    assert_eq!(row("new").status, Status::SourceOnly);
    assert_eq!(row("new").content_candidates.len(), 2);
    assert_eq!(row("renamed").status, Status::DestinationOnly);
    assert!(!report.executable);
    assert!(report.historical_only);
    // Even identical inode/mtime evidence on different drives cannot prove content.
    let mut metadata = source.clone();
    metadata.header.content_hashed = false;
    for e in &mut metadata.entries {
        e.sha256 = None;
    }
    assert!(
        compare(&metadata, &metadata)
            .unwrap()
            .rows
            .iter()
            .all(|r| r.status == Status::ContentUnknown)
    );
}

#[test]
fn compare_cli_works_offline_and_rejects_corrupt_input() {
    let f = Fixture::new();
    fs::write(f.root().join("clip"), "video").unwrap();
    let manifest = f.0.join("inventory.jsonl");
    f.scan(true).save_new(&manifest).unwrap();
    fs::remove_dir_all(f.root()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_safesync"))
        .args(["compare", "--json"])
        .arg(&manifest)
        .arg(&manifest)
        .output()
        .unwrap();
    assert!(output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["executable"], false);
    assert_eq!(report["rows"][0]["status"], "content_match");
    fs::write(&manifest, "corrupt\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_safesync"))
        .arg("compare")
        .arg(&manifest)
        .arg(&manifest)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
}

#[test]
fn comparison_rejects_unsafe_and_duplicate_paths_and_reports_scope_differences() {
    use safesync::{compare::compare, manifest::encode_path};
    let f = Fixture::new();
    fs::write(f.root().join("clip"), "video").unwrap();
    let source = f.scan(true);
    for path in ["../clip", "/clip", "a/./clip", "a//clip", "clip\0suffix"] {
        let mut invalid = source.clone();
        invalid.entries[0].path_base64 = encode_path(Path::new(path));
        assert!(compare(&source, &invalid).is_err(), "accepted {path:?}");
    }
    let mut duplicate = source.clone();
    duplicate.entries.push(duplicate.entries[0].clone());
    assert!(compare(&source, &duplicate).is_err());
    let mut other = source.clone();
    other.header.exclusions.push("different-scope".into());
    assert!(
        compare(&source, &other)
            .unwrap()
            .warnings
            .iter()
            .any(|w| w.contains("Exclusion policies differ"))
    );
}

#[test]
fn initial_plan_has_hand_checked_actions_and_byte_totals() {
    use safesync::plan::{Action, preview};
    let f = Fixture::new();
    for (name, content) in [
        ("keep", "same"),
        ("replace", "old"),
        ("unknown", "hmm"),
        ("extra", "retain"),
    ] {
        fs::write(f.root().join(name), content).unwrap();
    }
    let mut destination = f.scan(true);
    destination.header.volume.uuid = "destination".into();
    destination.header.content_hashed = false;
    destination
        .entries
        .iter_mut()
        .find(|e| e.path().unwrap() == Path::new("unknown"))
        .unwrap()
        .sha256 = None;
    fs::remove_file(f.root().join("extra")).unwrap();
    fs::write(f.root().join("replace"), "replacement").unwrap();
    fs::write(f.root().join("new"), "new").unwrap();
    let source = f.scan(true);
    let plan = preview(&source, &destination).unwrap();
    let action = |name: &str| {
        &plan
            .items
            .iter()
            .find(|i| i.path_display == name)
            .unwrap()
            .proposed_action
    };
    assert_eq!(action("keep"), &Action::KeepContent);
    assert_eq!(action("replace"), &Action::ReplaceWithHistory);
    assert_eq!(action("unknown"), &Action::ReviewContent);
    assert_eq!(action("new"), &Action::Copy);
    assert_eq!(action("extra"), &Action::PreserveDestination);
    assert_eq!(plan.summary.proposed_transfer_bytes, 14);
    assert_eq!(plan.summary.proposed_history_bytes, 3);
    assert_eq!(plan.summary.review, 1);
    assert!(plan.blockers.is_empty());
    assert_eq!(plan.exit_code(), 3);
    assert!(!plan.executable);
    assert_eq!(plan.source.generation, source.header.generation);
}

#[test]
fn plan_preserves_duplicate_alternate_content_for_review() {
    use safesync::plan::{Action, preview};
    let f = Fixture::new();
    fs::write(f.root().join("one"), "same").unwrap();
    fs::write(f.root().join("two"), "same").unwrap();
    let mut destination = f.scan(true);
    destination.header.volume.uuid = "destination".into();
    fs::remove_file(f.root().join("one")).unwrap();
    fs::rename(f.root().join("two"), f.root().join("new")).unwrap();
    let plan = preview(&f.scan(true), &destination).unwrap();
    let new = plan.items.iter().find(|i| i.path_display == "new").unwrap();
    assert_eq!(new.proposed_action, Action::ReviewAlternateContent);
    assert_eq!(new.alternate_content_paths_base64.len(), 2);
    assert_eq!(plan.summary.preserve_destination, 2);
    assert_eq!(plan.summary.proposed_transfer_bytes, 0);
}

#[test]
fn plan_blocks_namespace_collisions_unknown_unicode_and_skipped_scope() {
    use safesync::{manifest::encode_path, plan::preview};
    let f = Fixture::new();
    fs::write(f.root().join("a"), "content").unwrap();
    let mut source = f.scan(true);
    let mut destination = source.clone();
    destination.header.volume.uuid = "destination".into();
    source.entries[0].path_base64 = encode_path(Path::new("Folder/movie"));
    destination.entries[0].path_base64 = encode_path(Path::new("folder"));
    let plan = preview(&source, &destination).unwrap();
    assert!(
        plan.blockers
            .iter()
            .any(|b| b.reason.contains("case collision"))
    );
    assert!(
        plan.blockers
            .iter()
            .any(|b| b.reason.contains("required as a directory"))
    );
    assert_eq!(plan.exit_code(), 3);
    for name in ["café", "cafe\u{301}", ".SAFESYNC/file"] {
        source.entries[0].path_base64 = encode_path(Path::new(name));
        assert!(!preview(&source, &destination).unwrap().blockers.is_empty());
    }
    source.entries[0].path_base64 = encode_path(Path::new("safe"));
    destination.entries.clear();
    assert!(preview(&source, &destination).unwrap().blockers.is_empty());
    destination.header.skipped_symlinks = 1;
    assert!(
        preview(&source, &destination)
            .unwrap()
            .blockers
            .iter()
            .any(|b| b.reason.contains("omitted symlinks"))
    );
}

#[test]
fn plan_is_deterministic_and_rejects_overflow() {
    use safesync::plan::preview;
    let f = Fixture::new();
    fs::write(f.root().join("a"), "a").unwrap();
    fs::write(f.root().join("b"), "b").unwrap();
    let mut source = f.scan(false);
    let mut destination = source.clone();
    destination.header.volume.uuid = "destination".into();
    destination.entries.clear();
    let original = serde_json::to_value(preview(&source, &destination).unwrap()).unwrap();
    source.entries.reverse();
    assert_eq!(
        original,
        serde_json::to_value(preview(&source, &destination).unwrap()).unwrap()
    );
    source.entries[0].stamp.size = u64::MAX;
    assert!(preview(&source, &destination).is_err());
}

#[test]
fn plan_cli_reports_review_and_input_errors_without_touching_media() {
    let f = Fixture::new();
    fs::write(f.root().join("clip"), "video").unwrap();
    let source = f.scan(true);
    let mut destination = source.clone();
    destination.header.volume.uuid = "destination".into();
    let src = f.0.join("source.jsonl");
    let dst = f.0.join("destination.jsonl");
    source.save_new(&src).unwrap();
    destination.save_new(&dst).unwrap();
    fs::remove_dir_all(f.root()).unwrap();
    let run = |dest: &Path| {
        Command::new(env!("CARGO_BIN_EXE_safesync"))
            .args(["plan", "--json"])
            .arg(&src)
            .arg(dest)
            .output()
            .unwrap()
    };
    let output = run(&dst);
    assert_eq!(output.status.code(), Some(0));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["executable"], false);
    assert_eq!(report["summary"]["keep_content"], 1);
    assert_eq!(run(&src).status.code(), Some(3)); // same-volume blocker
    fs::write(&dst, b"corrupt\n").unwrap();
    let output = run(&dst);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
}

fn rename_history_fixture() -> (Fixture, Manifest, Manifest, Manifest, Manifest) {
    let f = Fixture::new();
    fs::write(f.root().join("old.mov"), b"video").unwrap();
    let old_source = f.scan(true);
    let mut old_destination = old_source.clone();
    old_destination.header.volume.uuid = "backup".into();
    old_destination.header.generation = "previous-destination".into();
    old_destination.entries[0].stamp.file_id = 90001;
    old_destination.entries[0].stamp.device = 999;
    fs::rename(f.root().join("old.mov"), f.root().join("new.mov")).unwrap();
    let source = f.scan(true);
    let mut destination = old_destination.clone();
    destination.header.generation = "current-destination".into();
    (f, old_source, old_destination, source, destination)
}

#[test]
fn history_finds_hash_and_metadata_rename_candidates_without_cross_volume_inode_matching() {
    use safesync::history::{Evidence, preview};
    let (_f, old_source, old_destination, mut source, mut destination) = rename_history_fixture();
    let report = preview(&old_source, &old_destination, &source, &destination).unwrap();
    assert_eq!(report.observations.len(), 1);
    assert_eq!(
        report.observations[0].evidence,
        Evidence::FullContentAtScanTime
    );
    assert_eq!(report.exit_code(), 3);
    assert!(!report.executable && !report.committed_relationship);
    source.header.content_hashed = false;
    source.entries[0].sha256 = None;
    source.entries[0].stamp.ctime_seconds += 100;
    // Device numbers can differ between mounts; volume UUID is the stable identity.
    source.entries[0].stamp.device += 100;
    destination.header.content_hashed = false;
    destination.entries[0].sha256 = None;
    let report = preview(&old_source, &old_destination, &source, &destination).unwrap();
    assert_eq!(
        report.observations[0].evidence,
        Evidence::IdentityAndMetadataOnly
    );
}

#[test]
fn history_rejects_changed_content_weak_prior_evidence_and_destination_replacement() {
    use safesync::history::{Evidence, preview};
    let (_f, old_source, old_destination, source, destination) = rename_history_fixture();
    for mutation in 0..5 {
        let mut previous = old_destination.clone();
        let mut src = source.clone();
        let mut dst = destination.clone();
        match mutation {
            0 => src.entries[0].sha256 = Some("0".repeat(64)),
            1 => src.entries[0].stamp.mtime_nanos += 1,
            2 => {
                previous.header.content_hashed = false;
                previous.entries[0].sha256 = None;
            }
            3 => dst.entries[0].stamp.file_id += 1,
            _ => dst.entries[0].sha256 = Some("0".repeat(64)),
        }
        let report = preview(&old_source, &previous, &src, &dst).unwrap();
        assert_eq!(
            report.observations[0].evidence,
            Evidence::ReviewRequired,
            "mutation {mutation}"
        );
    }
}

#[test]
fn history_preserves_ambiguity_cycles_and_disappearances() {
    use safesync::history::{Evidence, preview};
    let (_f, old_source, old_destination, source, destination) = rename_history_fixture();
    for mutation in 0..5 {
        let mut src = source.clone();
        let mut dst = destination.clone();
        match mutation {
            0 => {
                let mut link = src.entries[0].clone();
                link.path_base64 = safesync::manifest::encode_path(Path::new("hardlink"));
                src.entries.push(link);
            }
            1 => {
                let mut reused = old_source.entries[0].clone();
                reused.stamp.file_id += 1;
                src.entries.push(reused);
            }
            2 => {
                let mut occupied = src.entries[0].clone();
                occupied.stamp.file_id = 99001;
                dst.entries.push(occupied);
            }
            3 => src.entries.clear(),
            _ => dst.entries.clear(),
        }
        let report = preview(&old_source, &old_destination, &src, &dst).unwrap();
        assert_eq!(
            report.observations[0].evidence,
            Evidence::ReviewRequired,
            "mutation {mutation}"
        );
        assert!(!report.executable);
    }
}

#[test]
fn history_validates_scope_and_keeps_generations_explicit() {
    use safesync::history::preview;
    let (_f, old_source, old_destination, source, destination) = rename_history_fixture();
    for mutation in 0..3 {
        let mut src = source.clone();
        match mutation {
            0 => src.header.volume.uuid = "wrong".into(),
            1 => src.header.root_file_id += 1,
            _ => src.header.exclusions.push("new exclusion".into()),
        }
        assert!(preview(&old_source, &old_destination, &src, &destination).is_err());
    }
    let mut relocated = source.clone();
    relocated.header.root_base64 =
        safesync::manifest::encode_path(Path::new("/Volumes/RenamedMount"));
    let report = preview(&old_source, &old_destination, &relocated, &destination).unwrap();
    assert_eq!(
        report.previous_source.generation,
        old_source.header.generation
    );
    assert_eq!(report.current.source.generation, source.header.generation);
}

#[test]
fn rename_plan_cli_runs_offline_and_returns_review_status() {
    let (f, old_source, old_destination, source, destination) = rename_history_fixture();
    let mut files = Vec::new();
    for (n, manifest) in [old_source, old_destination, source, destination]
        .iter()
        .enumerate()
    {
        let path = f.0.join(format!("history-{n}.jsonl"));
        manifest.save_new(&path).unwrap();
        files.push(path);
    }
    fs::remove_dir_all(f.root()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_safesync"))
        .args(["plan-renames", "--json"])
        .args(&files)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(3));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        report["observations"][0]["evidence"],
        "full_content_at_scan_time"
    );
    assert_eq!(report["committed_relationship"], false);
    fs::write(&files[0], b"broken\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_safesync"))
        .arg("plan-renames")
        .args(&files)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
}

#[test]
fn journal_info_reports_clean_incomplete_and_corrupt_outcomes() {
    use safesync::journal::{Event, Journal};
    use std::io::Write;
    let f = Fixture::new();
    let path = f.0.join("run.journal");
    let mut journal = Journal::create(
        &path,
        Event::Start {
            run_id: "empty-run".into(),
            plan_sha256: "0".repeat(64),
            operation_ids: vec![],
        },
    )
    .unwrap();
    journal.append(Event::RunCommitted).unwrap();
    drop(journal);
    let inspect = || {
        Command::new(env!("CARGO_BIN_EXE_safesync"))
            .arg("journal-info")
            .arg(&path)
            .output()
            .unwrap()
    };
    let output = inspect();
    assert_eq!(output.status.code(), Some(0));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["state"]["committed"], true);
    fs::OpenOptions::new()
        .append(true)
        .open(&path)
        .unwrap()
        .write_all(b"partial")
        .unwrap();
    let output = inspect();
    assert_eq!(output.status.code(), Some(3));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["torn_tail"], true);
    assert_eq!(report["recovery_required"], true);
    fs::write(&path, b"not a journal").unwrap();
    assert_eq!(inspect().status.code(), Some(2));
    assert_eq!(fs::read(&path).unwrap(), b"not a journal");
}
