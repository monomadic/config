//! topaz-select-preset — pick a Topaz ffmpeg preset (enhancement, resolution,
//! interpolation, output format — the topaz-pick flow), render a short window
//! of it at a timestamp, step through the rendered frames as kitty images
//! against the matching source frames, and encode the whole clip through
//! topaz-encode while watching the newest frame it has written.
//!
//! Keys
//!   ↑/↓ j/k   move in the focused list        Tab / S-Tab   next / previous list
//!   Enter r   render the preview window        Esc           cancel a render / quit
//!   ←/→ h/l   previous / next frame            o  space      Topaz ↔ original
//!   s         side by side                     d             preset details
//!   z         zoom 1× 2× 4× 8×                 H J K L       pan the zoomed view
//!   , .       time −1s / +1s                   < >           time −10s / +10s
//!   e         encode with topaz-encode         c             print that command and quit
//!   q         quit
//! While encoding: p pause / continue, o open the output so far in mpv, Esc stop.

mod catalog;
mod encode;
mod logtail;
mod plan;
mod probe;
mod render;
mod view;

use anyhow::{anyhow, Context, Result};
use catalog::{Enhancement, Insight, Interpolation, OutputProfile};
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use probe::{res_available, res_options, Profile, ResOption};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use ratatui_image::picker::Picker;
use render::{Event as REvent, Job, RenderResult};
use std::collections::HashMap;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use view::{Images, Zoom};

/// A running (or just finished) full-clip encode, shown in place of the preview.
struct Encoding {
    job: encode::Encode,
    label: String,
    output: PathBuf,
    watch: Vec<PathBuf>,
    message: Option<String>,
    lines: Vec<String>,
    progress: encode::Progress,
    /// Output seconds and rate, from the probe — the fallback until
    /// topaz-encode's own totals arrive in the progress file.
    out_duration: f64,
    out_fps: f64,
    live: Option<PathBuf>,
    live_count: u64,
    live_at: Option<Instant>,
    done: Option<(bool, String)>,
    stopping: bool,
    confirm_cancel: bool,
    finished_in: Option<f64>,
    paused_since: Option<Instant>,
    paused_total: Duration,
}

impl Encoding {
    fn elapsed(&self) -> f64 {
        if let Some(t) = self.finished_in {
            return t;
        }
        let paused = self.paused_total + self.paused_since.map(|p| p.elapsed()).unwrap_or_default();
        self.job.started.elapsed().saturating_sub(paused).as_secs_f64()
    }

    /// (output seconds done, output seconds in all), counting what a resumed
    /// partial already held.
    fn position(&self) -> (f64, f64) {
        let p = &self.progress;
        let total = match p.total {
            Some(t) if t > 0.0 => p.resume_from + t,
            _ => self.out_duration,
        };
        ((p.resume_from + p.out_time).min(total.max(0.0)), total)
    }

