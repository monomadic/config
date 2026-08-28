//! Drawing (SPEC §7).
//!
//! Two-column labels and values at width, stacking below it. The thumbnail band
//! is the first thing dropped when the terminal is short, because the fields are
//! the point and the picture is not.

use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use ratatui_image::{protocol::StatefulProtocol, StatefulImage};

use crate::model::value::{Agg, Value};
use crate::ui::app::{App, Mode};
use crate::tags::plan::FilePlan;
use crate::ui::edit::{stars_glyphs, Validation};

const LABEL_COLS: u16 = 14;

/// Cells are about twice as tall as they are wide, so an image of pixel aspect
/// `a` needs `2 * rows * a` columns to keep its proportions. Sizing the band
/// this way is what lets a portrait clip render as a portrait picture instead
/// of a three-column sliver.
fn thumb_cols(rows: u16, aspect: f32) -> u16 {
    ((2.0 * rows as f32 * aspect).round() as u16).clamp(4, 40)
}

/// A portrait picture earns a taller band; a landscape one does not need it.
fn header_rows(area_h: u16, aspect: Option<f32>) -> u16 {
    if area_h < 20 {
        return 0;
    }
    match aspect {
        Some(a) if a < 0.95 => 6.max((area_h / 3).min(14)),
        _ => 6,
    }
}

fn dim() -> Style {
    Style::default().fg(Color::DarkGray)
}
fn label_style() -> Style {
    Style::default().fg(Color::Yellow)
}

pub fn draw(f: &mut Frame, app: &App, proto: Option<&mut StatefulProtocol>) {
    let area = f.area();
    // Header is dropped first on a short terminal; the fields survive longest.
    let header_h = header_rows(area.height, app.thumb_aspect);
    let chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(header_h),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .split(area);

    draw_title_bar(f, chunks[0], app);
    // The confirmation takes the whole frame: it is the last chance to read
    // what is about to happen, and it should not compete with the form.
    if let Some(plans) = &app.pending {
        draw_confirm(f, area, app, plans);
        return;
    }
    if let Some(results) = &app.results {
        draw_results(f, area, results);
        return;
    }
    if header_h > 0 {
        if app.inspector {
            draw_inspector(f, chunks[1], app);
        } else {
            draw_header(f, chunks[1], app, proto);
        }
    }
    draw_fields(f, chunks[2], app);
    draw_status(f, chunks[3], app);
}

fn draw_title_bar(f: &mut Frame, area: Rect, app: &App) {
    let view = match app.view {
        Some(i) => format!("file {}/{}", i + 1, app.files.len()),
        None => format!("{} file{}", app.files.len(), if app.files.len() == 1 { "" } else { "s" }),
    };
    let left = Span::styled(" tagform ", Style::default().add_modifier(Modifier::BOLD));
    let custom = if app.n_custom > 0 {
        format!(" · {} custom", app.n_custom)
    } else {
        String::new()
    };
    let mode = if app.mode == Mode::Edit { "EDIT" } else { "SELECT" };
    let right = Span::styled(format!("{view}{custom} · mdta · {mode} "), dim());
    let pad = (area.width as usize)
        .saturating_sub(left.content.len() + right.content.len());
    f.render_widget(
        Paragraph::new(Line::from(vec![left, Span::raw(" ".repeat(pad)), right])),
        area,
    );
}

