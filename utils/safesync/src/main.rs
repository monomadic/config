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
        } => {
            use safesync::enrollment::{EnrolledRoot, PairLease};
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
            output,
            save_local,
            exclude,
        } => {
            let root = root
                .canonicalize()
                .context("Cannot resolve source directory")?;
            ensure!(root.is_dir(), "Source must be a directory");
            let volume = filesystem::volume_for(&root)?;
            eprintln!(
                "Scanning {} on {:?}. {}",
                safe_display(&root),
                volume.name,
                if hash {
                    "Reading full contents for SHA-256 fingerprints."
                } else {
                    "Metadata only; content equality will remain unknown."
                }
            );
            let mut last = Instant::now();
            let inventory =
                safesync::scan::scan_with_exclusions(&root, volume.clone(), hash, &exclude, |p| {
                    if last.elapsed() >= Duration::from_secs(1) {
                        eprintln!(
                            "Scanned {} files · {} bytes {}",
                            p.files,
                            p.bytes,
                            if hash { "hashed" } else { "catalogued" }
                        );
                        last = Instant::now();
                    }
                })?;
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
