//! The screen for `sync` and `fill`: drive names and direction always visible,
//! a review list to approve, one row per drive while copying, then a summary.
//! Falls back to plain lines when stdout is not a terminal.
use crate::engine::{Control, Event, Kind, Overview, Phase, Summary, human};
use anyhow::Result;
use crossterm::event::{self, Event as Input, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
};
use std::{
    collections::VecDeque,
    io::IsTerminal,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender},
    },
    time::{Duration, Instant},
};

const DIM: Color = Color::Rgb(0x6B, 0x72, 0x80);
const LABEL: Color = Color::Rgb(0x9A, 0xA4, 0xB2);
const NAME: Color = Color::Rgb(0xE8, 0xEC, 0xF4);
const OK: Color = Color::Rgb(0x3B, 0xE3, 0x8B);
const WARN: Color = Color::Rgb(0xFF, 0xC2, 0x4B);
const ERR: Color = Color::Rgb(0xFF, 0x5C, 0x7A);
const PCT: Color = Color::Rgb(0x1E, 0xE6, 0xFF);
const FREE: Color = Color::Rgb(0xFF, 0x6F, 0xB5);
const SPEED: Color = Color::Rgb(0x8A, 0x5C, 0xFF);
const EMPTY: Color = Color::Rgb(0x2A, 0x2E, 0x3A);
// Neon stops: copying runs hot-pink → violet → cyan; a nearly full disk reads warm.
const COPY_STOPS: [(u8, u8, u8); 3] = [(0xFF, 0x2E, 0xC0), (0x8A, 0x5C, 0xFF), (0x1E, 0xE6, 0xFF)];
const DISK_STOPS: [(u8, u8, u8); 3] = [(0x22, 0xF5, 0xC8), (0x4F, 0x9C, 0xFF), (0xFF, 0x3C, 0x8A)];

fn gradient(stops: &[(u8, u8, u8)], t: f64) -> Color {
    let t = t.clamp(0.0, 1.0) * (stops.len() - 1) as f64;
    let i = (t.floor() as usize).min(stops.len() - 2);
    let f = t - i as f64;
    let (a, b) = (stops[i], stops[i + 1]);
    let mix = |x: u8, y: u8| (x as f64 + (y as f64 - x as f64) * f).round() as u8;
    Color::Rgb(mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
}

fn bar(width: usize, ratio: f64, stops: &[(u8, u8, u8)]) -> Vec<Span<'static>> {
    let width = width.max(1);
    let filled = ((ratio.clamp(0.0, 1.0) * width as f64).round() as usize).min(width);
    let mut spans = Vec::with_capacity(width);
    for i in 0..width {
        if i < filled {
            let t = if width > 1 {
                i as f64 / (width - 1) as f64
            } else {
                0.0
            };
            spans.push(Span::styled("█", Style::default().fg(gradient(stops, t))));
        } else {
            spans.push(Span::styled("░", Style::default().fg(EMPTY)));
        }
    }
    spans
}

/// Smoothed bytes/second from cumulative samples.
#[derive(Default)]
struct Speed {
    ewma: f64,
    last: Option<(Instant, u64)>,
}
impl Speed {
    fn sample(&mut self, now: Instant, total: u64) -> f64 {
        if let Some((then, bytes)) = self.last {
            let dt = now.duration_since(then).as_secs_f64();
            if dt > 0.05 {
                let inst = total.saturating_sub(bytes) as f64 / dt;
                self.ewma = if self.ewma == 0.0 {
                    inst
                } else {
                    0.3 * inst + 0.7 * self.ewma
                };
                self.last = Some((now, total));
            }
        } else {
            self.last = Some((now, total));
        }
        self.ewma
    }
}

#[derive(Default)]
struct Worker {
    path: String,
    kind: Option<Kind>,
    size: u64,
    bytes: u64,
    speed: Speed,
    active: bool,
}

#[derive(Default)]
struct ScanRow {
    files: usize,
    bytes: u64,
    reused: u64,
}

struct Model {
    verb: &'static str,
    sources: Vec<String>,
    destination: String,
    destination_path: Option<std::path::PathBuf>,
    phase: Phase,
    scans: Vec<ScanRow>,
    overview: Option<Overview>,
    list: ListState,
    workers: Vec<Worker>,
    done: usize,
    failed: usize,
    bytes: u64,
    total_bytes: u64,
    total_items: usize,
    total_speed: Speed,
    log: VecDeque<(bool, String)>,
    summary: Option<Summary>,
    error: Option<String>,
    disk: Option<(u64, u64)>,
    disk_checked: Instant,
}

