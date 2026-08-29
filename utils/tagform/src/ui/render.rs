//! Drawing (SPEC §7).
//!
//! The shape of a screen: a badge bar that reads as a title, a band of facts
//! about the file, the form itself, a shortcut strip for the current mode, and
//! one line of status. Every field paints its editable region, so the form
//! looks like a form before you focus anything.

use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;
use ratatui_image::{protocol::StatefulProtocol, StatefulImage};
use unicode_width::UnicodeWidthStr;

use crate::model::schema::Control;
use crate::model::value::{Agg, Value};
use crate::tags::plan::FilePlan;
use crate::ui::app::{App, Mode, Row, WriteResults};
use crate::ui::edit::{stars_glyphs, Validation};
use crate::ui::theme as t;

const LABEL_COLS: u16 = 15;
const GUTTER: u16 = 1;
/// Blank columns of field background either side of a value.
const PAD: u16 = 1;

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

pub fn draw(f: &mut Frame, app: &App, proto: Option<&mut StatefulProtocol>) {
    let area = f.area();
    let header_h = header_rows(area.height, app.thumb_aspect);
    let chunks = Layout::vertical([
        Constraint::Length(1),        // badge bar
        Constraint::Length(1),        // breathing room under it
        Constraint::Length(header_h), // thumbnail + file facts
        Constraint::Min(3),           // the form
        Constraint::Length(1),        // shortcuts for this mode
        Constraint::Length(1),        // status / validation
    ])
    .split(area);

    draw_badge_bar(f, chunks[0], app);

    // A dialog takes everything below the header: it is the whole message.
    if app.pending.is_some() || app.results.is_some() {
        let top = chunks[2].y;
        let body = Rect {
            x: area.x,
            y: top,
            width: area.width,
            height: area.height.saturating_sub(top.saturating_sub(area.y)),
        };
        if let Some(plans) = &app.pending {
            draw_confirm(f, body, app, plans);
        } else if let Some(r) = &app.results {
            draw_results(f, body, r);
        }
        return;
    }

    if header_h > 0 {
        if app.inspector {
            draw_inspector(f, chunks[2], app);
        } else {
            draw_header(f, chunks[2], app, proto);
        }
    }
    draw_fields(f, chunks[3], app);
    draw_shortcuts(f, chunks[4], app);
    draw_status(f, chunks[5], app);
}

/// The name sits in a filled badge and the bar carries its own background the
/// full width, so the header reads as a title rather than as one more row of
/// text competing with the form.
fn draw_badge_bar(f: &mut Frame, area: Rect, app: &App) {
    let view = match app.view {
        Some(i) => format!("file {}/{}", i + 1, app.files.len()),
        None => format!("{} file{}", app.files.len(), plural(app.files.len())),
    };
    let mut left = format!("  {view}");
    if app.n_custom > 0 {
        left.push_str(&format!(" · {} custom", app.n_custom));
    }
    let mut right = String::new();
    if !app.staged.is_empty() {
        right.push_str(&format!("{} staged · ", app.staged.len()));
    }
    right.push_str(&format!(
        "faststart {} · ",
        if app.faststart { "on" } else { "off" }
    ));
    let mode = if app.mode == Mode::Edit { "EDIT" } else { "SELECT" };
    let tail = format!("{mode}  ");

    let badge = " tagform ";
    let used = badge.width() + left.width() + right.width() + tail.width();
    let gap = (area.width as usize).saturating_sub(used);
    let bar = Style::default().bg(t::header_bg());

    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(
                badge,
                Style::default().bg(t::badge_bg()).fg(t::badge_fg()).add_modifier(Modifier::BOLD),
            ),
            Span::styled(left, bar.fg(t::header_fg())),
            Span::styled(" ".repeat(gap), bar),
            Span::styled(
                right,
                bar.fg(if app.staged.is_empty() { t::muted() } else { t::staged() }),
            ),
            Span::styled(
                tail,
                if app.mode == Mode::Edit {
                    bar.fg(t::accent()).add_modifier(Modifier::BOLD)
                } else {
                    bar.fg(t::muted())
                },
            ),
        ]))
        .style(bar),
        area,
    );
}