    fn pct(&self) -> f64 {
        if matches!(self.done, Some((true, _))) {
            return 1.0;
        }
        let (done, total) = self.position();
        if total > 0.0 { (done / total).clamp(0.0, 1.0) } else { 0.0 }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pane {
    Preset,
    Res,
    Interp,
    Output,
}

impl Pane {
    const ORDER: [Pane; 4] = [Pane::Preset, Pane::Res, Pane::Interp, Pane::Output];
    fn step(self, d: i32) -> Self {
        let i = Self::ORDER.iter().position(|p| *p == self).unwrap_or(0) as i32;
        Self::ORDER[(i + d).rem_euclid(4) as usize]
    }
}

/// What an earlier encode of the chosen output left on disk.
#[derive(Clone)]
enum Existing {
    Nothing,
    /// An interrupted encode: the .frag file (or a pre-.frag short output).
    Partial { path: PathBuf, kept: Option<f64>, of: f64 },
    Complete(PathBuf),
}

enum EncodeMode {
    Fresh,
    Resume,
    /// Move these to the Trash, then encode from scratch.
    Discard(Vec<PathBuf>),
}

enum Exit {
    Quit,
    PrintCommand(Vec<String>),
}

enum Row {
    Header(String),
    Preset(usize),
}

struct App {
    input: PathBuf,
    profile: Profile,
    presets: Vec<Enhancement>,
    rows: Vec<Row>,
    row_of: Vec<usize>,
    insights: HashMap<String, Insight>,
    res: Vec<ResOption>,
    two_x_is_4k: bool,
    interps: Vec<Interpolation>,
    outputs: Vec<OutputProfile>,
    preset_sel: usize,
    res_sel: usize,
    /// 0 is "None"; n is interps[n - 1].
    interp_sel: usize,
    out_sel: usize,
    pane: Pane,
    time: f64,
    window: u32,
    renders: HashMap<String, RenderResult>,
    job: Option<Job>,
    stage: Option<String>,
    note: Option<String>,
    frames_done: u64,
    frames_expected: u64,
    encoding: Option<Encoding>,
    error: Option<String>,
    /// Which rendered key is on screen (may lag the selection until Enter).
    shown: Option<String>,
    frame: usize,
    show_original: bool,
    split: bool,
    details: bool,
    zoom: Zoom,
    confirm: Option<Existing>,
    images: Images,
    scratch: PathBuf,
}

impl App {
    fn enh(&self) -> &Enhancement {
        &self.presets[self.preset_sel]
    }

    fn interp(&self) -> Option<&Interpolation> {
        self.interp_sel.checked_sub(1).and_then(|i| self.interps.get(i))
    }

    fn output(&self) -> &OutputProfile {
        &self.outputs[self.out_sel]
    }

    /// The resolution actually used for the selected preset: the selection when
    /// it supports it, else the nearest supported fallback (as the mpv menu).
    fn effective_res(&self) -> &ResOption {
        let scales = &self.enh().scales;
        let sel = &self.res[self.res_sel];
        if res_available(scales, sel, self.two_x_is_4k) {
            return sel;
        }
        const PRIORITY: &[(&str, &[&str])] = &[
            ("orig", &["orig", "2x", "4k", "4x"]),
            ("2x", &["2x", "4k", "4x", "orig"]),
            ("4k", &["4k", "2x", "4x", "orig"]),
            ("4x", &["4x", "4k", "2x", "orig"]),
        ];
        let order = PRIORITY.iter().find(|(k, _)| *k == sel.key).map(|(_, o)| *o).unwrap_or(&[]);
        for key in order {
            if let Some(o) = self.res.iter().find(|o| o.key == *key) {
                if res_available(scales, o, self.two_x_is_4k) {
                    return o;
                }
            }
        }
        &self.res[0]
    }

    fn filter(&self) -> String {
        plan::compose_filter(self.enh(), self.effective_res(), self.interp())
    }

    fn preset_name(&self) -> String {
        plan::preset_name(self.enh(), self.effective_res(), self.interp(), &self.output().display)
    }

    fn render_key(&self) -> String {
        let interp = self.interp().map(|i| i.slug.as_str()).unwrap_or("-");
        format!("{}|{}|{}|{:.3}", self.enh().slug, self.effective_res().key, interp, self.time)
    }

    /// Output seconds and frame rate of the encode, interpolation included.
    fn out_rate(&self) -> (f64, f64) {
        match self.interp() {
            Some(i) => {
                let (fps, slowmo) = i.rate();
                (self.profile.duration * slowmo, fps.unwrap_or(self.profile.fps))
            }
            None => (self.profile.duration, self.profile.fps),
        }
    }

    fn start_render(&mut self) {
        if self.job.is_some() {
            self.note = Some("already rendering — Esc cancels".into());
            return;
        }
        let key = self.render_key();
        if self.renders.contains_key(&key) {
            self.show(key);
            return;
        }
        let r = self.effective_res();
        let label = format!("{} {}", self.enh().display, r.label.split("  ").next().unwrap_or(r.key));
        let ratio = match self.interp() {
            Some(i) => {
                let (fps, slowmo) = i.rate();
                fps.map(|f| f / self.profile.fps).unwrap_or(1.0) * slowmo
            }
            None => 1.0,
        };
        self.error = None;
        self.stage = Some("starting".into());
        self.note = None;
        self.frames_done = 0;
        self.frames_expected = (self.window as f64 * ratio).round() as u64;
        self.job = Some(render::spawn(render::Request {
            input: self.input.clone(),
            label,
            filter: self.filter(),
            time: self.time,
            window: self.window,
            keep_interpolation: self.interp().is_some(),
            key,
        }));
    }

    /// Put a finished render on screen, keeping the frame position when the
    /// new render has the same shape (flicking between presets at one time).
    fn show(&mut self, key: String) {
        let Some(new) = self.renders.get(&key) else { return };
        let same_shape = self
            .shown_result()
            .map(|old| old.window_start == new.window_start && old.ratio == new.ratio)
            .unwrap_or(false);
        if !same_shape || !new.topaz_frames.contains_key(&self.frame) {
            self.frame = new.topaz_target();
        }
        self.note = Some(format!("rendered in {:.0}s", new.seconds));
        self.shown = Some(key);
    }

    /// A selection that was rendered before shows at once, without Enter.
    fn sync_shown(&mut self) {
        let key = self.render_key();
        if self.renders.contains_key(&key) && self.shown.as_deref() != Some(key.as_str()) {
            self.show(key);
        }
    }

    fn poll_job(&mut self) {
        let Some(job) = &self.job else { return };
        let mut done = None;
        while let Ok(ev) = job.rx.try_recv() {
            match ev {
                REvent::Stage(s) => {
                    self.stage = Some(s);
                    self.frames_done = 0;
                }
                REvent::Frame(n) => self.frames_done = n,
                REvent::Note(n) => self.note = Some(n),
                REvent::Done(r) => {
                    done = Some(r);
                    break;
                }
            }
        }
        if let Some(result) = done {
            let key = self.job.take().unwrap().key;
            self.stage = None;
            match result {
                Ok(r) => {
                    self.renders.insert(key.clone(), r);
                    self.show(key);
                }
                Err(e) => {
                    self.error = Some(e.to_string());
                    self.note = None;
                }
            }
        }
    }

    fn cancel(&mut self) {
        if let Some(job) = self.job.take() {
            job.cancel();
            self.stage = None;
            self.note = Some("render cancelled".into());
        }
    }

    fn shown_result(&self) -> Option<&RenderResult> {
        self.shown.as_ref().and_then(|k| self.renders.get(k))
    }

    fn step_frame(&mut self, delta: i32) {
        let Some(r) = self.shown_result() else { return };
        let keys: Vec<usize> = r.topaz_frames.keys().copied().collect();
        if keys.is_empty() {
            return;
        }
        let pos = keys.iter().position(|k| *k == self.frame).unwrap_or(0) as i32;
        let next = (pos + delta).clamp(0, keys.len() as i32 - 1) as usize;
        self.frame = keys[next];
    }

    fn shift_time(&mut self, delta: f64) {
        let max = if self.profile.duration > 0.0 { (self.profile.duration - 0.05).max(0.0) } else { f64::MAX };
        self.time = (self.time + delta).clamp(0.0, max);
        // The shown render stays up; the header names the new time, Enter
        // renders it, and a time rendered before comes straight back.
        self.sync_shown();
    }

    fn topaz_image(&self) -> Option<PathBuf> {
        let r = self.shown_result()?;
        r.topaz_frames.get(&self.frame).or_else(|| r.topaz_frames.values().next()).cloned()
    }

    fn original_image(&self) -> Option<PathBuf> {
        let r = self.shown_result()?;
        let src = r.source_offset(self.frame);
        r.original_frames.get(&src).or_else(|| r.original_frames.values().next()).cloned()
    }

    fn output_path(&self) -> PathBuf {
        let o = self.output();
        let ext = if o.ext.is_empty() { "mp4" } else { o.ext.as_str() };
        plan::output_path(&self.input, &self.preset_name(), ext)
    }

    /// What an earlier encode of the chosen output left behind — judged as
    /// topaz-encode judges it: within five seconds of the expected length is done.
    fn find_existing(&self) -> Existing {
        let output = self.output_path();
        let (expected, _) = self.out_rate();
        let complete = |d: Option<f64>| match d {
            Some(d) => expected <= 0.0 || d >= expected - 5.0,
            None => false,
        };
        if output.is_file() {
            let d = probe::duration(&output);
            if d.is_none() || complete(d) {
                return Existing::Complete(output);
            }
            return Existing::Partial { path: output, kept: d, of: expected };
        }
        let frag = plan::frag_path(&output);
        if frag.is_file() {
            return Existing::Partial { kept: probe::duration(&frag), path: frag, of: expected };
        }
        Existing::Nothing
    }

    /// The topaz-encode command for the current choice. `tui` adds what this
    /// tool needs to follow the encode; without it, it is the command to paste
    /// into a terminal (which names the output the same way by itself).
    fn encode_command(&self, tui: bool, resume: bool) -> Vec<String> {
        let o = self.output();
        let mut cmd = vec![catalog::zsh_bin().join("topaz-encode").to_string_lossy().into_owned()];
        if tui {
            cmd.extend([
                "--log".into(),
                "--output".into(), self.output_path().to_string_lossy().into_owned(),
                "--progress-file".into(), self.scratch.join("progress").to_string_lossy().into_owned(),
            ]);
        }
        if resume {
            cmd.push("--resume".into());
        }
        cmd.push("--nice".into());
        cmd.extend(["--preset_name".into(), self.preset_name(), "--filter_complex".into(), self.filter()]);
        if !o.ext.is_empty() {
            cmd.extend(["--output_ext".into(), o.ext.clone()]);
        }
        if !o.video_args.is_empty() {
            cmd.extend(["--video_args".into(), o.video_args.clone()]);
        }
        let e = self.enh();
        let metadata = if e.metadata.is_empty() { format!("videoai={}", e.display) } else { e.metadata.clone() };
        cmd.extend(["--metadata".into(), metadata]);
        cmd.extend(["--".into(), self.input.to_string_lossy().into_owned()]);
        cmd
    }

    fn start_encode(&mut self, mode: EncodeMode) {
        self.cancel(); // a preview render would only fight the encode for the GPU
        if let EncodeMode::Discard(paths) = &mode {
            for p in paths.iter().filter(|p| p.exists()) {
                if let Err(e) = encode::trash(p) {
                    self.error = Some(e);
                    return;
                }
            }
        }
        let output = self.output_path();
        let watch = vec![plan::frag_path(&output), plan::resume_tail_path(&output)];
        let (out_duration, out_fps) = self.out_rate();
        let job = encode::spawn(encode::Request {
            command: self.encode_command(true, matches!(mode, EncodeMode::Resume)),
            progress_file: self.scratch.join("progress"),
            watch: watch.clone(),
            scratch: self.scratch.clone(),
        });
        self.encoding = Some(Encoding {
            job,
            label: self.preset_name(),
            output,
            watch,
            message: Some(match mode {
                EncodeMode::Resume => "resuming the partial encode".into(),
                _ => "starting topaz-encode".into(),
            }),
            lines: Vec::new(),
            progress: encode::Progress::default(),
            out_duration,
            out_fps,
            live: None,
            live_count: 0,
            live_at: None,
            done: None,
            stopping: false,
            confirm_cancel: false,
            finished_in: None,
            paused_since: None,
            paused_total: Duration::ZERO,
        });
    }

    fn poll_encode(&mut self) {
        let Some(enc) = &mut self.encoding else { return };
        while let Ok(ev) = enc.job.rx.try_recv() {
            match ev {
                encode::Event::Line(line) => {
                    // topaz-encode marks each phase with a glyph (▶ encoding,
                    // ↻ resuming / finalizing / joining, ✓ ✗ ⚠); the rest of
                    // its transcript is detail, and its 5% "progress:" lines
                    // duplicate the progress file.
                    if line.starts_with(['▶', '↻', '✓', '✗', '⚠']) || line.starts_with("ERROR") {
                        enc.message = Some(line.clone());
                    }
                    enc.lines.push(line);
                    if enc.lines.len() > 40 {
                        enc.lines.remove(0);
                    }
                }
                encode::Event::Progress(p) => enc.progress = p,
                encode::Event::LiveFrame(path) => {
                    // Drop the previous live frame's images: a long encode
                    // would otherwise keep every one of them in memory.
                    if let Some(old) = enc.live.take() {
                        self.images.forget(&old);
                    }
                    enc.live = Some(path);
                    enc.live_count += 1;
                    enc.live_at = Some(Instant::now());
                }
                encode::Event::Done { ok } => {
                    enc.finished_in = Some(enc.elapsed());
                    enc.paused_since = None;
                    let detail = if ok {
                        enc.output.display().to_string()
                    } else if enc.stopping {
                        // topaz-encode reports the signal as a failure; here it was asked for.
                        enc.message = Some("■ stopped".into());
                        "stopped — the partial is kept, and e resumes it".into()
                    } else {
                        failure_line(&enc.lines)
                    };
                    enc.done = Some((ok, detail));
                }
            }
        }
    }

    fn toggle_pause(&mut self) {
        let Some(enc) = &mut self.encoding else { return };
        if enc.done.is_some() {
            return;
        }
        match enc.paused_since.take() {
            Some(since) => {
                enc.job.resume();
                enc.paused_total += since.elapsed();
            }
            None => {
                enc.job.pause();
                enc.paused_since = Some(Instant::now());
            }
        }
    }

    /// Open what the encode has written so far in mpv — the in-flight
    /// fragmented file, or the finished output.
    fn open_in_mpv(&mut self) {
        let Some(enc) = &mut self.encoding else { return };
        let file = if enc.output.is_file() && matches!(enc.done, Some((true, _))) {
            Some(enc.output.clone())
        } else {
            enc.watch.iter().filter(|p| p.is_file()).max_by_key(|p| p.metadata().and_then(|m| m.modified()).ok()).cloned()
        };
        let Some(file) = file else {
            enc.message = Some("nothing written yet".into());
            return;
        };
        let spawned = Command::new("mpv")
            .args(["--force-window=immediate", "--keep-open=yes", "--"])
            .arg(&file)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
        enc.message = Some(match spawned {
            Ok(_) => format!("opened {} in mpv", file.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()),
            Err(e) => format!("cannot run mpv: {e}"),
        });
    }
}

/// The line that says why an encode failed: topaz-encode's own ✗ / ERROR
/// headline if it printed one, else the last thing it said.
fn failure_line(lines: &[String]) -> String {
    lines
        .iter()
        .rev()
        .find(|l| l.starts_with('✗') || l.starts_with("ERROR"))
        .or_else(|| lines.iter().rev().find(|l| !l.starts_with("progress:")))
        .cloned()
        .unwrap_or_else(|| "encode failed".into())
}

/// Presets grouped under category headers; Original sits above the groups.
fn build_rows(presets: &[Enhancement]) -> (Vec<Row>, Vec<usize>) {
    let mut rows = Vec::new();
    let mut row_of = vec![0; presets.len()];
    let mut last: Option<&str> = None;
    for (i, p) in presets.iter().enumerate() {
        if !p.is_original() && last != Some(p.category.as_str()) {
            rows.push(Row::Header(catalog::category_label(&p.category)));
            last = Some(p.category.as_str());
        }
        row_of[i] = rows.len();
        rows.push(Row::Preset(i));
    }
    (rows, row_of)
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut input: Option<PathBuf> = None;
    let mut time: Option<f64> = None;
    let mut window: u32 = 8;
    let mut i = 0;
    while i < args.len() {
        let value = |i: usize, what: &str| args.get(i).cloned().ok_or_else(|| anyhow!("{what}"));
        match args[i].as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return Ok(());
            }
            "--time" | "-t" => {
                i += 1;
                time = Some(value(i, "--time needs seconds")?.parse().context("--time")?);
            }
            a if a.starts_with("--time=") => time = Some(a[7..].parse().context("--time")?),
            "--window" | "-w" => {
                i += 1;
                window = value(i, "--window needs a frame count")?.parse().context("--window")?;
            }
            a if a.starts_with("--window=") => window = a[9..].parse().context("--window")?,
            a if a.starts_with('-') && a.len() > 1 => return Err(anyhow!("unknown option: {a}\n{USAGE}")),
            // A file manager may pass a whole selection; this tool works on one
            // clip, and the first is the one the user was pointing at.
            a => {
                if input.is_none() {
                    input = Some(PathBuf::from(a));
                }
            }
        }
        i += 1;
    }
    if !(1..=64).contains(&window) {
        return Err(anyhow!("--window must be 1 to 64 frames"));
    }
    let input = input.ok_or_else(|| anyhow!("{USAGE}"))?;
    if !input.is_file() {
        return Err(anyhow!("not a file: {}", input.display()));
    }
    let input = input.canonicalize()?;

