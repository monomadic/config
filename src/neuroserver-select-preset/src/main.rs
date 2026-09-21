//! neuroserver-select-preset — pick a neuroserver preset (Starlight Precise
//! and friends), render its preview window at a timestamp, step through the
//! rendered frames as kitty images against the matching source frames, and
//! encode the whole clip with the chosen preset, resolution and output profile.
//!
//! The encoder (nsencode) is part of this binary. It also runs on its own as
//! `neuroserver-select-preset encode …`, or as `neuroserver-encode …` through
//! the symlink the installer makes: the name it is invoked by picks the mode.
//!
//! Keys
//!   ↑/↓ j/k   move in the focused list        Tab / S-Tab   next / previous list
//!   Enter r   render the preview window        Esc           cancel a render / quit
//!   ←/→ h/l   previous / next frame            o  space      Topaz ↔ original
//!   , .       time −1s / +1s                   < >           time −10s / +10s
//!   e         encode the whole clip            c             print the encode command and quit
//!   q         quit

mod catalog;
mod encode;
mod logtail;
mod nsencode;
mod probe;
mod render;

use anyhow::{anyhow, Context, Result};
use catalog::{neuroserver_presets, output_profiles, shell_quote, OutputProfile, Preset};
use nsencode::{Interrupted, Mode as EncodeMode};
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use probe::{res_available, res_options, Profile, ResOption};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::StatefulImage;
use render::{Event as REvent, Job, RenderResult};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

/// A running (or just finished) full-clip encode, shown in place of the preview.
struct Encoding {
    job: encode::Encode,
    label: String,
    stage: Option<String>,
    message: Option<String>,
    pct: u32,
    frame: u64,
    total_frames: u64,
    live: Option<PathBuf>,
    live_count: u64,
    done: Option<(bool, String)>,
    confirm_cancel: bool,
    finished_in: Option<f64>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Pane {
    Preset,
    Res,
    Output,
}

impl Pane {
    fn next(self) -> Self {
        match self {
            Pane::Preset => Pane::Res,
            Pane::Res => Pane::Output,
            Pane::Output => Pane::Preset,
        }
    }
    fn prev(self) -> Self {
        match self {
            Pane::Preset => Pane::Output,
            Pane::Res => Pane::Preset,
            Pane::Output => Pane::Res,
        }
    }
}

enum Exit {
    Quit,
    PrintCommand(Vec<String>),
}

struct App {
    input: PathBuf,
    profile: Profile,
    presets: Vec<Preset>,
    res: Vec<ResOption>,
    two_x_is_4k: bool,
    outputs: Vec<OutputProfile>,
    preset_sel: usize,
    res_sel: usize,
    out_sel: usize,
    pane: Pane,
    time: f64,
    renders: HashMap<String, RenderResult>,
    job: Option<Job>,
    stage: Option<String>,
    note: Option<String>,
    pct: Option<u32>,
    encoding: Option<Encoding>,
    error: Option<String>,
    /// Which rendered key is on screen (may lag the selection until Enter).
    shown: Option<String>,
    frame: usize,
    show_original: bool,
    confirm_encode: bool,
    interrupted: Option<Interrupted>,
    picker: Picker,
    images: HashMap<PathBuf, StatefulProtocol>,
}

impl App {
    fn preset(&self) -> &Preset {
        &self.presets[self.preset_sel]
    }