impl Model {
    fn new(verb: &'static str) -> Self {
        Self {
            verb,
            sources: Vec::new(),
            destination: String::new(),
            destination_path: None,
            phase: Phase::Scanning,
            scans: Vec::new(),
            overview: None,
            list: ListState::default(),
            workers: Vec::new(),
            done: 0,
            failed: 0,
            bytes: 0,
            total_bytes: 0,
            total_items: 0,
            total_speed: Speed::default(),
            log: VecDeque::new(),
            summary: None,
            error: None,
            disk: None,
            disk_checked: Instant::now() - Duration::from_secs(60),
        }
    }
    fn push_log(&mut self, ok: bool, line: String) {
        self.log.push_back((ok, line));
        while self.log.len() > 200 {
            self.log.pop_front();
        }
    }
    fn apply(&mut self, event: Event) {
        match event {
            Event::Drives {
                sources,
                destination,
                destination_path,
            } => {
                self.scans = sources.iter().map(|_| ScanRow::default()).collect();
                self.workers = sources.iter().map(|_| Worker::default()).collect();
                self.sources = sources;
                self.destination = destination;
                self.destination_path = Some(destination_path);
            }
            Event::Phase(phase) => self.phase = phase,
            Event::Scan {
                drive,
                files,
                bytes,
                reused,
            } => {
                if let Some(row) = self.scans.get_mut(drive) {
                    *row = ScanRow {
                        files,
                        bytes,
                        reused,
                    };
                }
            }
            Event::Planned(overview) => {
                self.total_bytes = overview.transfer_bytes;
                self.total_items = overview
                    .items
                    .iter()
                    .filter(|i| i.kind != Kind::Skip)
                    .count();
                self.list.select(if overview.items.is_empty() {
                    None
                } else {
                    Some(0)
                });
                self.overview = Some(overview);
            }
            Event::Start {
                worker,
                kind,
                path,
                size,
            } => {
                if let Some(w) = self.workers.get_mut(worker) {
                    *w = Worker {
                        path,
                        kind: Some(kind),
                        size,
                        active: true,
                        ..Worker::default()
                    };
                }
            }
            Event::Progress { worker, bytes } => {
                if let Some(w) = self.workers.get_mut(worker) {
                    let delta = bytes.saturating_sub(w.bytes);
                    w.bytes = bytes;
                    self.bytes += delta;
                    let now = Instant::now();
                    w.speed.sample(now, bytes);
                    self.total_speed.sample(now, self.bytes);
                }
            }
            Event::Finish { worker, error } => {
                if let Some(w) = self.workers.get_mut(worker) {
                    w.active = false;
                    let path = std::mem::take(&mut w.path);
                    // A rename or retire moves no bytes; count it as its size.
                    if let Some(kind) = w.kind {
                        if matches!(kind, Kind::Rename | Kind::Retire) && error.is_none() {
                        } else if error.is_none() && w.bytes < w.size {
                            self.bytes += w.size - w.bytes;
                        }
                        let label = match kind {
                            Kind::Copy => "copied",
                            Kind::Replace => "replaced",
                            Kind::Rename => "renamed",
                            Kind::Retire => "retired",
                            Kind::Skip => "skipped",
                        };
                        match error {
                            None => {
                                self.done += 1;
                                self.push_log(true, format!("{label} {path}"));
                            }
                            Some(error) => {
                                self.failed += 1;
                                self.push_log(false, format!("{path}: {error}"));
                            }
                        }
                    }
                }
            }
            Event::Log(line) => self.push_log(true, line),
            Event::Done(summary) => {
                self.phase = Phase::Done;
                self.summary = Some(summary);
            }
            Event::Failed(error) => {
                self.phase = Phase::Done;
                self.error = Some(error);
            }
        }
    }
    fn refresh_disk(&mut self) {
        if self.disk_checked.elapsed() < Duration::from_secs(2) {
            return;
        }
        self.disk_checked = Instant::now();
        if let Some(path) = &self.destination_path {
            self.disk = crate::copy::space(path).ok();
        }
    }
    fn eta(&self, speed: f64) -> String {
        if speed <= 0.0 || self.total_bytes <= self.bytes {
            return "—".into();
        }
        let secs = (self.total_bytes - self.bytes) as f64 / speed;
        duration(secs)
    }
}