    let presets = catalog::enhancements()?;
    let interps = catalog::interpolations()?;
    let outputs = catalog::output_profiles()?;
    let insights = catalog::insights();
    let profile = probe::probe(&input)?;
    let rs = res_options(&profile);
    let time = time.unwrap_or_else(|| {
        if profile.duration > 0.0 { (profile.duration * 0.1).min(10.0) } else { 10.0 }
    });
    let (rows, row_of) = build_rows(&presets);
    let scratch = std::env::temp_dir().join(format!("topaz-select-preset-{}", std::process::id()));
    std::fs::create_dir_all(&scratch)?;

    let picker = make_picker()?;

    let mut app = App {
        input,
        profile,
        presets,
        rows,
        row_of,
        insights,
        res: rs.options,
        two_x_is_4k: rs.two_x_is_4k,
        interps,
        outputs,
        preset_sel: 0,
        res_sel: rs.default,
        interp_sel: 0,
        out_sel: 0,
        pane: Pane::Preset,
        time,
        window,
        renders: HashMap::new(),
        job: None,
        stage: None,
        note: Some("Enter renders the preview window".into()),
        frames_done: 0,
        frames_expected: 0,
        encoding: None,
        error: None,
        shown: None,
        frame: 0,
        show_original: false,
        split: false,
        details: false,
        zoom: Zoom::default(),
        confirm: None,
        images: Images::new(picker),
        scratch: scratch.clone(),
    };
    // Start on the first real preset rather than Original.
    if app.presets.len() > 1 {
        app.preset_sel = 1;
    }

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout))?;
    let outcome = run_loop(&mut terminal, &mut app);
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    app.cancel();
    if let Some(enc) = &app.encoding {
        if enc.done.is_none() {
            enc.job.cancel();
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
    let _ = std::fs::remove_file(std::env::temp_dir().join(format!("topaz-select-preset-{}.log", std::process::id())));

    match outcome? {
        Exit::Quit => Ok(()),
        Exit::PrintCommand(cmd) => {
            println!("{}", cmd.iter().map(|a| catalog::shell_quote(a)).collect::<Vec<_>>().join(" "));
            Ok(())
        }
    }
}

/// kitty gets real images through its graphics protocol, which needs a query
/// answered by the terminal before raw mode; anything else (including a plain
/// pty with nothing listening, where that query would hang) gets half-block
/// cells with an assumed cell size and no query at all.
fn make_picker() -> Result<Picker> {
    let is_kitty = std::env::var("KITTY_WINDOW_ID").is_ok()
        || std::env::var("TERM").map(|t| t.starts_with("xterm-kitty")).unwrap_or(false)
        || std::env::var("TERM_PROGRAM").map(|t| t == "kitty").unwrap_or(false);
    if is_kitty && std::env::var("TOPAZ_SELECT_PRESET_NO_QUERY").is_err() {
        return Picker::from_query_stdio().context("querying kitty for image support");
    }
    Ok(Picker::halfblocks())
}

const USAGE: &str = "usage: topaz-select-preset FILE [--time SECONDS] [--window FRAMES]

Pick a Topaz ffmpeg preset — enhancement, resolution, interpolation, output —
preview a short window of it at a timestamp (kitty images, one frame at a time,
Topaz against source), then encode the clip with topaz-encode and watch the
newest frame it writes.

  --time SECONDS    where the preview window sits (default: 10% in, at most 10s)
  --window FRAMES   source frames per preview render, 1-64 (default 8)
";

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<Exit> {
    loop {
        app.poll_job();
        app.poll_encode();
        terminal.draw(|f| draw(f, app))?;
        if !event::poll(Duration::from_millis(120))? {
            continue;
        }
        let CEvent::Key(key) = event::read()? else { continue };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        if let Some(exit) = handle_key(app, key) {
            return Ok(exit);
        }
    }
}

fn handle_key(app: &mut App, key: KeyEvent) -> Option<Exit> {
    // While an encode is up it owns the keyboard: nothing here may start a
    // second GPU job, and leaving has to be deliberate.
    if let Some(enc) = &mut app.encoding {
        if enc.done.is_some() {
            match key.code {
                KeyCode::Char('q') => return Some(Exit::Quit),
                KeyCode::Enter | KeyCode::Esc => {
                    if let Some(old) = enc.live.take() {
                        app.images.forget(&old);
                    }
                    app.encoding = None;
                }
                KeyCode::Char('o') => app.open_in_mpv(),
                _ => {}
            }
        } else if enc.confirm_cancel {
            enc.confirm_cancel = false;
            if key.code == KeyCode::Char('y') {
                enc.job.cancel();
                enc.stopping = true;
                enc.paused_since = None;
                enc.message = Some("stopping…".into());
            }
        } else {
            match key.code {
                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('x') => enc.confirm_cancel = true,
                KeyCode::Char('p') => app.toggle_pause(),
                KeyCode::Char('o') => app.open_in_mpv(),
                KeyCode::Char('z') => app.zoom.cycle(),
                _ => pan_key(app, key),
            }
        }
        return None;
    }
    if let Some(existing) = app.confirm.take() {
        match (existing, key.code) {
            // Nothing to lose: y or Enter just starts.
            (Existing::Nothing, KeyCode::Char('y') | KeyCode::Enter) => app.start_encode(EncodeMode::Fresh),
            // Work on disk: resuming is the safe default, discarding needs its own key.
            (Existing::Partial { .. }, KeyCode::Char('r') | KeyCode::Enter) => app.start_encode(EncodeMode::Resume),
            (Existing::Partial { path, .. }, KeyCode::Char('x')) => {
                let tail = plan::resume_tail_path(&app.output_path());
                app.start_encode(EncodeMode::Discard(vec![path, tail]));
            }
            (Existing::Complete(path), KeyCode::Char('x')) => {
                let out = app.output_path();
                app.start_encode(EncodeMode::Discard(vec![path, plan::frag_path(&out), plan::resume_tail_path(&out)]));
            }
            _ => {}
        }
        return None;
    }
    match key.code {
        KeyCode::Char('q') => return Some(Exit::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Some(Exit::Quit),
        KeyCode::Esc => {
            if app.job.is_some() {
                app.cancel();
            } else if app.details {
                app.details = false;
            } else {
                return Some(Exit::Quit);
            }
        }
        KeyCode::Tab => app.pane = app.pane.step(1),
        KeyCode::BackTab => app.pane = app.pane.step(-1),
        KeyCode::Down | KeyCode::Char('j') => move_sel(app, 1),
        KeyCode::Up | KeyCode::Char('k') => move_sel(app, -1),
        KeyCode::PageDown => move_sel(app, 10),
        KeyCode::PageUp => move_sel(app, -10),
        KeyCode::Enter | KeyCode::Char('r') => app.start_render(),
        KeyCode::Left | KeyCode::Right if key.modifiers.contains(KeyModifiers::SHIFT) => pan_key(app, key),
        KeyCode::Left | KeyCode::Char('h') => app.step_frame(-1),
        KeyCode::Right | KeyCode::Char('l') => app.step_frame(1),
        KeyCode::Char('o') | KeyCode::Char(' ') => app.show_original = !app.show_original,
        KeyCode::Char('s') => app.split = !app.split,
        KeyCode::Char('d') => app.details = !app.details,
        KeyCode::Char('z') => app.zoom.cycle(),
        KeyCode::Char(',') => app.shift_time(-1.0),
        KeyCode::Char('.') => app.shift_time(1.0),
        KeyCode::Char('<') => app.shift_time(-10.0),
        KeyCode::Char('>') => app.shift_time(10.0),
        KeyCode::Char('e') => {
            app.error = None;
            app.confirm = Some(app.find_existing());
        }
        KeyCode::Char('c') => return Some(Exit::PrintCommand(app.encode_command(false, false))),
        KeyCode::Char(ch @ '0'..='9') => {
            let n = ch.to_digit(10).unwrap_or(0) as usize;
            match app.pane {
                Pane::Res if (1..=app.res.len()).contains(&n) => app.res_sel = n - 1,
                Pane::Interp if n <= app.interps.len() => app.interp_sel = n,
                Pane::Output if (1..=app.outputs.len()).contains(&n) => app.out_sel = n - 1,
                _ => {}
            }
            app.sync_shown();
        }
        _ => pan_key(app, key),
    }
    None
}

/// H J K L (or shift-arrows) pan a zoomed view.
fn pan_key(app: &mut App, key: KeyEvent) {
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    let (dx, dy) = match key.code {
        KeyCode::Char('H') => (-1.0, 0.0),
        KeyCode::Char('L') => (1.0, 0.0),
        KeyCode::Char('K') => (0.0, -1.0),
        KeyCode::Char('J') => (0.0, 1.0),
        KeyCode::Left if shift => (-1.0, 0.0),
        KeyCode::Right if shift => (1.0, 0.0),
        KeyCode::Up if shift => (0.0, -1.0),
        KeyCode::Down if shift => (0.0, 1.0),
        _ => return,
    };
    app.zoom.pan(dx, dy);
}

fn move_sel(app: &mut App, delta: i32) {
    let (sel, len) = match app.pane {
        Pane::Preset => (&mut app.preset_sel, app.presets.len()),
        Pane::Res => (&mut app.res_sel, app.res.len()),
        Pane::Interp => (&mut app.interp_sel, app.interps.len() + 1),
        Pane::Output => (&mut app.out_sel, app.outputs.len()),
    };
    if len == 0 {
        return;
    }
    *sel = if delta.abs() > 1 {
        (*sel as i32 + delta).clamp(0, len as i32 - 1) as usize
    } else {
        (*sel as i32 + delta).rem_euclid(len as i32) as usize
    };
    app.sync_shown();
}

// ---------------------------------------------------------------- drawing

const ACCENT: Color = Color::Rgb(0x0a, 0x84, 0xff);
const DIM: Color = Color::DarkGray;
const TRACK: Color = Color::Rgb(0x22, 0x22, 0x26);

fn draw(f: &mut Frame, app: &mut App) {
    app.images.begin_frame();
    let area = f.area();
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(44), Constraint::Min(20)])
        .split(area);
    draw_lists(f, app, cols[0]);
    if app.encoding.is_some() {
        draw_encode(f, app, cols[1]);
    } else {
        draw_preview(f, app, cols[1]);
    }
    if let Some(existing) = app.confirm.clone() {
        draw_confirm(f, app, &existing, area);
    }
}

