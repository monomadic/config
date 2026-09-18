//! A whole-clip encode through the Topaz app's neuroserver process — the
//! backend that serves Starlight Precise and the other generative models the
//! ffmpeg tvai_up filter cannot reach. It is the full-clip counterpart of the
//! still path in topaz-preview-frame. topaz-encode stays ffmpeg-only; a
//! neuroserver encode has none of its plumbing (no fragments, no live pause)
//! because neuroserver owns the whole pipeline from decode to encode.
//!
//! One neuroserver --once job, run the way the Topaz app runs an export:
//! neuroserver decodes, enhances and encodes the video itself
//! (--ffmpeg-encoding carries the codec arguments), then a second ffmpeg pass
//! muxes the source audio and the videoai metadata tag onto the result, like
//! the app's cleanup pass.
//!
//! Interrupted encodes. The video is written as fragmented MP4/MOV (added to
//! the codec arguments unless they already set -movflags), so what neuroserver
//! had written survives a crash or a kill and can be read back — which is also
//! what lets the TUI show a live frame. Starlight works in chunks (about 100
//! frames at 4K) and writes a chunk only when it is finished, so at most one
//! chunk of work is lost. Resuming keeps the surviving frames as a part file,
//! renders only the frames after them, and joins the parts losslessly before
//! the audio mux. neuroserver's own --resume flag is not used: it is accepted
//! for Starlight Precise 2.6 and then ignored, re-rendering from frame zero
//! over the partial. An interrupted encode is never discarded without being
//! asked to: silently deleting hours of rendering is not a default.
//!
//! Used by the TUI in-process, and as a CLI via `neuroserver-select-preset
//! encode …` or the `neuroserver-encode` symlink the installer creates.

use crate::catalog::{self, shell_quote};
use crate::logtail::{parse_line, LogEvent};
use crate::probe;
use anyhow::{anyhow, bail, Context, Result};
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub const FRAG_FLAGS: &str = "-movflags frag_keyframe+empty_moov+delay_moov";

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Mode {
    /// Refuse if an interrupted encode of this output exists.
    #[default]
    Fresh,
    /// Keep what an interrupted encode wrote and render only the rest.
    Resume,
    /// Discard an interrupted encode and start over.
    Restart,
}

/// What the caller asks for; `plan` resolves the defaults.
#[derive(Clone, Debug, Default)]
pub struct Options {
    pub input: PathBuf,
    pub model: String,
    pub store: String,
    pub params: Option<String>,
    pub size: Option<(u32, u32)>,
    pub output_profile: Option<String>,
    pub video_args: Option<String>,
    pub ext: Option<String>,
    pub preset_name: Option<String>,
    pub metadata: Option<String>,
    pub output: Option<PathBuf>,
    pub start_frame: Option<u64>,
    pub end_frame: Option<u64>,
    pub nice: Option<i32>,
    pub log_file: Option<PathBuf>,
    pub mode: Mode,
}

pub enum Event {
    /// A step of our own: salvaging, joining, muxing.
    Stage(String),
    /// Something worth reading that does not stop the encode.
    Warn(String),
    /// neuroserver's JSON progress line.
    Progress { pct: Option<u32>, frame: Option<u64>, message: Option<String> },
}

/// Everything resolved: paths, sizes, frame range, the Topaz app's binaries.
pub struct Plan {
    pub input: PathBuf,
    pub output: PathBuf,
    /// neuroserver's video-only result, staged beside the output so the final
    /// rename stays on one volume.
    pub video_only: PathBuf,
    pub log: PathBuf,
    pub ext: String,
    pub model: String,
    pub store: String,
    pub params: Option<String>,
    pub filters: String,
    pub video_args: String,
    /// `key=value` for the mux pass.
    pub metadata: String,
    pub out_size: (u32, u32),
    pub start_frame: u64,
    pub end_frame: u64,
    pub nice: Option<i32>,
    pub mode: Mode,
    ns_dir: PathBuf,
    ns_bin: PathBuf,
    ns_store: PathBuf,
    models_dir: PathBuf,
    ffmpeg: PathBuf,
}

