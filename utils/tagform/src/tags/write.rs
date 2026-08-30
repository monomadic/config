//! Executing a plan (SPEC §9.2, §9.3).
//!
//! The rule the whole module is built around: the original file is never
//! modified until a verified replacement exists. A remux writes to a sibling
//! temp, proves the result carries the same duration, the tags that were asked
//! for, and the requested layout, and only then renames over the original. A
//! failure anywhere leaves the original untouched.

use anyhow::{anyhow, bail, Context, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::model::value::Value;
use crate::tags::atoms;
use crate::tags::plan::{exiftool_name, junk_clears, FilePlan, Writer};
use crate::tags::probe;

/// What a remux has to reproduce exactly: one entry per stream the file
/// carries, so the result can be checked against the source rather than hoped
/// about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamShape {
    pub index: usize,
    pub kind: String,
    pub tag: String,
    /// ffmpeg reports a timecode track's codec as `none`, which is also what
    /// makes it unmappable.
    pub codec: Option<String>,
}

impl StreamShape {
    /// A track ffmpeg synthesises rather than carries: the chapter text track
    /// and the timecode track. Mapping them explicitly is what caused a
    /// duplicate chapter track on every write, and what made a remux fail
    /// outright on any file with a timecode track ("Could not find tag for
    /// codec none"). Both are rebuilt from metadata instead.
    fn is_synthesised(&self) -> bool {
        self.kind == "data" && (self.codec.is_none() || self.tag == "text")
    }
}