fn duration(secs: f64) -> String {
    let s = secs.round() as u64;
    if s >= 3600 {
        format!("{}h{:02}m", s / 3600, (s % 3600) / 60)
    } else if s >= 60 {
        format!("{}m{:02}s", s / 60, s % 60)
    } else {
        format!("{s}s")
    }
}

fn rate(bytes_per_second: f64) -> String {
    format!("{}/s", human(bytes_per_second as u64))
}

fn kind_span(kind: Kind) -> Span<'static> {
    let (text, color) = match kind {
        Kind::Copy => ("copy   ", OK),
        Kind::Replace => ("replace", WARN),
        Kind::Rename => ("rename ", PCT),
        Kind::Retire => ("retire ", FREE),
        Kind::Skip => ("skip   ", DIM),
    };
    Span::styled(text, Style::default().fg(color))
}

fn truncate(s: &str, n: usize) -> String {
    let count = s.chars().count();
    if count <= n || n < 2 {
        return s.into();
    }
    let keep: String = s.chars().skip(count - (n - 1)).collect();
    format!("…{keep}")
}

fn draw(frame: &mut Frame, model: &mut Model) {
    let area = frame.area();
    let width = area.width as usize;
    let rows = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(3),
        Constraint::Length(2),
    ])
    .split(area);
    draw_header(frame, rows[0], model);
    match model.phase {
        Phase::Scanning => draw_scanning(frame, rows[1], model),
        Phase::Review => draw_review(frame, rows[1], model, width),
        Phase::Transfer | Phase::Indexing => draw_transfer(frame, rows[1], model, width),
        Phase::Done => draw_done(frame, rows[1], model, width),
    }
    draw_footer(frame, rows[2], model, width);
}

fn draw_header(frame: &mut Frame, area: Rect, model: &Model) {
    let mut spans = vec![Span::styled(
        format!("safesync {} ", model.verb),
        Style::default().fg(NAME).add_modifier(Modifier::BOLD),
    )];
    for (i, source) in model.sources.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" + ", Style::default().fg(DIM)));
        }
        spans.push(Span::styled(
            source.clone(),
            Style::default().fg(PCT).add_modifier(Modifier::BOLD),
        ));
    }
    spans.push(Span::styled("  →  ", Style::default().fg(LABEL)));
    spans.push(Span::styled(
        model.destination.clone(),
        Style::default().fg(FREE).add_modifier(Modifier::BOLD),
    ));
    let phase = match model.phase {
        Phase::Scanning => "scanning",
        Phase::Review => "review",
        Phase::Transfer => "copying",
        Phase::Indexing => "writing indexes",
        Phase::Done => "done",
    };
    spans.push(Span::styled(
        format!("   {phase}"),
        Style::default().fg(DIM),
    ));
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_scanning(frame: &mut Frame, area: Rect, model: &Model) {
    let mut lines = Vec::new();
    for (name, row) in model.sources.iter().zip(&model.scans) {
        lines.push(Line::from(vec![
            Span::styled(format!("{name:<18}"), Style::default().fg(NAME)),
            Span::styled(
                format!("{:>8} files  ", row.files),
                Style::default().fg(LABEL),
            ),
            Span::styled(
                format!("{:>10}", human(row.bytes)),
                Style::default().fg(PCT),
            ),
            Span::styled(
                if row.reused > 0 {
                    format!("   {} fingerprints reused", row.reused)
                } else {
                    String::new()
                },
                Style::default().fg(DIM),
            ),
        ]));
    }
    if model.sources.len() == 1 {
        lines.push(Line::from(Span::styled(
            format!("{:<18}reading its index", model.destination),
            Style::default().fg(DIM),
        )));
    }
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "Only new or changed files are read; everything else comes from the last index.",
        Style::default().fg(DIM),
    )));
    frame.render_widget(Paragraph::new(lines), area);
}

