use anyhow::{Context, Result, ensure};
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::Shell;
use safesync::{
    drive::{self, Drive, Role},
    drives::Inventory,
    drives_ui::{self, Action},
    engine::{self, FillOptions, SyncOptions},
    lookup::{self, Query},
    manifest::Manifest,
    scan::Hashing,
    ui,
};
use std::{
    ffi::OsString,
    fs,
    io::{IsTerminal, Read},
    path::{Path, PathBuf},
};

#[derive(Parser)]
#[command(
    version,
    about = "Indexed one-way media sync between sentinel-marked drives."
)]
struct Cli {
    /// Hide icons in the interactive drives screen.
    #[arg(long, global = true)]
    no_icons: bool,
    /// With no subcommand: the drives screen.
    #[command(subcommand)]
    command: Option<Command>,
}
#[derive(Subcommand)]
enum Command {
    /// Every disk on this Mac with its role and index state; assign roles, start
    /// scans, and search every saved index. The default when no subcommand is given.
    Drives,
    /// Write a drive's sentinel: its role, and for a backup, the source it mirrors.
    Init {
        root: PathBuf,
        #[arg(long, value_enum)]
        role: Role,
        /// Backup only: the source drive this one mirrors.
        #[arg(long)]
        source: Option<PathBuf>,
    },
    /// Show a drive's sentinel and current index.
    Show { root: PathBuf },
    /// Index a drive: publishes to ROOT/.safesync and keeps a copy on this Mac.
    Scan {
        root: PathBuf,
        /// Read every file that has no fingerprint yet (new or changed files only).
        #[arg(long)]
        hash: bool,
        /// Read every file again, ignoring known fingerprints.
        #[arg(long, requires = "hash")]
        rehash: bool,
    },
    /// Copy what the source has and the backup lacks. Never writes to the source.
    Sync {
        source: PathBuf,
        backup: PathBuf,
        /// Read back every copied file and compare fingerprints.
        #[arg(long)]
        verify: bool,
        /// Fingerprint new files while scanning instead of trusting size and mtime.
        #[arg(long)]
        hash: bool,
        #[arg(long, requires = "hash")]
        rehash: bool,
        /// Skip the review screen.
        #[arg(long, short)]
        yes: bool,
        /// Extra literal paths (relative to the roots) to leave out this time.
        #[arg(long)]
        exclude: Vec<PathBuf>,
    },
    /// Copy from one or more library drives onto a scratch disk, reading from
    /// each drive in parallel. Selection from arguments or NUL/newline-separated stdin.
    Fill {
        #[arg(long, required = true)]
        from: Vec<PathBuf>,
        #[arg(long)]
        to: PathBuf,
        #[arg(long)]
        verify: bool,
        #[arg(long, short)]
        yes: bool,
        /// Files or folders to copy; omit for everything.
        select: Vec<PathBuf>,
        /// Read the selection from stdin.
        #[arg(long)]
        stdin: bool,
    },
    /// Search saved indexes while the drives are unplugged.
    Lookup {
        #[arg(long)]
        manifest: Vec<PathBuf>,
        /// Exact basename.
        #[arg(long, conflicts_with = "file", required_unless_present = "file")]
        name: Option<OsString>,
        /// Fingerprint this file and find the same content under any name.
        #[arg(long, conflicts_with = "name", required_unless_present = "name")]
        file: Option<PathBuf>,
        #[arg(long)]
        json: bool,
    },
    /// Compare two indexes without touching either drive.
    Compare {
        source: PathBuf,
        destination: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// Header and file count of an index file.
    Info { manifest: PathBuf },
    /// List the indexes saved on this Mac.
    Manifests,
    /// Print a shell completion script to stdout.
    Completions { shell: Shell },
}

fn saved_manifests() -> Result<Vec<PathBuf>> {
    let directory = drive::library()?;
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

/// The drives screen, handing off to scan/sync and back until the user quits.
fn drives(icons: bool) -> Result<i32> {
    if !(std::io::stdout().is_terminal() && std::io::stdin().is_terminal()) {
        drives_ui::print(&Inventory::load());
        return Ok(0);
    }
    let mut focus: Option<PathBuf> = None;
    loop {
        match drives_ui::run(focus.as_deref(), icons)? {
            Action::Quit => return Ok(0),
            Action::Scan { root, hash } => {
                focus = Some(root.clone());
                let hashing = if hash {
                    Hashing::Missing
                } else {
                    Hashing::Known
                };
                ui::run("scan", true, move |control| {
                    engine::index_drive(root, hashing, false, control)
                })?;
            }
            Action::Sync { source, backup } => {
                focus = Some(backup.clone());
                let options = SyncOptions {
                    source,
                    backup,
                    hashing: Hashing::Known,
                    rehash: false,
                    verify: false,
                    exclude: Vec::new(),
                };
                ui::run("sync", false, move |control| engine::sync(options, control))?;
            }
        }
    }
}

fn run(cli: Cli) -> Result<i32> {
    let Some(command) = cli.command else {
        return drives(!cli.no_icons);
    };
    match command {
        Command::Drives => return drives(!cli.no_icons),
        Command::Init { root, role, source } => {
            let source = source.as_deref().map(Drive::open).transpose()?;
            let drive = Drive::init(&root, role, source.as_ref())?;
            println!(
                "{:?} is now a {:?} drive. Sentinel: {}",
                drive.sentinel.name,
                drive.sentinel.role,
                safe_display(&drive.metadata_dir().join("drive.toml"))
            );
            println!("Next: safesync scan {}", safe_display(&drive.root));
        }
        Command::Show { root } => {
            let drive = Drive::open(&root)?;
            println!("{}", toml::to_string_pretty(&drive.sentinel)?);
            match drive.index() {
                Ok(index) => println!(
                    "Index: {} files, {} · scanned {} · {}",
                    index.entries.len(),
                    engine::human(index.entries.iter().map(|e| e.stamp.size).sum()),
                    index.header.finished_unix,
                    if index.header.content_hashed {
                        "all fingerprinted"
                    } else {
                        "size and mtime only"
                    }
                ),
                Err(error) => println!("{error:#}"),
            }
        }
        Command::Scan { root, hash, rehash } => {
            let hashing = if hash {
                Hashing::Missing
            } else {
                Hashing::Known
            };
            return ui::run("scan", true, move |control| {
                engine::index_drive(root, hashing, rehash, control)
            });
        }
        Command::Sync {
            source,
            backup,
            verify,
            hash,
            rehash,
            yes,
            exclude,
        } => {
            let options = SyncOptions {
                source,
                backup,
                hashing: if hash {
                    Hashing::Missing
                } else {
                    Hashing::Known
                },
                rehash,
                verify,
                exclude,
            };
            return ui::run("sync", yes, move |control| engine::sync(options, control));
        }
        Command::Fill {
            from,
            to,
            verify,
            yes,
            mut select,
            stdin,
        } => {
            if stdin {
                let mut input = Vec::new();
                std::io::stdin().read_to_end(&mut input)?;
                select.extend(engine::parse_selection(&input));
                ensure!(!select.is_empty(), "Nothing selected on stdin");
            }
            let options = FillOptions {
                from,
                destination: to,
                select,
                verify,
            };
            return ui::run("fill", yes, move |control| engine::fill(options, control));
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
                "No saved indexes yet; scan a drive first."
            );
            let inventories: Vec<_> = paths
                .iter()
                .map(|p| Manifest::load(p))
                .collect::<Result<_>>()?;
            let query = match &name {
                Some(name) => Query::Name(name),
                None => Query::File(file.as_deref().context("Missing query")?),
            };
            let report = lookup::lookup(&inventories, query)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                for found in &report.matches {
                    println!(
                        "{:?} · {:?}\n  {} · scanned at Unix {}",
                        found.volume_name, found.path_display, found.evidence, found.scanned_unix
                    );
                }
                println!(
                    "{} · {} index(es) · {} unhashed same-size candidate(s)",
                    report.verdict, report.manifests_searched, report.unverified_candidates
                );
            }
            return Ok(report.exit_code());
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
                for warning in &report.warnings {
                    println!("{warning}");
                }
                for row in &report.rows {
                    println!("{:?} · {:?}", row.status, row.path_display);
                }
                println!("{} paths compared.", report.rows.len());
            }
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
                    "{}\n  {:?} · {} files · scanned at Unix {} · {}",
                    safe_display(&path),
                    inventory.header.volume.name,
                    inventory.entries.len(),
                    inventory.header.finished_unix,
                    if inventory.header.content_hashed {
                        "fingerprinted"
                    } else {
                        "size and mtime only"
                    }
                );
            }
        }
        Command::Completions { shell } => {
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                "safesync",
                &mut std::io::stdout(),
            );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icons_default_on_and_can_be_disabled_before_or_after_drives() {
        assert!(!Cli::try_parse_from(["safesync"]).unwrap().no_icons);
        for args in [
            vec!["safesync", "--no-icons"],
            vec!["safesync", "--no-icons", "drives"],
            vec!["safesync", "drives", "--no-icons"],
        ] {
            assert!(Cli::try_parse_from(args).unwrap().no_icons);
        }
    }
}
