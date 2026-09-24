//! `safesync drives`: every disk on the machine, its role and the state of its
//! index; assign a role to an unmarked disk, start a scan, or search every
//! saved index for a file while the drives are in a drawer.
//!
//! The screen never writes media. Its only writes are a new sentinel (`r`, on
//! an unmarked disk) and, through `Action::Scan`, an index published by the
//! ordinary scan screen.
use crate::{
    drive::{Drive, Role},
    drives::{Inventory, Marking, Relation, Row, Section, Tone, ago, date, group, time},
    engine::human,
    ui::{DIM, ERR, FREE, LABEL, NAME, OK, PCT, SPEED, WARN},
};
use anyhow::Result;
use crossterm::event::{self, Event as Input, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Wrap},
};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    time::Duration,
};

/// What the caller does after the screen closes.
pub enum Action {
    Quit,
    /// Run the scan screen on this root (`hash`: read files that have no fingerprint yet).
    Scan {
        root: PathBuf,
        hash: bool,
    },
}

const ROLES: [(Role, &str, &str); 3] = [
    (
        Role::Source,
        "source ",
        "the library. never written except its own index",
    ),
    (
        Role::Backup,
        "backup ",
        "mirror of one source. written only by `sync`",
    ),
    (
        Role::Scratch,
        "scratch",
        "working disk. written only by `fill`",
    ),
];
const SEARCH_LIMIT: usize = 500;
// SF Symbols glyphs supplied for the terminal's font fallback.
const DRIVE_LOCAL: &str = "􀤂";
const DRIVE_ALERT: &str = "􁘧";
const DRIVE_CURRENT: &str = "􀩐";
const DRIVE_ADD: &str = "􀩎";
const APP_ICON: &str = "􀊯";

fn drive_icon(row: &Row, now: u64) -> &'static str {
    // Offline records cannot establish a drive's current health or capacity.
    if !row.online() {
        return DRIVE_LOCAL;
    }
    if matches!(row.state(now).1, Tone::Warn | Tone::Err)
        || matches!(row.marking, Marking::NoIdentity)
        || matches!(&row.relation, Relation::Source { backups } if backups.is_empty())
    {
        return DRIVE_ALERT;
    }
    if row.assign_refusal().is_none() {
        return DRIVE_ADD;
    }
    if matches!(&row.relation, Relation::Backup { source_online: true, estimate: Some(estimate), .. } if estimate.actions == 0)
    {
        return DRIVE_CURRENT;
    }
    DRIVE_LOCAL
}