fn draw_review(frame: &mut Frame, area: Rect, model: &mut Model, width: usize) {
    let Some(overview) = &model.overview else {
        return;
    };
    let parts = Layout::vertical([
        Constraint::Length(overview.notes.len() as u16 + 2),
        Constraint::Min(1),
    ])
    .split(area);
    let mut notes: Vec<Line> = vec![Line::from(vec![
        Span::styled(
            format!("{} actions", model.total_items),
            Style::default().fg(NAME).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  ·  {} to copy", human(overview.transfer_bytes)),
            Style::default().fg(PCT),
        ),
        Span::styled(
            match model.disk {
                Some((free, _)) => format!("  ·  {} free on {}", human(free), model.destination),
                None => String::new(),
            },
            Style::default().fg(FREE),
        ),
    ])];
    for note in &overview.notes {
        notes.push(Line::from(Span::styled(
            note.clone(),
            Style::default().fg(LABEL),
        )));
    }
    notes.push(Line::default());
    frame.render_widget(Paragraph::new(notes), parts[0]);
    let items: Vec<ListItem> = overview
        .items
        .iter()
        .map(|item| {
            let size = if item.size > 0 {
                format!("{:>10}  ", human(item.size))
            } else {
                " ".repeat(12)
            };
            ListItem::new(Line::from(vec![
                kind_span(item.kind),
                Span::styled(size, Style::default().fg(LABEL)),
                Span::styled(
                    truncate(&item.path, width.saturating_sub(22)),
                    Style::default().fg(NAME),
                ),
            ]))
        })
        .collect();
    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(Style::default().fg(EMPTY)),
        )
        .highlight_style(Style::default().bg(Color::Rgb(0x1A, 0x1E, 0x2A)));
    frame.render_stateful_widget(list, parts[1], &mut model.list);
}

fn draw_transfer(frame: &mut Frame, area: Rect, model: &Model, width: usize) {
    let worker_rows = model.workers.len() as u16 * 3;
    let parts = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(worker_rows),
        Constraint::Length(2),
        Constraint::Min(1),
    ])
    .split(area);
    let bar_width = width.saturating_sub(2).max(10);
    let ratio = if model.total_bytes > 0 {
        model.bytes as f64 / model.total_bytes as f64
    } else {
        0.0
    };
    let speed = model.total_speed.ewma;
    let mut overall = vec![Line::from(vec![
        Span::styled(
            format!("{:>5.1}%", ratio * 100.0),
            Style::default().fg(PCT).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!("  {} / {}", human(model.bytes), human(model.total_bytes)),
            Style::default().fg(LABEL),
        ),
        Span::styled(format!("  {}", rate(speed)), Style::default().fg(SPEED)),
        Span::styled(
            format!("  eta {}", model.eta(speed)),
            Style::default().fg(LABEL),
        ),
        Span::styled(
            format!("  {}/{} done", model.done, model.total_items),
            Style::default().fg(DIM),
        ),
        Span::styled(
            if model.failed > 0 {
                format!("  {} failed", model.failed)
            } else {
                String::new()
            },
            Style::default().fg(ERR),
        ),
    ])];
    overall.push(Line::from(bar(bar_width, ratio, &COPY_STOPS)));
    frame.render_widget(Paragraph::new(overall), parts[0]);

    let mut lines = Vec::new();
    for (name, worker) in model.sources.iter().zip(&model.workers) {
        let label = if model.sources.len() > 1 {
            format!("{name}: ")
        } else {
            String::new()
        };
        if worker.active {
            let r = if worker.size > 0 {
                worker.bytes as f64 / worker.size as f64
            } else {
                1.0
            };
            lines.push(Line::from(vec![
                Span::styled(label, Style::default().fg(PCT)),
                worker.kind.map(kind_span).unwrap_or_default(),
                Span::styled(
                    format!(" {}", truncate(&worker.path, width.saturating_sub(40))),
                    Style::default().fg(NAME),
                ),
                Span::styled(
                    format!(
                        "  {} / {}  {}",
                        human(worker.bytes),
                        human(worker.size),
                        rate(worker.speed.ewma)
                    ),
                    Style::default().fg(LABEL),
                ),
            ]));
            lines.push(Line::from(bar(bar_width, r, &COPY_STOPS)));
        } else {
            lines.push(Line::from(Span::styled(
                format!("{label}idle"),
                Style::default().fg(DIM),
            )));
            lines.push(Line::default());
        }
        lines.push(Line::default());
    }
    frame.render_widget(Paragraph::new(lines), parts[1]);

    if let Some((free, total)) = model.disk {
        let used = total.saturating_sub(free) as f64 / total.max(1) as f64;
        let mut spans = vec![
            Span::styled(
                format!("{} ", model.destination),
                Style::default().fg(LABEL),
            ),
            Span::styled(format!("{} free", human(free)), Style::default().fg(FREE)),
            Span::raw(" "),
        ];
        spans.extend(bar(
            bar_width.saturating_sub(model.destination.len() + 20),
            used,
            &DISK_STOPS,
        ));
        frame.render_widget(Paragraph::new(Line::from(spans)), parts[2]);
    }

    let height = parts[3].height as usize;
    let lines: Vec<Line> = model
        .log
        .iter()
        .rev()
        .take(height)
        .rev()
        .map(|(ok, line)| {
            Line::from(Span::styled(
                truncate(line, width),
                Style::default().fg(if *ok { DIM } else { ERR }),
            ))
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), parts[3]);
}