/// What an earlier, interrupted encode of the same output left behind.
#[derive(Clone, Copy, Debug)]
pub struct Interrupted {
    pub frames: u64,
    pub parts: usize,
}

// ------------------------------------------------------------------ paths

/// "<dir>/<stem minus trailing [tags]> [Topaz - <preset>].<ext>" — topaz-encode's
/// tag convention, so outputs of the two sit together.
pub fn output_path(input: &Path, preset_name: &str, ext: &str) -> PathBuf {
    let stem = input.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let mut clean = stem.trim_end().to_string();
    while clean.ends_with(']') {
        match clean.rfind('[') {
            Some(i) => clean = clean[..i].trim_end().to_string(),
            None => break,
        }
    }
    if clean.trim().is_empty() {
        clean = stem;
    }
    let safe: String = preset_name.chars().map(|c| if c == '/' || c == ':' { '-' } else { c }).collect();
    input
        .parent()
        .unwrap_or(Path::new("."))
        .join(format!("{clean} [Topaz - {safe}].{ext}"))
}

/// "<output minus extension>.<suffix>".
fn beside(output: &Path, suffix: &str) -> PathBuf {
    let stem = output.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    output.with_file_name(format!("{stem}.{suffix}"))
}

pub fn video_only_path(output: &Path, ext: &str) -> PathBuf {
    beside(output, &format!("ns-video.{ext}"))
}

pub fn default_log(output: &Path) -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let stem = output.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_else(|| "neuroserver".into());
    PathBuf::from(home).join("Library/Logs/topaz-batch").join(format!("{stem}.log"))
}

fn part_path(output: &Path, ext: &str, n: usize) -> PathBuf {
    beside(output, &format!("ns-part-{n:03}.{ext}"))
}

/// "<output stem>.ns-part-NNN.<ext>": frames salvaged from earlier interrupted
/// runs, in order.
fn part_files(output: &Path, ext: &str) -> Vec<PathBuf> {
    let Some(dir) = output.parent() else { return Vec::new() };
    let stem = output.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    let prefix = format!("{stem}.ns-part-");
    let suffix = format!(".{ext}");
    let mut parts: Vec<PathBuf> = fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| {
                    let name = p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
                    name.strip_prefix(&prefix)
                        .and_then(|rest| rest.strip_suffix(&suffix))
                        .is_some_and(|n| n.len() == 3 && n.bytes().all(|b| b.is_ascii_digit()))
                })
                .collect()
        })
        .unwrap_or_default();
    parts.sort();
    parts
}

/// Frames that survive from an interrupted encode of `output`: the fragmented
/// partial plus any part files earlier resumes salvaged.
pub fn interrupted(output: &Path, ext: &str) -> Option<Interrupted> {
    let parts = part_files(output, ext);
    let partial = video_only_path(output, ext);
    let leftover = if partial.is_file() { probe::count_frames(&partial) } else { 0 };
    if leftover == 0 && parts.is_empty() {
        return None;
    }
    let frames = leftover + parts.iter().map(|p| probe::count_frames(p)).sum::<u64>();
    Some(Interrupted { frames, parts: parts.len() })
}

// ------------------------------------------------------------------ plan

