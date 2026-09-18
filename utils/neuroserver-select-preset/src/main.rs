//! neuroserver-select-preset — pick a neuroserver preset (Starlight Precise
//! and friends), render its preview window at a timestamp, step through the
//! rendered frames as kitty images against the matching source frames, and
//! hand the chosen preset, resolution and output profile to neuroserver-encode.
//!
//! Keys
//!   ↑/↓ j/k   move in the focused list        Tab / S-Tab   next / previous list
//!   Enter r   render the preview window        Esc           cancel a render / quit
//!   ←/→ h/l   previous / next frame            o  space      Topaz ↔ original
//!   , .       time −1s / +1s                   < >           time −10s / +10s
//!   e         encode with neuroserver-encode   c             print that command and quit
//!   q         quit

mod catalog;
mod probe;
mod render;

use anyhow::{anyhow, Context, Result};
use catalog::{neuroserver_presets, output_profiles, shell_quote, OutputProfile, Preset};
use crossterm::event::{self, Event as CEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use probe::{res_available, res_options, Profile, ResOption};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{Frame, Terminal};
use ratatui_image::picker::Picker;
use ratatui_image::protocol::StatefulProtocol;
use ratatui_image::StatefulImage;
use render::{Event as REvent, Job, RenderResult};
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

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
    Encode(Vec<String>),
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
    error: Option<String>,
    /// Which rendered key is on screen (may lag the selection until Enter).
    shown: Option<String>,
    frame: usize,
    show_original: bool,
    confirm_encode: bool,
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

    fn encode_command(&self) -> Vec<String> {
        let p = self.preset();
        let r = self.effective_res();
        let o = &self.outputs[self.out_sel];
        let mut cmd = vec![
            catalog::zsh_bin().join("neuroserver-encode").to_string_lossy().into_owned(),
            "--input".into(), self.input.to_string_lossy().into_owned(),
            "--model".into(), p.ns_model.clone(),
            "--store".into(), p.ns_store.clone(),
            "--output-profile".into(), o.slug.clone(),
            "--preset-name".into(), format!("{} - {}", self.preset_label(), o.display),
            "--nice".into(),
        ];
        if let Some(params) = &p.ns_params {
            cmd.extend(["--params".into(), params.clone()]);
        }
        if r.w > 0 {
            cmd.extend(["--size".into(), format!("{}x{}", r.w, r.h)]);
        }
        if !p.metadata.is_empty() {
            cmd.extend(["--metadata".into(), p.metadata.clone()]);
        }
        cmd
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
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
            a => input = Some(PathBuf::from(a)),
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
        error: None,
        shown: None,
        frame: 0,
        show_original: false,
        confirm_encode: false,
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

    match outcome? {
        Exit::Quit => Ok(()),
        Exit::PrintCommand(cmd) => {
            println!("{}", cmd.iter().map(|a| shell_quote(a)).collect::<Vec<_>>().join(" "));
            Ok(())
        }
        Exit::Encode(cmd) => {
            eprintln!("{}", cmd.iter().map(|a| shell_quote(a)).collect::<Vec<_>>().join(" "));
            let status = Command::new(&cmd[0]).args(&cmd[1..]).status().context("running neuroserver-encode")?;
            if status.success() { Ok(()) } else { Err(anyhow!("neuroserver-encode exited with {status}")) }
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

Pick a neuroserver preset, preview its rendered window at a timestamp (kitty
images, one frame at a time, Topaz against source), then encode the clip with
neuroserver-encode.
";

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, app: &mut App) -> Result<Exit> {
    loop {
        app.poll_job();
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
    if app.confirm_encode {
        match key.code {
            KeyCode::Char('y') | KeyCode::Enter => return Some(Exit::Encode(app.encode_command())),
            _ => {
                app.confirm_encode = false;
                return None;
            }
        }
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
        KeyCode::Char('e') => app.confirm_encode = true,
        KeyCode::Char('c') => return Some(Exit::PrintCommand(app.encode_command())),
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
const DIM: Color = Color::DarkGray;

fn draw(f: &mut Frame, app: &mut App) {
    let area = f.area();
    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(40), Constraint::Min(20)])
        .split(area);
    draw_lists(f, app, cols[0]);
    draw_preview(f, app, cols[1]);
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
        Line::from(vec![key("e"), Span::raw(" encode  "), key("c"), Span::raw(" print cmd  "), key("q"), Span::raw(" quit")]),
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
        .constraints([Constraint::Min(4), Constraint::Length(1), Constraint::Length(1)])
        .split(inner);

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

fn draw_confirm(f: &mut Frame, app: &App, area: Rect) {
    let w = area.width.min(70);
    let h = 7;
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
        Line::from(vec![key("y"), Span::raw(" / "), key("↵"), Span::raw("  encode the whole clip with neuroserver-encode      "), key("any"), Span::raw(" cancel")]),
    ];
    f.render_widget(ratatui::widgets::Clear, rect);
    f.render_widget(
        Paragraph::new(text).wrap(Wrap { trim: true }).block(
            Block::default().borders(Borders::ALL).border_style(Style::default().fg(ACCENT)).title(" Encode? "),
        ),
        rect,
    );
}