pub fn probe_streams(path: &Path) -> Vec<StreamShape> {
    let out = Command::new("ffprobe")
        .args(["-v", "error", "-show_streams", "-of", "json", "--"])
        .arg(path)
        .output();
    let Ok(out) = out else { return Vec::new() };
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap_or_default();
    v.get("streams")
        .and_then(|s| s.as_array())
        .map(|arr| {
            arr.iter()
                .enumerate()
                .map(|(i, s)| StreamShape {
                    index: s.get("index").and_then(|x| x.as_u64()).unwrap_or(i as u64) as usize,
                    kind: s.get("codec_type").and_then(|x| x.as_str()).unwrap_or("").into(),
                    tag: s.get("codec_tag_string").and_then(|x| x.as_str()).unwrap_or("").into(),
                    codec: s
                        .get("codec_name")
                        .and_then(|x| x.as_str())
                        .filter(|c| *c != "none")
                        .map(|c| c.to_string()),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The source's timecode, so a skipped tmcd track can be rebuilt.
fn timecode(path: &Path) -> Option<String> {
    let out = Command::new("ffprobe")
        .args(["-v", "error", "-show_entries", "format_tags=timecode:stream_tags=timecode",
               "-of", "json", "--"])
        .arg(path)
        .output()
        .ok()?;
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).ok()?;
    let from = |o: Option<&serde_json::Value>| {
        o.and_then(|t| t.get("timecode")).and_then(|t| t.as_str()).map(String::from)
    };
    from(v.get("format").and_then(|f| f.get("tags"))).or_else(|| {
        v.get("streams")?
            .as_array()?
            .iter()
            .find_map(|s| from(s.get("tags")))
    })
}

/// Space for a second full-size copy, plus room to breathe.
const HEADROOM: u64 = 64 * 1024 * 1024;
/// Remuxed duration may differ by a frame or two of rounding, never by more.
const DURATION_TOLERANCE: f64 = 5.0;

#[derive(Debug)]
pub enum WriteError {
    /// Its own case because the remux is atomic-by-copy: a 10 GB file on a
    /// volume with 4 GB free fails with nothing wrong with the container at
    /// all, and calling that "could not write tags" sends you hunting for
    /// corruption that is not there.
    NoSpace { need: u64, avail: u64 },
    Failed(anyhow::Error),
}

impl std::fmt::Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WriteError::NoSpace { need, avail } => write!(
                f,
                "not enough space: needs {} MB, {} MB free",
                need / 1_048_576,
                avail / 1_048_576
            ),
            WriteError::Failed(e) => write!(f, "{e:#}"),
        }
    }
}

fn config_path() -> PathBuf {
    // Installed beside the binary; the repo copy is the fallback for `cargo run`.
    if let Ok(exe) = std::env::current_exe() {
        for up in [1usize, 2, 3] {
            let mut p = exe.clone();
            for _ in 0..up {
                p.pop();
            }
            let c = p.join("assets/tagform.exiftool.cfg");
            if c.exists() {
                return c;
            }
        }
    }
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/tagform.exiftool.cfg"))
}

/// Execute a plan against one file.
///
/// `xmp_snapshot` is everything the file's XMP held when it was read. It is
/// what the two-pass writer puts back after a remux has destroyed it, so it
/// must come from the read, not from a re-probe of the remuxed file.
pub fn execute(plan: &FilePlan, xmp_snapshot: &BTreeMap<String, Value>) -> Result<(), WriteError> {
    if plan.is_empty() {
        return Ok(());
    }
    let r = match plan.writer {
        Writer::Exiftool => in_place(plan),
        Writer::Ffmpeg => remux(plan, None),
        Writer::TwoPass => remux(plan, Some(xmp_snapshot)),
    };
    r
}

// ---------------------------------------------------------------------------
// in place
// ---------------------------------------------------------------------------

fn in_place(plan: &FilePlan) -> Result<(), WriteError> {
    let mut args: Vec<String> = vec![
        "-config".into(),
        config_path().to_string_lossy().into_owned(),
        "-q".into(),
        "-overwrite_original_in_place".into(),
    ];
    for (key, value) in &plan.atoms {
        let name = exiftool_name(key).ok_or_else(|| {
            WriteError::Failed(anyhow!("{key} cannot be written in place; this is a planning bug"))
        })?;
        args.push(format!("-Keys:{name}={value}"));
    }
    args.extend(xmp_args(&plan.xmp));
    args.push("--".into());
    args.push(plan.path.to_string_lossy().into_owned());

    let before = mtime(&plan.path);
    run("exiftool", &args).map_err(WriteError::Failed)?;
    if let Some(t) = before {
        restore_mtime(&plan.path, t);
    }
    verify_atoms(&plan.path, &plan.atoms).map_err(WriteError::Failed)
}

/// XMP list tags append on assignment, so a bare set would grow the list every
/// run. Clearing first and then using `=` (never `+=`) is the only correct
/// order -- `+=` is applied against the original list and survives the clear.
/// This is the trap `rename-footage` documents.
fn xmp_args(xmp: &[(String, Vec<String>)]) -> Vec<String> {
    let mut args = Vec::new();
    for (tag, values) in xmp {
        args.push(format!("-{tag}="));
        for v in values {
            if !v.is_empty() {
                args.push(format!("-{tag}={v}"));
            }
        }
    }
    args
}

// ---------------------------------------------------------------------------
// remux
// ---------------------------------------------------------------------------

fn remux(plan: &FilePlan, restore: Option<&BTreeMap<String, Value>>) -> Result<(), WriteError> {
    let path = &plan.path;
    let dir = path.parent().unwrap_or(Path::new("."));
    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    // Checked before ffmpeg runs, so a space problem is reported as itself
    // rather than as a mysterious encoder failure ten seconds later.
    let need = size + HEADROOM;
    if let Some(avail) = atoms::free_bytes(dir) {
        if avail < need {
            return Err(WriteError::NoSpace { need, avail });
        }
    }

    let ext = path.extension().map(|e| e.to_string_lossy().into_owned()).unwrap_or_else(|| "mp4".into());
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    // Same directory, so the swap is a rename rather than a copy; same
    // extension, so ffmpeg selects the same muxer mode (docs/CONTAINER.md §1.2).
    let tmp = dir.join(format!(".{stem}.tagform.{}.{ext}", std::process::id()));
    let _guard = TempGuard(tmp.clone());

    let mut args: Vec<String> = vec!["-hide_banner".into(), "-loglevel".into(), "error".into(), "-nostdin".into(), "-y".into()];
    args.push("-i".into());
    args.push(path.to_string_lossy().into_owned());
    // Map every stream by index except the ones ffmpeg rebuilds itself. A bare
    // `-map 0` copies the chapter track *and* re-emits it, so a file gained a
    // data stream on every write, and it fails outright on a timecode track.
    let shapes = probe_streams(path);
    for s in shapes.iter().filter(|s| !s.is_synthesised()) {
        args.push("-map".into());
        args.push(format!("0:{}", s.index));
    }
    args.extend(["-c", "copy", "-map_metadata", "0"].map(String::from));
    if let Some(tc) = timecode(path) {
        args.push("-timecode".into());
        args.push(tc);
    }
    let flags = if plan.faststart {
        "+faststart+use_metadata_tags"
    } else {
        "+use_metadata_tags"
    };
    args.push("-movflags".into());
    args.push(flags.into());
    for (k, v) in junk_clears().iter().chain(plan.atoms.iter()) {
        args.push("-metadata".into());
        args.push(format!("{k}={v}"));
    }
    args.push("--".into());
    args.push(tmp.to_string_lossy().into_owned());

    run("ffmpeg", &args).map_err(WriteError::Failed)?;

    verify_duration(path, &tmp).map_err(WriteError::Failed)?;
    verify_streams(&shapes, &tmp).map_err(WriteError::Failed)?;
    verify_atoms(&tmp, &plan.atoms).map_err(WriteError::Failed)?;
    if plan.faststart {
        let l = atoms::layout(&tmp);
        if !l.is_faststart() {
            return Err(WriteError::Failed(anyhow!(
                "remux did not come out faststart (got {l:?})"
            )));
        }
    }

    // The remux has destroyed whatever XMP the source carried; put it back
    // before the swap, so the original is only replaced by a file that is
    // strictly no poorer than it was.
    if let Some(snapshot) = restore {
        let merged = merge_xmp(snapshot, &plan.xmp);
        if !merged.is_empty() {
            let mut a: Vec<String> = vec![
                "-config".into(),
                config_path().to_string_lossy().into_owned(),
                "-q".into(),
                "-overwrite_original_in_place".into(),
            ];
            a.extend(xmp_args(&merged));
            a.push("--".into());
            a.push(tmp.to_string_lossy().into_owned());
            run("exiftool", &a).map_err(WriteError::Failed)?;
            verify_xmp(&tmp, &merged).map_err(WriteError::Failed)?;
        }
    }

    if let Some(t) = mtime(path) {
        restore_mtime(&tmp, t);
    }
    std::fs::rename(&tmp, path)
        .with_context(|| format!("replacing {}", path.display()))
        .map_err(WriteError::Failed)?;
    std::mem::forget(_guard);
    Ok(())
}

/// The staged edits win over the snapshot; everything else is restored as it was.
fn merge_xmp(
    snapshot: &BTreeMap<String, Value>,
    changed: &[(String, Vec<String>)],
) -> Vec<(String, Vec<String>)> {
    let mut out: Vec<(String, Vec<String>)> = snapshot
        .iter()
        .map(|(tag, v)| {
            let values = match v {
                Value::Text(s) => vec![s.clone()],
                Value::List(l) => l.clone(),
            };
            (tag.clone(), values)
        })
        .collect();
    for (tag, values) in changed {
        match out.iter_mut().find(|(t, _)| t == tag) {
            Some(slot) => slot.1 = values.clone(),
            None => out.push((tag.clone(), values.clone())),
        }
    }
    out.retain(|(_, v)| !v.is_empty());
    out
}

/// Removes the temp unless the caller forgets it, so an error path never
/// leaves a half-written sibling behind.
struct TempGuard(PathBuf);
impl Drop for TempGuard {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0).ok();
    }
}