fn draw_header(f: &mut Frame, area: Rect, app: &App, proto: Option<&mut StatefulProtocol>) {
    let idx = app.current_file();
    let Some(file) = app.files.get(idx) else { return };

    let want = app.thumb_aspect.map(|a| thumb_cols(area.height, a)).unwrap_or(0);
    let has_thumb = proto.is_some() && want > 0 && area.width > want + 20;
    let cols = Layout::horizontal([
        Constraint::Length(if has_thumb { want } else { 0 }),
        Constraint::Min(10),
    ])
    .split(area);

    if has_thumb {
        if let Some(p) = proto {
            f.render_stateful_widget(StatefulImage::default(), cols[0], p);
        }
    }

    let name = file_label(&file.path);
    let dir = file
        .path
        .parent()
        .map(|d| d.to_string_lossy().to_string())
        .unwrap_or_default();
    let summary = app.media.get(idx).map(|m| m.summary()).unwrap_or_default();
    let pad = if has_thumb { "  " } else { " " };

    let lines = vec![
        Line::from(Span::styled(
            format!("{pad}{name}"),
            Style::default().fg(t::header_fg()).add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(
            format!("{pad}{}", if summary.is_empty() { "probing…".into() } else { summary }),
            Style::default().fg(t::muted()),
        )),
        Line::from(Span::styled(format!("{pad}{dir}"), Style::default().fg(t::path()))),
    ];
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), cols[1]);
}

/// The answer to "what does ‹multiple› actually contain" -- the thing the old
/// fzf-based tagger could only show in a preview pane.
fn draw_inspector(f: &mut Frame, area: Rect, app: &App) {
    let Some(row) = app.rows.get(app.focus) else { return };
    let mut lines = vec![Line::from(vec![
        Span::styled(format!(" {} ", row.label), Style::default().bg(t::rule()).fg(t::label_focus())),
        Span::styled("  per file", Style::default().fg(t::muted())),
    ])];

    match &row.agg {
        Agg::Mixed { values } => {
            for (i, v) in values.iter().enumerate() {
                let shown = match v {
                    Some(Value::Text(s)) => s.clone(),
                    Some(Value::List(l)) => l.join(" · "),
                    None => "—".into(),
                };
                let style = if v.is_some() {
                    Style::default().fg(t::value())
                } else {
                    Style::default().fg(t::value_empty())
                };
                lines.push(Line::from(vec![
                    Span::raw(" "),
                    Span::styled(t::fit(&shown, 34), style),
                    Span::styled(
                        app.files.get(i).map(|f| file_label(&f.path)).unwrap_or_default(),
                        Style::default().fg(t::muted()),
                    ),
                ]));
            }
        }
        Agg::Same { .. } => lines.push(Line::from(Span::styled(
            " identical in every file",
            Style::default().fg(t::muted()),
        ))),
        Agg::Absent => lines.push(Line::from(Span::styled(
            " present in no file",
            Style::default().fg(t::value_empty()),
        ))),
    }
    f.render_widget(Paragraph::new(lines), area);
}