    /// The resolution actually used for the selected preset: the selection when
    /// it supports it, else the nearest supported fallback (as the mpv menu).
    fn effective_res(&self) -> &ResOption {
        let scales = &self.preset().scales;
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

    fn render_key(&self) -> String {
        format!("{}|{}|{:.3}", self.preset().slug, self.effective_res().key, self.time)
    }

    fn preset_label(&self) -> String {
        let r = self.effective_res();
        format!("{} {}", self.preset().display, r.label.split("  ").next().unwrap_or(r.key))
    }

    fn start_render(&mut self) {
        if self.job.is_some() {
            self.note = Some("already rendering — Esc cancels".into());
            return;
        }
        let key = self.render_key();
        if self.renders.contains_key(&key) {
            self.shown = Some(key);
            self.frame = self.renders[self.shown.as_ref().unwrap()].target;
            return;
        }
        let res = self.effective_res().clone();
        let size = if res.w > 0 { Some((res.w, res.h)) } else { None };
        self.error = None;
        self.stage = Some("starting".into());
        self.note = None;
        self.pct = None;
        self.job = Some(render::spawn(render::Request {
            input: self.input.clone(),
            preset: self.preset().clone(),
            preset_label: self.preset_label(),
            size,
            time: self.time,
            key,
        }));
    }

    fn poll_job(&mut self) {
        let Some(job) = &self.job else { return };
        let mut done = None;
        while let Ok(ev) = job.rx.try_recv() {
            match ev {
                REvent::Stage(s) => {
                    self.stage = Some(s);
                    self.note = None;
                }
                REvent::Note(n) => self.note = Some(n),
                REvent::Pct(p) => self.pct = Some(p),
                REvent::Done(r) => {
                    done = Some(r);
                    break;
                }
            }
        }
        if let Some(result) = done {
            let key = self.job.take().unwrap().key;
            self.stage = None;
            self.pct = None;
            match result {
                Ok(r) => {
                    self.frame = r.target;
                    self.note = Some(format!("rendered in {:.0}s", r.seconds));
                    self.renders.insert(key.clone(), r);
                    self.shown = Some(key);
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
            self.pct = None;
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
        // A different timestamp means different frames: the shown render stays
        // visible, but the header names the new time and Enter renders it.
    }

    fn current_image_path(&self) -> Option<PathBuf> {
        let r = self.shown_result()?;
        let map = if self.show_original { &r.original_frames } else { &r.topaz_frames };
        map.get(&self.frame)
            .or_else(|| map.values().next())
            .cloned()
    }

    fn encode_name(&self) -> String {
        format!("{} - {}", self.preset_label(), self.outputs[self.out_sel].display)
    }

    fn encode_ext(&self) -> &str {
        let o = &self.outputs[self.out_sel];
        if o.ext.is_empty() { "mp4" } else { o.ext.as_str() }
    }

    fn encode_output(&self) -> PathBuf {
        nsencode::output_path(&self.input, &self.encode_name(), self.encode_ext())
    }

    /// Frames that survive from an interrupted encode of the current output.
    fn find_interrupted(&self) -> Option<Interrupted> {
        nsencode::interrupted(&self.encode_output(), self.encode_ext())
    }

    /// The encode for the current choice. The encoder adds the
    /// fragmented-container flags that the live frame (encode.rs) and resuming
    /// both depend on.
    fn encode_options(&self, mode: EncodeMode) -> nsencode::Options {
        let p = self.preset();
        let r = self.effective_res();
        let output = self.encode_output();
        nsencode::Options {
            input: self.input.clone(),
            model: p.ns_model.clone(),
            store: p.ns_store.clone(),
            params: p.ns_params.clone(),
            size: (r.w > 0).then_some((r.w, r.h)),
            output_profile: Some(self.outputs[self.out_sel].slug.clone()),
            preset_name: Some(self.encode_name()),
            metadata: Some(p.metadata.clone()).filter(|m| !m.is_empty()),
            log_file: Some(nsencode::default_log(&output)),
            output: Some(output),
            nice: Some(19),
            mode,
            ..Default::default()
        }
    }

    fn start_encode(&mut self, mode: EncodeMode) {
        self.cancel(); // a preview render would only fight the encode for the GPU
        let output = self.encode_output();
        let partial = nsencode::video_only_path(&output, self.encode_ext());
        let scratch = std::env::temp_dir().join(format!("neuroserver-select-preset-live-{}", std::process::id()));
        let job = encode::spawn(encode::Request { options: self.encode_options(mode), output, partial, scratch });
        self.encoding = Some(Encoding {
            job,
            label: self.encode_name(),
            stage: None,
            message: Some(match (mode, self.interrupted) {
                (EncodeMode::Resume, Some(i)) => format!("resuming after {} kept frames", i.frames),
                _ => "starting neuroserver".into(),
            }),
            pct: 0,
            frame: 0,
            total_frames: self.profile.total_frames(),
            live: None,
            live_count: 0,
            done: None,
            confirm_cancel: false,
            finished_in: None,
        });
    }

    fn poll_encode(&mut self) {
        let Some(enc) = &mut self.encoding else { return };
        while let Ok(ev) = enc.job.rx.try_recv() {
            match ev {
                // Our own steps (salvage, join, mux) outrank neuroserver's last message.
                encode::Event::Stage(s) => {
                    enc.stage = Some(s);
                    enc.message = None;
                }
                encode::Event::Progress { pct, frame, message } => {
                    if let Some(p) = pct {
                        enc.pct = p.min(100);
                    }
                    if let Some(f) = frame {
                        enc.frame = f;
                    }
                    if message.is_some() {
                        enc.message = message;
                    }
                }
                encode::Event::LiveFrame(path) => {
                    // Drop the previous live frame's decoded image: a long
                    // encode would otherwise keep every one of them in memory.
                    if let Some(old) = enc.live.take() {
                        self.images.remove(&old);
                    }
                    enc.live = Some(path);
                    enc.live_count += 1;
                }
                encode::Event::Done { ok, detail } => {
                    enc.finished_in = Some(enc.job.started.elapsed().as_secs_f64());
                    if ok {
                        enc.pct = 100;
                    }
                    enc.done = Some((ok, detail));
                }
            }
        }
    }
}

fn main() -> Result<()> {
    let argv0 = std::env::args().next().unwrap_or_default();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if argv0.rsplit('/').next() == Some("neuroserver-encode") {
        return nsencode::cli(&args);
    }
    if args.first().map(String::as_str) == Some("encode") {
        return nsencode::cli(&args[1..]);
    }
    let mut input: Option<PathBuf> = None;
    let mut time: Option<f64> = None;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print!("{}", USAGE);
                return Ok(());
            }
            "--time" | "-t" => {
                i += 1;
                time = Some(args.get(i).and_then(|v| v.parse().ok()).ok_or_else(|| anyhow!("--time needs seconds"))?);
            }
            a if a.starts_with("--time=") => time = Some(a[7..].parse().context("--time")?),
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
    let input = input.ok_or_else(|| anyhow!("usage: neuroserver-select-preset FILE [--time SECONDS]"))?;
    if !input.is_file() {
        return Err(anyhow!("not a file: {}", input.display()));
    }
    let input = input.canonicalize()?;

    let presets = neuroserver_presets()?;
    let outputs = output_profiles()?;
    let profile = probe::probe(&input)?;
    let rs = res_options(&profile);
    let time = time.unwrap_or_else(|| {
        if profile.duration > 0.0 { (profile.duration * 0.1).min(10.0) } else { 10.0 }
    });

    let picker = make_picker()?;

    let mut app = App {
        input,
        profile,
        presets,
        res: rs.options,
        two_x_is_4k: rs.two_x_is_4k,
        outputs,
        preset_sel: 0,
        res_sel: rs.default,
        out_sel: 0,
        pane: Pane::Preset,
        time,
        renders: HashMap::new(),
        job: None,
        stage: None,
        note: Some("Enter renders the preview window".into()),
        pct: None,
        encoding: None,
        error: None,
        shown: None,
        frame: 0,
        show_original: false,
        confirm_encode: false,
        interrupted: None,
        picker,
        images: HashMap::new(),
    };

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
    let _ = std::fs::remove_dir_all(
        std::env::temp_dir().join(format!("neuroserver-select-preset-live-{}", std::process::id())),
    );

    match outcome? {
        Exit::Quit => Ok(()),
        Exit::PrintCommand(args) => {
            println!("{}", command_line(args));
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
    if is_kitty && std::env::var("NEUROSERVER_SELECT_PRESET_NO_QUERY").is_err() {
        return Picker::from_query_stdio().context("querying kitty for image support");
    }
    Ok(Picker::halfblocks())
}

const USAGE: &str = "usage: neuroserver-select-preset FILE [--time SECONDS]
       neuroserver-select-preset encode --help

Pick a neuroserver preset, preview its rendered window at a timestamp (kitty
images, one frame at a time, Topaz against source), then encode the whole clip.
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
                KeyCode::Enter | KeyCode::Esc => app.encoding = None,
                _ => {}
            }
        } else if enc.confirm_cancel {
            match key.code {
                KeyCode::Char('y') => {
                    enc.job.cancel();
                    enc.confirm_cancel = false;
                    enc.message = Some("cancelling…".into());
                }
                _ => enc.confirm_cancel = false,
            }
        } else if matches!(key.code, KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('x')) {
            enc.confirm_cancel = true;
        }
        return None;
    }
    if app.confirm_encode {
        app.confirm_encode = false;
        match (app.interrupted.is_some(), key.code) {
            // Nothing to lose: y or Enter just starts.
            (false, KeyCode::Char('y') | KeyCode::Enter) => app.start_encode(EncodeMode::Fresh),
            // Work on disk: resuming is the safe default, discarding needs its own key.
            (true, KeyCode::Char('r') | KeyCode::Enter) => app.start_encode(EncodeMode::Resume),
            (true, KeyCode::Char('x')) => app.start_encode(EncodeMode::Restart),
            _ => {}
        }
        return None;
    }
    let shift = key.modifiers.contains(KeyModifiers::SHIFT);
    match key.code {
        KeyCode::Char('q') => return Some(Exit::Quit),
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => return Some(Exit::Quit),
        KeyCode::Esc => {
            if app.job.is_some() {
                app.cancel();
            } else {
                return Some(Exit::Quit);
            }
        }
        KeyCode::Tab => app.pane = app.pane.next(),
        KeyCode::BackTab => app.pane = app.pane.prev(),
        KeyCode::Down | KeyCode::Char('j') => move_sel(app, 1),
        KeyCode::Up | KeyCode::Char('k') => move_sel(app, -1),
        KeyCode::Enter | KeyCode::Char('r') => app.start_render(),
        KeyCode::Left | KeyCode::Char('h') => app.step_frame(-1),
        KeyCode::Right | KeyCode::Char('l') => app.step_frame(1),
        KeyCode::Char('o') | KeyCode::Char(' ') => app.show_original = !app.show_original,
        KeyCode::Char(',') => app.shift_time(-1.0),
        KeyCode::Char('.') => app.shift_time(1.0),
        KeyCode::Char('<') => app.shift_time(-10.0),
        KeyCode::Char('>') => app.shift_time(10.0),
        KeyCode::Char('e') => {
            app.interrupted = app.find_interrupted();
            app.confirm_encode = true;
        }
        KeyCode::Char('c') => return Some(Exit::PrintCommand(app.encode_options(EncodeMode::Fresh).to_args())),
        KeyCode::Char('y') => {
            app.note = Some(match copy_to_clipboard(&command_line(app.encode_options(EncodeMode::Fresh).to_args())) {
                Ok(()) => "command copied to the clipboard".into(),
                Err(e) => format!("copy failed: {e}"),
            });
        }
        KeyCode::Char('1'..='9') if !shift => {
            let n = key.code.to_string().parse::<usize>().unwrap_or(1) - 1;
            match app.pane {
                Pane::Preset if n < app.presets.len() => app.preset_sel = n,
                Pane::Res if n < app.res.len() => app.res_sel = n,
                Pane::Output if n < app.outputs.len() => app.out_sel = n,
                _ => {}
            }
        }
        _ => {}
    }
    None
}

fn move_sel(app: &mut App, delta: i32) {
    let (sel, len) = match app.pane {
        Pane::Preset => (&mut app.preset_sel, app.presets.len()),
        Pane::Res => (&mut app.res_sel, app.res.len()),
        Pane::Output => (&mut app.out_sel, app.outputs.len()),
    };
    if len == 0 {
        return;
    }
    *sel = (*sel as i32 + delta).rem_euclid(len as i32) as usize;
}

// ---------------------------------------------------------------- drawing

const ACCENT: Color = Color::Rgb(0x0a, 0x84, 0xff);
/// Secondary text. An explicit grey rather than the palette's DarkGray (bright
/// black), which many dark themes draw at or near the background colour.
const DIM: Color = Color::Rgb(0x8c, 0x93, 0xa3);

/// The `c` / `y` command as one paste-able shell line: the encoder's own CLI,
/// which this binary also is.
fn command_line(args: Vec<String>) -> String {
    ["neuroserver-select-preset".to_string(), "encode".to_string()]
        .into_iter()
        .chain(args)
        .map(|a| shell_quote(&a))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Put `text` on the macOS clipboard.
fn copy_to_clipboard(text: &str) -> Result<()> {
    use std::io::Write;
    let mut child = Command::new("pbcopy").stdin(Stdio::piped()).spawn()?;
    child.stdin.take().expect("piped stdin").write_all(text.as_bytes())?;
    if !child.wait()?.success() {
        return Err(anyhow!("pbcopy failed"));
    }
    Ok(())
}

fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(40), Constraint::Min(20)])
        .split(area);
    draw_lists(f, app, cols[0]);
    if app.encoding.is_some() {
        draw_encode(f, app, cols[1]);
    } else {
        draw_preview(f, app, cols[1]);
    }
    if app.confirm_encode {
        draw_confirm(f, app, area);
    }
}

