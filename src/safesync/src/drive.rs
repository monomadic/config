//! The sentinel: `ROOT/.safesync/drive.toml`. It is the drive's config and the
//! thing that makes a copy in the wrong direction impossible — every command
//! that writes media opens both sentinels first and checks the roles.
use crate::{
    filesystem::{self, Volume},
    manifest::Manifest,
    scan::HashCache,
};
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
};

pub const METADATA_DIR: &str = ".safesync";
const SENTINEL: &str = "drive.toml";
// Indexes are cheap to regenerate; a few generations are kept for the hash
// cache and as a fallback when the newest one is damaged.
const KEPT_GENERATIONS: usize = 3;
const DEFAULT_EXCLUDES: &[&str] = &[
    ".Trashes",
    ".TemporaryItems",
    ".Spotlight-V100",
    ".DocumentRevisions-V100",
    ".fseventsd",
    ".rclone",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// The library. Media on it is never written, renamed or removed.
    Source,
    /// A mirror of exactly one source. Only `sync` writes to it.
    Backup,
    /// A working disk that `fill` may copy onto.
    Scratch,
}

/// What `sync` does with files on the backup that the source no longer has.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Extras {
    #[default]
    Keep,
    /// Move them into `.safesync/history/` on the backup.
    History,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sentinel {
    pub role: Role,
    pub name: String,
    pub volume_uuid: String,
    /// Backup only: the one source this drive mirrors.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_uuid: Option<String>,
    /// Literal paths relative to the root, skipped by every scan.
    #[serde(default)]
    pub exclude: Vec<PathBuf>,
    #[serde(default)]
    pub extras: Extras,
}

#[derive(Debug)]
pub struct Drive {
    pub root: PathBuf,
    pub volume: Volume,
    pub sentinel: Sentinel,
}

pub fn library() -> Result<PathBuf> {
    Ok(
        PathBuf::from(std::env::var_os("HOME").context("HOME is not set")?)
            .join("Library/Application Support/safesync/manifests"),
    )
}

impl Drive {
    pub fn init(root: &Path, role: Role, source: Option<&Drive>) -> Result<Self> {
        let root = root.canonicalize().context("Cannot resolve drive root")?;
        let volume = filesystem::volume_for(&root)?;
        let directory = root.join(METADATA_DIR);
        let path = directory.join(SENTINEL);
        ensure!(
            !path.exists(),
            "{:?} already has a sentinel; edit or remove {:?} deliberately",
            root,
            path
        );
        let source_uuid = match (role, source) {
            (Role::Backup, Some(source)) => {
                ensure!(
                    source.sentinel.role == Role::Source,
                    "{:?} is not a source drive",
                    source.root
                );
                ensure!(
                    source.volume.uuid != volume.uuid,
                    "A backup must live on a different volume from its source"
                );
                Some(source.volume.uuid.clone())
            }
            (Role::Backup, None) => bail!("A backup needs --source ROOT: the one drive it mirrors"),
            (_, Some(_)) => bail!("--source only applies to --role backup"),
            (_, None) => None,
        };
        let sentinel = Sentinel {
            role,
            name: volume.name.clone(),
            volume_uuid: volume.uuid.clone(),
            source_uuid,
            exclude: DEFAULT_EXCLUDES.iter().map(PathBuf::from).collect(),
            extras: Extras::Keep,
        };
        fs::create_dir_all(&directory)?;
        let text = format!(
            "# safesync sentinel. The role decides which way media may be copied:\n\
             #   source  - never written\n\
             #   backup  - written only by `safesync sync` from source_uuid\n\
             #   scratch - written only by `safesync fill`\n\
             # volume_uuid pins this file to this disk; a copied sentinel is refused.\n\
             # extras = \"keep\" | \"history\": files on a backup its source no longer has.\n\n{}",
            toml::to_string_pretty(&sentinel)?
        );
        fs::write(&path, text)?;
        Ok(Self {
            root,
            volume,
            sentinel,
        })
    }

    pub fn open(root: &Path) -> Result<Self> {
        let root = root.canonicalize().context("Cannot resolve drive root")?;
        let path = root.join(METADATA_DIR).join(SENTINEL);
        let text = fs::read_to_string(&path).with_context(|| {
            format!("{root:?} has no sentinel; run `safesync init {root:?} --role ...` first")
        })?;
        let sentinel: Sentinel =
            toml::from_str(&text).with_context(|| format!("Invalid sentinel {path:?}"))?;
        let volume = filesystem::volume_for(&root)?;
        ensure!(
            sentinel.volume_uuid == volume.uuid,
            "Sentinel {:?} belongs to volume {} but this is {} — it was copied here; refusing",
            path,
            sentinel.volume_uuid,
            volume.uuid
        );
        Ok(Self {
            root,
            volume,
            sentinel,
        })
    }