enum Mode {
    Table,
    Details {
        scroll: u16,
    },
    Help {
        scroll: u16,
    },
    Role {
        row: usize,
        choice: usize,
    },
    Source {
        row: usize,
        sources: Vec<usize>,
        choice: usize,
    },
    Search {
        query: String,
        selected: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Focus {
    Section(String),
    Drive(usize),
}

struct Screen {
    icons: bool,
    inventory: Inventory,
    sections: Vec<Section>,
    collapsed: HashSet<String>,
    selected: Focus,
    mode: Mode,
    /// Shown in place of the key hints until the next key: `(ok, text)`.
    status: Option<(bool, String)>,
    view_size: (u16, u16),
}

impl Screen {
    fn new() -> Self {
        Self::from_inventory(Inventory::load())
    }
    fn from_inventory(inventory: Inventory) -> Self {
        let mut screen = Self {
            icons: true,
            sections: inventory.sections(),
            inventory,
            collapsed: HashSet::from(["system".into()]),
            selected: Focus::Drive(0),
            mode: Mode::Table,
            status: None,
            view_size: (80, 20),
        };
        let items = screen.items();
        screen.selected = items
            .iter()
            .find(|i| matches!(i, Focus::Drive(_)))
            .or(items.first())
            .cloned()
            .unwrap_or(Focus::Drive(0));
        screen
    }
    fn name(&self, row: &Row) -> String {
        if self.icons {
            format!(
                "{}  {}",
                drive_icon(row, self.inventory.loaded_unix),
                row.name
            )
        } else {
            row.name.clone()
        }
    }
    fn items(&self) -> Vec<Focus> {
        let mut items = Vec::new();
        for section in &self.sections {
            items.push(Focus::Section(section.key.clone()));
            if !self.collapsed.contains(&section.key) {
                items.extend(section.rows.iter().map(|i| Focus::Drive(*i)));
            }
        }
        items
    }
    fn focus_row(&mut self, index: usize) {
        if let Some(section) = self.sections.iter().find(|s| s.rows.contains(&index)) {
            self.collapsed.remove(&section.key);
        }
        self.selected = Focus::Drive(index);
    }
    fn move_selection(&mut self, delta: isize) {
        let items = self.items();
        let current = items.iter().position(|i| *i == self.selected).unwrap_or(0);
        let next = current
            .saturating_add_signed(delta)
            .min(items.len().saturating_sub(1));
        if let Some(item) = items.get(next) {
            self.selected = item.clone();
        }
    }
    fn fold(&mut self, expand: Option<bool>) {
        let key = match &self.selected {
            Focus::Section(key) => Some(key.clone()),
            Focus::Drive(i) => self
                .sections
                .iter()
                .find(|s| s.rows.contains(i))
                .map(|s| s.key.clone()),
        };
        if let Some(key) = key {
            let open = expand.unwrap_or_else(|| self.collapsed.contains(&key));
            if open {
                self.collapsed.remove(&key);
            } else {
                self.collapsed.insert(key.clone());
                self.selected = Focus::Section(key);
            }
        }
    }
    fn reload(&mut self) {
        self.replace_inventory(Inventory::load());
    }
    fn replace_inventory(&mut self, inventory: Inventory) {
        let uuid = self.row().and_then(|r| r.uuid.clone());
        self.inventory = inventory;
        self.sections = self.inventory.sections();
        if let Some(index) = uuid.and_then(|u| {
            self.inventory
                .rows
                .iter()
                .position(|r| r.uuid.as_deref() == Some(&u))
        }) {
            self.focus_row(index);
        } else if !self.items().contains(&self.selected) || matches!(self.selected, Focus::Drive(_))
        {
            self.selected = self.items().first().cloned().unwrap_or(Focus::Drive(0));
        }
    }
    fn row(&self) -> Option<&Row> {
        self.row_index().and_then(|i| self.inventory.rows.get(i))
    }
    fn row_index(&self) -> Option<usize> {
        match self.selected {
            Focus::Drive(i) => Some(i),
            Focus::Section(_) => None,
        }
    }
    fn scroll_limit(&self) -> u16 {
        let lines = match self.mode {
            Mode::Details { .. } => details_lines(self),
            Mode::Help { .. } => help_lines(self),
            _ => return 0,
        };
        Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .line_count(self.view_size.0)
            .saturating_sub(self.view_size.1 as usize)
            .min(u16::MAX as usize) as u16
    }
    fn say(&mut self, ok: bool, text: impl Into<String>) {
        self.status = Some((ok, text.into()));
    }

    /// Write the sentinel. `source` is the row a backup mirrors.
    fn assign(&mut self, row: usize, role: Role, source: Option<usize>) {
        let Some(root) = self.inventory.rows[row].path.clone() else {
            return;
        };
        let result = (|| -> Result<Drive> {
            let source = match source {
                Some(i) => {
                    let path = self.inventory.rows[i]
                        .path
                        .as_deref()
                        .ok_or_else(|| anyhow::anyhow!("source is not mounted"))?;
                    Some(Drive::open(path)?)
                }
                None => None,
            };
            Drive::init(&root, role, source.as_ref())
        })();
        self.mode = Mode::Table;
        match result {
            Ok(drive) => {
                self.reload();
                self.say(
                    true,
                    format!(
                        "{} is now a {} drive. Next: s to scan it.",
                        drive.sentinel.name,
                        ROLES[role_index(role)].1.trim()
                    ),
                );
            }
            Err(error) => self.say(false, format!("{error:#}")),
        }
    }

    /// Returns `Some` when the screen should close.
    fn key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Option<Action> {
        self.status = None;
        let quit = code == KeyCode::Esc
            || code == KeyCode::Char('q')
            || (code == KeyCode::Char('c') && modifiers.contains(KeyModifiers::CONTROL));
        let max_scroll = self.scroll_limit();
        match &mut self.mode {
            Mode::Table => match code {
                KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
                KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
                KeyCode::PageDown => self.move_selection(10),
                KeyCode::PageUp => self.move_selection(-10),
                KeyCode::Left | KeyCode::Char('h') => self.fold(Some(false)),
                KeyCode::Right | KeyCode::Char('l') => self.fold(Some(true)),
                KeyCode::Enter | KeyCode::Char(' ')
                    if matches!(self.selected, Focus::Section(_)) =>
                {
                    self.fold(None)
                }
                KeyCode::Enter | KeyCode::Char('d') if self.row().is_some() => {
                    self.mode = Mode::Details { scroll: 0 }
                }
                KeyCode::Char('?') => self.mode = Mode::Help { scroll: 0 },
                KeyCode::Char('r') => {
                    if let Some(row) = self.row() {
                        match row.assign_refusal() {
                            None => {
                                self.mode = Mode::Role {
                                    row: self.row_index().unwrap(),
                                    choice: 0,
                                }
                            }
                            Some(why) => {
                                let text = format!("{}: {why}", row.name);
                                self.say(false, text);
                            }
                        }
                    }
                }
                KeyCode::Char('s') | KeyCode::Char('S') => {
                    if let Some(row) = self.row() {
                        match (row.scan_refusal(), &row.path) {
                            (None, Some(path)) => {
                                return Some(Action::Scan {
                                    root: path.clone(),
                                    hash: code == KeyCode::Char('S'),
                                });
                            }
                            (why, _) => {
                                let text =
                                    format!("{}: {}", row.name, why.unwrap_or("not mounted"));
                                self.say(false, text);
                            }
                        }
                    }
                }
                KeyCode::Char('/') => {
                    if self.inventory.catalog.is_empty() {
                        self.say(false, "No indexes to search yet; scan a drive first.");
                    } else {
                        self.mode = Mode::Search {
                            query: String::new(),
                            selected: 0,
                        };
                    }
                }
                KeyCode::Char('R') => {
                    self.reload();
                    self.say(true, "Reloaded.");
                }
                _ if quit => return Some(Action::Quit),
                _ => {}
            },
            Mode::Details { scroll } | Mode::Help { scroll } => match code {
                KeyCode::Down | KeyCode::Char('j') => {
                    *scroll = scroll.saturating_add(1).min(max_scroll)
                }
                KeyCode::Up | KeyCode::Char('k') => *scroll = scroll.saturating_sub(1),
                KeyCode::PageDown => *scroll = scroll.saturating_add(10).min(max_scroll),
                KeyCode::PageUp => *scroll = scroll.saturating_sub(10),
                KeyCode::Home => *scroll = 0,
                KeyCode::Char('d') | KeyCode::Char('?') | KeyCode::Enter => self.mode = Mode::Table,
                _ if quit => self.mode = Mode::Table,
                _ => {}
            },
            Mode::Role { row, choice } => match code {
                KeyCode::Down | KeyCode::Char('j') => *choice = (*choice + 1).min(ROLES.len() - 1),
                KeyCode::Up | KeyCode::Char('k') => *choice = choice.saturating_sub(1),
                KeyCode::Enter => {
                    let (row, role) = (*row, ROLES[*choice].0);
                    if role == Role::Backup {
                        let sources = self.inventory.sources();
                        if sources.is_empty() {
                            self.mode = Mode::Table;
                            self.say(
                                false,
                                "A backup mirrors one source, and no source drive is mounted.",
                            );
                        } else {
                            self.mode = Mode::Source {
                                row,
                                sources,
                                choice: 0,
                            };
                        }
                    } else {
                        self.assign(row, role, None);
                    }
                }
                _ if quit => self.mode = Mode::Table,
                _ => {}
            },
            Mode::Source {
                row,
                sources,
                choice,
            } => match code {
                KeyCode::Down | KeyCode::Char('j') => {
                    *choice = (*choice + 1).min(sources.len() - 1);
                }
                KeyCode::Up | KeyCode::Char('k') => *choice = choice.saturating_sub(1),
                KeyCode::Enter => {
                    let (row, source) = (*row, sources[*choice]);
                    self.assign(row, Role::Backup, Some(source));
                }
                _ if quit => {
                    self.mode = Mode::Role {
                        row: *row,
                        choice: role_index(Role::Backup),
                    }
                }
                _ => {}
            },
            Mode::Search { query, selected } => match code {
                KeyCode::Esc => self.mode = Mode::Table,
                KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                    return Some(Action::Quit);
                }
                KeyCode::Down => *selected += 1,
                KeyCode::Up => *selected = selected.saturating_sub(1),
                KeyCode::PageDown => *selected += 20,
                KeyCode::PageUp => *selected = selected.saturating_sub(20),
                KeyCode::Backspace => {
                    query.pop();
                    *selected = 0;
                }
                KeyCode::Char(c) => {
                    query.push(c);
                    *selected = 0;
                }
                KeyCode::Enter => {
                    // Jump to the drive that holds the highlighted file.
                    let results = self.inventory.catalog.search(query, SEARCH_LIMIT);
                    if let Some(record) =
                        results.get((*selected).min(results.len().saturating_sub(1)))
                    {
                        let row = record.row;
                        self.focus_row(row);
                        self.mode = Mode::Table;
                    }
                }
                _ => {}
            },
        }
        None
    }
}

fn role_index(role: Role) -> usize {
    ROLES.iter().position(|(r, _, _)| *r == role).unwrap_or(0)
}

fn tone(tone: Tone) -> Color {
    match tone {
        Tone::Ok => OK,
        Tone::Warn => WARN,
        Tone::Err => ERR,
        Tone::Dim => DIM,
        Tone::Neutral => LABEL,
    }
}

fn role_color(role: Option<Role>) -> Color {
    match role {
        Some(Role::Source) => PCT,
        Some(Role::Backup) => FREE,
        Some(Role::Scratch) => SPEED,
        None => DIM,
    }
}

fn short_uuid(uuid: &str) -> String {
    match (
        uuid.get(..4),
        uuid.len().checked_sub(4).and_then(|i| uuid.get(i..)),
    ) {
        (Some(head), Some(tail)) if uuid.len() > 12 => format!("{head}-…-{tail}"),
        _ => uuid.into(),
    }
}

// Keep the beginning of status text (especially its count), and measure
// terminal cells rather than bytes or Unicode scalar count.
fn shorten(text: &str, width: usize) -> String {
    if Line::from(text).width() <= width {
        return text.to_owned();
    }
    if width == 0 {
        return String::new();
    }
    let mut result = String::new();
    let mut cells = 0;
    for ch in text.chars() {
        let size = Span::raw(ch.to_string()).width();
        if cells + size > width - 1 {
            break;
        }
        result.push(ch);
        cells += size;
    }
    result.push('…');
    result
}
fn pad(text: &str, width: usize) -> String {
    let text = shorten(text, width);
    let remaining = width.saturating_sub(Line::from(text.as_str()).width());
    format!("{text}{}", " ".repeat(remaining))
}
fn right(text: &str, width: usize) -> String {
    let text = shorten(text, width);
    let remaining = width.saturating_sub(Line::from(text.as_str()).width());
    format!("{}{text}", " ".repeat(remaining))
}

fn draw(frame: &mut Frame, screen: &mut Screen) {
    let area = frame.area();
    let width = area.width as usize;
    let rows = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(3),
        Constraint::Length(2),
    ])
    .split(area);
    screen.view_size = (rows[1].width, rows[1].height);
    let max_scroll = screen.scroll_limit();
    if let Mode::Details { scroll } | Mode::Help { scroll } = &mut screen.mode {
        *scroll = (*scroll).min(max_scroll);
    }
    draw_header(frame, rows[0], screen);
    match &screen.mode {
        Mode::Table => draw_table(frame, rows[1], screen, width),
        Mode::Details { scroll } => draw_details(frame, rows[1], screen, *scroll),
        Mode::Help { scroll } => {
            frame.render_widget(
                Paragraph::new(help_lines(screen))
                    .wrap(Wrap { trim: false })
                    .scroll((*scroll, 0)),
                rows[1],
            );
        }
        Mode::Role { row, choice } => draw_role(frame, rows[1], screen, *row, *choice),
        Mode::Source {
            row,
            sources,
            choice,
        } => draw_source(frame, rows[1], screen, *row, sources, *choice),
        Mode::Search { query, selected } => {
            draw_search(frame, rows[1], screen, query, *selected, width)
        }
    }
    draw_footer(frame, rows[2], screen, width);
}