fn draw_fields(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::TOP).border_style(Style::default().fg(t::rule()));
    let inner = block.inner(area);
    f.render_widget(block, area);
    if inner.width <= LABEL_COLS + GUTTER + 2 * PAD + 4 {
        return;
    }

    let height = inner.height as usize;
    let start = if app.focus >= height { app.focus + 1 - height } else { 0 };
    // The box is the full width; the text sits inside it with a blank column of
    // its own background either side, so it reads as an input rather than as a
    // block of colour butted straight up against the label.
    let value_w = inner.width.saturating_sub(1 + LABEL_COLS + GUTTER + 1) as usize;
    let text_w = value_w.saturating_sub(2 * PAD as usize);
    let value_x = inner.x + 1 + LABEL_COLS + GUTTER;
    let mut cursor: Option<(u16, u16)> = None;

    let mut lines = Vec::new();
    for (i, row) in app.rows.iter().enumerate().skip(start).take(height) {
        let focused = i == app.focus;
        let editing = focused && app.mode == Mode::Edit;
        let staged = app.is_staged(&row.key);
        let custom = row.def.is_none();
        let readonly = !row.editable();

        // The marker column says where you are, and nothing else. It used to
        // carry the staged dot as well, which put an edit indicator in the
        // caret's column -- so a staged row looked mis-caretted, and a row that
        // was both staged and focused lost its indicator entirely because the
        // caret won. Edited-ness is carried by the label colour instead.
        let (marker, marker_fg) = if editing {
            ("▶", t::accent())
        } else if focused {
            ("▍", t::accent())
        } else {
            (" ", t::rule())
        };

        // Staged outranks focus here precisely so it survives being focused.
        let label_fg = match (staged, focused, custom) {
            (true, _, _) => t::staged(),
            (false, true, _) => t::label_focus(),
            (false, false, true) => t::label_custom(),
            (false, false, false) => t::label(),
        };
        let label_style = if focused {
            Style::default().fg(label_fg).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(label_fg)
        };

        // Every control paints its editable region, so the form reads as a form
        // rather than as a list of colons.
        let bg = if readonly {
            t::input_bg_readonly()
        } else if editing {
            t::input_bg_edit()
        } else if focused {
            t::input_bg_focus()
        } else {
            t::input_bg()
        };

        let (raw, fg) = if editing {
            let (text, cur) = app
                .editor
                .as_ref()
                .map(|e| e.display())
                .unwrap_or_else(|| (String::new(), None));
            if let Some(c) = cur {
                let x = value_x + PAD + (c as u16).min(text_w.saturating_sub(1) as u16);
                cursor = Some((x, inner.y + (i - start) as u16));
            }
            let fg = match app.validation() {
                Validation::Error(_) => t::error(),
                Validation::Warn(_) => t::warn(),
                Validation::Ok => t::value(),
            };
            (text, fg)
        } else {
            match display_row(app, row) {
                Some(v) if staged => (v, t::staged()),
                Some(v) if row.is_mixed() => (v, t::mixed()),
                Some(v) if readonly => (v, t::muted()),
                Some(v) => (v, t::value()),
                None => ("—".into(), t::value_empty()),
            }
        };
        // Star colour belongs to stars. An empty rating draws the same "—" as
        // every other empty field and must look like one.
        let has_value = app.shown_value(row).is_some();
        let value_fg = if row.control == Control::Stars && !editing && has_value {
            t::star()
        } else {
            fg
        };

        lines.push(Line::from(vec![
            Span::styled(marker, Style::default().fg(marker_fg)),
            Span::styled(
                t::fit(
                    &if custom { t::short_key(&row.label) } else { row.label.clone() },
                    LABEL_COLS as usize,
                ),
                label_style,
            ),
            Span::raw(" "),
            Span::styled(" ".repeat(PAD as usize), Style::default().bg(bg)),
            Span::styled(t::fit(&raw, text_w), Style::default().bg(bg).fg(value_fg)),
            Span::styled(" ".repeat(PAD as usize), Style::default().bg(bg)),
        ]));
    }
    f.render_widget(Paragraph::new(lines), inner);
    if let Some((x, y)) = cursor {
        f.set_cursor_position((x, y));
    }
}

/// An unfocused row shows the staged edit if there is one, else what is on disk.
fn display_row(app: &App, row: &Row) -> Option<String> {
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
            l.iter().map(|x| format!("#{x}")).collect::<Vec<_>>().join(" ")
        }
        (Value::List(l), _) => l.join(" · "),
        (Value::Text(s), Control::Enum) => app.enum_label(row, s).unwrap_or_else(|| s.clone()),
        (Value::Text(s), _) => s.replace('\n', " "),
    })
}