fn pane_block(app: &App, name: &str, pane: Pane) -> Block<'static> {
    let focused = app.pane == pane;
    let title = Span::styled(
        format!(" {name} "),
        if focused { Style::default().fg(ACCENT).add_modifier(Modifier::BOLD) } else { Style::default().fg(DIM) },
    );
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if focused { ACCENT } else { DIM }))
        .title(Line::from(title))
}

fn numbered(n: usize, text: String, style: Style) -> ListItem<'static> {
    ListItem::new(Line::from(vec![
        Span::styled(format!("{n} "), Style::default().fg(DIM)),
        Span::styled(text, style),
    ]))
}

fn draw_list(f: &mut Frame, items: Vec<ListItem<'static>>, block: Block<'static>, sel: usize, area: Rect) {
    let mut st = ListState::default().with_selected(Some(sel));
    f.render_stateful_widget(
        List::new(items)
            .block(block)
            .highlight_style(Style::default().bg(ACCENT).fg(Color::White))
            .highlight_symbol("▸ "),
        area,
        &mut st,
    );
}

fn draw_lists(f: &mut Frame, app: &App, area: Rect) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),
            Constraint::Length(app.res.len() as u16 + 2),
            Constraint::Length(app.interps.len() as u16 + 3),
            Constraint::Length(app.outputs.len() as u16 + 2),
            Constraint::Length(7),
        ])
        .split(area);

    // Presets, grouped; rendered ones get a check mark, like the mpv menu's
    // cached stills.
    let items: Vec<ListItem> = app
        .rows
        .iter()
        .map(|row| match row {
            Row::Header(label) => ListItem::new(Line::from(Span::styled(
                format!(" {label}"),
                Style::default().fg(DIM).add_modifier(Modifier::BOLD),
            ))),
            Row::Preset(i) => {
                let p = &app.presets[*i];
                let prefix = format!("{}|", p.slug);
                let mark = if app.renders.keys().any(|k| k.starts_with(&prefix)) { "  ✓" } else { "" };
                ListItem::new(Line::from(vec![
                    Span::raw(format!("  {}", p.display)),
                    Span::styled(mark, Style::default().fg(Color::Green)),
                ]))
            }
        })
        .collect();
    draw_list(f, items, pane_block(app, "Enhancement", Pane::Preset), app.row_of[app.preset_sel], rows[0]);

    // Resolution: rows the selected preset cannot reach are dimmed; the one that
    // would actually be used (fallback) is marked.
    let eff = app.effective_res().key;
    let items: Vec<ListItem> = app
        .res
        .iter()
        .enumerate()
        .map(|(i, o)| {
            let ok = res_available(&app.enh().scales, o, app.two_x_is_4k);
            let style = if ok { Style::default() } else { Style::default().fg(DIM) };
            let used = if o.key == eff && app.res[app.res_sel].key != eff { "  ← used" } else { "" };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", i + 1), Style::default().fg(DIM)),
                Span::styled(o.label.clone(), style),
                Span::styled(used, Style::default().fg(Color::Yellow)),
            ]))
        })
        .collect();
    draw_list(f, items, pane_block(app, "Resolution", Pane::Res), app.res_sel, rows[1]);

    let mut items = vec![numbered(0, "None (keep source frame rate)".into(), Style::default())];
    items.extend(app.interps.iter().enumerate().map(|(i, p)| numbered(i + 1, p.display.clone(), Style::default())));
    draw_list(f, items, pane_block(app, "Interpolation", Pane::Interp), app.interp_sel, rows[2]);

    let items = app.outputs.iter().enumerate().map(|(i, o)| numbered(i + 1, o.display.clone(), Style::default())).collect();
    draw_list(f, items, pane_block(app, "Output", Pane::Output), app.out_sel, rows[3]);

    let help = Paragraph::new(vec![
        Line::from(vec![key("↵"), Span::raw(" render  "), key("←→"), Span::raw(" frame  "), key("o"), Span::raw(" orig  "), key("s"), Span::raw(" split")]),
        Line::from(vec![key("z"), Span::raw(" zoom  "), key("HJKL"), Span::raw(" pan  "), key(", ."), Span::raw(" ±1s  "), key("< >"), Span::raw(" ±10s")]),
        Line::from(vec![key("d"), Span::raw(" details  "), key("e"), Span::raw(" encode  "), key("c"), Span::raw(" cmd  "), key("q"), Span::raw(" quit")]),
        Line::from(Span::styled(app.enh().blurb.clone(), Style::default().fg(DIM))),
    ])
    .wrap(Wrap { trim: true })
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(DIM)));
    f.render_widget(help, rows[4]);
}

