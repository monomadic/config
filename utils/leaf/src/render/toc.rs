use crate::{app::App, theme::app_theme};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

pub(super) fn render_toc_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let theme = app_theme();
    app.refresh_toc_cache();
    let toc_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(area);

    f.render_widget(
        Paragraph::new("")
            .style(Style::default().bg(theme.ui.toc_bg))
            .block(
                Block::default()
                    .borders(Borders::RIGHT | Borders::BOTTOM)
                    .border_style(Style::default().fg(theme.ui.toc_border))
                    .style(Style::default().bg(theme.ui.toc_bg)),
            ),
        toc_chunks[0],
    );
    f.render_widget(
        Paragraph::new(app.toc_display_lines().to_vec())
            .style(Style::default().bg(theme.ui.toc_bg))
            .block(
                Block::default()
                    .borders(Borders::RIGHT)
                    .border_style(Style::default().fg(theme.ui.toc_border))
                    .style(Style::default().bg(theme.ui.toc_bg)),
            ),
        toc_chunks[1],
    );
    f.render_widget(
        Paragraph::new(vec![app.toc_header_line().clone()])
            .style(Style::default().bg(theme.ui.toc_bg)),
        Rect {
            x: toc_chunks[0].x,
            y: toc_chunks[0].y.saturating_add(1),
            width: toc_chunks[0].width.saturating_sub(1),
            height: 1,
        },
    );
}

pub(crate) fn toc_header_line() -> Line<'static> {
    let theme = app_theme();
    Line::from(vec![Span::styled(
        "  TABLE OF CONTENTS",
        Style::default()
            .fg(theme.ui.toc_header_fg)
            .bg(theme.ui.toc_bg)
            .add_modifier(Modifier::BOLD),
    )])
}

pub(crate) fn build_toc_line_with_index(
    entry: &crate::markdown::toc::TocEntry,
    display_level: u8,
    top_level_index: Option<usize>,
    active: bool,
) -> Line<'static> {
    let theme = app_theme();
    let active_bg = theme.ui.toc_active_bg;
    let inactive_bg = theme.ui.toc_inactive_bg;

    match display_level {
        1 => {
            let index = top_level_index.unwrap_or(0) + 1;
            let title = crate::markdown::truncate_display_width(&entry.title, 18);
            let bg = if active { active_bg } else { inactive_bg };
            Line::from(vec![
                Span::styled(
                    if active { "▎" } else { " " },
                    Style::default().fg(theme.ui.toc_accent).bg(bg),
                ),
                Span::styled("  ", Style::default().bg(bg)),
                Span::styled(
                    format!("{index:02}"),
                    Style::default()
                        .fg(if active {
                            theme.ui.toc_accent
                        } else {
                            theme.ui.toc_index_inactive
                        })
                        .bg(bg)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" ", Style::default().bg(bg)),
                Span::styled(
                    title,
                    Style::default()
                        .fg(if active {
                            theme.ui.toc_primary_active
                        } else {
                            theme.ui.toc_primary_inactive
                        })
                        .bg(bg)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        }
        _ => Line::from(vec![
            Span::styled(
                if active { "▎" } else { " " },
                Style::default().fg(theme.ui.toc_accent),
            ),
            Span::raw("     "),
            Span::styled(
                "•",
                Style::default().fg(if active {
                    theme.ui.toc_accent
                } else {
                    theme.ui.toc_secondary_inactive
                }),
            ),
            Span::raw(" "),
            Span::styled(
                crate::markdown::truncate_display_width(&entry.title, 18),
                Style::default()
                    .fg(if active {
                        theme.ui.toc_secondary_text_active
                    } else {
                        theme.ui.toc_secondary_text_inactive
                    })
                    .add_modifier(if active {
                        Modifier::BOLD
                    } else {
                        Modifier::empty()
                    }),
            ),
        ]),
    }
}