const SELECTED_BG: Color = Color::Rgb(0x32, 0x25, 0x52);
const HEADER_BG: Color = Color::Rgb(0x33, 0x25, 0x5C);
const BADGE_FG: Color = Color::Rgb(0x1B, 0x10, 0x30);
const BAR_BG: Color = Color::Rgb(0x2A, 0x1F, 0x4A);

fn draw_header(frame: &mut Frame, area: Rect, screen: &Screen) {
    let groups = screen
        .sections
        .iter()
        .filter(|s| s.key.starts_with("source:"))
        .count();
    let unassigned = screen
        .inventory
        .rows
        .iter()
        .filter(|r| matches!(r.marking, Marking::Unmarked))
        .count();
    let offline = screen.inventory.rows.iter().filter(|r| !r.online()).count();
    let mut summary = format!(
        "  {groups} sync group{} · {unassigned} unassigned",
        if groups == 1 { "" } else { "s" }
    );
    if offline > 0 {
        summary.push_str(&format!(" · {offline} offline"));
    }
    if !screen.inventory.warnings.is_empty() {
        summary.push_str(&format!(
            " · {} warnings (? to read)",
            screen.inventory.warnings.len()
        ));
    }
    let badge = if screen.icons {
        format!(" {APP_ICON}  safesync ")
    } else {
        " safesync ".into()
    };
    let help = if area.width as usize >= Line::from(badge.as_str()).width() + 9 {
        " ? help  "
    } else {
        ""
    };
    let gap = (area.width as usize).saturating_sub(Line::from(badge.as_str()).width() + help.len());
    let spans = vec![
        Span::styled(badge, Style::default().bg(FREE).fg(BADGE_FG).bold()),
        Span::raw(" ".repeat(gap)),
        Span::styled(help, Style::default().fg(FREE).bold()),
    ];
    frame.render_widget(
        Paragraph::new(Line::from(spans)).style(Style::default().bg(HEADER_BG).fg(LABEL)),
        Rect {
            height: area.height.min(1),
            ..area
        },
    );
    if area.height > 1 {
        frame.render_widget(
            Paragraph::new(label(&summary)),
            Rect {
                y: area.y + 1,
                height: area.height - 1,
                ..area
            },
        );
    }
}

fn selection_style(selected: bool) -> Style {
    if selected {
        Style::default().bg(SELECTED_BG).bold()
    } else {
        Style::default()
    }
}

