use crate::theme::MarkdownTheme;
use ratatui::{
    style::Style,
    text::{Line, Span},
};

use super::blocks::{block_prefix, trim_paragraph_gap_before_block};
use super::width::display_width;
use super::wrapping::push_wrapped_prefixed_lines;
use super::LastBlock;

#[derive(Clone, Copy)]
pub(super) enum ListKind {
    Unordered,
    Ordered(u64),
}

pub(super) struct ItemState {
    pub(super) marker_emitted: bool,
    pub(super) continuation_indent: usize,
    pub(super) checkbox: Option<bool>,
}

pub(super) fn list_item_prefix(
    in_bq: bool,
    list_stack: &[ListKind],
    item_stack: &mut [ItemState],
    theme: &MarkdownTheme,
    marker_color: Option<ratatui::style::Color>,
) -> Vec<Span<'static>> {
    let mut prefix = block_prefix(in_bq, theme, marker_color);
    let Some(item) = item_stack.last_mut() else {
        return prefix;
    };

    if item.marker_emitted {
        prefix.push(Span::raw(" ".repeat(item.continuation_indent)));
        return prefix;
    }

    let depth = list_stack.len();
    prefix.push(Span::raw("  ".repeat(depth.saturating_sub(1))));

    let marker = match item.checkbox {
        Some(true) => "☑ ".to_string(),
        Some(false) => "☐ ".to_string(),
        None => match list_stack.last().copied().unwrap_or(ListKind::Unordered) {
            ListKind::Unordered => match depth {
                1 => "• ".to_string(),
                2 => "◦ ".to_string(),
                _ => "▸ ".to_string(),
            },
            ListKind::Ordered(n) => format!("{n}. "),
        },
    };
    item.continuation_indent = "  ".repeat(depth.saturating_sub(1)).len() + display_width(&marker);
    item.marker_emitted = true;

    let marker_style = match item.checkbox {
        Some(true) => Style::default().fg(theme.task_checked),
        Some(false) => Style::default().fg(theme.task_unchecked),
        None => match list_stack.last().copied().unwrap_or(ListKind::Unordered) {
            ListKind::Unordered => match depth {
                1 => Style::default().fg(theme.list_level_1),
                2 => Style::default().fg(theme.list_level_2),
                _ => Style::default().fg(theme.list_level_3),
            },
            ListKind::Ordered(_) => Style::default().fg(theme.ordered_list),
        },
    };
    prefix.push(Span::styled(marker, marker_style));
    prefix
}

#[allow(clippy::too_many_arguments)]
pub(super) fn flush_list_item_spans(
    lines: &mut Vec<Line<'static>>,
    spans: &mut Vec<Span<'static>>,
    list_stack: &[ListKind],
    item_stack: &mut [ItemState],
    blockquote_depth: usize,
    render_width: usize,
    theme: &MarkdownTheme,
    marker_color: Option<ratatui::style::Color>,
) {
    if spans.is_empty() {
        return;
    }

    let first_prefix = list_item_prefix(
        blockquote_depth > 0,
        list_stack,
        item_stack,
        theme,
        marker_color,
    );
    let continuation_prefix = list_item_prefix(
        blockquote_depth > 0,
        list_stack,
        item_stack,
        theme,
        marker_color,
    );
    push_wrapped_prefixed_lines(
        lines,
        spans,
        first_prefix,
        continuation_prefix,
        render_width,
    );
}

pub(super) fn start_list(
    lines: &mut Vec<Line<'static>>,
    last_block: LastBlock,
    list_stack: &mut Vec<ListKind>,
    start: Option<u64>,
) {
    trim_paragraph_gap_before_block(lines, last_block);
    list_stack.push(match start {
        Some(n) => ListKind::Ordered(n),
        None => ListKind::Unordered,
    });
}

pub(super) fn end_list(lines: &mut Vec<Line<'static>>, list_stack: &mut Vec<ListKind>) {
    list_stack.pop();
    if list_stack.is_empty() {
        lines.push(Line::from(""));
    }
}

pub(super) fn start_item(item_stack: &mut Vec<ItemState>) {
    item_stack.push(ItemState {
        marker_emitted: false,
        continuation_indent: 0,
        checkbox: None,
    });
}

#[allow(clippy::too_many_arguments)]
pub(super) fn end_item(
    lines: &mut Vec<Line<'static>>,
    spans: &mut Vec<Span<'static>>,
    list_stack: &mut [ListKind],
    item_stack: &mut Vec<ItemState>,
    blockquote_depth: usize,
    render_width: usize,
    theme: &MarkdownTheme,
    marker_color: Option<ratatui::style::Color>,
) {
    flush_list_item_spans(
        lines,
        spans,
        list_stack,
        item_stack,
        blockquote_depth,
        render_width,
        theme,
        marker_color,
    );
    item_stack.pop();
    if let Some(ListKind::Ordered(next)) = list_stack.last_mut() {
        *next += 1;
    }
}