/// The keys that matter right now, and only those. Which keys are live depends
/// on the mode, so a fixed strip would be wrong half the time.
fn draw_shortcuts(f: &mut Frame, area: Rect, app: &App) {
    let pairs: &[(&str, &str)] = if app.mode == Mode::Edit {
        &[
            ("⏎", "save"),
            ("⇥", "save + next"),
            ("←→", "cycle"),
            ("esc", "cancel"),
            ("^c", "quit"),
        ]
    } else {
        &[
            ("jk", "move"),
            ("⏎", "edit"),
            ("w", "write"),
            ("m", "merge"),
            ("p", "inspect"),
            ("][", "file"),
            ("a", "all"),
            ("u", "undo"),
            ("r", "revert"),
            ("c", "theme"),
            ("f", "fast"),
            ("q", "quit"),
        ]
    };
    // Drop hints that do not fit rather than letting the strip run off the
    // edge: a half-rendered key name is worse than one fewer hint.
    let mut spans = vec![Span::raw(" ")];
    let mut used = 1usize;
    let mut dropped = 0usize;
    for (k, d) in pairs {
        let key = format!(" {k} ");
        let desc = format!(" {d}  ");
        let w = key.width() + desc.width();
        if used + w + 2 > area.width as usize {
            dropped += 1;
            continue;
        }
        used += w;
        spans.push(Span::styled(
            key,
            Style::default().bg(t::rule()).fg(t::accent()).add_modifier(Modifier::BOLD),
        ));
        spans.push(Span::styled(desc, Style::default().fg(t::muted())));
    }
    if dropped > 0 {
        spans.push(Span::styled("…", Style::default().fg(t::rule())));
    }
    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

fn draw_status(f: &mut Frame, area: Rect, app: &App) {
    // Validation is about the field under the cursor, so it outranks the
    // transient status line -- but only while a field is actually open.
    let live = if app.mode == Mode::Edit { app.validation() } else { Validation::Ok };
    let (text, fg) = match live {
        Validation::Error(m) => (m, t::error()),
        Validation::Warn(m) => (m, t::warn()),
        Validation::Ok if !app.status.is_empty() => (app.status.clone(), t::muted()),
        Validation::Ok => (String::new(), t::muted()),
    };
    f.render_widget(
        Paragraph::new(Line::from(Span::styled(format!(" {text}"), Style::default().fg(fg)))),
        area,
    );
}

/// The plan, in the terms the user thinks in: which field, to what, and by
/// which route. The route matters because it is the difference between an
/// in-place update and a full rewrite of a multi-gigabyte file.
fn draw_confirm(f: &mut Frame, area: Rect, app: &App, plans: &[FilePlan]) {
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            format!(" Write {} file{} ", plans.len(), plural(plans.len())),
            Style::default().bg(t::accent()).fg(t::badge_fg()).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];

    for row in &app.rows {
        let Some(v) = app.staged.get(&row.key) else { continue };
        let shown = match v {
            Value::List(l) if l.is_empty() => "removed".to_string(),
            Value::List(l) => l.join(", "),
            Value::Text(s) if s.is_empty() => "removed".to_string(),
            Value::Text(s) => s.clone(),
        };
        let mut spans = vec![
            Span::styled(format!("  {}", t::fit(&row.label, 14)), Style::default().fg(t::label())),
            Span::styled("→ ", Style::default().fg(t::muted())),
            Span::styled(shown, Style::default().fg(t::staged())),
        ];
        // Replacing one value is an edit; replacing several distinct ones is a
        // different act, and this is the last place to notice it.
        let n = app.overwrites(row);
        if n > 1 {
            spans.push(Span::styled(
                format!("   replaces {n} distinct values"),
                Style::default().fg(t::warn()),
            ));
        }
        lines.push(Line::from(spans));
    }
    lines.push(Line::from(""));

    for p in plans {
        lines.push(Line::from(vec![
            Span::styled(format!("  {}", t::fit(&file_label(&p.path), 28)), Style::default().fg(t::header_fg())),
            Span::styled(t::fit(p.writer.label(), 22), Style::default().fg(t::accent())),
            Span::styled(p.why, Style::default().fg(t::muted())),
        ]));
        lines.push(Line::from(Span::styled(
            format!("  {}{:?}", " ".repeat(28), p.layout),
            Style::default().fg(t::rule()),
        )));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        format!(
            "  faststart {} · originals replaced only after the result is verified",
            if app.faststart { "on" } else { "off" }
        ),
        Style::default().fg(t::muted()),
    )));
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ⏎ ", Style::default().bg(t::rule()).fg(t::accent()).add_modifier(Modifier::BOLD)),
        Span::styled(" write   ", Style::default().fg(t::value())),
        Span::styled(" esc ", Style::default().bg(t::rule()).fg(t::accent()).add_modifier(Modifier::BOLD)),
        Span::styled(" cancel", Style::default().fg(t::value())),
    ]));

    f.render_widget(
        Paragraph::new(lines).block(
            Block::default().borders(Borders::ALL).border_style(Style::default().fg(t::accent())),
        ),
        area,
    );
}

/// What actually happened, per file. A one-line status is fine for one file and
/// useless for forty: a batch needs to say which ones failed and why, without
/// the successes scrolling them away.
fn draw_results(f: &mut Frame, area: Rect, r: &WriteResults) {
    let total = r.ok.len() + r.failed.len();
    let ok = r.failed.is_empty();
    let mut lines: Vec<Line> = vec![
        Line::from(Span::styled(
            format!(" Wrote {} of {} ", r.ok.len(), total),
            Style::default()
                .bg(if ok { t::staged() } else { t::error() })
                .fg(t::badge_fg())
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
    ];
    for p in &r.ok {
        lines.push(Line::from(vec![
            Span::styled("  ✓ ", Style::default().fg(t::staged())),
            Span::styled(file_label(p), Style::default().fg(t::value())),
        ]));
    }
    for (p, err) in &r.failed {
        lines.push(Line::from(vec![
            Span::styled("  ✕ ", Style::default().fg(t::error())),
            Span::styled(t::fit(&file_label(p), 26), Style::default().fg(t::error())),
            Span::styled(err.clone(), Style::default().fg(t::muted())),
        ]));
    }
    if !ok {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "  Files that failed are unchanged; nothing was half-written.",
            Style::default().fg(t::muted()),
        )));
    }
    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "  any key to continue",
        Style::default().fg(t::value()).add_modifier(Modifier::BOLD),
    )));

    f.render_widget(
        Paragraph::new(lines).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if ok { t::staged() } else { t::error() })),
        ),
        area,
    );
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

fn file_label(p: &std::path::Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
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