fn highlight(mut line: Line<'static>, selected: bool, width: usize) -> Line<'static> {
    if selected {
        line.spans
            .push(Span::raw(" ".repeat(width.saturating_sub(line.width()))));
    }
    line.style(selection_style(selected))
}

fn backup_status(row: &Row) -> Option<(String, Color)> {
    let Relation::Backup {
        behind,
        source_name,
        estimate,
        ..
    } = &row.relation
    else {
        return None;
    };
    if let Some(warning) = row.space_warning() {
        return Some((warning, WARN));
    }
    if row.online() && !row.writable {
        return Some(("Read-only volume".into(), WARN));
    }
    if let Some(estimate) = estimate {
        if estimate.actions == 0 {
            return Some(("No pending changes · saved indexes".into(), LABEL));
        }
        if *behind == Some(0) {
            return Some((
                format!(
                    "{} pending changes · saved indexes",
                    group(estimate.actions as u64)
                ),
                WARN,
            ));
        }
    }
    Some(match behind {
        Some(n) => (
            format!(
                "{} missing / size-changed · saved indexes",
                group(*n as u64)
            ),
            if *n == 0 { LABEL } else { WARN },
        ),
        None if source_name.is_none() => ("Source unknown · comparison unavailable".into(), WARN),
        None => ("Index missing · comparison unavailable".into(), WARN),
    })
}

fn draw_table(frame: &mut Frame, area: Rect, screen: &Screen, width: usize) {
    // Keep the summary anchored above the footer; short terminals devote their
    // space to navigation and expose the same information through d/Enter.
    let detail_height = if area.height >= 19 {
        10
    } else if area.height >= 14 {
        7
    } else {
        0
    };
    let parts =
        Layout::vertical([Constraint::Min(0), Constraint::Length(detail_height)]).split(area);
    let table = parts[0];
    let capacity = width >= 76;
    let role = width >= 48;
    let age = width >= 36;
    let reserved =
        4 + if role { 11 } else { 0 } + if age { 14 } else { 0 } + if capacity { 23 } else { 0 };
    let preferred_name_width = screen
        .inventory
        .rows
        .iter()
        .map(|r| Line::from(screen.name(r)).width())
        .max()
        .unwrap_or(16)
        .clamp(16, 30);
    let name_width = width
        .saturating_sub(reserved)
        .clamp(4, preferred_name_width);
    let mut heading = format!("    {}", pad("DRIVE", name_width));
    if role {
        heading.push_str("  ROLE     ");
    }
    if age {
        heading.push_str("  LAST SCAN   ");
    }
    if capacity {
        heading.push_str(&right("FREE / CAPACITY", 23));
    }
    frame.render_widget(
        Paragraph::new(label(&heading)),
        Rect {
            height: table.height.min(1),
            ..table
        },
    );

    let mut lines = Vec::new();
    let mut selected_line = 0;
    let mut selected_section = None;
    for section in &screen.sections {
        if !lines.is_empty() {
            lines.push(Line::default());
        }
        let selected = screen.selected == Focus::Section(section.key.clone());
        let collapsed = screen.collapsed.contains(&section.key);
        let title = format!(
            "{} {}{}",
            if collapsed { "▸" } else { "▾" },
            section.title,
            if collapsed {
                format!(" ({})", section.rows.len())
            } else {
                String::new()
            }
        );
        let section_line = Line::from(value(
            shorten(&title, width),
            if section.key == "system" { LABEL } else { NAME },
        ));
        if selected {
            selected_line = lines.len();
        }
        lines.push(
            highlight(section_line.clone(), selected, width)
                .style(selection_style(selected).bold()),
        );
        if collapsed {
            continue;
        }
        for i in &section.rows {
            let row = &screen.inventory.rows[*i];
            let selected = screen.selected == Focus::Drive(*i);
            if selected {
                selected_line = lines.len();
                selected_section = Some(section_line.clone());
            }
            let mut spans = vec![
                value(if selected { "  ▸ " } else { "    " }, FREE),
                value(
                    pad(&screen.name(row), name_width),
                    if matches!(row.marking, Marking::Boot) {
                        DIM
                    } else {
                        NAME
                    },
                ),
            ];
            if role {
                spans.push(value(
                    format!("  {}", pad(row.role_text(), 9)),
                    role_color(row.role()),
                ));
            }
            if age {
                spans.push(label(&format!(
                    "  {}",
                    pad(
                        &row.index
                            .as_ref()
                            .map(|i| ago(i.finished_unix, screen.inventory.loaded_unix))
                            .unwrap_or_else(|| "never".into()),
                        12
                    )
                )));
            }
            if capacity {
                spans.push(label(&right(
                    &if row.online() {
                        format!("{} / {}", human(row.free), human(row.total))
                    } else {
                        "offline".into()
                    },
                    23,
                )));
            }
            lines.push(highlight(Line::from(spans), selected, width));
            let status = if !row.online() {
                let (text, color) = backup_status(row).unwrap_or_else(|| (String::new(), LABEL));
                Some((
                    format!(
                        "Offline · saved index{}",
                        if text.is_empty() {
                            text
                        } else {
                            format!(" · {text}")
                        }
                    ),
                    color,
                ))
            } else if let Some(status) = backup_status(row) {
                Some(status)
            } else if !row.writable
                || matches!(
                    row.marking,
                    Marking::Invalid(_) | Marking::Copied(_) | Marking::NoIdentity
                )
                || (row.role().is_some() && row.index.is_none())
            {
                let (text, color) = row.state(screen.inventory.loaded_unix);
                Some((text, tone(color)))
            } else if matches!(&row.relation, Relation::Source { backups } if backups.is_empty()) {
                Some(("No linked backup".into(), WARN))
            } else {
                None
            };
            if let Some((text, color)) = status {
                lines.push(Line::from(value(
                    format!("      {}", shorten(&text, width.saturating_sub(6))),
                    color,
                )));
            }
        }
    }
    if lines.is_empty() {
        lines.push(Line::from(label("  No volumes found.")));
    }
    let height = table.height.saturating_sub(1) as usize;
    // Leave a line below the selected drive for its comparison status.
    let first = selected_line
        .saturating_sub(height.saturating_sub(2))
        .min(lines.len().saturating_sub(height));
    let mut visible: Vec<_> = lines.into_iter().skip(first).take(height).collect();
    if first > 0
        && selected_line > first
        && let Some(title) = selected_section
        && !visible.is_empty()
    {
        visible[0] = title;
    }
    frame.render_widget(
        Paragraph::new(visible),
        Rect {
            y: table.y.saturating_add(1),
            height: table.height.saturating_sub(1),
            ..table
        },
    );
    if detail_height > 0 {
        draw_summary(frame, parts[1], screen);
    }
}

fn label(text: &str) -> Span<'static> {
    Span::styled(text.to_owned(), Style::default().fg(LABEL))
}
fn dim(text: impl Into<String>) -> Span<'static> {
    Span::styled(text.into(), Style::default().fg(DIM))
}
fn value(text: impl Into<String>, color: Color) -> Span<'static> {
    Span::styled(text.into(), Style::default().fg(color))
}

fn summary_lines(screen: &Screen, row: &Row) -> Vec<Line<'static>> {
    let relation = match &row.relation {
        Relation::Backup {
            source_name,
            source_online,
            ..
        } => format!(
            " · backup of {} · source {}",
            source_name.as_deref().unwrap_or("unknown source"),
            if *source_online {
                "mounted"
            } else {
                "offline / unavailable"
            }
        ),
        Relation::Source { backups } => format!(
            " · source · {} linked backup{}",
            backups.len(),
            if backups.len() == 1 { "" } else { "s" }
        ),
        _ => format!(
            " · {}",
            if row.role() == Some(Role::Scratch) {
                "scratch drive"
            } else if row.online() {
                "mounted"
            } else {
                "offline · saved index"
            }
        ),
    };
    let mut lines = vec![Line::from(vec![
        value(screen.name(row), NAME),
        label(&relation),
    ])];
    if let Some((status, color)) = backup_status(row) {
        lines.push(Line::from(value(status, color)));
        lines.push(Line::from(label(
            "Live content not compared; using saved indexes.",
        )));
        let source_age = row
            .source_uuid()
            .and_then(|uuid| {
                screen
                    .inventory
                    .rows
                    .iter()
                    .find(|r| r.uuid.as_deref() == Some(uuid))
            })
            .and_then(|r| r.index.as_ref())
            .map(|i| ago(i.finished_unix, screen.inventory.loaded_unix))
            .unwrap_or_else(|| "unknown".into());
        let backup_age = row
            .index
            .as_ref()
            .map(|i| ago(i.finished_unix, screen.inventory.loaded_unix))
            .unwrap_or_else(|| "never".into());
        lines.push(Line::from(label(&format!(
            "Last scans: source {source_age} · backup {backup_age}"
        ))));
    } else {
        let message = match &row.marking {
            Marking::Unmarked if !row.writable => "Read-only volume · cannot assign a role".into(),
            Marking::Unmarked => "Unassigned · press r to assign a role".into(),
            Marking::Boot => "System volume · intentionally excluded".into(),
            Marking::NoIdentity => "No stable volume UUID · cannot configure this volume".into(),
            Marking::Invalid(_) => "Invalid sentinel · press d for the error".into(),
            Marking::Copied(_) => {
                "Copied sentinel · operations refused; press d for details".into()
            }
            Marking::Offline if row.role().is_none() => {
                "Role not recorded in this older index; reconnect and scan to record it.".into()
            }
            _ => row.state(screen.inventory.loaded_unix).0,
        };
        lines.push(Line::from(label(&message)));
    }
    if let Some(index) = &row.index {
        lines.push(Line::from(label(&format!(
            "{} indexed files · {} · {}",
            group(index.files as u64),
            human(index.bytes),
            if index.content_hashed {
                "all fingerprinted"
            } else {
                "fingerprints incomplete"
            }
        ))));
    }
    if let Some(path) = &row.path {
        lines.push(Line::from(label(&format!(
            "{} · {} free / {}",
            path.display(),
            human(row.free),
            human(row.total)
        ))));
    } else {
        lines.push(Line::from(label("Offline · showing the last saved index")));
    }
    lines
}