pub fn plan(o: &Options) -> Result<Plan> {
    if !o.input.is_file() {
        bail!("not a file: {}", o.input.display());
    }
    let input = o.input.canonicalize()?;
    if o.model.is_empty() {
        bail!("--model is required");
    }
    if o.store.is_empty() {
        bail!("--store is required");
    }

    // Output profile: one of the catalog's output/ presets, or explicit arguments.
    let (mut ext, mut video_args) = (o.ext.clone(), o.video_args.clone());
    if let Some(slug) = &o.output_profile {
        let p = catalog::output_profiles()?
            .into_iter()
            .find(|p| p.slug == *slug)
            .ok_or_else(|| anyhow!("unknown output profile: {slug}"))?;
        ext.get_or_insert(p.ext);
        video_args.get_or_insert(p.video_args);
    }
    let mut video_args = video_args
        .filter(|a| !a.trim().is_empty())
        .ok_or_else(|| anyhow!("--output-profile or --video-args is required"))?;
    let ext = ext.filter(|e| !e.is_empty()).unwrap_or_else(|| "mp4".into());
    if !video_args.contains("-movflags") && matches!(ext.as_str(), "mp4" | "mov" | "m4v") {
        video_args = format!("{video_args} {FRAG_FLAGS}");
    }
    let preset_name = o.preset_name.clone().unwrap_or_else(|| o.model.clone());

    let app = probe::topaz_app().ok_or_else(|| {
        anyhow!("Topaz app not found. Set TOPAZ_APP, or install /Applications/Topaz Video.app")
    })?;
    let macos = app.join("Contents/MacOS");
    let models_dir = app.join("Contents/Resources/models");
    let ns_dir = macos.join("neuroserver");
    let ns_bin = ns_dir.join("neuroserver");
    let ns_store = models_dir.join("models");
    if !ns_bin.is_file() {
        bail!("neuroserver not found: {}", ns_bin.display());
    }
    let has_weights = fs::read_dir(ns_store.join(&o.store))
        .map(|rd| rd.filter_map(|e| e.ok()).any(|e| e.file_name().to_string_lossy().starts_with("blob.")))
        .unwrap_or(false);
    if !has_weights {
        bail!(
            "neuroserver model {} has no weights here: {}\n\
             A neuroserver model downloads only when the Topaz app runs it once — run any \
             short export with this model in the app, then retry.",
            o.model,
            ns_store.join(&o.store).display()
        );
    }

    let profile = probe::probe(&input)?;
    let start_frame = o.start_frame.unwrap_or(0);
    let end_frame = o.end_frame.unwrap_or_else(|| profile.total_frames());
    let out_size = match o.size {
        Some(s) => s,
        None if profile.width > 0 && profile.height > 0 => (profile.width, profile.height),
        None => bail!("cannot read the source size — pass --size WxH"),
    };

    let output = o.output.clone().unwrap_or_else(|| output_path(&input, &preset_name, &ext));
    let log = o.log_file.clone().unwrap_or_else(|| default_log(&output));
    let mut metadata = o.metadata.clone().filter(|m| !m.is_empty()).unwrap_or_else(|| {
        format!("videoai=Enhanced using {}. Changed resolution to {}x{}", o.model, out_size.0, out_size.1)
    });
    if !metadata.contains('=') {
        metadata = format!("videoai={metadata}");
    }

    let mut filters = format!("[{{\"model\": \"{}\"}}]", o.model);
    if let Some(params) = o.params.as_deref().map(str::trim).filter(|p| !p.is_empty()) {
        let inner = params
            .strip_prefix('{')
            .and_then(|p| p.strip_suffix('}'))
            .ok_or_else(|| anyhow!("--params must be a JSON object: {params}"))?
            .trim();
        if !inner.is_empty() {
            filters = format!("[{{\"model\": \"{}\", {inner}}}]", o.model);
        }
    }

    Ok(Plan {
        video_only: video_only_path(&output, &ext),
        input,
        output,
        log,
        ext,
        model: o.model.clone(),
        store: o.store.clone(),
        params: o.params.clone(),
        filters,
        video_args,
        metadata,
        out_size,
        start_frame,
        end_frame,
        nice: o.nice,
        mode: o.mode,
        ns_dir,
        ns_bin,
        ns_store,
        models_dir,
        ffmpeg: macos.join("ffmpeg"),
    })
}

impl Plan {
    fn ns_command(&self, start: u64) -> Command {
        let mut cmd = match self.nice {
            Some(n) => {
                let mut c = Command::new("nice");
                c.arg("-n").arg(n.to_string()).arg(&self.ns_bin);
                c
            }
            None => Command::new(&self.ns_bin),
        };
        cmd.arg("--once")
            .arg("--input-path").arg(&self.input)
            .arg("--output-path").arg(&self.video_only)
            .arg("--start-frame-idx").arg(start.to_string())
            .arg("--end-frame-idx").arg(self.end_frame.to_string())
            .arg("--ffmpeg-preproc-filters").arg("")
            .arg("--filters").arg(&self.filters)
            .arg("--output-width").arg(self.out_size.0.to_string())
            .arg("--output-height").arg(self.out_size.1.to_string())
            .arg("--upscale-factor").arg("1")
            .arg("--ffmpeg-encoding").arg(&self.video_args)
            .arg("--videoai-metadata").arg(self.metadata.split_once('=').map_or("", |(_, v)| v))
            .arg("--device").arg("0")
            // neuroserver resolves its modules relative to cwd and finds
            // weights only via TOPAZ_MODEL_STORE.
            .current_dir(&self.ns_dir)
            .env("TOPAZ_MODEL_STORE", &self.ns_store)
            .env("TVAI_MODEL_DIR", &self.models_dir)
            .env("TVAI_MODEL_DATA_DIR", &self.models_dir)
            .env("LC_NUMERIC", "C")
            .stdin(Stdio::null());
        cmd
    }