fn draw_lists(f: &mut Frame, app: &App, area: Rect) {
    let preset_h = (app.presets.len() as u16 + 2).min(area.height / 2);
    let res_h = app.res.len() as u16 + 2;
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(preset_h),
            Constraint::Length(res_h),
            Constraint::Min(4),
            Constraint::Length(6),
        ])
        .split(area);

    let title = |name: &str, pane: Pane| -> Line {
        if app.pane == pane {
            Line::from(Span::styled(format!(" {name} "), Style::default().fg(ACCENT).add_modifier(Modifier::BOLD)))
        } else {
            Line::from(Span::styled(format!(" {name} "), Style::default().fg(DIM)))
        }
    };
    let border = |pane: Pane| {
        Style::default().fg(if app.pane == pane { ACCENT } else { DIM })
    };

    // Presets: rendered ones get a check mark, like the mpv menu's cached stills.
    let items: Vec<ListItem> = app
        .presets
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let mark = if app.renders.keys().any(|k| k.starts_with(&format!("{}|", p.slug))) { "✓" } else { " " };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", i + 1), Style::default().fg(DIM)),
                Span::raw(p.display.clone()),
                Span::styled(format!("  {mark}"), Style::default().fg(Color::Green)),
            ]))
        })
        .collect();
    let mut st = ListState::default().with_selected(Some(app.preset_sel));
    f.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL).border_style(border(Pane::Preset)).title(title("Preset", Pane::Preset)))
            .highlight_style(Style::default().bg(ACCENT).fg(Color::White))
            .highlight_symbol("▸ "),
        rows[0],
        &mut st,
    );

    // Resolution: rows the selected preset cannot reach are dimmed; the one that
    // would actually be used (fallback) is marked.
    let eff = app.effective_res().key;
    let items: Vec<ListItem> = app
        .res
        .iter()
        .enumerate()
        .map(|(i, o)| {
            let ok = res_available(&app.preset().scales, o, app.two_x_is_4k);
            let style = if ok { Style::default() } else { Style::default().fg(DIM) };
            let used = if o.key == eff && app.res[app.res_sel].key != eff { "  ← used" } else { "" };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", i + 1), Style::default().fg(DIM)),
                Span::styled(o.label.clone(), style),
                Span::styled(used.to_string(), Style::default().fg(Color::Yellow)),
            ]))
        })
        .collect();
    let mut st = ListState::default().with_selected(Some(app.res_sel));
    f.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL).border_style(border(Pane::Res)).title(title("Resolution", Pane::Res)))
            .highlight_style(Style::default().bg(ACCENT).fg(Color::White))
            .highlight_symbol("▸ "),
        rows[1],
        &mut st,
    );

    let items: Vec<ListItem> = app
        .outputs
        .iter()
        .enumerate()
        .map(|(i, o)| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{} ", i + 1), Style::default().fg(DIM)),
                Span::raw(o.display.clone()),
            ]))
        })
        .collect();
    let mut st = ListState::default().with_selected(Some(app.out_sel));
    f.render_stateful_widget(
        List::new(items)
            .block(Block::default().borders(Borders::ALL).border_style(border(Pane::Output)).title(title("Output", Pane::Output)))
            .highlight_style(Style::default().bg(ACCENT).fg(Color::White))
            .highlight_symbol("▸ "),
        rows[2],
        &mut st,
    );

    let help = Paragraph::new(vec![
        Line::from(vec![key("↵"), Span::raw(" render  "), key("←→"), Span::raw(" frame  "), key("o"), Span::raw(" original")]),
        Line::from(vec![key(", ."), Span::raw(" ±1s  "), key("< >"), Span::raw(" ±10s  "), key("⇥"), Span::raw(" pane")]),
        Line::from(vec![key("e"), Span::raw(" encode  "), key("c"), Span::raw(" cmd  "), key("y"), Span::raw(" copy  "), key("q"), Span::raw(" quit")]),
        Line::from(Span::styled(app.preset().blurb.clone(), Style::default().fg(DIM))),
    ])
    .wrap(Wrap { trim: true })
    .block(Block::default().borders(Borders::ALL).border_style(Style::default().fg(DIM)));
    f.render_widget(help, rows[3]);
}