fn key(k: &str) -> Span<'static> {
    Span::styled(k.to_string(), Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
}

fn centre_line(f: &mut Frame, text: &str, area: Rect) {
    f.render_widget(
        Paragraph::new(Span::styled(text.to_string(), Style::default().fg(DIM))).alignment(Alignment::Center),
        Rect { y: area.y + area.height / 2, height: 1.min(area.height), ..area },
    );
}

fn draw_image(f: &mut Frame, app: &mut App, path: &Path, area: Rect) {
    if let Err(e) = app.images.render(f, path, app.zoom, area) {
        centre_line(f, &e, area);
    }
}

fn draw_preview(f: &mut Frame, app: &mut App, area: Rect) {
    let name = app.input.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let zoom = if app.zoom.level > 1 {
        format!("  ·  zoom {}× at {:.0}%,{:.0}%", app.zoom.level, app.zoom.cx * 100.0, app.zoom.cy * 100.0)
    } else {
        String::new()
    };
    let header = format!(
        " {}  ·  {}×{} @ {:.3}fps  ·  t = {:.2}s{zoom} ",
        name, app.profile.width, app.profile.height, app.profile.fps, app.time
    );
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(DIM))
        .title(Line::from(Span::styled(header, Style::default().fg(Color::White))));
    let inner = block.inner(area);
    f.render_widget(block, area);

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(4), Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    if app.details {
        draw_details(f, app, rows[0]);
    } else if app.shown_result().is_none() {
        let msg = if app.job.is_some() { "rendering…" } else { "no preview yet — Enter renders the window at t" };
        centre_line(f, msg, rows[0]);
    } else if app.split {
        let halves = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Length(1), Constraint::Percentage(50)])
            .split(rows[0]);
        for (side, original) in [(halves[0], true), (halves[2], false)] {
            let parts = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(1), Constraint::Min(2)])
                .split(side);
            let (label, colour) = if original { ("original", Color::Yellow) } else { ("topaz", ACCENT) };
            f.render_widget(
                Paragraph::new(Span::styled(label, Style::default().fg(colour).add_modifier(Modifier::BOLD))).alignment(Alignment::Center),
                parts[0],
            );
            let path = if original { app.original_image() } else { app.topaz_image() };
            if let Some(path) = path {
                draw_image(f, app, &path, parts[1]);
            }
        }
    } else {
        let path = if app.show_original { app.original_image() } else { app.topaz_image() };
        if let Some(path) = path {
            draw_image(f, app, &path, rows[0]);
        }
    }

    // Frame strip: one glyph per rendered frame, the requested one ringed and
    // frames interpolation invented drawn small.
    if let Some(r) = app.shown_result() {
        let mut spans: Vec<Span> = Vec::new();
        let which = if app.split { "split" } else if app.show_original { "original" } else { "topaz" };
        let colour = if app.show_original && !app.split { Color::Yellow } else { ACCENT };
        spans.push(Span::styled(format!(" {which:<8} "), Style::default().fg(colour).add_modifier(Modifier::BOLD)));
        let target = r.topaz_target();
        for idx in r.topaz_frames.keys() {
            let invented = r.ratio > 1.0 && (*idx as f64 / r.ratio).fract().abs() > 1e-6;
            let glyph = match (*idx == app.frame, *idx == target, invented) {
                (true, true, _) => "◉",
                (false, true, _) => "◎",
                (true, false, false) => "●",
                (false, false, false) => "○",
                (true, false, true) => "•",
                (false, false, true) => "·",
            };
            let style = if *idx == app.frame { Style::default().fg(ACCENT) } else { Style::default().fg(DIM) };
            spans.push(Span::styled(format!("{glyph} "), style));
        }
        let pos = app.frame as f64 / r.ratio;
        let src = r.window_start as f64 + pos;
        let where_ = if pos.fract().abs() > 1e-6 {
            format!("invented, between source frames {} and {}", src.floor(), src.floor() + 1.0)
        } else {
            format!("source frame {}", src.round())
        };
        spans.push(Span::styled(
            format!("  {}  ·  {:.3}s", where_, src / app.profile.fps),
            Style::default().fg(DIM),
        ));
        f.render_widget(Paragraph::new(Line::from(spans)), rows[1]);
    }

    // Status: stage · frame counter · note, or the error.
    let status: Line = if let Some(err) = &app.error {
        Line::from(Span::styled(format!(" ✗ {err}"), Style::default().fg(Color::Red)))
    } else if let Some(stage) = &app.stage {
        let mut spans = vec![Span::styled(" ● ", Style::default().fg(ACCENT)), Span::raw(stage.clone())];
        if app.frames_done > 0 {
            spans.push(Span::styled(format!("  ·  frame {}", app.frames_done), Style::default().fg(DIM)));
        } else if stage.starts_with("enhancing") {
            spans.push(Span::styled("  ·  loading the model", Style::default().fg(DIM)));
        }
        if let Some(note) = &app.note {
            spans.push(Span::styled(format!("  ·  {note}"), Style::default().fg(Color::Yellow)));
        }
        Line::from(spans)
    } else {
        let shown_is_current = app.shown.as_deref() == Some(app.render_key().as_str());
        let mut spans = vec![Span::styled(format!(" {}", app.note.clone().unwrap_or_default()), Style::default().fg(DIM))];
        if app.shown.is_some() && !shown_is_current {
            spans.push(Span::styled(
                "   (showing an earlier render — Enter renders the current selection)",
                Style::default().fg(Color::Yellow),
            ));
        }
        Line::from(spans)
    };
    f.render_widget(Paragraph::new(status), rows[2]);

    if app.job.is_some() {
        let enhancing = app.stage.as_deref().map(|s| s.starts_with("enhancing")).unwrap_or(false);
        let (ratio, label) = if enhancing && app.frames_expected > 0 {
            let n = app.frames_done.min(app.frames_expected);
            (n as f64 / app.frames_expected as f64, format!("{n} / {} frames", app.frames_expected))
        } else {
            (0.0, app.stage.clone().unwrap_or_default())
        };
        f.render_widget(
            Gauge::default().gauge_style(Style::default().fg(ACCENT).bg(TRACK)).ratio(ratio).label(label),
            rows[3],
        );
    }
}

