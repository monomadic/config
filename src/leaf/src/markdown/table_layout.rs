use ratatui::{
    style::{Modifier, Style},
    text::Span,
};
use unicode_width::UnicodeWidthChar;

use super::tables::{CellFragment, CellInlineStyle};
use super::width::{display_width, expand_tabs};
use super::LINK_MARKER;

pub(super) fn fragments_display_width(frags: &[CellFragment]) -> usize {
    frags.iter().map(|f| f.display_width()).sum()
}

pub(super) fn min_table_cell_width(frags: &[CellFragment]) -> usize {
    let mut max_width = 4usize;
    for frag in frags {
        let w = if frag.is_text() {
            frag.rendered_text()
                .split_whitespace()
                .map(display_width)
                .max()
                .unwrap_or(0)
                .min(12)
        } else {
            frag.display_width()
        };
        max_width = max_width.max(w);
    }
    max_width
}

pub(super) fn fit_table_widths(
    col_widths: &mut [usize],
    min_widths: &[usize],
    render_width: usize,
) {
    if col_widths.is_empty() {
        return;
    }

    let col_count = col_widths.len();
    let border_width = 3 * col_count + 1;
    let available = render_width.saturating_sub(border_width).max(col_count);
    let min_total: usize = min_widths.iter().sum();

    if min_total >= available {
        let mut widths = vec![1; col_count];
        let mut remaining = available.saturating_sub(col_count);
        let mut order: Vec<usize> = (0..col_count).collect();
        order.sort_by_key(|&idx| std::cmp::Reverse(min_widths[idx]));
        for idx in order {
            if remaining == 0 {
                break;
            }
            let extra = (min_widths[idx].saturating_sub(1)).min(remaining);
            widths[idx] += extra;
            remaining -= extra;
        }
        col_widths.copy_from_slice(&widths);
        return;
    }

    while col_widths.iter().sum::<usize>() > available {
        let Some((idx, _)) = col_widths
            .iter()
            .enumerate()
            .filter(|(idx, width)| **width > min_widths[*idx])
            .max_by_key(|(_, width)| **width)
        else {
            break;
        };
        col_widths[idx] -= 1;
    }
}

