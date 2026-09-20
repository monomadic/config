use anyhow::{Context, Result, ensure};
use clap::{Parser, Subcommand};
use safesync::{
    filesystem,
    lookup::{self, Query},
    manifest::{Manifest, generation},
};
use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

#[derive(Parser)]
#[command(
    version,
    about = "Portable manifests and offline file lookup. No media copying or deletion."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    /// Verify filesystems and live file preconditions for an unstarted prepared run. No media writes.
    CheckRun {
        source: PathBuf,
        destination: PathBuf,
        run_id: String,
        /// Leave this many bytes for metadata, journals and safety (default: 1 GiB).
        #[arg(long, default_value_t = safesync::capacity::DEFAULT_RESERVE_BYTES)]
        reserve_bytes: u64,
    },
    /// Verify and hash both drives, then save a copy/replacement plan and journal. No execution.
    PrepareRun {
        source: PathBuf,
        destination: PathBuf,
        #[arg(long)]
        exclude: Vec<PathBuf>,
    },
    /// Validate the immutable plan and its bound journal. Never execute or repair.
    RunInfo { directory: PathBuf },
    /// Inspect a stopped journal; never replay, truncate or repair it.
    JournalInfo { journal: PathBuf },
    /// Verify, hash and record matching same-path replicas on the destination.
    RelationshipAdopt {
        source: PathBuf,
        destination: PathBuf,
        #[arg(long)]
        exclude: Vec<PathBuf>,
    },
    /// Validate an adoption baseline and review it against two current manifests.
    PlanRelationship {
        baseline: PathBuf,
        source: PathBuf,
        destination: PathBuf,
    },
    /// Analyze possible renames using four historical snapshots. Never applies changes.
    PlanRenames {
        previous_source: PathBuf,
        previous_destination: PathBuf,
        source: PathBuf,
        destination: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Verify both filesystems, scan enrolled roots and publish drive-owned catalogs.
    CatalogRefresh {
        source: PathBuf,
        destination: PathBuf,
        #[arg(long)]
        hash: bool,
        /// Read every file again instead of reusing fingerprints from earlier catalogs.
        #[arg(long, requires = "hash")]
        rehash: bool,
        #[arg(long)]
        exclude: Vec<PathBuf>,
    },
    /// Read the current drive-owned catalog under an enrollment lease.
    Catalog { root: PathBuf },
    /// Preview initial-adoption proposals from two manifests. Never applies changes.
    Plan {
        source: PathBuf,
        destination: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Check enrolled drive direction and acquire/release session locks. No writes.
    CheckPair {
        source: PathBuf,
        destination: PathBuf,
        /// Run diskutil verifyVolume on both drives before acquiring drive leases.
        #[arg(long)]
        verify_filesystem: bool,
    },
    /// Record a fixed drive role at a local APFS volume root. Writes metadata only.
    Enroll {
        root: PathBuf,
        #[arg(long, value_enum)]
        role: safesync::enrollment::DriveRole,
    },
    /// Inspect and validate a drive's enrollment against its mounted volume identity.
    Enrollment { root: PathBuf },
    /// Compare historical regular-file snapshots. Does not produce an executable plan.
    Compare {
        source: PathBuf,
        destination: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Inventory regular files; errors abort publication. Never follows symlinks.
    Scan {
        root: PathBuf,
        /// Hash every file's complete contents for accurate offline content lookup.
        #[arg(long)]
        hash: bool,
        /// Read every file again instead of reusing fingerprints from earlier scans
        /// of this volume whose file ID, size and mtime are unchanged.
        #[arg(long, requires = "hash")]
        rehash: bool,
        /// New manifest path; default: ROOT/.safesync/manifest-GENERATION.jsonl.
        #[arg(long)]
        output: Option<PathBuf>,
        /// Also export a read-only inventory snapshot to this Mac's manifest library.
        #[arg(long)]
        save_local: bool,
        /// Exclude a literal relative file/subtree (repeatable; no glob patterns).
        #[arg(long)]
        exclude: Vec<PathBuf>,
    },
    /// Save a validated manifest snapshot locally, without the original drive.
    Export {
        manifest: PathBuf,
        /// New output path; default: this Mac's safesync manifest library.
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Search historical manifests, never claiming that an offline file is present now.
    Lookup {
        /// Repeat to select manifests. Defaults to all snapshots in the local library.
        #[arg(long, short = 'm')]
        manifest: Vec<PathBuf>,
        /// Exact, case-sensitive basename match; does not compare contents.
        #[arg(long, conflicts_with = "file", required_unless_present = "file")]
        name: Option<OsString>,
        /// Hash this file and compare complete-content SHA-256 fingerprints.
        #[arg(long, conflicts_with = "name", required_unless_present = "name")]
        file: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Show the identity, scope, exclusions and hash coverage of a manifest.
    Info { manifest: PathBuf },
    /// List saved snapshots on this Mac; does not open the original drives.
    Manifests,
}
fn library() -> Result<PathBuf> {
    Ok(
        PathBuf::from(std::env::var_os("HOME").context("HOME is not set")?)
            .join("Library/Application Support/safesync/manifests"),
    )
}
fn local_output() -> Result<PathBuf> {
    let directory = library()?;
    fs::create_dir_all(&directory)?;
    Ok(directory.join(format!("snapshot-{}.jsonl", generation())))
}
fn saved_manifests() -> Result<Vec<PathBuf>> {
    let directory = library()?;
    if !directory.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.file_type()?.is_file() && entry.path().extension().is_some_and(|e| e == "jsonl") {
            paths.push(entry.path());
        }
    }
    paths.sort();
    Ok(paths)
}
fn safe_display(path: &Path) -> String {
    format!("{:?}", path.as_os_str())
}
fn run(cli: Cli) -> Result<i32> {
    match cli.command {
        Command::CheckRun {
            source,
            destination,
            run_id,
            reserve_bytes,
        } => {
            let session = safesync::maintenance::verify_pair(&source, &destination)?;
            let report = session.check_run_with_reserve(&run_id, reserve_bytes)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            return Ok(report.exit_code());
        }
        Command::PrepareRun {
            source,
            destination,
            exclude,
        } => {
            let session = safesync::maintenance::verify_pair(&source, &destination)?;
            let prepared = session.prepare_run(&exclude)?;
            println!("{}", serde_json::to_string_pretty(&prepared)?);
        }
        Command::RunInfo { directory } => {
            let report = safesync::run::inspect(&directory)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            return Ok(if report.journal.recovery_required {
                3
            } else {
                0
            });
        }
        Command::JournalInfo { journal } => {
            let report = safesync::journal::inspect(&journal)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            return Ok(if report.recovery_required { 3 } else { 0 });
        }
        Command::RelationshipAdopt {
            source,
            destination,
            exclude,
        } => {
            let verified = safesync::maintenance::verify_pair(&source, &destination)?;
            let adoption = verified.adopt_relationship(&exclude)?;
            println!("{}", serde_json::to_string_pretty(&adoption)?);
        }
        Command::PlanRelationship {
            baseline,
            source,
            destination,
        } => {
            let baseline = safesync::relationship::Baseline::load(&baseline)?;
            let report =
                baseline.preview(&Manifest::load(&source)?, &Manifest::load(&destination)?)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            return Ok(report.exit_code());
        }
        Command::PlanRenames {
            previous_source,
            previous_destination,
            source,
            destination,
            json,
        } => {
            let report = safesync::history::preview(
                &Manifest::load(&previous_source)?,
                &Manifest::load(&previous_destination)?,
                &Manifest::load(&source)?,
                &Manifest::load(&destination)?,
            )?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!(
                    "Historical rename review: {:?} → {:?}",
                    report.current.source.volume.name, report.current.destination.volume.name
                );
                for warning in &report.warnings {
                    println!("{warning}");
                }
                for observation in &report.observations {
                    let target = observation
                        .current_source
                        .as_ref()
                        .map(|e| e.path())
                        .transpose()?;
                    println!(
                        "{} → {} · {:?}\n  {}",
                        safe_display(&observation.previous_source.path()?),
                        target
                            .as_deref()
                            .map(safe_display)
                            .unwrap_or_else(|| "unknown".into()),
                        observation.evidence,
                        observation.reason
                    );
                }
                println!(
                    "{} observation(s) · {} current-plan blocker(s) · {} previous-plan blocker(s). No changes applied.",
                    report.observations.len(),
                    report.current.blockers.len(),
                    report.previous_blockers.len()
                );
            }
            return Ok(report.exit_code());
        }
        Command::CatalogRefresh {
            source,
            destination,
            hash,
            rehash,
            exclude,
        } => {
            let verified = safesync::maintenance::verify_pair(&source, &destination)?;
            let (source, destination) = verified.refresh_catalogs(hash, !rehash, &exclude)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "source": source, "destination": destination,
                    "filesystem_checks": verified.checks, "media_changed": false,
                    "relationship_committed": false
                }))?
            );
        }
        Command::Catalog { root } => {
            use safesync::enrollment::{DriveLease, EnrolledRoot};
            let root = EnrolledRoot::open(&root)?;
            let role = root.inspect()?.role;
            let lease = DriveLease::acquire(root, role)?;
            let (current, inventory) = safesync::catalog::load(&lease)?
                .context("No current catalog; use catalog-refresh first")?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &serde_json::json!({ "current": current, "header": inventory.header, "files": inventory.entries.len() })
                )?
            );
        }
        Command::Plan {
            source,
            destination,
            json,
        } => {
            let preview =
                safesync::plan::preview(&Manifest::load(&source)?, &Manifest::load(&destination)?)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&preview)?);
            } else {
                println!(
                    "Historical plan preview: {:?} → {:?}",
                    preview.source.volume.name, preview.destination.volume.name
                );
                for warning in &preview.warnings {
                    println!("{warning}");
                }
                for blocker in &preview.blockers {
                    println!("BLOCKED: {}", blocker.reason);
                    for path in &blocker.paths_base64 {
                        println!(
                            "  {}",
                            safe_display(&safesync::manifest::decode_path(path)?)
                        );
                    }
                }
                for item in &preview.items {
                    println!("{:?} · {:?}", item.proposed_action, item.path_display);
                }
                let s = &preview.summary;
                println!(
                    "{} keep content · {} copy · {} replace with history · {} review · {} preserve destination",
                    s.keep_content, s.copies, s.replacements, s.review, s.preserve_destination
                );
                println!(
                    "Proposed payload: {} bytes · predecessors: {} bytes (not a free-space estimate)",
                    s.proposed_transfer_bytes, s.proposed_history_bytes
                );
                println!(
                    "{} blocker(s). Historical preview only; no changes applied.",
                    preview.blockers.len()
                );
            }
            return Ok(preview.exit_code());
        }
        Command::CheckPair {
            source,
            destination,
            verify_filesystem,
        } => {
            use safesync::enrollment::{EnrolledRoot, PairLease};
            if verify_filesystem {
                let verified = safesync::maintenance::verify_pair(&source, &destination)?;
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "source": verified.pair().source.enrollment(),
                        "destination": verified.pair().destination.enrollment(),
                        "identity_and_roles_valid": true,
                        "filesystem_checked": true,
                        "filesystem_checks": verified.checks,
                        "executable": false,
                        "locks": "held for this check only; released on exit"
                    }))?
                );
                return Ok(0);
            }
            let pair = PairLease::acquire(
                EnrolledRoot::open(&source)?,
                EnrolledRoot::open(&destination)?,
            )?;
            pair.revalidate()?;
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "source": pair.source.enrollment(),
                    "destination": pair.destination.enrollment(),
                    "identity_and_roles_valid": true,
                    "filesystem_checked": false,
                    "executable": false,
                    "locks": "held for this check only; released on exit"
                }))?
            );
        }
        Command::Enroll { root, role } => {
            let record = safesync::enrollment::EnrolledRoot::open(&root)?.enroll(role)?;
            println!("{}", serde_json::to_string_pretty(&record)?);
        }
        Command::Enrollment { root } => {
            let record = safesync::enrollment::EnrolledRoot::open(&root)?.inspect()?;
            println!("{}", serde_json::to_string_pretty(&record)?);
        }
        Command::Compare {
            source,
            destination,
            json,
        } => {
            let report = safesync::compare::compare(
                &Manifest::load(&source)?,
                &Manifest::load(&destination)?,
            )?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!(
                    "Historical comparison: {:?} → {:?}",
                    report.source.volume.name, report.destination.volume.name
                );
                for warning in &report.warnings {
                    println!("{warning}");
                }
                for row in &report.rows {
                    println!(
                        "{:?} · {:?} · {} alternate content match(es)",
                        row.status,
                        row.path_display,
                        row.content_candidates.len()
                    );
                }
                println!("{} paths compared. No media changed.", report.rows.len());
            }
        }
        Command::Scan {
            root,
            hash,
            rehash,
            output,
            save_local,
            exclude,
        } => {
            let root = root
                .canonicalize()
                .context("Cannot resolve source directory")?;
            ensure!(root.is_dir(), "Source must be a directory");
            let volume = filesystem::volume_for(&root)?;
            // Earlier scans of this volume, on the drive and in the local library.
            let mut cache = safesync::scan::HashCache::new(&volume);
            if hash && !rehash {
                cache.add_directory(&root.join(".safesync"));
                cache.add_directory(&library()?);
            }
            eprintln!(
                "Scanning {} on {:?}. {}",
                safe_display(&root),
                volume.name,
                if !hash {
                    "Metadata only; content equality will remain unknown.".to_string()
                } else if cache.is_empty() {
                    "Reading full contents for SHA-256 fingerprints.".to_string()
                } else {
                    format!(
                        "Reading only new or changed files; {} fingerprints known (--rehash reads everything).",
                        cache.len()
                    )
                }
            );
            let mut last = Instant::now();
            let inventory = safesync::scan::scan_with_reuse(
                &root,
                volume.clone(),
                hash,
                &exclude,
                Some(&cache),
                |p| {
                    if last.elapsed() >= Duration::from_secs(1) {
                        eprintln!(
                            "Scanned {} files · {} bytes {} · {} fingerprints reused",
                            p.files,
                            p.bytes,
                            if hash { "hashed" } else { "catalogued" },
                            p.reused
                        );
                        last = Instant::now();
                    }
                },
            )?;
            ensure!(
                filesystem::volume_for(&root)?.uuid == volume.uuid,
                "Volume changed during scan"
            );
            let output = if let Some(output) = output {
                output
            } else {
                let directory = root.join(".safesync");
                if let Ok(meta) = fs::symlink_metadata(&directory) {
                    ensure!(
                        meta.is_dir() && !meta.file_type().is_symlink(),
                        "Unsafe .safesync directory"
                    );
                }
                fs::create_dir_all(&directory)?;
                directory.join(format!("manifest-{}.jsonl", inventory.header.generation))
            };
            inventory.save_new(&output)?;
            if inventory.header.reused_hashes > 0 {
                eprintln!(
                    "Reused {} fingerprints from earlier scans (file ID, size and mtime unchanged).",
                    inventory.header.reused_hashes
                );
            }
            eprintln!(
                "Saved {} regular files. Skipped {} symlinks, {} special entries and {} mounted subtrees.",
                inventory.entries.len(),
                inventory.header.skipped_symlinks,
                inventory.header.skipped_special,
                inventory.header.skipped_mounts
            );
            println!("Manifest: {}", safe_display(&output));
            if save_local {
                let local = local_output()?;
                inventory.export(&local)?;
                println!("Offline snapshot: {}", safe_display(&local));
            }
        }
        Command::Export { manifest, output } => {
            let inventory = Manifest::load(&manifest)?;
            let output = match output {
                Some(path) => path,
                None => local_output()?,
            };
            inventory.export(&output)?;
            println!("Offline snapshot: {}", safe_display(&output));
        }
        Command::Lookup {
            manifest,
            name,
            file,
            json,
        } => {
            let paths = if manifest.is_empty() {
                saved_manifests()?
            } else {
                manifest
            };
            ensure!(
                !paths.is_empty(),
                "No saved manifests. Use scan --save-local or export first."
            );
            let inventories: Vec<_> = paths
                .iter()
                .map(|p| Manifest::load(p))
                .collect::<Result<_>>()?;
            let query = if let Some(name) = name.as_ref() {
                Query::Name(name)
            } else {
                Query::File(file.as_deref().context("Missing query")?)
            };
            let report = lookup::lookup(&inventories, query)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("Historical offline inventory — current presence is not checked.");
                for found in &report.matches {
                    println!(
                        "{:?} [{}] · {:?}\n  {} · scanned at Unix {} · generation {}",
                        found.volume_name,
                        found.volume_uuid,
                        found.path_display,
                        found.evidence,
                        found.scanned_unix,
                        found.generation
                    );
                }
                println!(
                    "{} · {} manifest(s) · {} unhashed same-size candidate(s)",
                    report.verdict, report.manifests_searched, report.unverified_candidates
                );
            }
            return Ok(report.exit_code());
        }
        Command::Info { manifest } => {
            let inventory = Manifest::load(&manifest)?;
            println!("{}", serde_json::to_string_pretty(&inventory.header)?);
            println!("Files: {}", inventory.entries.len());
        }
        Command::Manifests => {
            for path in saved_manifests()? {
                let inventory = Manifest::load(&path)?;
                println!(
                    "{}\n  {:?} [{}] · {} files · scanned at Unix {} · {}",
                    safe_display(&path),
                    inventory.header.volume.name,
                    inventory.header.volume.uuid,
                    inventory.entries.len(),
                    inventory.header.finished_unix,
                    if inventory.header.content_hashed {
                        "full-content hashes"
                    } else {
                        "metadata only"
                    }
                );
            }
        }
    }
    Ok(0)
}
fn main() {
    match run(Cli::parse()) {
        Ok(code) => std::process::exit(code),
        Err(error) => {
            eprintln!("safesync: {error:#}");
            std::process::exit(2);
        }
    }
}
