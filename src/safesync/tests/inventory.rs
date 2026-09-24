use safesync::{
    drives,
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

fn rescan(fixture: &Fixture, cache: &scan::HashCache) -> Manifest {
    scan::scan_with_reuse(
        &fixture.root(),
        Fixture::volume(),
        true,
        &[],
        Some(cache),
        |_| {},
    )
    .unwrap()
}

#[test]
fn unchanged_file_reuses_its_fingerprint_even_after_a_rename() {
    let fixture = Fixture::new();
    fs::write(fixture.root().join("a.mov"), b"video").unwrap();
    let mut previous = fixture.scan(true);
    // A marker digest proves the second scan trusted the cache and did not read.
    let marker = "0".repeat(64);
    previous.entries[0].sha256 = Some(marker.clone());
    let mut cache = scan::HashCache::new(&Fixture::volume());
    cache.add(&previous);
    fs::rename(fixture.root().join("a.mov"), fixture.root().join("b.mov")).unwrap();

    let current = rescan(&fixture, &cache);
    assert_eq!(current.header.reused_hashes, 1);
    assert_eq!(current.entries[0].path().unwrap(), Path::new("b.mov"));
    assert_eq!(current.entries[0].sha256, Some(marker));
}

#[test]
fn changed_or_new_files_are_read_again() {
    let fixture = Fixture::new();
    fs::write(fixture.root().join("a.mov"), b"video").unwrap();
    let mut previous = fixture.scan(true);
    previous.entries[0].sha256 = Some("0".repeat(64));
    let mut cache = scan::HashCache::new(&Fixture::volume());
    cache.add(&previous);
    fs::write(fixture.root().join("a.mov"), b"longer video").unwrap();
    fs::write(fixture.root().join("new.mov"), b"video").unwrap();

    let current = rescan(&fixture, &cache);
    assert_eq!(current.header.reused_hashes, 0);
    for entry in &current.entries {
        let (_, digest) =
            filesystem::hash_path(&fixture.root().join(entry.path().unwrap())).unwrap();
        assert_eq!(entry.sha256, Some(digest));
    }
}

#[test]
fn cache_ignores_other_volumes_metadata_only_scans_and_disagreement() {
    let fixture = Fixture::new();
    fs::write(fixture.root().join("a.mov"), b"video").unwrap();
    let honest = fixture.scan(true);
    let mut foreign = honest.clone();
    foreign.header.volume.uuid = "another-volume".into();
    foreign.entries[0].sha256 = Some("0".repeat(64));

    let mut cache = scan::HashCache::new(&Fixture::volume());
    cache.add(&foreign);
    cache.add(&fixture.scan(false));
    assert!(cache.is_empty());

    // Two earlier scans that disagree about one file version: trust neither.
    let mut disputed = honest.clone();
    disputed.entries[0].sha256 = Some("1".repeat(64));
    cache.add(&honest);
    cache.add(&disputed);
    assert!(cache.is_empty());
    let current = rescan(&fixture, &cache);
    assert_eq!(current.header.reused_hashes, 0);
    assert_eq!(current.entries[0].sha256, honest.entries[0].sha256);

    let mut wrong = scan::HashCache::new(&foreign.header.volume);
    wrong.add(&foreign);
    assert!(
        scan::scan_with_reuse(
            &fixture.root(),
            Fixture::volume(),
            true,
            &[],
            Some(&wrong),
            |_| {}
        )
        .is_err()
    );
}

fn put(root: &Path, relative: &str, content: &[u8]) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

#[test]
fn catalog_search_is_case_insensitive_and_word_wise() {
    let fixture = Fixture::new();
    put(&fixture.root(), "Clips/2026/Holiday Beach.mov", b"aaaa");
    put(&fixture.root(), "Clips/2025/holiday-city.MOV", b"bb");
    put(&fixture.root(), "Stills/beach.jpg", b"c");
    let manifest = fixture.scan(false);
    let mut catalog = drives::Catalog::new();
    catalog.add(3, &manifest);
    assert_eq!(catalog.len(), 3);

    let hits = catalog.search("HOLIDAY", 10);
    assert_eq!(hits.len(), 2);
    assert!(hits.iter().all(|r| r.row == 3 && !r.hashed));
    // Every word must appear somewhere in the relative path.
    let hits = catalog.search("beach 2026", 10);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].path, "Clips/2026/Holiday Beach.mov");
    assert_eq!(hits[0].size, 4);
    assert!(catalog.search("   ", 10).is_empty());
    assert!(catalog.search("nothing", 10).is_empty());
    assert_eq!(catalog.search("o", 2).len(), 2, "limit is honoured");
}