pub(super) fn wrap_table_cell(frags: &[CellFragment], width: usize) -> Vec<Vec<CellFragment>> {
    if width == 0 {
        return vec![vec![]];
    }
    if frags.is_empty() {
        return vec![vec![]];
    }

    let mut lines: Vec<Vec<CellFragment>> = Vec::new();
    let mut current_line: Vec<CellFragment> = Vec::new();
    let mut current_width = 0usize;
    let mut glue = false;
    let space_frag = || CellFragment::Text(" ".to_string(), CellInlineStyle::default(), false);

    for frag in frags {
        match frag {
            CellFragment::Text(t, style, adj) => {
                let expanded = expand_tabs(t, 0);
                let style = *style;
                let adj = *adj;
                let mut first_word = true;
                for word in expanded.split_whitespace() {
                    let word_width = display_width(word);

                    if word_width > width {
                        if !current_line.is_empty() || current_width > 0 {
                            lines.push(std::mem::take(&mut current_line));
                            current_width = 0;
                        }
                        glue = false;
                        first_word = false;
                        let mut chunk = String::new();
                        let mut chunk_width = 0usize;
                        for ch in word.chars() {
                            let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
                            if chunk_width + ch_width > width && !chunk.is_empty() {
                                lines.push(vec![CellFragment::Text(
                                    std::mem::take(&mut chunk),
                                    style,
                                    false,
                                )]);
                                chunk_width = 0;
                            }
                            chunk.push(ch);
                            chunk_width += ch_width;
                        }
                        if !chunk.is_empty() {
                            current_line.push(CellFragment::Text(chunk, style, false));
                            current_width = chunk_width;
                        }
                        continue;
                    }

                    let suppress = adj && first_word;
                    first_word = false;
                    let needs_sep = current_width > 0 && !glue && !suppress;
                    glue = false;
                    let sep = if needs_sep { 1 } else { 0 };
                    if current_width + sep + word_width > width && current_width > 0 {
                        lines.push(std::mem::take(&mut current_line));
                        current_width = 0;
                    } else if needs_sep {
                        current_line.push(CellFragment::Text(" ".to_string(), style, false));
                        current_width += 1;
                    }
                    current_line.push(CellFragment::Text(word.to_string(), style, false));
                    current_width += word_width;
                }
            }
            CellFragment::LinkMarker(_) => {
                if current_width + 1 > width && current_width > 0 {
                    lines.push(std::mem::take(&mut current_line));
                    current_width = 0;
                }
                if current_width > 0 {
                    current_line.push(space_frag());
                    current_width += 1;
                }
                current_line.push(frag.clone());
                current_width += 1;
                glue = true;
            }
            CellFragment::Code(_, adj)
            | CellFragment::InlineMath(_, adj)
            | CellFragment::Mark(_, adj) => {
                let frag_width = frag.display_width();
                let adj = *adj;
                let sep = if current_width == 0 || adj { 0 } else { 1 };
                if current_width + sep + frag_width > width && current_width > 0 {
                    lines.push(std::mem::take(&mut current_line));
                    current_width = 0;
                }
                if current_width > 0 && !adj {
                    current_line.push(space_frag());
                    current_width += 1;
                }
                current_line.push(frag.clone());
                current_width += frag_width;
            }
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }
    if lines.is_empty() {
        lines.push(vec![]);
    }
    lines
}

pub(super) fn align_cell(
    frags: &[CellFragment],
    width: usize,
    align: pulldown_cmark::Alignment,
    base_style: Style,
    is_header: bool,
    theme: &crate::theme::MarkdownTheme,
) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut content_width = 0usize;

    for frag in frags {
        match frag {
            CellFragment::Text(t, inline, _) => {
                let expanded = expand_tabs(t, 0);
                content_width += display_width(&expanded);
                let mut style = base_style;
                if inline.bold {
                    style = style.add_modifier(Modifier::BOLD);
                    if !is_header {
                        style = style.fg(theme.strong_text);
                    }
                }
                if inline.italic {
                    style = style.add_modifier(Modifier::ITALIC);
                }
                if inline.strikethrough {
                    style = style.add_modifier(Modifier::CROSSED_OUT);
                }
                if inline.link {
                    style = style.fg(theme.link_text).add_modifier(Modifier::UNDERLINED);
                }
                spans.push(Span::styled(expanded, style));
            }
            CellFragment::LinkMarker(inline) => {
                let style = Style::default()
                    .fg(theme.link_icon)
                    .add_modifier(inline.modifiers());
                spans.push(Span::styled(LINK_MARKER, style));
                content_width += display_width(LINK_MARKER);
            }
            CellFragment::Code(_, _)
            | CellFragment::InlineMath(_, _)
            | CellFragment::Mark(_, _) => {
                let styled = format!(" {} ", frag.rendered_text());
                content_width += display_width(&styled);
                let (fg, bg) = match frag {
                    CellFragment::Code(..) => (theme.inline_code_fg, theme.inline_code_bg),
                    CellFragment::Mark(..) => (theme.mark_fg, theme.mark_bg),
                    _ => (theme.latex_inline_fg, theme.latex_inline_bg),
                };
                spans.push(Span::styled(styled, Style::default().fg(fg).bg(bg)));
            }
        }
    }

    if content_width < width {
        let pad = width - content_width;
        match align {
            pulldown_cmark::Alignment::Right => {
                spans.insert(0, Span::styled(" ".repeat(pad), base_style));
            }
            pulldown_cmark::Alignment::Center => {
                let l = pad / 2;
                spans.insert(0, Span::styled(" ".repeat(l), base_style));
                spans.push(Span::styled(" ".repeat(pad - l), base_style));
            }
            _ => {
                spans.push(Span::styled(" ".repeat(pad), base_style));
            }
        }
    }

    spans
}