fn key(k: &str) -> Span<'static> {
    Span::styled(k.to_string(), Style::default().fg(ACCENT).add_modifier(Modifier::BOLD))
}

fn draw_preview(f: &mut Frame, app: &mut App, area: Rect) {
    let name = app.input.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let header = format!(
        " {}  ·  {}×{} @ {:.3}fps  ·  t = {:.2}s ",
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
    if app.job.is_some() {
        let pct = app.pct.unwrap_or(0).min(100);
        f.render_widget(
            Gauge::default()
                .gauge_style(Style::default().fg(ACCENT).bg(Color::Rgb(0x22, 0x22, 0x26)))
                .ratio(pct as f64 / 100.0)
                .label(format!("{pct}%")),
            rows[3],
        );
    }

    // The image.
    match app.current_image_path() {
        Some(path) => {
            if !app.images.contains_key(&path) {
                match image::ImageReader::open(&path).and_then(|r| r.decode().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))) {
                    Ok(img) => {
                        let proto = app.picker.new_resize_protocol(img);
                        app.images.insert(path.clone(), proto);
                    }
                    Err(e) => {
                        f.render_widget(Paragraph::new(format!("cannot load {}: {e}", path.display())).alignment(Alignment::Center), rows[0]);
                    }
                }
            }
            if let Some(proto) = app.images.get_mut(&path) {
                f.render_stateful_widget(StatefulImage::default(), rows[0], proto);
            }
        }
        None => {
            let msg = if app.job.is_some() {
                "rendering…"
            } else {
                "no preview yet — Enter renders the window at t"
            };
            f.render_widget(
                Paragraph::new(Span::styled(msg, Style::default().fg(DIM))).alignment(Alignment::Center),
                Rect { y: rows[0].y + rows[0].height / 2, height: 1, ..rows[0] },
            );
        }
    }

    // Frame strip: one glyph per rendered frame, the requested one ringed.
    let strip: Line = match app.shown_result() {
        Some(r) => {
            let mut spans: Vec<Span> = Vec::new();
            let which = if app.show_original { "original" } else { "topaz" };
            spans.push(Span::styled(format!(" {which:<8} "), Style::default().fg(if app.show_original { Color::Yellow } else { ACCENT }).add_modifier(Modifier::BOLD)));
            for (idx, _) in &r.topaz_frames {
                let cur = *idx == app.frame;
                let tgt = *idx == r.target;
                let glyph = match (cur, tgt) {
                    (true, true) => "◉",
                    (true, false) => "●",
                    (false, true) => "◎",
                    (false, false) => "○",
                };
                let style = if cur { Style::default().fg(ACCENT) } else { Style::default().fg(DIM) };
                spans.push(Span::styled(format!("{glyph} "), style));
            }
            let abs = r.window_start + app.frame as u64;
            spans.push(Span::styled(
                format!("  frame {} of window  ·  source frame {}  ·  {:.3}s", app.frame, abs, abs as f64 / app.profile.fps),
                Style::default().fg(DIM),
            ));
            Line::from(spans)
        }
        None => Line::from(""),
    };
    f.render_widget(Paragraph::new(strip), rows[1]);

    // Status: stage · note, or the error.
    let status: Line = if let Some(err) = &app.error {
        Line::from(Span::styled(format!(" ✗ {err}"), Style::default().fg(Color::Red)))
    } else if let Some(stage) = &app.stage {
        let note = app.note.clone().unwrap_or_default();
        Line::from(vec![
            Span::styled(" ● ", Style::default().fg(ACCENT)),
            Span::raw(stage.clone()),
            Span::styled(if note.is_empty() { String::new() } else { format!("  ·  {note}") }, Style::default().fg(DIM)),
        ])
    } else {
        let shown_is_current = app.shown.as_deref() == Some(app.render_key().as_str());
        let mut spans = vec![Span::styled(format!(" {}", app.note.clone().unwrap_or_default()), Style::default().fg(DIM))];
        if app.shown.is_some() && !shown_is_current {
            spans.push(Span::styled("   (showing an earlier render — Enter renders the current selection)", Style::default().fg(Color::Yellow)));
        }
        Line::from(spans)
    };
    f.render_widget(Paragraph::new(status), rows[2]);
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
    let title = format!(" encoding  ·  {} ", enc.label);
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT))
        .title(Line::from(Span::styled(title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD))));
    let inner = block.inner(area);
    f.render_widget(block, area);
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(4), Constraint::Length(1), Constraint::Length(1), Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

    // Image: the newest decoded frame once one exists; before that the preview
    // still for this selection if there is one, clearly labelled as not final.
    let live = enc.live.clone();
    let fallback = if live.is_none() { app.current_image_path() } else { None };
    let caption: Line = if live.is_some() {
        Line::from(vec![
            Span::styled(" live ", Style::default().fg(Color::Rgb(0x10, 0x10, 0x14)).bg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(format!("  newest frame written to the output  ·  update {}", enc.live_count), Style::default().fg(DIM)),
        ])
    } else if fallback.is_some() {
        Line::from(vec![
            Span::styled(" preview ", Style::default().fg(Color::Rgb(0x10, 0x10, 0x14)).bg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("  not the encode — frames land a chunk (~100 frames) at a time, after each chunk's decode pass", Style::default().fg(DIM)),
        ])
    } else {
        Line::from(Span::styled(
            " no frames yet — Starlight works in chunks of ~100 frames and writes each one only after its decode pass",
            Style::default().fg(DIM),
        ))
    };
    if let Some(path) = live.or(fallback) {
        if !app.images.contains_key(&path) {
            if let Ok(img) = image::ImageReader::open(&path).and_then(|r| r.decode().map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))) {
                let proto = app.picker.new_resize_protocol(img);
                app.images.insert(path.clone(), proto);
            }
        }
        if let Some(proto) = app.images.get_mut(&path) {
            f.render_stateful_widget(StatefulImage::default(), rows[0], proto);
        }
    }
    let Some(enc) = &app.encoding else { return };
    f.render_widget(Paragraph::new(caption), rows[1]);

    // Phase, frame count, elapsed and a rate-based ETA.
    let elapsed = enc.finished_in.unwrap_or_else(|| enc.job.started.elapsed().as_secs_f64());
    let eta = if enc.done.is_none() && enc.pct >= 2 {
        format!("  ·  about {} left", fmt_dur(elapsed * (100 - enc.pct) as f64 / enc.pct as f64))
    } else {
        String::new()
    };
    let frames = if enc.total_frames > 0 {
        format!("frame {} / {}", enc.frame.min(enc.total_frames), enc.total_frames)
    } else {
        format!("frame {}", enc.frame)
    };
    let phase = enc.message.clone().or_else(|| enc.stage.clone()).unwrap_or_default();
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(format!(" {phase}"), Style::default().fg(Color::White)),
            Span::styled(format!("  ·  {frames}  ·  {} elapsed{eta}", fmt_dur(elapsed)), Style::default().fg(DIM)),
        ])),
        rows[2],
    );

    let (color, label) = match &enc.done {
        Some((true, _)) => (Color::Green, "done".to_string()),
        Some((false, _)) => (Color::Red, "failed".to_string()),
        None => (ACCENT, format!("{}%", enc.pct)),
    };
    f.render_widget(
        Gauge::default()
            .gauge_style(Style::default().fg(color).bg(Color::Rgb(0x22, 0x22, 0x26)))
            .ratio(enc.pct.min(100) as f64 / 100.0)
            .label(label),
        rows[3],
    );

    let foot: Line = match &enc.done {
        Some((true, path)) => Line::from(vec![
            Span::styled(" ✓ ", Style::default().fg(Color::Green)),
            Span::raw(path.clone()),
            Span::styled("    ↵ back   q quit", Style::default().fg(DIM)),
        ]),
        Some((false, why)) => Line::from(vec![
            Span::styled(format!(" ✗ {why}"), Style::default().fg(Color::Red)),
            Span::styled("    ↵ back   q quit", Style::default().fg(DIM)),
        ]),
        None if enc.confirm_cancel => Line::from(Span::styled(
            " cancel this encode and lose the work so far?   y cancel   any other key keeps going",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        None => Line::from(Span::styled(format!(" → {}    Esc cancels", enc.job.output.display()), Style::default().fg(DIM))),
    };
    f.render_widget(Paragraph::new(foot), rows[4]);
}

fn draw_confirm(f: &mut Frame, app: &App, area: Rect) {
    let w = area.width.min(78);
    let h = if app.interrupted.is_some() { 9 } else { 7 };
    let rect = Rect {
        x: area.x + (area.width.saturating_sub(w)) / 2,
        y: area.y + (area.height.saturating_sub(h)) / 2,
        width: w,
        height: h,
    };
    let o = &app.outputs[app.out_sel];
    let text = vec![
        Line::from(Span::styled(app.preset_label(), Style::default().add_modifier(Modifier::BOLD))),
        Line::from(format!("{}  →  {}", app.effective_res().label, o.display)),
        Line::from(Span::styled(format!("{}", app.input.display()), Style::default().fg(DIM))),
        Line::from(""),
    ];
    let mut text = text;
    match app.interrupted {
        Some(i) => {
            let parts = if i.parts > 0 { format!(" (+{} earlier part file{})", i.parts, if i.parts == 1 { "" } else { "s" }) } else { String::new() };
            text.push(Line::from(Span::styled(
                format!("An interrupted encode of this output kept {} frames{parts}.", i.frames),
                Style::default().fg(Color::Yellow),
            )));
            text.push(Line::from(""));
            text.push(Line::from(vec![
                key("r"), Span::raw(" / "), key("↵"), Span::raw("  resume, render only the rest     "),
                key("x"), Span::raw("  discard and restart     "),
                key("any"), Span::raw(" cancel"),
            ]));
        }
        None => text.push(Line::from(vec![
            key("y"), Span::raw(" / "), key("↵"), Span::raw("  encode the whole clip      "),
            key("any"), Span::raw(" cancel"),
        ])),
    }
    f.render_widget(ratatui::widgets::Clear, rect);
    f.render_widget(
        Paragraph::new(text).wrap(Wrap { trim: true }).block(
            Block::default().borders(Borders::ALL).border_style(Style::default().fg(ACCENT)).title(" Encode? "),
        ),
        rect,
    );
}