    fn mux_command(&self) -> Command {
        let mut cmd = Command::new(&self.ffmpeg);
        cmd.args(["-hide_banner", "-nostdin", "-y", "-loglevel", "error", "-i"])
            .arg(&self.video_only)
            .arg("-i")
            .arg(&self.input)
            .args(["-map", "0:v", "-map", "1:a:0?", "-c:v", "copy", "-c:a", "copy", "-map_metadata", "0", "-metadata"])
            .arg(&self.metadata)
            .args(["-movflags", "use_metadata_tags", "-fps_mode", "passthrough"])
            .arg(&self.output);
        cmd
    }

    fn ffmpeg(&self) -> Command {
        let mut cmd = Command::new(&self.ffmpeg);
        cmd.args(["-hide_banner", "-nostdin", "-y", "-loglevel", "error"]);
        cmd
    }
}

fn quote_command(cmd: &Command) -> String {
    std::iter::once(cmd.get_program())
        .chain(cmd.get_args())
        .map(|a| shell_quote(&a.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(" ")
}

// ------------------------------------------------------------------ cancel

/// Cancels a running encode. With `isolated`, every child starts its own
/// process group and cancel signals the whole group, so nothing neuroserver
/// spawned is orphaned — what the TUI needs, since its raw-mode terminal
/// delivers no SIGINT. Without it children stay in the caller's group and a
/// terminal Ctrl-C reaches all of them, which is right for the CLI.
#[derive(Clone, Default)]
pub struct Cancel(Arc<CancelInner>);

#[derive(Default)]
struct CancelInner {
    isolated: bool,
    cancelled: AtomicBool,
    pgid: Mutex<Option<u32>>,
}

impl Cancel {
    pub fn isolated() -> Self {
        Cancel(Arc::new(CancelInner { isolated: true, ..Default::default() }))
    }

    pub fn cancel(&self) {
        self.0.cancelled.store(true, Ordering::SeqCst);
        if let Some(pgid) = *self.0.pgid.lock().unwrap() {
            let _ = Command::new("kill").arg("-TERM").arg(format!("-{pgid}")).status();
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.0.cancelled.load(Ordering::SeqCst)
    }

    fn spawn(&self, cmd: &mut Command) -> Result<Child> {
        if self.is_cancelled() {
            bail!("cancelled");
        }
        if self.0.isolated {
            cmd.process_group(0);
        }
        let child = cmd.spawn().with_context(|| format!("cannot run {}", cmd.get_program().to_string_lossy()))?;
        *self.0.pgid.lock().unwrap() = self.0.isolated.then_some(child.id());
        // A cancel that landed between the check and the spawn.
        if self.is_cancelled() {
            self.cancel();
        }
        Ok(child)
    }

    fn wait(&self, mut child: Child) -> Result<std::process::ExitStatus> {
        let status = child.wait();
        *self.0.pgid.lock().unwrap() = None;
        Ok(status?)
    }
}

/// Run an ffmpeg step to completion, its output appended to the log.
fn step(cancel: &Cancel, mut cmd: Command, log: &Path, what: &str) -> Result<()> {
    let out = OpenOptions::new().create(true).append(true).open(log)?;
    cmd.stdin(Stdio::null()).stdout(out.try_clone()?).stderr(out);
    let status = cancel.wait(cancel.spawn(&mut cmd)?)?;
    if cancel.is_cancelled() {
        bail!("cancelled");
    }
    if !status.success() {
        bail!("{what} failed — see {}", log.display());
    }
    Ok(())
}

// ------------------------------------------------------------------ run

/// Why neuroserver left no output, when the log knows: a bare "no output"
/// sends people hunting.
fn failure_reason(log: &Path) -> Option<String> {
    let text = String::from_utf8_lossy(&fs::read(log).ok()?).into_owned();
    if text.contains("broadcast_shapes") {
        return Some(
            "the source is smaller than the model's 640-pixel encoder tile — Starlight Precise cannot tile a frame that small".into(),
        );
    }
    text.lines()
        .filter(|l| {
            (l.contains("ERROR:local_model_service:") || l.contains("MODEL_INFERENCE_ERROR")) && !l.contains("Traceback")
        })
        .last()
        .map(|l| l.trim().trim_start_matches("ERROR:local_model_service:").to_string())
}

/// Salvage an interrupted partial into the next part file, or discard it,
/// as `plan.mode` says. Returns the part files to join, in order.
fn prepare(plan: &Plan, cancel: &Cancel, on: &mut dyn FnMut(Event)) -> Result<Vec<PathBuf>> {
    let mut parts = part_files(&plan.output, &plan.ext);
    let vo = &plan.video_only;
    let leftover = if vo.is_file() { probe::count_frames(vo) } else { 0 };

    match plan.mode {
        Mode::Fresh if leftover > 0 || !parts.is_empty() => {
            let extra = if parts.is_empty() {
                String::new()
            } else {
                format!("\n  plus {} earlier part file(s)", parts.len())
            };
            bail!(
                "an interrupted encode of this output exists: {leftover} readable frames in\n  {}{extra}\n\
                 Rerun with --resume to keep them and render only the rest, or --restart to discard them.",
                vo.file_name().unwrap_or_default().to_string_lossy()
            );
        }
        Mode::Fresh => {}
        Mode::Restart => {
            let _ = fs::remove_file(vo);
            for p in parts.drain(..) {
                let _ = fs::remove_file(p);
            }
        }
        Mode::Resume if leftover > 0 => {
            // A kill can land mid-write and leave the last packet torn: it is
            // still counted as a packet, but it does not decode, and copied into
            // the final file it costs a frame and shifts every frame after it.
            // So decode the partial once; if fewer frames decode than there are
            // packets, cut back and let the tail be rendered again. A partial
            // that decodes cleanly (a polite cancel) is kept whole.
            on(Event::Stage("checking the interrupted encode".into()));
            let mut keep = leftover;
            let decoded = probe::count_decoded_frames(vo);
            if decoded.is_none_or(|d| d < leftover) {
                let how = match (decoded, probe::has_b_frames(vo)) {
                    // No B-frames (VideoToolbox HEVC here, and ProRes is
                    // intra-only): every frame depends only on earlier ones, so
                    // the decodable prefix can be kept exactly.
                    (Some(d), Some(0)) if d > 0 => {
                        keep = d;
                        "up to the damaged frame"
                    }
                    // Reordered streams cannot be cut mid-GOP; a keyframe
                    // boundary always can.
                    _ => {
                        keep = probe::last_keyframe(vo);
                        "back to the last keyframe"
                    }
                };
                on(Event::Warn(format!(
                    "partial ends in a damaged frame — keeping {keep} of {leftover} frames ({how})"
                )));
            }
            if keep > 0 {
                let next = part_path(&plan.output, &plan.ext, parts.len() + 1);
                let mut cmd = plan.ffmpeg();
                cmd.arg("-i").arg(vo).args(["-map", "0:v", "-c", "copy", "-frames:v"]).arg(keep.to_string()).arg(&next);
                step(cancel, cmd, &plan.log, "salvaging the partial")
                    .with_context(|| format!("could not salvage {}", vo.display()))?;
                parts.push(next);
            }
            let _ = fs::remove_file(vo);
        }
        Mode::Resume => {
            let _ = fs::remove_file(vo);
        }
    }
    Ok(parts)
}

/// Run the encode to the finished file. `on` hears progress; a cancel leaves
/// whatever neuroserver wrote on disk, to be resumed.
pub fn run(plan: &Plan, cancel: &Cancel, on: &mut dyn FnMut(Event)) -> Result<PathBuf> {
    if let Some(dir) = plan.log.parent() {
        fs::create_dir_all(dir)?;
    }
    let parts = prepare(plan, cancel, on)?;
    let done_frames: u64 = parts.iter().map(|p| probe::count_frames(p)).sum();
    let start = plan.start_frame + done_frames;
    let expected = plan.end_frame.saturating_sub(plan.start_frame);

    // A resumed encode keeps the earlier run's log; a new one starts it afresh.
    if done_frames > 0 {
        let mut log = OpenOptions::new().create(true).append(true).open(&plan.log)?;
        writeln!(log, "### resuming at frame {start} ({done_frames} frames in {} part file(s))", parts.len())?;
    } else {
        fs::write(&plan.log, "")?;
    }

    if start < plan.end_frame {
        on(Event::Stage("starting neuroserver".into()));
        // stdout and stderr share one pipe: every line goes to the log verbatim,
        // and the JSON progress lines are parsed on the way through.
        let (reader, writer) = std::io::pipe()?;
        let mut cmd = plan.ns_command(start);
        cmd.stdout(writer.try_clone()?).stderr(writer);
        let child = cancel.spawn(&mut cmd)?;
        drop(cmd); // our copies of the write end, or the reader never sees EOF
        let mut log = OpenOptions::new().create(true).append(true).open(&plan.log)?;
        let mut reader = BufReader::new(reader);
        let mut buf = Vec::new();
        while reader.read_until(b'\n', &mut buf)? > 0 {
            let _ = log.write_all(&buf);
            if let Some(LogEvent::Progress { pct, frame, message }) = parse_line(String::from_utf8_lossy(&buf).trim_end()) {
                on(Event::Progress { pct, frame, message });
            }
            buf.clear();
        }
        let status = cancel.wait(child)?;
        if cancel.is_cancelled() {
            bail!("cancelled — what was written is kept; encode again to resume");
        }
        let written = if plan.video_only.is_file() { probe::count_frames(&plan.video_only) } else { 0 };
        if written == 0 {
            let reason = failure_reason(&plan.log).map(|r| format!(": {r}")).unwrap_or_default();
            bail!("neuroserver produced no output{reason}\nsee {}", plan.log.display());
        }
        // A crash mid-clip still leaves a readable partial: don't finish it as
        // if it were the whole clip.
        if !status.success() && done_frames + written < expected {
            let reason = failure_reason(&plan.log).map(|r| format!(": {r}")).unwrap_or_default();
            bail!(
                "neuroserver stopped after {written} of {} frames{reason}\nthe partial is kept — encode again to resume; see {}",
                plan.end_frame - start,
                plan.log.display()
            );
        }
    } else if parts.is_empty() {
        bail!("nothing to encode: frame range {}-{} is empty", plan.start_frame, plan.end_frame);
    } else {
        on(Event::Stage("every frame is already rendered — joining".into()));
    }

    // Join the salvaged parts and this run's video, losslessly, into video_only.
    if !parts.is_empty() {
        let list = beside(&plan.output, "ns-concat.txt");
        let joined = beside(&plan.output, &format!("ns-joined.{}", plan.ext));
        let pieces: Vec<&PathBuf> = parts
            .iter()
            .chain(std::iter::once(&plan.video_only))
            .filter(|p| fs::metadata(p).is_ok_and(|m| m.len() > 0))
            .collect();
        on(Event::Stage(format!("joining {} parts", pieces.len())));
        let text: String = pieces
            .iter()
            .map(|p| format!("file '{}'\n", p.to_string_lossy().replace('\'', "'\\''")))
            .collect();
        fs::write(&list, text)?;
        let mut cmd = plan.ffmpeg();
        cmd.args(["-f", "concat", "-safe", "0", "-i"]).arg(&list).args(["-map", "0:v", "-c", "copy"]).arg(&joined);
        step(cancel, cmd, &plan.log, "joining the parts")?;
        let total = probe::count_frames(&joined);
        if total != expected {
            bail!(
                "joined video has {total} frames, expected {expected} — parts kept, nothing deleted\nsee {}",
                plan.log.display()
            );
        }
        fs::rename(&joined, &plan.video_only)?;
        let _ = fs::remove_file(&list);
    }

    on(Event::Stage("muxing audio + metadata".into()));
    step(cancel, plan.mux_command(), &plan.log, "mux pass")?;
    let _ = fs::remove_file(&plan.video_only);
    for p in &parts {
        let _ = fs::remove_file(p);
    }
    if let Ok(mtime) = fs::metadata(&plan.input).and_then(|m| m.modified()) {
        let _ = fs::File::options().write(true).open(&plan.output).and_then(|f| f.set_modified(mtime));
    }
    Ok(plan.output.clone())
}

// ------------------------------------------------------------------ CLI

pub const USAGE: &str = "usage:
  neuroserver-encode --input file --model name --store dir [--params json] [--size WxH]
      [--output-profile slug | --video-args args --ext ext] [--preset-name name]
      [--metadata key=value] [--output file] [--start-frame N] [--end-frame N]
      [--nice[=N]] [--log-file path] [--resume | --restart] [--dry-run]

  (also: neuroserver-select-preset encode …)

Encode a whole clip through the Topaz app's neuroserver: neuroserver decodes,
enhances and encodes, then an ffmpeg pass muxes the source audio and the videoai
metadata tag onto the result, like the app's own export.

  --model / --store / --params  as in the preset (ns_model, ns_store, ns_params)
  --size WxH                    output size (default: source size)
  --output-profile slug         an output/ preset (hevc-cbr-40mbps, prores-422-proxy...)
  --video-args / --ext          or the codec arguments and extension directly
  --output file                 default: beside the input as \"<stem> [Topaz - <preset>].<ext>\"
  --start-frame / --end-frame   frame index range (default: the whole clip)
  --nice[=N]                    run neuroserver at niceness N (default 19)
  --log-file path               default: ~/Library/Logs/topaz-batch/<output stem>.log
  --resume                      continue an interrupted encode of the same output
  --restart                     discard an interrupted encode and start over
  --dry-run                     print the commands and exit

The video is written as fragmented MP4/MOV, so an interrupted encode keeps what
neuroserver wrote (at most one ~100-frame chunk is lost). If one is found and
neither --resume nor --restart is given, nothing is touched and the choice is
asked for.
";

impl Options {
    /// The CLI arguments that reproduce these options.
    pub fn to_args(&self) -> Vec<String> {
        let mut a: Vec<String> = vec!["--input".into(), self.input.to_string_lossy().into_owned()];
        let mut opt = |k: &str, v: Option<String>| {
            if let Some(v) = v {
                a.push(k.into());
                a.push(v);
            }
        };
        opt("--model", Some(self.model.clone()));
        opt("--store", Some(self.store.clone()));
        opt("--params", self.params.clone());
        opt("--size", self.size.map(|(w, h)| format!("{w}x{h}")));
        opt("--output-profile", self.output_profile.clone());
        opt("--video-args", self.video_args.clone());
        opt("--ext", self.ext.clone());
        opt("--preset-name", self.preset_name.clone());
        opt("--metadata", self.metadata.clone());
        opt("--output", self.output.as_ref().map(|p| p.to_string_lossy().into_owned()));
        opt("--start-frame", self.start_frame.map(|n| n.to_string()));
        opt("--end-frame", self.end_frame.map(|n| n.to_string()));
        opt("--log-file", self.log_file.as_ref().map(|p| p.to_string_lossy().into_owned()));
        if let Some(n) = self.nice {
            a.push(format!("--nice={n}"));
        }
        match self.mode {
            Mode::Fresh => {}
            Mode::Resume => a.push("--resume".into()),
            Mode::Restart => a.push("--restart".into()),
        }
        a
    }
}

fn parse_size(s: &str) -> Result<(u32, u32)> {
    let (w, h) = s.split_once('x').ok_or_else(|| anyhow!("--size must be WxH: {s}"))?;
    Ok((w.parse().context("--size width")?, h.parse().context("--size height")?))
}

const CYAN: &str = "\x1b[36m";
const YELLOW: &str = "\x1b[33m";
const GREEN: &str = "\x1b[32m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

/// `neuroserver-encode ARGS`: parse, plan, print, run.
pub fn cli(args: &[String]) -> Result<()> {
    let mut o = Options::default();
    let mut dry_run = false;
    let (mut resume, mut restart) = (false, false);
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        let mut value = || -> Result<String> {
            i += 1;
            args.get(i).cloned().ok_or_else(|| anyhow!("{a} needs a value"))
        };
        match a {
            "-h" | "--help" | "help" => {
                print!("{USAGE}");
                return Ok(());
            }
            "--input" => o.input = PathBuf::from(value()?),
            "--model" => o.model = value()?,
            "--store" => o.store = value()?,
            "--params" => o.params = Some(value()?),
            "--size" => o.size = Some(parse_size(&value()?)?),
            "--output-profile" => o.output_profile = Some(value()?),
            "--video-args" => o.video_args = Some(value()?),
            "--ext" => o.ext = Some(value()?),
            "--preset-name" => o.preset_name = Some(value()?),
            "--metadata" => o.metadata = Some(value()?),
            "--output" => o.output = Some(PathBuf::from(value()?)),
            "--start-frame" => o.start_frame = Some(value()?.parse().context("--start-frame")?),
            "--end-frame" => o.end_frame = Some(value()?.parse().context("--end-frame")?),
            "--log-file" => o.log_file = Some(PathBuf::from(value()?)),
            "--nice" => o.nice = Some(19),
            a if a.starts_with("--nice=") => o.nice = Some(a[7..].parse().context("--nice")?),
            "--dry-run" => dry_run = true,
            "--resume" => resume = true,
            "--restart" => restart = true,
            _ => bail!("unknown option: {a}\n\n{USAGE}"),
        }
        i += 1;
    }
    if o.input.as_os_str().is_empty() {
        bail!("--input is required\n\n{USAGE}");
    }
    o.mode = match (resume, restart) {
        (true, true) => bail!("--resume and --restart are mutually exclusive"),
        (true, false) => Mode::Resume,
        (false, true) => Mode::Restart,
        _ => Mode::Fresh,
    };

    let plan = plan(&o)?;
    let parts = part_files(&plan.output, &plan.ext);
    println!("{CYAN}neuroserver encode{RESET}");
    println!("  {BOLD}input:{RESET}  {}", plan.input.display());
    println!(
        "  {BOLD}model:{RESET}  {}  ({}){}",
        plan.model,
        plan.store,
        plan.params.as_deref().map(|p| format!("  {p}")).unwrap_or_default()
    );
    println!(
        "  {BOLD}size:{RESET}   {}x{}   frames {}-{}",
        plan.out_size.0, plan.out_size.1, plan.start_frame, plan.end_frame
    );
    if plan.mode == Mode::Resume && !parts.is_empty() {
        println!("  {BOLD}resume:{RESET} {} earlier part file(s) kept", parts.len());
    }
    println!("  {BOLD}output:{RESET} {}", plan.output.display());
    println!("  {BOLD}log:{RESET}    {}", plan.log.display());
    if dry_run {
        let done: u64 = parts.iter().map(|p| probe::count_frames(p)).sum();
        println!("{}", quote_command(&plan.ns_command(plan.start_frame + done)));
        println!("{}", quote_command(&plan.mux_command()));
        return Ok(());
    }

    let started = Instant::now();
    let mut on_line = false;
    let result = run(&plan, &Cancel::default(), &mut |ev| match ev {
        Event::Progress { pct, frame, message } => {
            print!(
                "\r  {:<24} frame {:<6} {:>3}%   {}s ",
                message.unwrap_or_default(),
                frame.map(|f| f.to_string()).unwrap_or_default(),
                pct.map(|p| p.to_string()).unwrap_or_default(),
                started.elapsed().as_secs()
            );
            let _ = std::io::stdout().flush();
            on_line = true;
        }
        Event::Stage(s) | Event::Warn(s) if std::mem::take(&mut on_line) => {
            println!("\n  {BOLD}{s}{RESET}");
        }
        Event::Stage(s) => println!("  {BOLD}{s}{RESET}"),
        Event::Warn(s) => println!("  {YELLOW}{s}{RESET}"),
    });
    if on_line {
        println!();
    }
    let output = result?;
    println!("{GREEN}done{RESET}  {}  ({}s)", output.display(), started.elapsed().as_secs());
    Ok(())
}