fn draw_header(f: &mut Frame, area: Rect, app: &App, proto: Option<&mut StatefulProtocol>) {
    let idx = app.current_file();
    let Some(file) = app.files.get(idx) else { return };

    let want = app.thumb_aspect.map(|a| thumb_cols(area.height, a)).unwrap_or(0);
    let has_thumb = proto.is_some() && want > 0 && area.width > want + 20;
    let cols = if has_thumb {
        Layout::horizontal([Constraint::Length(want), Constraint::Min(10)]).split(area)
    } else {
        Layout::horizontal([Constraint::Length(0), Constraint::Min(10)]).split(area)
    };

    if has_thumb {
        if let Some(p) = proto {
            f.render_stateful_widget(StatefulImage::default(), cols[0], p);
        }
    }

    let name = file
        .path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    let dir = file
        .path
        .parent()
        .map(|d| d.to_string_lossy().to_string())
        .unwrap_or_default();
    let summary = app.media.get(idx).map(|m| m.summary()).unwrap_or_default();

    let lines = vec![
        Line::from(Span::styled(
            name,
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            if summary.is_empty() { "probing…".into() } else { summary },
            dim(),
        )),
        Line::from(Span::styled(dir, dim())),
    ];
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), cols[1]);
}

/// The answer to "what does ‹multiple› actually contain" -- the thing the old
/// fzf-based tagger could only show in a preview pane.
fn draw_inspector(f: &mut Frame, area: Rect, app: &App) {
    let Some(row) = app.rows.get(app.focus) else { return };
    let mut lines = vec![Line::from(vec![
        Span::styled(format!("{} ", row.label), label_style()),
        Span::styled("per file", dim()),
    ])];

    match &row.agg {
        Agg::Mixed { values } => {
            for (i, v) in values.iter().enumerate() {
                let name = app.files[i]
                    .path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default();
                let shown = match v {
                    Some(Value::Text(s)) => s.clone(),
                    Some(Value::List(l)) => l.join(" · "),
                    None => "—".into(),
                };
                lines.push(Line::from(vec![
                    Span::styled(format!("{shown:<28} "), Style::default()),
                    Span::styled(name, dim()),
                ]));
            }
        }
        Agg::Same { .. } => lines.push(Line::from(Span::styled(
            "identical in every file",
            dim(),
        ))),
        Agg::Absent => lines.push(Line::from(Span::styled("present in no file", dim()))),
    }
    f.render_widget(Paragraph::new(lines), area);
}

fn draw_fields(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::TOP).border_style(dim());
    let inner = block.inner(area);
    f.render_widget(block, area);

    let height = inner.height as usize;
    // Keep the focused row on screen without re-centring on every move.
    let start = if app.focus >= height { app.focus + 1 - height } else { 0 };
    let value_col = inner.x + 1 + LABEL_COLS;
    let mut cursor: Option<(u16, u16)> = None;

    let mut lines = Vec::new();
    for (i, row) in app.rows.iter().enumerate().skip(start).take(height) {
        let focused = i == app.focus;
        let editing = focused && app.mode == Mode::Edit;
        let staged = app.is_staged(&row.key);
        let is_custom = row.def.is_none();

        // A staged row carries a dot so a change is visible even when the row
        // is scrolled away from the cursor.
        // The caret says "this field is open"; the bar only says "you are here".
        let marker = if editing {
            "▶"
        } else if focused {
            "▍"
        } else if staged {
            "●"
        } else {
            " "
        };
        let marker_style = if editing {
            Style::default().fg(Color::Cyan)
        } else if staged {
            Style::default().fg(Color::Yellow)
        } else {
            Style::default().fg(Color::Green)
        };

        let (text, style) = if editing {
            let (t, cur) = app
                .editor
                .as_ref()
                .map(|e| e.display())
                .unwrap_or_else(|| (String::new(), None));
            if let Some(c) = cur {
                let row_y = inner.y + (i - start) as u16;
                let x = value_col + c as u16;
                if x < inner.right() {
                    cursor = Some((x, row_y));
                }
            }
            let st = match app.validation() {
                Validation::Error(_) => Style::default().fg(Color::Red),
                Validation::Warn(_) => Style::default().fg(Color::Yellow),
                Validation::Ok if staged => Style::default().fg(Color::Green),
                Validation::Ok => Style::default(),
            };
            (t, st)
        } else {
            match display_row(app, row) {
                Some(v) if staged => (v, Style::default().fg(Color::Green)),
                Some(v) if row.is_mixed() => (
                    v,
                    Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC),
                ),
                Some(v) => (v, Style::default()),
                None => ("—".into(), dim()),
            }
        };

        lines.push(Line::from(vec![
            Span::styled(marker, marker_style),
            Span::styled(
                format!("{:<w$}", row.label, w = LABEL_COLS as usize),
                match (focused, is_custom) {
                    (true, false) if editing => {
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                    }
                    (true, false) => label_style().add_modifier(Modifier::BOLD),
                    (false, false) => label_style(),
                    (true, true) => dim().add_modifier(Modifier::BOLD),
                    (false, true) => dim(),
                },
            ),
            Span::styled(text, style),
        ]));
    }
    f.render_widget(Paragraph::new(lines), inner);
    if let Some((x, y)) = cursor {
        f.set_cursor_position((x, y));
    }
}

