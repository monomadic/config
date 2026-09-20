use ratatui::text::Line;
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub(super) const TAB_STOP: usize = 4;

pub(crate) fn line_plain_text(line: &Line<'_>) -> String {
    line.spans.iter().map(|s| s.content.as_ref()).collect()
}

pub(super) fn is_code_gutter_span(content: &str) -> bool {
    let inner = content.strip_prefix('│').and_then(|s| s.strip_suffix('│'));
    match inner {
        Some(s) if !s.is_empty() => {
            let mut has_digit = false;
            for b in s.bytes() {
                if b.is_ascii_digit() {
                    has_digit = true;
                } else if b != b' ' {
                    return false;
                }
            }
            has_digit
        }
        _ => false,
    }
}

pub(crate) fn line_searchable_text(line: &Line<'_>) -> String {
    use super::LINK_MARKER;

    fn collect_filtered(spans: &[ratatui::text::Span<'_>]) -> String {
        spans
            .iter()
            .filter(|s| s.content.as_ref() != LINK_MARKER)
            .map(|s| s.content.as_ref())
            .collect()
    }

    let spans = &line.spans;
    let has_pipe = spans.iter().take(4).any(|s| s.content.contains('│'));
    if !has_pipe {
        return collect_filtered(spans);
    }
    let mut gutter_end = None;
    for (i, span) in spans.iter().enumerate() {
        if is_code_gutter_span(span.content.as_ref()) {
            gutter_end = Some(i);
            break;
        }
    }
    let Some(ge) = gutter_end else {
        return collect_filtered(spans);
    };
    let last = spans.len().saturating_sub(1);
    let start = ge + 1;
    if start > last {
        return String::new();
    }
    let end = if last > start && spans[last].content.as_ref() == "│" {
        last
    } else {
        spans.len()
    };
    collect_filtered(&spans[start..end])
}

pub(crate) fn build_searchable_lines(lines: &[Line<'_>]) -> Vec<String> {
    lines.iter().map(line_searchable_text).collect()
}

pub(crate) fn truncate_display_width(text: &str, max_width: usize) -> String {
    if display_width(text) <= max_width {
        return text.to_string();
    }
    if max_width == 0 {
        return String::new();
    }

    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let ch_w = UnicodeWidthChar::width(ch).unwrap_or(0);
        if used + ch_w > max_width.saturating_sub(1) {
            break;
        }
        out.push(ch);
        used += ch_w;
    }
    out.push('\u{2026}');
    out
}

pub(crate) fn display_width(text: &str) -> usize {
    let mut width = 0;
    let mut parts = text.split('\t').peekable();
    while let Some(segment) = parts.next() {
        width += UnicodeWidthStr::width(segment);
        if parts.peek().is_some() {
            width += TAB_STOP - (width % TAB_STOP);
        }
    }
    width
}

pub(super) fn expand_tabs(text: &str, start_width: usize) -> String {
    if !text.contains('\t') {
        return text.to_string();
    }

    let mut out = String::new();
    let mut width = start_width;
    let mut parts = text.split('\t').peekable();
    while let Some(segment) = parts.next() {
        out.push_str(segment);
        width += UnicodeWidthStr::width(segment);
        if parts.peek().is_some() {
            let spaces = TAB_STOP - (width % TAB_STOP);
            out.push_str(&" ".repeat(spaces));
            width += spaces;
        }
    }
    out
}