fn draw_details(f: &mut Frame, app: &App, area: Rect) {
    let e = app.enh();
    let head = Style::default().fg(ACCENT).add_modifier(Modifier::BOLD);
    let dim = Style::default().fg(DIM);
    let mut text = vec![Line::from(vec![
        Span::styled(e.display.clone(), Style::default().add_modifier(Modifier::BOLD)),
        Span::styled(
            if e.category.is_empty() { String::new() } else { format!("   {}", catalog::category_label(&e.category)) },
            dim,
        ),
    ])];
    text.push(Line::from(Span::styled(e.blurb.clone(), dim)));
    if let Some(ins) = app.insights.get(&e.slug) {
        let mut section = |title: &str, body: &str| {
            text.push(Line::from(""));
            text.push(Line::from(Span::styled(title.to_string(), head)));
            text.push(Line::from(body.to_string()));
        };
        if let Some(s) = &ins.strategy {
            section("Strategy", s);
        }
        for (k, v) in &ins.notes {
            section(k, v);
        }
        if let Some(w) = &ins.watch {
            section("Watch", w);
        }
        if let Some((slug, note)) = &ins.vs {
            let name = app.presets.iter().find(|p| &p.slug == slug).map(|p| p.display.clone()).unwrap_or_else(|| slug.clone());
            section(&format!("vs {name}"), note);
        }
    }
    if let Some(i) = app.interp() {
        text.push(Line::from(""));
        text.push(Line::from(Span::styled(format!("+ {}", i.display), head)));
        let about = i.metadata.split_once("] ").map(|(_, t)| t).unwrap_or(&i.metadata);
        text.push(Line::from(about.to_string()));
    }
    text.push(Line::from(""));
    text.push(Line::from(Span::styled("Filter", head)));
    text.push(Line::from(Span::styled(app.filter(), Style::default().fg(Color::Gray))));
    text.push(Line::from(""));
    text.push(Line::from(Span::styled("Encodes to", head)));
    text.push(Line::from(Span::styled(
        app.output_path().file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
        Style::default().fg(Color::Gray),
    )));
    let padded = Rect { x: area.x + 2, width: area.width.saturating_sub(4), ..area };
    f.render_widget(Paragraph::new(text).wrap(Wrap { trim: false }), padded);
}