/// An unfocused row shows the staged edit if there is one, else what is on disk.
fn display_row(app: &App, row: &crate::ui::app::Row) -> Option<String> {
    use crate::model::schema::Control;
    let v = app.shown_value(row)?;
    Some(match (&v, row.control) {
        (_, Control::Stars) => stars_glyphs(
            match &v {
                Value::Text(s) => s.trim().parse::<u8>().unwrap_or(0),
                _ => 0,
            }
            .min(5),
        ),
        (Value::List(l), Control::HashTags) => {
            l.iter().map(|t| format!("#{t}")).collect::<Vec<_>>().join(" ")
        }
        (Value::List(l), _) => l.join(" · "),
        (Value::Text(s), Control::Enum) => {
            app.enum_label(row, s).unwrap_or_else(|| s.clone())
        }
        (Value::Text(s), _) if row.is_mixed() && !app.is_staged(&row.key) => s.clone(),
        (Value::Text(s), _) => s.replace('\n', " "),
    })
}

fn draw_status(f: &mut Frame, area: Rect, app: &App) {
    const SELECT_HELP: &str =
        "jk move · ⏎ edit · w write · m merge · p inspect · ][ file · a all · u undo · q quit";
    const EDIT_HELP: &str = "⏎ save · ⇥ save and next · esc cancel · ⌃c quit";
    let help = if app.mode == Mode::Edit { EDIT_HELP } else { SELECT_HELP };
    // Validation is about the field under the cursor, so it outranks the
    // transient status line, which in turn outranks the help.
    let live = if app.mode == Mode::Edit { app.validation() } else { Validation::Ok };
    let (text, style) = match live {
        Validation::Error(m) => (m, Style::default().fg(Color::Red)),
        Validation::Warn(m) => (m, Style::default().fg(Color::Yellow)),
        Validation::Ok if !app.status.is_empty() => (app.status.clone(), dim()),
        Validation::Ok => (help.to_string(), dim()),
    };
    let dirty = if app.staged.is_empty() {
        String::new()
    } else {
        format!(
            " faststart:{} · {} staged ",
            if app.faststart { "on" } else { "off" },
            app.staged.len()
        )
    };
    let pad = (area.width as usize).saturating_sub(text.chars().count() + dirty.chars().count());
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(text, style),
            Span::raw(" ".repeat(pad)),
            Span::styled(dirty, Style::default().fg(Color::Yellow)),
        ]))
        .alignment(Alignment::Left),
        area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portrait_gets_a_taller_band_than_landscape() {
        assert!(header_rows(40, Some(0.5625)) > header_rows(40, Some(1.78)));
    }

    #[test]
    fn short_terminals_drop_the_band_entirely() {
        assert_eq!(header_rows(18, Some(0.56)), 0);
    }

    /// A 16:9 picture is wide; a 9:16 one is narrow. The point of the fix.
    #[test]
    fn columns_follow_the_aspect() {
        assert!(thumb_cols(6, 1.78) > thumb_cols(6, 0.5625));
    }

    #[test]
    fn columns_stay_within_bounds() {
        assert!(thumb_cols(6, 0.01) >= 4);
        assert!(thumb_cols(60, 10.0) <= 40);
    }
}