    pub fn metadata_dir(&self) -> PathBuf {
        self.root.join(METADATA_DIR)
    }

    fn generations(&self) -> Vec<PathBuf> {
        listing(&self.metadata_dir(), |name| {
            name.starts_with("index-") && name.ends_with(".jsonl")
        })
    }

    /// The newest readable index on the drive: the source of truth for its contents.
    pub fn index(&self) -> Result<Manifest> {
        for path in self.generations().iter().rev() {
            match Manifest::load(path) {
                Ok(manifest) if manifest.header.volume.uuid == self.volume.uuid => {
                    return Ok(manifest);
                }
                Ok(_) => eprintln!("safesync: ignoring index from another volume: {path:?}"),
                Err(error) => eprintln!("safesync: ignoring damaged index {path:?}: {error:#}"),
            }
        }
        bail!(
            "{:?} has no index yet; run `safesync scan {:?}`",
            self.sentinel.name,
            self.root
        )
    }

    /// Fingerprints from this drive's earlier indexes and the local copies of them.
    pub fn hash_cache(&self) -> Result<HashCache> {
        let mut cache = HashCache::new(&self.volume);
        cache.add_directory(&self.metadata_dir());
        cache.add_directory(&library()?);
        Ok(cache)
    }

    /// Write the index onto the drive, keep a copy on this Mac for offline
    /// lookup, and drop generations beyond the last few in both places.
    pub fn publish(&self, manifest: &Manifest) -> Result<PathBuf> {
        ensure!(
            manifest.header.volume.uuid == self.volume.uuid,
            "Index belongs to another volume"
        );
        let generation = &manifest.header.generation;
        let path = self
            .metadata_dir()
            .join(format!("index-{generation}.jsonl"));
        manifest.save_new(&path)?;
        prune(self.generations());

        let library = library()?;
        fs::create_dir_all(&library)?;
        let tag = format!(".{}.", self.volume.uuid);
        let name: String = self
            .sentinel
            .name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        manifest.export(&library.join(format!("{name}{tag}{generation}.jsonl")))?;
        prune(listing(&library, |file| file.contains(&tag)));
        Ok(path)
    }
}

fn listing(directory: &Path, keep: impl Fn(&str) -> bool) -> Vec<PathBuf> {
    let mut paths: Vec<_> = fs::read_dir(directory)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|entry| entry.file_type().is_ok_and(|t| t.is_file()))
        .filter(|entry| entry.file_name().to_str().is_some_and(&keep))
        .map(|entry| entry.path())
        .collect();
    // A generation is NANOS-PID-SEQ, so its leading number orders files by age.
    paths.sort_by_key(|path| {
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        stem.rsplitn(3, '-')
            .nth(2)
            .and_then(|rest| rest.rsplit(['-', '.']).next()?.parse::<u128>().ok())
            .unwrap_or(0)
    });
    paths
}

fn prune(generations: Vec<PathBuf>) {
    let surplus = generations.len().saturating_sub(KEPT_GENERATIONS);
    for path in &generations[..surplus] {
        let _ = fs::remove_file(path);
    }
}

/// `sync` may only run from a source to the backup that names it.
pub fn check_sync(source: &Drive, backup: &Drive) -> Result<()> {
    ensure!(
        source.sentinel.role == Role::Source,
        "{:?} is a {:?} drive, not a source — wrong direction?",
        source.sentinel.name,
        source.sentinel.role
    );
    ensure!(
        backup.sentinel.role == Role::Backup,
        "{:?} is a {:?} drive, not a backup — wrong direction?",
        backup.sentinel.name,
        backup.sentinel.role
    );
    ensure!(
        backup.sentinel.source_uuid.as_deref() == Some(source.volume.uuid.as_str()),
        "{:?} backs up a different source, not {:?}",
        backup.sentinel.name,
        source.sentinel.name
    );
    Ok(())
}

/// `fill` may write anywhere except onto a source or a backup. The nearest
/// sentinel at or above the destination, on the same volume, decides.
pub fn check_fill_destination(destination: &Path) -> Result<()> {
    let destination = destination
        .canonicalize()
        .context("Destination does not exist")?;
    let device = fs::metadata(&destination)?.dev();
    for ancestor in destination.ancestors() {
        if fs::metadata(ancestor)?.dev() != device {
            break;
        }
        if ancestor.join(METADATA_DIR).join(SENTINEL).exists() {
            let drive = Drive::open(ancestor)?;
            ensure!(
                drive.sentinel.role == Role::Scratch,
                "{:?} is a {:?} drive; fill only writes to scratch disks",
                drive.sentinel.name,
                drive.sentinel.role
            );
            break;
        }
    }
    Ok(())
}
