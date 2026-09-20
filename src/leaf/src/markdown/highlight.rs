use crate::theme::MarkdownTheme;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};
use std::{borrow::Cow, ops::Range};

use super::width;
use super::LINK_MARKER;

pub(super) fn find_match_char_ranges(
    text_chars: &[char],
    query_lower: &[Vec<char>],
) -> Vec<Range<usize>> {
    let qlen = query_lower.len();
    if qlen == 0 || text_chars.len() < qlen {
        return Vec::new();
    }
    let mut ranges = Vec::new();
    let mut i = 0;
    while i + qlen <= text_chars.len() {
        let matched = text_chars[i..i + qlen]
            .iter()
            .zip(query_lower)
            .all(|(t, ql)| t.to_lowercase().eq(ql.iter().copied()));
        if matched {
            ranges.push(i..i + qlen);
            i += qlen;
        } else {
            i += 1;
        }
    }
    ranges
}

pub(super) fn content_span_start(spans: &[Span<'_>]) -> usize {
    let has_pipe = spans.iter().take(4).any(|s| s.content.contains('│'));
    if !has_pipe {
        return 0;
    }
    for (i, span) in spans.iter().enumerate() {
        if width::is_code_gutter_span(span.content.as_ref()) {
            return i + 1;
        }
    }
    0
}

pub(super) fn overlapping_ranges(ranges: &[Range<usize>], start: usize, end: usize) -> bool {
    ranges.iter().any(|r| r.start < end && r.end > start)
}

pub(crate) fn highlight_line<'a>(line: &Line<'a>, theme: &MarkdownTheme, query: &str) -> Line<'a> {
    let spans = &line.spans;
    let content_start = content_span_start(spans);

    let text_chars: Vec<char> = spans[content_start..]
        .iter()
        .filter(|s| s.content.as_ref() != LINK_MARKER)
        .flat_map(|s| s.content.chars())
        .collect();
    let query_lower: Vec<Vec<char>> = query.chars().map(|c| c.to_lowercase().collect()).collect();
    let ranges = find_match_char_ranges(&text_chars, &query_lower);
    if ranges.is_empty() {
        return line.clone();
    }

    let line_bg = theme.search_highlight_bg;
    let match_style_extra = Style::default()
        .bg(theme.search_match_bg)
        .add_modifier(Modifier::BOLD);
    let mut result: Vec<Span<'a>> = Vec::with_capacity(spans.len());

    for span in &spans[..content_start] {
        result.push(Span::styled(span.content.clone(), span.style.bg(line_bg)));
    }

    let mut char_offset: usize = 0;
    for span in &spans[content_start..] {
        if span.content.as_ref() == LINK_MARKER {
            result.push(span.clone());
            continue;
        }
        let span_len = span.content.chars().count();
        let span_end = char_offset + span_len;
        let base_style = span.style.bg(line_bg);

        if !overlapping_ranges(&ranges, char_offset, span_end) {
            result.push(Span::styled(span.content.clone(), base_style));
            char_offset = span_end;
            continue;
        }

        let span_chars: Vec<char> = span.content.chars().collect();
        let mut pos = 0;
        for range in &ranges {
            if range.end <= char_offset || range.start >= span_end {
                continue;
            }
            let local_start = range.start.saturating_sub(char_offset);
            let local_end = (range.end - char_offset).min(span_len);

            if local_start > pos {
                let text: String = span_chars[pos..local_start].iter().collect();
                result.push(Span::styled(Cow::Owned(text), base_style));
            }
            let text: String = span_chars[local_start..local_end].iter().collect();
            result.push(Span::styled(
                Cow::Owned(text),
                span.style.patch(match_style_extra),
            ));
            pos = local_end;
        }
        if pos < span_len {
            let text: String = span_chars[pos..].iter().collect();
            result.push(Span::styled(Cow::Owned(text), base_style));
        }

        char_offset = span_end;
    }

    Line::from(result)
}