fn draw_summary(frame: &mut Frame, area: Rect, screen: &Screen) {
    let mut lines = vec![Line::from(dim("─".repeat(area.width as usize)))];
    if let Some(row) = screen.row() {
        lines.extend(summary_lines(screen, row));
    } else if let Focus::Section(key) = &screen.selected
        && let Some(section) = screen.sections.iter().find(|s| &s.key == key)
    {
        lines.push(Line::from(value(section.title.clone(), NAME)));
        let online = section
            .rows
            .iter()
            .filter(|i| screen.inventory.rows[**i].online())
            .count();
        lines.push(Line::from(label(&format!(
            "{} drives · {online} mounted · {} offline",
            section.rows.len(),
            section.rows.len() - online
        ))));
        lines.push(Line::from(label(
            "Enter to expand / collapse · ↓ to select a drive",
        )));
    }
    // Detailed text is available in a scrollable view. Keep this pane compact.
    frame.render_widget(Paragraph::new(lines), area);
}

fn details_lines(screen: &Screen) -> Vec<Line<'static>> {
    let Some(row) = screen.row() else {
        return Vec::new();
    };
    let mut lines = summary_lines(screen, row);
    lines.push(Line::default());
    lines.push(Line::from(value("Index details", NAME)));
    if let Some(uuid) = &row.uuid {
        lines.push(Line::from(label(&format!("Volume UUID: {uuid}"))));
    }
    lines.push(Line::from(label(&format!(
        "Filesystem: {}",
        row.filesystem.to_uppercase()
    ))));
    if let Marking::Invalid(error) = &row.marking {
        lines.push(Line::from(value(error.clone(), ERR)));
    }
    if let Marking::Copied(sentinel) = &row.marking {
        lines.push(Line::from(value(format!("Sentinel names {} instead of this volume. Remove .safesync/drive.toml deliberately before assigning a role.", sentinel.volume_uuid), ERR)));
    }
    if let Some(index) = &row.index {
        lines.push(Line::from(label(&format!(
            "index-{}.jsonl",
            index.generation
        ))));
        lines.push(Line::from(label(&format!(
            "Scanned {} {} → {} {}",
            date(index.started_unix),
            time(index.started_unix),
            date(index.finished_unix),
            time(index.finished_unix)
        ))));
        if row.online() {
            lines.push(Line::from(label(&format!(
                "{} generations on drive",
                index.generations
            ))));
        }
        lines.push(Line::from(label(&format!(
            "Index saved on this Mac: {}",
            if index.saved_locally { "yes" } else { "no" }
        ))));
        lines.push(Line::from(label(&format!(
            "{} fingerprints reused",
            group(index.reused_hashes)
        ))));
        let skipped: Vec<String> = [
            (index.skipped_symlinks, "symlinks"),
            (index.skipped_special, "special files"),
            (index.skipped_mounts, "mounts"),
        ]
        .into_iter()
        .filter(|(n, _)| *n > 0)
        .map(|(n, what)| format!("{n} {what}"))
        .collect();
        if !skipped.is_empty() {
            lines.push(Line::from(label(&format!(
                "Skipped: {}",
                skipped.join(" · ")
            ))));
        }
        lines.push(Line::from(label("Recorded scan exclusions:")));
        for exclusion in &index.exclusions {
            lines.push(Line::from(label(&format!("  {exclusion}"))));
        }
    } else {
        lines.push(Line::from(label("No index yet.")));
    }
    lines
}

fn draw_details(frame: &mut Frame, area: Rect, screen: &Screen, scroll: u16) {
    let paragraph = Paragraph::new(details_lines(screen)).wrap(Wrap { trim: false });
    let max = paragraph
        .line_count(area.width)
        .saturating_sub(area.height as usize)
        .min(u16::MAX as usize) as u16;
    frame.render_widget(paragraph.scroll((scroll.min(max), 0)), area);
}

fn help_lines(screen: &Screen) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(value("Drive navigation", NAME)),
        Line::from(label("↑↓ / j k   Select a drive or section")),
        Line::from(label("←→ / h l   Collapse / expand a section")),
        Line::from(label("Enter      Toggle a section or open drive details")),
        Line::from(label(
            "s          Scan metadata and reuse known fingerprints",
        )),
        Line::from(label(
            "S          Scan and fingerprint files missing a fingerprint",
        )),
        Line::from(label("r          Assign a role to an unassigned drive")),
        Line::from(label(
            "d          Index details; ↑↓ / PgUp / PgDn to scroll",
        )),
        Line::from(label(
            "/          Search all saved indexes, including offline drives",
        )),
        Line::from(label("R          Reload mounted drives and saved indexes")),
        Line::from(label(
            "q / Esc    Back from a view; quit from the drive list",
        )),
        Line::default(),
        Line::from(label(
            "Missing / size-changed counts compare saved paths and sizes.",
        )),
        Line::from(label(
            "Fingerprinted means a digest was recorded, not freshly verified.",
        )),
    ];
    lines.push(Line::from(label(
        "Sync estimates also use recorded timestamps/fingerprints; they are not live checks.",
    )));
    if screen.icons {
        lines.push(Line::default());
        lines.push(Line::from(label(&format!(
            "{DRIVE_LOCAL}  Drive   {DRIVE_ALERT}  Needs attention   {DRIVE_ADD}  Can assign a role"
        ))));
        lines.push(Line::from(label(&format!(
            "{DRIVE_CURRENT}  No pending backup changes in saved indexes (both drives mounted)"
        ))));
        lines.push(Line::from(label(
            "Use --no-icons if your terminal font cannot display these symbols.",
        )));
    }
    for warning in &screen.inventory.warnings {
        lines.push(Line::from(value(warning.clone(), WARN)));
    }
    lines
}