fn fmt_dur(secs: f64) -> String {
    let s = secs.max(0.0) as u64;
    if s >= 3600 {
        format!("{}h {:02}m", s / 3600, (s % 3600) / 60)
    } else if s >= 60 {
        format!("{}m {:02}s", s / 60, s % 60)
    } else {
        format!("{s}s")
    }
}

fn draw_encode(f: &mut Frame, app: &mut App, area: Rect) {
    let Some(enc) = &app.encoding else { return };
    let paused = enc.paused_since.is_some();
    let title = format!(" {}  ·  {} ", if paused { "paused" } else { "encoding" }, enc.label);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if paused { Color::Yellow } else { ACCENT }))
        .title(Line::from(Span::styled(title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD))));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(4), Constraint::Length(1), Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    // Image: the newest decoded frame once one exists; before that the preview
    // for this selection if there is one, clearly labelled as not the encode.
    let live = enc.live.clone();
    let fallback = if live.is_none() { app.topaz_image() } else { None };
    let caption: Line = if live.is_some() {
        let age = enc.live_at.map(|t| format!("  ·  {} ago", fmt_dur(t.elapsed().as_secs_f64()))).unwrap_or_default();
        Line::from(vec![
            Span::styled(" live ", Style::default().fg(Color::Black).bg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  newest frame written to the output  ·  update {}{age}", enc.live_count), Style::default().fg(DIM)),
        ])
    } else if fallback.is_some() {
        Line::from(vec![
            Span::styled(" preview ", Style::default().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("  not the encode — the live frame appears once the first fragment is written", Style::default().fg(DIM)),
        ])
    } else {
        Line::from(Span::styled(" no frames yet — the model loads before the first frame is written", Style::default().fg(DIM)))
    };
    if let Some(path) = live.or(fallback) {
        draw_image(f, app, &path, rows[0]);
    }
    let Some(enc) = &app.encoding else { return };
    f.render_widget(Paragraph::new(caption), rows[1]);

    // Phase, frames, speed, elapsed and an ETA from ffmpeg's own speed.
    let p = &enc.progress;
    let (done, total) = enc.position();
    let elapsed = enc.elapsed();
    let frames_total = (total * enc.out_fps).round() as u64;
    let frame_now = if matches!(enc.done, Some((true, _))) { frames_total } else { (done * enc.out_fps).round() as u64 };
    let mut stats = vec![if frames_total > 0 {
        format!("frame {} / {}", frame_now.min(frames_total), frames_total)
    } else {
        format!("frame {frame_now}")
    }];
    if p.fps > 0.0 {
        stats.push(format!("{:.2} fps", p.fps));
    }
    if p.speed > 0.0 {
        stats.push(format!("{:.3}x", p.speed));
    }
    stats.push(format!("{} elapsed", fmt_dur(elapsed)));
    if enc.done.is_none() && !paused && p.speed > 0.0 && total > done {
        stats.push(format!("about {} left", fmt_dur((total - done) / p.speed)));
    }
    let phase = enc.message.clone().unwrap_or_default();
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!(" {phase}"), Style::default().fg(Color::White)),
            Span::styled(format!("  ·  {}", stats.join("  ·  ")), Style::default().fg(DIM)),
        ])),
        rows[2],
    );

    let pct = enc.pct();
    let (colour, label) = match &enc.done {
        Some((true, _)) => (Color::Green, "done".to_string()),
        Some((false, _)) if enc.stopping => (Color::Yellow, "stopped".to_string()),
        Some((false, _)) => (Color::Red, "failed".to_string()),
        None if paused => (Color::Yellow, format!("paused at {:.1}%", pct * 100.0)),
        None => (ACCENT, format!("{:.1}%", pct * 100.0)),
    };
    f.render_widget(Gauge::default().gauge_style(Style::default().fg(colour).bg(TRACK)).ratio(pct).label(label), rows[3]);

    let foot: Line = match &enc.done {
        Some((true, path)) => Line::from(vec![
            Span::styled(" ✓ ", Style::default().fg(Color::Green)),
            Span::raw(path.clone()),
            Span::styled("    ↵ back   o mpv   q quit", Style::default().fg(DIM)),
        ]),
        Some((false, why)) => Line::from(vec![
            Span::styled(format!(" {} {why}", if enc.stopping { "■" } else { "✗" }), Style::default().fg(if enc.stopping { Color::Yellow } else { Color::Red })),
            Span::styled("    ↵ back   q quit", Style::default().fg(DIM)),
        ]),
        None if enc.confirm_cancel => Line::from(Span::styled(
            " stop this encode? the partial is kept and resumes next time   y stop   any other key keeps going",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        None => Line::from(Span::styled(
            format!(" → {}    p {}   o mpv   z zoom   Esc stop", enc.output.display(), if paused { "continue" } else { "pause" }),
            Style::default().fg(DIM),
        )),
    };
    f.render_widget(Paragraph::new(foot), rows[4]);
}

fn draw_confirm(f: &mut Frame, app: &App, existing: &Existing, area: Rect) {
    let w = area.width.min(84);
    let h = if matches!(existing, Existing::Nothing) { 8 } else { 10 };
    let rect = Rect {
        x: area.x + (area.width.saturating_sub(w)) / 2,
        y: area.y + (area.height.saturating_sub(h)) / 2,
        width: w,
        height: h,
    };
    let output = app.output_path();
    let mut text = vec![
        Line::from(Span::styled(app.preset_name(), Style::default().add_modifier(Modifier::BOLD))),
        Line::from(Span::styled(app.filter(), Style::default().fg(DIM))),
        Line::from(format!("→ {}", output.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default())),
        Line::from(""),
    ];
    match existing {
        Existing::Nothing => text.push(Line::from(vec![
            key("y"), Span::raw(" / "), key("↵"), Span::raw("  encode the whole clip with topaz-encode      "),
            key("any"), Span::raw(" cancel"),
        ])),
        Existing::Partial { path, kept, of } => {
            let how_much = match kept {
                Some(k) if *of > 0.0 => format!("{} of {} ({:.0}%)", fmt_dur(*k), fmt_dur(*of), k / of * 100.0),
                Some(k) => fmt_dur(*k),
                None => "an unknown amount".into(),
            };
            text.push(Line::from(Span::styled(
                format!("An interrupted encode of this output kept {how_much}:"),
                Style::default().fg(Color::Yellow),
            )));
            text.push(Line::from(Span::styled(
                path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default(),
                Style::default().fg(DIM),
            )));
            text.push(Line::from(""));
            text.push(Line::from(vec![
                key("r"), Span::raw(" / "), key("↵"), Span::raw("  resume, encode only the rest     "),
                key("x"), Span::raw("  trash it and restart     "),
                key("any"), Span::raw(" cancel"),
            ]));
        }
        Existing::Complete(_) => {
            text.push(Line::from(Span::styled("This output already exists and is complete.", Style::default().fg(Color::Yellow))));
            text.push(Line::from(""));
            text.push(Line::from(vec![
                key("x"), Span::raw("  move it to the Trash and encode again     "),
                key("any"), Span::raw(" cancel"),
            ]));
        }
    }
    f.render_widget(Clear, rect);
    f.render_widget(
        Paragraph::new(text).wrap(Wrap { trim: true }).block(
            Block::default().borders(Borders::ALL).border_style(Style::default().fg(ACCENT)).title(" Encode? "),
        ),
        rect,
    );
}