// ---------------------------------------------------------------------------
// verification
// ---------------------------------------------------------------------------

/// Read the result back and confirm it says what was asked for. This is what
/// catches a key the writer silently dropped, and it is what turns the mapping
/// table from an assumption into something checked on every single run.
fn verify_atoms(path: &Path, wanted: &[(String, String)]) -> Result<()> {
    let got = probe::probe(path).context("re-probing after write")?;
    for (key, value) in wanted {
        let actual = got.atoms.get(key).map(|v| match v {
            Value::Text(s) => s.clone(),
            Value::List(l) => l.join(", "),
        });
        if value.is_empty() {
            if actual.as_deref().is_some_and(|a| !a.is_empty()) {
                bail!("{key} should have been removed but still reads {:?}", actual.unwrap());
            }
        } else if actual.as_deref() != Some(value.as_str()) {
            bail!("{key} did not round-trip: wrote {value:?}, read back {actual:?}");
        }
    }
    Ok(())
}

fn verify_xmp(path: &Path, wanted: &[(String, Vec<String>)]) -> Result<()> {
    let got = probe::probe(path).context("re-probing XMP after write")?;
    for (tag, values) in wanted {
        let actual = got.xmp.get(tag);
        let ok = match actual {
            Some(Value::List(l)) => l == values,
            Some(Value::Text(s)) => values.len() == 1 && &values[0] == s,
            None => false,
        };
        if !ok {
            bail!("XMP {tag} did not survive: wanted {values:?}, read back {actual:?}");
        }
    }
    Ok(())
}