fn draw_role(frame: &mut Frame, area: Rect, screen: &Screen, row: usize, choice: usize) {
    let row = &screen.inventory.rows[row];
    let mut lines = vec![
        Line::default(),
        Line::from(vec![
            Span::raw("  "),
            label("Assign a role to  "),
            Span::styled(
                screen.name(row),
                Style::default().fg(NAME).add_modifier(Modifier::BOLD),
            ),
            dim(format!(
                "  ({}, {}, {})",
                row.path
                    .as_deref()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                row.filesystem.to_uppercase(),
                human(row.total)
            )),
        ]),
        Line::default(),
    ];
    for (i, (role, name, blurb)) in ROLES.iter().enumerate() {
        let selected = i == choice;
        lines.push(Line::from(vec![
            Span::styled(
                if selected { "  ▸ " } else { "    " },
                Style::default().fg(PCT),
            ),
            Span::styled(
                format!("{name}   "),
                Style::default()
                    .fg(role_color(Some(*role)))
                    .add_modifier(if selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
            value(*blurb, if selected { NAME } else { LABEL }),
        ]));
    }
    lines.push(Line::default());
    lines.push(Line::from(dim(format!(
        "  Writes .safesync/drive.toml pinned to volume {}. Nothing else on the disk is touched.",
        row.uuid.as_deref().map(short_uuid).unwrap_or_default()
    ))));
    frame.render_widget(Paragraph::new(lines), area);
}

fn draw_source(
    frame: &mut Frame,
    area: Rect,
    screen: &Screen,
    row: usize,
    sources: &[usize],
    choice: usize,
) {
    let row = &screen.inventory.rows[row];
    let mut lines = vec![
        Line::default(),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(
                screen.name(row),
                Style::default().fg(NAME).add_modifier(Modifier::BOLD),
            ),
            label("  →  backup of"),
        ]),
        Line::default(),
    ];
    for (i, source) in sources.iter().enumerate() {
        let source = &screen.inventory.rows[*source];
        let selected = i == choice;
        let same_volume = source.uuid.is_some() && source.uuid == row.uuid;
        let mut spans = vec![
            Span::styled(
                if selected { "  ▸ " } else { "    " },
                Style::default().fg(PCT),
            ),
            Span::styled(
                pad(&screen.name(source), 22),
                Style::default()
                    .fg(if selected { NAME } else { LABEL })
                    .add_modifier(if selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
            value("source   ", PCT),
        ];
        match &source.index {
            Some(index) => spans.push(label(&format!(
                "{} files   {}   index {}",
                group(index.files as u64),
                human(index.bytes),
                date(index.finished_unix)
            ))),
            None => spans.push(dim("no index yet")),
        }
        if same_volume {
            spans.push(value("   same volume: refused", ERR));
        }
        lines.push(Line::from(spans));
    }
    lines.push(Line::default());
    lines.push(Line::from(dim(
        "  A backup must live on a different volume from its source. Only `sync` from that source will ever write to it.",
    )));
    frame.render_widget(Paragraph::new(lines), area);
}

fn draw_search(
    frame: &mut Frame,
    area: Rect,
    screen: &Screen,
    query: &str,
    selected: usize,
    width: usize,
) {
    let results = screen.inventory.catalog.search(query, SEARCH_LIMIT);
    let selected = selected.min(results.len().saturating_sub(1));
    let mut lines = vec![
        Line::from(vec![
            Span::raw("  "),
            label("search  "),
            Span::styled(
                format!("{query}▏"),
                Style::default().fg(NAME).add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(dim(if query.trim().is_empty() {
            format!(
                "  Type part of a file name or path. Every saved index is searched — {} files across {} drives, mounted or not.",
                group(screen.inventory.catalog.len() as u64),
                screen
                    .inventory
                    .rows
                    .iter()
                    .filter(|r| r.index.is_some())
                    .count()
            )
        } else if results.is_empty() {
            "  Nothing recorded under that name. Indexes describe what was there at scan time, not the disk right now.".into()
        } else if results.len() >= SEARCH_LIMIT {
            format!("  first {SEARCH_LIMIT} matches — keep typing to narrow")
        } else {
            format!(
                "  {} match{}",
                results.len(),
                if results.len() == 1 { "" } else { "es" }
            )
        })),
        Line::default(),
    ];
    let height = (area.height as usize).saturating_sub(lines.len());
    let first = selected
        .saturating_sub(height.saturating_sub(1))
        .min(results.len().saturating_sub(height));
    let name_width = results
        .iter()
        .map(|r| Line::from(screen.name(&screen.inventory.rows[r.row])).width())
        .max()
        .unwrap_or(4)
        .clamp(4, 24);
    for (i, record) in results.iter().enumerate().skip(first).take(height) {
        let row = &screen.inventory.rows[record.row];
        let is_selected = i == selected;
        let online = row.online();
        let fixed = 2 + name_width + 2 + 9 + 2 + 8 + 2;
        let path_width = width.saturating_sub(fixed).max(10);
        lines.push(Line::from(vec![
            Span::styled(
                if is_selected { "▸ " } else { "  " },
                Style::default().fg(PCT),
            ),
            Span::styled(
                pad(&screen.name(row), name_width),
                Style::default()
                    .fg(if online { role_color(row.role()) } else { DIM })
                    .add_modifier(if is_selected {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
            Span::raw("  "),
            value(
                pad(if online { "mounted" } else { "offline" }, 9),
                if online { OK } else { DIM },
            ),
            Span::raw("  "),
            value(right(&human(record.size), 8), LABEL),
            Span::raw("  "),
            Span::styled(
                shorten(&record.path, path_width),
                Style::default().fg(if is_selected { NAME } else { LABEL }),
            ),
        ]));
    }
    frame.render_widget(Paragraph::new(lines), area);
}

fn draw_footer(frame: &mut Frame, area: Rect, screen: &Screen, width: usize) {
    let keys = match &screen.mode {
        Mode::Table => {
            let action = match screen.row() {
                Some(row) if row.assign_refusal().is_none() => "r Assign role · d Details",
                Some(row) if row.scan_refusal().is_none() => "s Scan · S Fingerprint · d Details",
                Some(_) => "d Details",
                None => "Enter Expand/collapse",
            };
            format!("{action}   / Search   ? Help   q Quit")
        }
        Mode::Details { .. } => "↑↓ Scroll   PgUp/PgDn Page   Home Top   d/Esc Back".into(),
        Mode::Help { .. } => "↑↓ Scroll   ?/Esc Back".into(),
        Mode::Role { .. } => "↑↓ Choose   Enter Confirm   Esc Cancel".into(),
        Mode::Source { .. } => "↑↓ Choose   Enter Confirm   Esc Back".into(),
        Mode::Search { .. } => "Type to search   ↑↓ Select   Enter Go to drive   Esc Back".into(),
    };
    let first = match &screen.status {
        Some((ok, text)) => value(shorten(text, width), if *ok { OK } else { ERR }),
        None => dim(if matches!(screen.mode, Mode::Table) {
            "↑↓ Select   ←→ Fold   R Reload".into()
        } else {
            "─".repeat(width)
        }),
    };
    let keys = if width < 60 && !matches!(screen.mode, Mode::Table) {
        match screen.mode {
            Mode::Details { .. } | Mode::Help { .. } => "↑↓ Scroll  Esc Back",
            Mode::Role { .. } | Mode::Source { .. } => "↑↓ Choose  Enter Confirm  Esc Back",
            Mode::Search { .. } => "↑↓ Select  Enter Open  Esc Back",
            Mode::Table => unreachable!(),
        }
        .to_owned()
    } else if matches!(screen.mode, Mode::Table) && width < 40 {
        "? Help  q Quit".to_owned()
    } else if matches!(screen.mode, Mode::Table) && width < 76 {
        "Enter Open  / Find  ? Help  q Quit".to_owned()
    } else {
        keys
    };
    frame.render_widget(
        Paragraph::new(Line::from(first)),
        Rect {
            height: area.height.min(1),
            ..area
        },
    );
    if area.height > 1 {
        let spans: Vec<_> = keys
            .split_inclusive(' ')
            .map(|part| {
                let shortcut = matches!(
                    part.trim(),
                    "s" | "S"
                        | "d"
                        | "r"
                        | "R"
                        | "/"
                        | "?"
                        | "q"
                        | "↑↓"
                        | "←→"
                        | "Enter"
                        | "Esc"
                        | "d/Esc"
                        | "?/Esc"
                        | "PgUp/PgDn"
                        | "Home"
                );
                Span::styled(
                    part,
                    Style::default().fg(if shortcut { FREE } else { LABEL }),
                )
            })
            .collect();
        frame.render_widget(
            Paragraph::new(Line::from(spans)).style(Style::default().bg(BAR_BG)),
            Rect {
                y: area.y + 1,
                height: 1,
                ..area
            },
        );
    }
}

/// The interactive screen. Returns when the user quits or asks for a scan.
/// `focus` is the mount point to select first, when it is still there.
pub fn run(focus: Option<&Path>, icons: bool) -> Result<Action> {
    let mut screen = Screen::new();
    screen.icons = icons;
    if let Some(i) = focus.and_then(|f| {
        screen
            .inventory
            .rows
            .iter()
            .position(|r| r.path.as_deref() == Some(f))
    }) {
        screen.focus_row(i);
    }
    let mut terminal = ratatui::init();
    let result = (|| -> Result<Action> {
        // A previous screen may have left its frame in the alternate buffer;
        // a fresh Terminal only paints non-blank cells, so wipe it first.
        crossterm::execute!(
            std::io::stdout(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )?;
        loop {
            terminal.draw(|frame| draw(frame, &mut screen))?;
            if event::poll(Duration::from_millis(250))?
                && let Input::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
                && let Some(action) = screen.key(key.code, key.modifiers)
            {
                return Ok(action);
            }
        }
    })();
    ratatui::restore();
    result
}

/// The table as plain text, for a pipe or a script.
pub fn print(inventory: &Inventory) {
    let (headings, cells) = inventory.table();
    let widths: Vec<usize> = (0..headings.len())
        .map(|c| {
            cells
                .iter()
                .map(|row| row[c].chars().count())
                .chain(std::iter::once(headings[c].len()))
                .max()
                .unwrap_or(0)
        })
        .collect();
    let line = |row: &[String]| {
        row.iter()
            .enumerate()
            .map(|(c, text)| {
                if c + 1 == row.len() {
                    text.clone()
                } else if matches!(c, 3 | 4 | 6 | 7) {
                    right(text, widths[c])
                } else {
                    pad(text, widths[c])
                }
            })
            .collect::<Vec<_>>()
            .join("  ")
    };
    println!(
        "{}",
        line(&headings.iter().map(|h| h.to_string()).collect::<Vec<_>>())
    );
    for row in &cells {
        println!("{}", line(row));
    }
    for warning in &inventory.warnings {
        eprintln!("safesync: {warning}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        drive::{Extras, Sentinel},
        drives::{Catalog, IndexStats},
        manifest::RecordedDrive,
    };
    use ratatui::{Terminal, backend::TestBackend};

    fn drive(name: &str, uuid: &str, role: Option<Role>, source: Option<&str>) -> Row {
        Row {
            name: name.into(),
            uuid: Some(uuid.into()),
            path: Some(PathBuf::from(format!("/Volumes/{name}"))),
            filesystem: "apfs".into(),
            total: 1_100_000_000,
            free: 784_200_000,
            writable: true,
            marking: role
                .map(|role| {
                    Marking::Valid(Sentinel {
                        role,
                        name: name.into(),
                        volume_uuid: uuid.into(),
                        source_uuid: source.map(str::to_owned),
                        exclude: vec![],
                        extras: Extras::Keep,
                    })
                })
                .unwrap_or(Marking::Unmarked),
            recorded: None,
            index: role.map(|_| IndexStats {
                generation: "123-1-1".into(),
                files: 7,
                bytes: 285_200_000,
                started_unix: 100,
                finished_unix: 200,
                content_hashed: true,
                reused_hashes: 0,
                skipped_symlinks: 0,
                skipped_special: 0,
                skipped_mounts: 0,
                exclusions: vec!["Any directory or file named .safesync".into()],
                generations: 1,
                saved_locally: true,
            }),
            relation: match role {
                Some(Role::Source) => Relation::Source {
                    backups: vec!["DemoBackup".into()],
                },
                Some(Role::Backup) => Relation::Backup {
                    source_name: Some("DemoTower".into()),
                    source_online: true,
                    behind: Some(0),
                    estimate: Some(crate::drives::SyncEstimate {
                        actions: 0,
                        transfer_bytes: 0,
                    }),
                },
                _ => Relation::None,
            },
        }
    }
    fn inventory() -> Inventory {
        let mut boot = drive("Macintosh HD", "boot", None, None);
        boot.marking = Marking::Boot;
        Inventory {
            rows: vec![
                boot,
                drive("DemoBackup", "backup", Some(Role::Backup), Some("source")),
                drive("DemoTower", "source", Some(Role::Source), None),
                drive("Tower", "tower", None, None),
                drive("Tower Backup", "tower-backup", None, None),
            ],
            catalog: Catalog::new(),
            loaded_unix: 54_200,
            warnings: vec![],
        }
    }
    fn key(screen: &mut Screen, key: KeyCode) {
        screen.key(key, KeyModifiers::NONE);
    }

    #[test]
    fn icons_prioritize_alerts_and_never_treat_offline_or_unknown_state_as_current() {
        let mut inv = inventory();
        let now = inv.loaded_unix;
        assert_eq!(drive_icon(&inv.rows[0], now), DRIVE_LOCAL);
        assert_eq!(drive_icon(&inv.rows[1], now), DRIVE_CURRENT);
        assert_eq!(drive_icon(&inv.rows[2], now), DRIVE_LOCAL);
        assert_eq!(drive_icon(&inv.rows[3], now), DRIVE_ADD);
        inv.rows[3].writable = false;
        assert_eq!(drive_icon(&inv.rows[3], now), DRIVE_ALERT);
        let backup = &mut inv.rows[1];
        if let Relation::Backup { estimate, .. } = &mut backup.relation {
            *estimate = Some(crate::drives::SyncEstimate {
                actions: 1,
                transfer_bytes: backup.free + 1,
            });
        }
        assert_eq!(drive_icon(backup, now), DRIVE_ALERT);
        assert!(
            backup_status(backup)
                .unwrap()
                .0
                .contains("Not enough space")
        );
        // Matching paths/sizes alone cannot give a check mark if a fingerprint
        // or timestamp difference still requires a replacement.
        backup.free = u64::MAX;
        assert_eq!(drive_icon(backup, now), DRIVE_ALERT);
        assert!(backup_status(backup).unwrap().0.contains("pending changes"));
        backup.path = None;
        backup.marking = Marking::Offline;
        assert_eq!(drive_icon(backup, now), DRIVE_LOCAL);
        assert!(backup.space_warning().is_none());
        backup.path = Some("/Volumes/DemoBackup".into());
        backup.marking = Marking::Invalid("broken sentinel".into());
        assert_eq!(drive_icon(backup, now), DRIVE_ALERT);
    }

    #[test]
    fn no_icons_removes_symbols_from_the_list_details_and_help() {
        let mut screen = Screen::from_inventory(inventory());
        screen.focus_row(1);
        let (text, _) = render(&mut screen, 100, 30);
        for icon in [DRIVE_LOCAL, DRIVE_CURRENT, DRIVE_ADD] {
            assert!(text.contains(icon));
        }
        screen.icons = false;
        for mode in [
            Mode::Table,
            Mode::Details { scroll: 0 },
            Mode::Help { scroll: 0 },
            Mode::Role { row: 3, choice: 0 },
            Mode::Source {
                row: 3,
                sources: vec![2],
                choice: 0,
            },
        ] {
            screen.mode = mode;
            let (text, _) = render(&mut screen, 100, 40);
            for icon in [APP_ICON, DRIVE_LOCAL, DRIVE_ALERT, DRIVE_CURRENT, DRIVE_ADD] {
                assert!(!text.contains(icon));
            }
        }
    }
    fn render(screen: &mut Screen, width: u16, height: u16) -> (String, ratatui::buffer::Buffer) {
        let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
        terminal.draw(|frame| draw(frame, screen)).unwrap();
        let buffer = terminal.backend().buffer().clone();
        let text = buffer
            .content
            .chunks(width.max(1) as usize)
            .map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n");
        (text, buffer)
    }
    #[test]
    fn groups_use_uuid_links_not_names_and_keep_offline_members() {
        let mut inv = inventory();
        let mut offline = drive("Cold backup", "cold", Some(Role::Backup), Some("source"));
        offline.path = None;
        offline.marking = Marking::Offline;
        offline.recorded = Some(RecordedDrive {
            role: Role::Backup,
            source_uuid: Some("source".into()),
        });
        inv.rows.push(offline);
        inv.rows
            .push(drive("DemoTower", "other-source", Some(Role::Source), None));
        inv.rows.push(drive(
            "Other backup",
            "other-backup",
            Some(Role::Backup),
            Some("other-source"),
        ));
        let sections = inv.sections();
        let group = sections.iter().find(|s| s.key == "source:source").unwrap();
        assert_eq!(group.title, "Sync group: DemoTower");
        assert_eq!(group.rows, [2, 5, 1]);
        assert_eq!(
            sections
                .iter()
                .find(|s| s.key == "source:other-source")
                .unwrap()
                .rows,
            [6, 7]
        );
        assert_eq!(
            sections
                .iter()
                .find(|s| s.key == "unassigned")
                .unwrap()
                .rows,
            [3, 4]
        );
        assert_eq!(inv.rows[5].role(), Some(Role::Backup));
        assert!(inv.rows[5].scan_refusal().is_some());
        assert!(inv.rows[5].assign_refusal().is_some());
        assert!(inv.rows[5].sentinel().is_none());
        let mut members: Vec<_> = sections.iter().flat_map(|s| s.rows.clone()).collect();
        members.sort();
        assert_eq!(members, (0..inv.rows.len()).collect::<Vec<_>>());
    }
    #[test]
    fn missing_sources_and_legacy_offline_indexes_are_not_unassigned() {
        let mut inv = inventory();
        inv.rows.remove(2);
        let mut legacy = drive("Legacy", "legacy", None, None);
        legacy.path = None;
        legacy.marking = Marking::Offline;
        inv.rows.push(legacy);
        let sections = inv.sections();
        let group = sections.iter().find(|s| s.key == "source:source").unwrap();
        assert!(group.title.contains("Unknown source"));
        assert_eq!(group.rows, [1]);
        assert_eq!(
            sections.iter().find(|s| s.key == "offline").unwrap().rows,
            [4]
        );
    }
    #[test]
    fn navigation_uses_visible_order_and_folded_sections_never_scan() {
        let mut screen = Screen::from_inventory(inventory());
        assert_eq!(screen.selected, Focus::Drive(2));
        key(&mut screen, KeyCode::Down);
        assert_eq!(screen.selected, Focus::Drive(1));
        assert!(
            matches!(screen.key(KeyCode::Char('s'), KeyModifiers::NONE), Some(Action::Scan { root, hash: false }) if root.ends_with("DemoBackup"))
        );
        key(&mut screen, KeyCode::Left);
        assert_eq!(screen.selected, Focus::Section("source:source".into()));
        assert!(screen.key(KeyCode::Char('s'), KeyModifiers::NONE).is_none());
        key(&mut screen, KeyCode::Down);
        assert_eq!(screen.selected, Focus::Section("unassigned".into()));
        key(&mut screen, KeyCode::Up);
        key(&mut screen, KeyCode::Enter);
        key(&mut screen, KeyCode::Down);
        assert_eq!(screen.selected, Focus::Drive(2));
        assert!(
            !screen.items().contains(&Focus::Drive(0)),
            "system starts collapsed"
        );
    }
    #[test]
    fn search_reveals_a_drive_inside_a_collapsed_group() {
        let mut screen = Screen::from_inventory(inventory());
        let root = std::env::temp_dir().join(crate::manifest::generation());
        std::fs::create_dir(&root).unwrap();
        std::fs::write(root.join("clip.mov"), b"video").unwrap();
        let manifest = crate::scan::scan(
            &root,
            crate::filesystem::Volume {
                uuid: "backup".into(),
                name: "DemoBackup".into(),
                filesystem: "apfs".into(),
            },
            false,
            |_| {},
        )
        .unwrap();
        screen.inventory.catalog.add(1, &manifest);
        std::fs::remove_dir_all(root).unwrap();
        screen.collapsed.insert("source:source".into());
        key(&mut screen, KeyCode::Char('/'));
        for ch in "clip".chars() {
            key(&mut screen, KeyCode::Char(ch));
        }
        key(&mut screen, KeyCode::Enter);
        assert_eq!(screen.selected, Focus::Drive(1));
        assert!(screen.items().contains(&Focus::Drive(1)));
        assert!(matches!(screen.mode, Mode::Table));
    }
    #[test]
    fn reload_tracks_uuid_through_reorder_and_renamed_group() {
        let mut screen = Screen::from_inventory(inventory());
        screen.focus_row(1);
        let mut inv = inventory();
        inv.rows[2].name = "Renamed source".into();
        inv.rows.swap(1, 4);
        screen.replace_inventory(inv);
        assert_eq!(screen.selected, Focus::Drive(4));
        assert_eq!(screen.row().unwrap().uuid.as_deref(), Some("backup"));
        assert!(
            screen
                .sections
                .iter()
                .any(|s| s.title == "Sync group: Renamed source")
        );
    }
    #[test]
    fn renderer_preserves_selected_row_footer_and_honest_status_on_resize() {
        let mut screen = Screen::from_inventory(inventory());
        screen.focus_row(1);
        for (width, height) in [(120, 30), (80, 24), (60, 18), (40, 12), (20, 8), (1, 1)] {
            let (text, buffer) = render(&mut screen, width, height);
            assert!(!text.contains("in sync"));
            assert!(!text.contains("nothing to copy"));
            if width >= 40 {
                assert!(text.contains("DemoBackup"), "{width}x{height}\n{text}");
                assert!(text.contains("q Quit"), "{width}x{height}\n{text}");
                assert!(buffer.content.iter().any(|cell| cell.bg == SELECTED_BG));
                let selected = buffer
                    .content
                    .chunks(width as usize)
                    .find(|row| row.iter().any(|cell| cell.bg == SELECTED_BG))
                    .unwrap();
                assert!(
                    selected.iter().all(|cell| cell.bg == SELECTED_BG),
                    "selection must span the row"
                );
            }
            if width >= 80 {
                assert!(text.contains("saved indexes"));
                assert!(text.contains("content not compared"));
                assert!(!text.contains("index-123"));
                assert!(!text.contains("Macintosh HD"));
            }
        }
    }
    #[test]
    fn long_lists_keep_the_selection_visible_and_details_scroll_to_the_end() {
        let mut inv = inventory();
        for i in 0..30 {
            inv.rows.push(drive(
                &format!("Spare {i:02}"),
                &format!("spare-{i}"),
                None,
                None,
            ));
        }
        let mut screen = Screen::from_inventory(inv);
        for _ in 0..100 {
            let (text, _) = render(&mut screen, 80, 16);
            if let Some(row) = screen.row() {
                assert!(text.contains(&row.name), "{}\n{text}", row.name);
            }
            key(&mut screen, KeyCode::Down);
        }
        screen.focus_row(1);
        key(&mut screen, KeyCode::Char('d'));
        render(&mut screen, 40, 12);
        for _ in 0..100 {
            key(&mut screen, KeyCode::PageDown);
        }
        let (text, _) = render(&mut screen, 40, 12);
        assert!(text.contains(".safesync"), "{text}");
        assert!(matches!(screen.mode, Mode::Details { scroll } if scroll == screen.scroll_limit()));
        key(&mut screen, KeyCode::Esc);
        assert!(matches!(screen.mode, Mode::Table));
    }
}