fn draw_done(frame: &mut Frame, area: Rect, model: &Model, width: usize) {
    let mut lines = Vec::new();
    if let Some(error) = &model.error {
        lines.push(Line::from(Span::styled(
            format!("Failed: {error}"),
            Style::default().fg(ERR).add_modifier(Modifier::BOLD),
        )));
    } else if let Some(summary) = &model.summary {
        let headline = if summary.cancelled {
            ("Cancelled", WARN)
        } else if summary.failed > 0 {
            ("Finished with errors", ERR)
        } else if summary.done == 0 && model.verb != "scan" {
            ("Nothing to do — already in sync", OK)
        } else {
            ("Done", OK)
        };
        lines.push(Line::from(Span::styled(
            headline.0,
            Style::default().fg(headline.1).add_modifier(Modifier::BOLD),
        )));
        let speed = if summary.seconds > 0.0 {
            summary.bytes as f64 / summary.seconds
        } else {
            0.0
        };
        lines.push(Line::from(vec![
            Span::styled(format!("{} files", summary.done), Style::default().fg(NAME)),
            Span::styled(
                if summary.bytes > 0 {
                    format!("  ·  {}", human(summary.bytes))
                } else {
                    String::new()
                },
                Style::default().fg(PCT),
            ),
            Span::styled(
                format!("  ·  {}", duration(summary.seconds)),
                Style::default().fg(LABEL),
            ),
            Span::styled(
                if summary.bytes > 0 {
                    format!("  ·  {}", rate(speed))
                } else {
                    String::new()
                },
                Style::default().fg(SPEED),
            ),
            Span::styled(
                if summary.failed > 0 {
                    format!("  ·  {} failed", summary.failed)
                } else {
                    String::new()
                },
                Style::default().fg(ERR),
            ),
        ]));
    }
    lines.push(Line::default());
    let height = (area.height as usize).saturating_sub(lines.len());
    for (ok, line) in model.log.iter().rev().take(height).rev() {
        lines.push(Line::from(Span::styled(
            truncate(line, width),
            Style::default().fg(if *ok { DIM } else { ERR }),
        )));
    }
    frame.render_widget(Paragraph::new(lines), area);
}

fn draw_footer(frame: &mut Frame, area: Rect, model: &Model, width: usize) {
    let keys = match model.phase {
        Phase::Scanning => "esc quit",
        Phase::Review => "enter start   ↑↓ scroll   esc cancel",
        Phase::Transfer | Phase::Indexing => "esc stop after the current file",
        Phase::Done => "any key to exit",
    };
    let lines = vec![
        Line::from(bar(width, 0.0, &COPY_STOPS)),
        Line::from(Span::styled(keys, Style::default().fg(DIM))),
    ];
    frame.render_widget(Paragraph::new(lines), area);
}

pub struct Session {
    pub control: Control,
    events: Receiver<Event>,
    confirm: Sender<bool>,
    cancel: Arc<AtomicBool>,
}
impl Session {
    pub fn new() -> Self {
        let (events_tx, events) = mpsc::channel();
        let (confirm, confirm_rx) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        Self {
            control: Control {
                events: events_tx,
                confirm: Mutex::new(confirm_rx),
                cancel: cancel.clone(),
            },
            events,
            confirm,
            cancel,
        }
    }
}
impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