/// The plan, in the terms the user thinks in: which field, to what, and by
/// which route. The route matters because it is the difference between an
/// in-place update and a full rewrite of a multi-gigabyte file.
fn draw_confirm(f: &mut Frame, area: Rect, app: &App, plans: &[FilePlan]) {
    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        format!(
            " Write {} file{}",
            plans.len(),
            if plans.len() == 1 { "" } else { "s" }
        ),
        Style::default().add_modifier(Modifier::BOLD),
    )));
    lines.push(Line::from(""));

    for row in &app.rows {
        let Some(v) = app.staged.get(&row.key) else { continue };
        let shown = match v {
            Value::List(l) if l.is_empty() => "removed".to_string(),
            Value::List(l) => l.join(", "),
            Value::Text(s) if s.is_empty() => "removed".to_string(),
            Value::Text(s) => s.clone(),
        };
        let mut spans = vec![
            Span::styled(format!("  {:<14}", row.label), label_style()),
            Span::styled("→ ", dim()),
            Span::styled(shown, Style::default().fg(Color::Green)),
        ];
        // Replacing one value is an edit; replacing several distinct ones is a
        // different act, and this is the last place to notice it.
        let n = app.overwrites(row);
        if n > 1 {
            spans.push(Span::styled(
                format!("   replaces {n} distinct values"),
                Style::default().fg(Color::Yellow),
            ));
        }
        lines.push(Line::from(spans));
    }
    lines.push(Line::from(""));

    for p in plans {
        lines.push(Line::from(vec![
            Span::styled(format!("  {:<28}", file_label(&p.path)), Style::default().fg(Color::Cyan)),
            Span::styled(format!("{:<22}", p.writer.label()), Style::default().fg(Color::Magenta)),
            Span::styled(p.why, dim()),
        ]));
        // Say what the container is now, so "faststart on" is visibly either a
        // no-op or the reason this file is being rewritten rather than nudged.
        lines.push(Line::from(Span::styled(
            format!("  {:<28}{:?}", "", p.layout),
            dim(),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "  faststart {} · originals replaced in place after verification",
            if app.faststart { "on" } else { "off" }
        ),
        dim(),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  ⏎ write   esc cancel",
        Style::default().add_modifier(Modifier::BOLD),
    )));

    f.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        ),
        area,
    );
}

fn file_label(p: &std::path::Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
}

/// What actually happened, per file. A one-line status is fine for one file and
/// useless for forty: a batch needs to say which ones failed and why, without
/// the successes scrolling them away.
fn draw_results(f: &mut Frame, area: Rect, r: &crate::ui::app::WriteResults) {
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            format!(
                " Wrote {} of {}",
                r.ok.len(),
                r.ok.len() + r.failed.len()
            ),
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    for p in &r.ok {
        lines.push(Line::from(vec![
            Span::styled("  ✓ ", Style::default().fg(Color::Green)),
            Span::raw(file_label(p)),
        ]));
    }
    for (p, err) in &r.failed {
        lines.push(Line::from(vec![
            Span::styled("  ✕ ", Style::default().fg(Color::Red)),
            Span::styled(format!("{:<26}", file_label(p)), Style::default().fg(Color::Red)),
            Span::styled(err.clone(), dim()),
        ]));
    }
    if !r.failed.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Files that failed are unchanged; nothing was half-written.",
            dim(),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  any key to continue",
        Style::default().add_modifier(Modifier::BOLD),
    )));

    let border = if r.failed.is_empty() { Color::Green } else { Color::Red };
    f.render_widget(
        Paragraph::new(lines).block(
            Block::default().borders(Borders::ALL).border_style(Style::default().fg(border)),
        ),
        area,
    );
}