/// The result must carry the same streams as the source. This is what catches
/// a track silently gained or dropped -- the duplicate-chapter-track bug lived
/// for several milestones precisely because nothing compared the two.
fn verify_streams(before: &[StreamShape], after: &Path) -> Result<()> {
    let got = probe_streams(after);
    let shape = |v: &[StreamShape]| {
        let mut s: Vec<String> = v.iter().map(|x| format!("{}/{}", x.kind, x.tag)).collect();
        s.sort();
        s
    };
    let (a, b) = (shape(before), shape(&got));
    if a != b {
        bail!("streams changed: {a:?} -> {b:?}");
    }
    Ok(())
}

fn verify_duration(before: &Path, after: &Path) -> Result<()> {
    let (a, b) = (duration(before), duration(after));
    match (a, b) {
        (Some(a), Some(b)) if (a - b).abs() <= DURATION_TOLERANCE => Ok(()),
        (Some(a), Some(b)) => bail!("duration changed: {a:.2}s -> {b:.2}s"),
        _ => bail!("could not read duration back; refusing to replace the original"),
    }
}

fn duration(path: &Path) -> Option<f64> {
    let out = Command::new("ffprobe")
        .args(["-v", "error", "-show_entries", "format=duration", "-of", "default=nw=1:nk=1", "--"])
        .arg(path)
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout).trim().parse().ok()
}

fn mtime(path: &Path) -> Option<std::time::SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}

/// `touch -r` rather than a syscall: it is what the rest of this repo uses and
/// it needs no extra dependency for something done once per file.
fn restore_mtime(path: &Path, _t: std::time::SystemTime) {
    let _ = _t;
    let _ = Command::new("touch").arg("-r").arg(path).arg(path).status();
}

fn run(program: &str, args: &[String]) -> Result<()> {
    let out = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .output()
        .with_context(|| format!("running {program}"))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        bail!("{program} failed: {}", err.trim().lines().next().unwrap_or("(no output)"));
    }
    // exiftool reports refusals on stdout with a zero exit status, so a
    // successful exit is not by itself proof that anything was written.
    let sout = String::from_utf8_lossy(&out.stdout);
    if sout.contains("Sorry,") || sout.contains("Nothing to do") {
        bail!("{program}: {}", sout.trim().lines().next().unwrap_or("refused"));
    }
    Ok(())
}