#[test]
fn behind_counts_paths_the_backup_lacks_or_holds_at_another_size() {
    let source = Fixture::new();
    put(&source.root(), "a.mov", b"1234");
    put(&source.root(), "b.mov", b"12");
    put(&source.root(), "c.mov", b"1");
    let backup = Fixture::new();
    put(&backup.root(), "a.mov", b"1234"); // same
    put(&backup.root(), "b.mov", b"123"); // other size
    put(&backup.root(), "extra.mov", b"x"); // backup-only never counts
    let ahead = drives::behind(&source.scan(false), &backup.scan(false));
    assert_eq!(ahead, 2);
    assert_eq!(drives::behind(&source.scan(false), &source.scan(false)), 0);
}

#[test]
fn saved_sync_estimate_counts_replacements_but_not_renames_as_transfer() {
    use safesync::{drive::Extras, drives::SyncEstimate, manifest::encode_path};
    let fixture = Fixture::new();
    put(&fixture.root(), "clip.mov", b"1234");
    let source = fixture.scan(true);
    let mut backup = source.clone();
    let current = SyncEstimate::from_indexes(&source, &backup, Extras::Keep).unwrap();
    assert_eq!((current.actions, current.transfer_bytes), (0, 0));
    backup.entries[0].path_base64 = encode_path(Path::new("old.mov"));
    let rename = SyncEstimate::from_indexes(&source, &backup, Extras::Keep).unwrap();
    assert_eq!((rename.actions, rename.transfer_bytes), (1, 0));
    backup = source.clone();
    backup.entries[0].sha256 = Some("0".repeat(64));
    assert_eq!(drives::behind(&source, &backup), 0, "same path and size");
    let replace = SyncEstimate::from_indexes(&source, &backup, Extras::Keep).unwrap();
    assert_eq!((replace.actions, replace.transfer_bytes), (1, 4));
}

#[test]
fn numbers_and_ages_format_for_the_table() {
    assert_eq!(drives::group(0), "0");
    assert_eq!(drives::group(999), "999");
    assert_eq!(drives::group(1000), "1 000");
    assert_eq!(drives::group(48210), "48 210");
    assert_eq!(drives::group(1_234_567), "1 234 567");
    assert_eq!(drives::ago(100, 130), "just now");
    assert_eq!(drives::ago(100, 100 + 5 * 60), "5m ago");
    assert_eq!(drives::ago(100, 100 + 3 * 3600), "3h ago");
    assert_eq!(drives::ago(100, 100 + 9 * 86_400), "9d ago");
    assert_eq!(
        drives::ago(200, 100),
        "just now",
        "clock skew is not negative"
    );
    assert_eq!(drives::date(0).len(), 10);
}

#[test]
fn hashing_reports_exact_totals_after_the_walk() {
    let fixture = Fixture::new();
    put(&fixture.root(), "a.bin", &[1u8; 3 * 1024 * 1024]);
    put(&fixture.root(), "b.bin", &[2u8; 1024 * 1024]);
    put(&fixture.root(), "c.bin", b"tiny");
    let mut walk_reports = 0;
    let mut hash_reports: Vec<scan::HashProgress> = Vec::new();
    let manifest = scan::scan_with_reuse(
        &fixture.root(),
        Fixture::volume(),
        scan::Hashing::Missing,
        &[],
        None,
        |p| match p.hashing {
            None => {
                walk_reports += 1;
                assert!(hash_reports.is_empty(), "walk reports never follow hashing");
            }
            Some(h) => hash_reports.push(h),
        },
    )
    .unwrap();
    assert_eq!(walk_reports, 3);
    assert!(manifest.header.content_hashed);
    let total = 4 * 1024 * 1024 + 4;
    let last = hash_reports.last().unwrap();
    assert_eq!((last.files_total, last.bytes_total), (3, total));
    assert_eq!((last.files_done, last.bytes_done), (3, total));
    assert!(
        hash_reports
            .windows(2)
            .all(|w| w[0].bytes_done <= w[1].bytes_done
                && w[0].files_done <= w[1].files_done
                && w[0].bytes_total == w[1].bytes_total),
        "progress is monotonic and totals are fixed"
    );
    assert!(
        hash_reports.len() > 4,
        "a multi-megabyte file reports progress within the file, got {}",
        hash_reports.len()
    );

    // Known fingerprints are carried over and never re-read: no hashing phase.
    let mut cache = scan::HashCache::new(&Fixture::volume());
    cache.add(&manifest);
    let mut hashed = false;
    let again = scan::scan_with_reuse(
        &fixture.root(),
        Fixture::volume(),
        scan::Hashing::Missing,
        &[],
        Some(&cache),
        |p| hashed |= p.hashing.is_some_and(|h| h.files_total > 0),
    )
    .unwrap();
    assert!(!hashed);
    assert_eq!(again.header.reused_hashes, 3);
}
