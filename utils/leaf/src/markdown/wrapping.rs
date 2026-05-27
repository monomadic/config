use super::width::display_width;
use ratatui::{
    style::Style,
    text::{Line, Span},
};
use unicode_width::UnicodeWidthChar;

pub(super) fn push_wrapped_prefixed_lines(
    lines: &mut Vec<Line<'static>>,
    body_spans: &mut Vec<Span<'static>>,
    first_prefix: Vec<Span<'static>>,
    continuation_prefix: Vec<Span<'static>>,
    render_width: usize,
) {
    if body_spans.is_empty() {
        return;
    }

    let first_prefix_width: usize = first_prefix
        .iter()
        .map(|span| display_width(span.content.as_ref()))
        .sum();
    let continuation_prefix_width: usize = continuation_prefix
        .iter()
        .map(|span| display_width(span.content.as_ref()))
        .sum();
    let max_width = render_width
        .saturating_sub(first_prefix_width.max(continuation_prefix_width))
        .max(8);

    let total_width: usize = body_spans
        .iter()
        .map(|s| display_width(s.content.as_ref()))
        .sum();
    if total_width <= max_width {
        let mut all = first_prefix;
        all.append(body_spans);
        lines.push(Line::from(all));
        return;
    }

    let mut current_prefix = first_prefix.clone();
    let mut next_prefix = continuation_prefix.clone();
    let mut current_width = 0usize;
    let mut body_started = false;

    let push_current = |lines: &mut Vec<Line<'static>>,
                        current_prefix: &mut Vec<Span<'static>>,
                        next_prefix: &mut Vec<Span<'static>>,
                        body_started: &mut bool,
                        current_width: &mut usize| {
        if *body_started {
            lines.push(Line::from(std::mem::take(current_prefix)));
            *current_prefix = next_prefix.clone();
            *body_started = false;
            *current_width = 0;
        }
    };

    for span in body_spans.drain(..) {
        let style = span.style;
        let mut token = String::new();
        let mut token_is_space = false;

        let mut flush_token = |token: &mut String,
                               token_is_space: bool,
                               lines: &mut Vec<Line<'static>>,
                               current_prefix: &mut Vec<Span<'static>>,
                               body_started: &mut bool,
                               current_width: &mut usize| {
            if token.is_empty() {
                return;
            }

            let token_width = display_width(token);
            if token_is_space {
                let keep_styled_padding = style.bg.is_some();
                if (*body_started || keep_styled_padding)
                    && *current_width + token_width <= max_width
                {
                    current_prefix.push(Span::styled(std::mem::take(token), style));
                    *current_width += token_width;
                    *body_started = true;
                } else {
                    token.clear();
                }
                return;
            }

            if *body_started && *current_width + token_width > max_width {
                push_current(
                    lines,
                    current_prefix,
                    &mut next_prefix,
                    body_started,
                    current_width,
                );
            }

            if token_width <= max_width {
                current_prefix.push(Span::styled(std::mem::take(token), style));
                *current_width += token_width;
                *body_started = true;
                return;
            }

            let mut chunk = String::new();
            let mut chunk_width = 0usize;
            for ch in token.chars() {
                let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
                let would_overflow = if *body_started {
                    *current_width + chunk_width + ch_width > max_width
                } else {
                    chunk_width + ch_width > max_width
                };
                if would_overflow {
                    if !chunk.is_empty() {
                        current_prefix.push(Span::styled(std::mem::take(&mut chunk), style));
                        *body_started = true;
                    }
                    push_current(
                        lines,
                        current_prefix,
                        &mut next_prefix,
                        body_started,
                        current_width,
                    );
                    chunk_width = 0;
                }

                chunk.push(ch);
                chunk_width += ch_width;
            }

            if !chunk.is_empty() {
                current_prefix.push(Span::styled(chunk, style));
                *current_width += chunk_width;
                *body_started = true;
            }
            token.clear();
        };

        for ch in span.content.chars() {
            let is_space = ch.is_whitespace();
            if token.is_empty() {
                token_is_space = is_space;
            } else if token_is_space != is_space {
                flush_token(
                    &mut token,
                    token_is_space,
                    lines,
                    &mut current_prefix,
                    &mut body_started,
                    &mut current_width,
                );
                token_is_space = is_space;
            }
            token.push(ch);
        }

        flush_token(
            &mut token,
            token_is_space,
            lines,
            &mut current_prefix,
            &mut body_started,
            &mut current_width,
        );
    }

    if body_started {
        lines.push(Line::from(current_prefix));
    }
}

pub(super) fn push_wrapped_code_lines(
    lines: &mut Vec<Line<'static>>,
    content_spans: Vec<Span<'static>>,
    first_prefix: Vec<Span<'static>>,
    continuation_prefix: Vec<Span<'static>>,
    suffix_style: Style,
    available_content_width: usize,
) {
    let mut chars: Vec<(char, Style)> = Vec::new();
    for span in &content_spans {
        let style = span.style;
        for ch in span.content.chars() {
            chars.push((ch, style));
        }
    }

    if chars.is_empty() {
        let pad = " ".repeat(available_content_width + 1);
        let mut row = first_prefix;
        row.push(Span::raw(format!(" {pad}")));
        row.push(Span::styled("│", suffix_style));
        lines.push(Line::from(row));
        return;
    }

    let max_w = available_content_width.max(1);
    let mut pos = 0;
    let mut first_prefix = Some(first_prefix);

    while pos < chars.len() {
        let prefix = first_prefix
            .take()
            .unwrap_or_else(|| continuation_prefix.clone());

        let mut row_chars: Vec<(char, Style)> = Vec::new();
        let mut row_width = 0;

        while pos < chars.len() {
            let (ch, st) = chars[pos];
            let ch_w = UnicodeWidthChar::width(ch).unwrap_or(0);
            if row_width + ch_w > max_w && row_width > 0 {
                break;
            }
            row_chars.push((ch, st));
            row_width += ch_w;
            pos += 1;
        }

        let mut row = prefix;
        row.push(Span::raw(" "));

        let mut current_style: Option<Style> = None;
        let mut current_text = String::new();
        for (ch, st) in &row_chars {
            if current_style == Some(*st) {
                current_text.push(*ch);
            } else {
                if !current_text.is_empty() {
                    row.push(Span::styled(
                        std::mem::take(&mut current_text),
                        current_style.unwrap(),
                    ));
                }
                current_style = Some(*st);
                current_text.push(*ch);
            }
        }
        if !current_text.is_empty() {
            row.push(Span::styled(current_text, current_style.unwrap()));
        }

        let pad = max_w.saturating_sub(row_width);
        row.push(Span::raw(" ".repeat(pad + 1)));
        row.push(Span::styled("│", suffix_style));
        lines.push(Line::from(row));
    }
}