/// Drives `work` on a background thread and shows it. `yes` skips the review.
/// Returns the exit status: 0 done, 1 cancelled or partial, 2 failed.
pub fn run(
    verb: &'static str,
    yes: bool,
    work: impl FnOnce(Control) + Send + 'static,
) -> Result<i32> {
    let Session {
        control,
        events,
        confirm,
        cancel,
    } = Session::new();
    std::thread::spawn(move || work(control));
    let mut model = Model::new(verb);
    if std::io::stdout().is_terminal() && std::io::stdin().is_terminal() {
        run_screen(&mut model, events, confirm, cancel, yes)?;
    } else {
        run_plain(&mut model, events, confirm, yes);
    }
    Ok(if model.error.is_some() {
        2
    } else if model
        .summary
        .as_ref()
        .is_some_and(|s| s.cancelled || s.failed > 0)
    {
        1
    } else {
        0
    })
}

fn run_screen(
    model: &mut Model,
    events: Receiver<Event>,
    confirm: Sender<bool>,
    cancel: Arc<AtomicBool>,
    yes: bool,
) -> Result<()> {
    let mut terminal = ratatui::init();
    let result = (|| -> Result<()> {
        let mut answered = false;
        loop {
            while let Ok(event) = events.try_recv() {
                model.apply(event);
            }
            if model.phase == Phase::Review && yes && !answered {
                answered = true;
                let _ = confirm.send(true);
            }
            model.refresh_disk();
            terminal.draw(|frame| draw(frame, model))?;
            if event::poll(Duration::from_millis(100))?
                && let Input::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                let quit = key.code == KeyCode::Esc
                    || key.code == KeyCode::Char('q')
                    || (key.code == KeyCode::Char('c')
                        && key.modifiers.contains(KeyModifiers::CONTROL));
                match model.phase {
                    Phase::Done => return Ok(()),
                    Phase::Review => match key.code {
                        KeyCode::Enter | KeyCode::Char('y') if !answered => {
                            answered = true;
                            let _ = confirm.send(true);
                        }
                        KeyCode::Down | KeyCode::Char('j') => model.list.select_next(),
                        KeyCode::Up | KeyCode::Char('k') => model.list.select_previous(),
                        KeyCode::PageDown => model.list.scroll_down_by(20),
                        KeyCode::PageUp => model.list.scroll_up_by(20),
                        _ if quit && !answered => {
                            answered = true;
                            let _ = confirm.send(false);
                        }
                        _ => {}
                    },
                    _ if quit => {
                        cancel.store(true, Ordering::Relaxed);
                        model.push_log(false, "Stopping after the current file…".into());
                    }
                    _ => {}
                }
            }
        }
    })();
    ratatui::restore();
    result
}

fn run_plain(model: &mut Model, events: Receiver<Event>, confirm: Sender<bool>, yes: bool) {
    let mut last = Instant::now();
    for event in events {
        match &event {
            Event::Drives {
                sources,
                destination,
                ..
            } => {
                eprintln!(
                    "safesync {}: {} → {}",
                    model.verb,
                    sources.join(" + "),
                    destination
                );
            }
            Event::Phase(phase) => eprintln!("[{phase:?}]"),
            Event::Scan {
                drive,
                files,
                bytes,
                ..
            } => {
                if last.elapsed() >= Duration::from_secs(1) {
                    last = Instant::now();
                    eprintln!(
                        "  {}: {files} files, {}",
                        model.sources.get(*drive).cloned().unwrap_or_default(),
                        human(*bytes)
                    );
                }
            }
            Event::Planned(overview) => {
                for note in &overview.notes {
                    eprintln!("  {note}");
                }
                for item in &overview.items {
                    eprintln!(
                        "  {:<8}{:>10}  {}",
                        format!("{:?}", item.kind).to_lowercase(),
                        human(item.size),
                        item.path
                    );
                }
                eprintln!("  {} to copy", human(overview.transfer_bytes));
            }
            Event::Start { path, size, .. } => eprintln!("  → {path} ({})", human(*size)),
            Event::Finish {
                error: Some(error), ..
            } => eprintln!("  FAILED: {error}"),
            Event::Failed(error) => eprintln!("safesync: {error}"),
            Event::Done(summary) => eprintln!(
                "{} files, {} in {}{}",
                summary.done,
                human(summary.bytes),
                duration(summary.seconds),
                if summary.cancelled {
                    " (cancelled)"
                } else {
                    ""
                }
            ),
            _ => {}
        }
        model.apply(event);
        if model.phase == Phase::Review {
            if yes {
                let _ = confirm.send(true);
            } else {
                eprintln!("Not a terminal: pass --yes to start without review.");
                let _ = confirm.send(false);
            }
        }
    }
}
