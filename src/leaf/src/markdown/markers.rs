use crate::theme::MarkdownTheme;
use ratatui::{style::Style, text::Span};

pub(super) struct CustomMarker {
    pub(super) open: &'static str,
    pub(super) close: &'static str,
    pub(super) style_fn: fn(&MarkdownTheme) -> Style,
    pub(super) pad: bool,
}

fn mark_style(theme: &MarkdownTheme) -> Style {
    Style::default().fg(theme.mark_fg).bg(theme.mark_bg)
}

pub(super) const MARK_MARKER: CustomMarker = CustomMarker {
    open: "==",
    close: "==",
    style_fn: mark_style,
    pad: true,
};

fn is_valid_content(content: &str) -> bool {
    !content.is_empty()
        && !content.contains('\n')
        && !content.starts_with(' ')
        && !content.ends_with(' ')
}

struct MarkerMatch<'a> {
    open_pos: usize,
    close_rel: usize,
    marker: &'a CustomMarker,
}

fn find_first_marker<'a>(text: &str, markers: &'a [CustomMarker]) -> Option<MarkerMatch<'a>> {
    let mut best: Option<MarkerMatch<'a>> = None;

    for marker in markers {
        if let Some(open_pos) = text.find(marker.open) {
            let after_open = open_pos + marker.open.len();
            if let Some(close_rel) = text[after_open..].find(marker.close) {
                let content = &text[after_open..after_open + close_rel];
                if is_valid_content(content) {
                    let dominated = best.as_ref().is_some_and(|b| b.open_pos <= open_pos);
                    if !dominated {
                        best = Some(MarkerMatch {
                            open_pos,
                            close_rel,
                            marker,
                        });
                    }
                }
            }
        }
    }

    best
}

pub(super) fn push_custom_marker_spans(
    text: &str,
    markers: &[CustomMarker],
    fallback_style: Style,
    theme: &MarkdownTheme,
    spans: &mut Vec<Span<'static>>,
) {
    let mut remaining = text;

    while !remaining.is_empty() {
        let Some(m) = find_first_marker(remaining, markers) else {
            spans.push(Span::styled(remaining.to_string(), fallback_style));
            break;
        };

        let after_open = m.open_pos + m.marker.open.len();
        let content = &remaining[after_open..after_open + m.close_rel];
        let after_close = after_open + m.close_rel + m.marker.close.len();

        if m.open_pos > 0 {
            spans.push(Span::styled(
                remaining[..m.open_pos].to_string(),
                fallback_style,
            ));
        }

        let display = if m.marker.pad {
            format!(" {} ", content)
        } else {
            content.to_string()
        };
        spans.push(Span::styled(display, (m.marker.style_fn)(theme)));

        remaining = &remaining[after_close..];
    }
}

pub(super) enum MarkerSegment<'a> {
    Text(&'a str),
    Mark(&'a str),
}

pub(super) fn split_marker_segments<'a>(
    text: &'a str,
    markers: &[CustomMarker],
) -> Vec<MarkerSegment<'a>> {
    let mut segments = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        let Some(m) = find_first_marker(remaining, markers) else {
            segments.push(MarkerSegment::Text(remaining));
            break;
        };

        let after_open = m.open_pos + m.marker.open.len();
        let content = &remaining[after_open..after_open + m.close_rel];
        let after_close = after_open + m.close_rel + m.marker.close.len();

        if m.open_pos > 0 {
            segments.push(MarkerSegment::Text(&remaining[..m.open_pos]));
        }
        segments.push(MarkerSegment::Mark(content));

        remaining = &remaining[after_close..];
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_theme() -> MarkdownTheme {
        crate::theme::theme_by_preset(crate::theme::ThemePreset::OceanDark).markdown
    }

    fn fallback() -> Style {
        Style::default()
    }

    fn collect(text: &str, markers: &[CustomMarker], theme: &MarkdownTheme) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        push_custom_marker_spans(text, markers, fallback(), theme, &mut spans);
        spans
    }

    #[test]
    fn no_markers_returns_single_span() {
        let theme = test_theme();
        let spans = collect("hello world", &[], &theme);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content.as_ref(), "hello world");
    }

    #[test]
    fn mark_in_middle_produces_three_spans() {
        let theme = test_theme();
        let spans = collect("before ==marked== after", &[MARK_MARKER], &theme);
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].content.as_ref(), "before ");
        assert_eq!(spans[1].content.as_ref(), " marked ");
        assert!(spans[1].style.bg.is_some());
        assert_eq!(spans[2].content.as_ref(), " after");
    }

    #[test]
    fn consecutive_marks() {
        let theme = test_theme();
        let spans = collect("==one== ==two==", &[MARK_MARKER], &theme);
        assert_eq!(spans.len(), 3);
        assert_eq!(spans[0].content.as_ref(), " one ");
        assert_eq!(spans[1].content.as_ref(), " ");
        assert_eq!(spans[2].content.as_ref(), " two ");
    }

    #[test]
    fn unclosed_marker_is_literal() {
        let theme = test_theme();
        let spans = collect("==unclosed", &[MARK_MARKER], &theme);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content.as_ref(), "==unclosed");
    }

    #[test]
    fn empty_content_is_literal() {
        let theme = test_theme();
        let spans = collect("====", &[MARK_MARKER], &theme);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content.as_ref(), "====");
    }

    #[test]
    fn spaces_around_content_is_literal() {
        let theme = test_theme();
        let spans = collect("== text ==", &[MARK_MARKER], &theme);
        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].content.as_ref(), "== text ==");
    }

    #[test]
    fn split_segments_returns_text_and_mark() {
        let segs = split_marker_segments("before ==marked== after", &[MARK_MARKER]);
        assert_eq!(segs.len(), 3);
        assert!(matches!(segs[0], MarkerSegment::Text("before ")));
        assert!(matches!(segs[1], MarkerSegment::Mark("marked")));
        assert!(matches!(segs[2], MarkerSegment::Text(" after")));
    }

    #[test]
    fn split_segments_no_markers() {
        let segs = split_marker_segments("plain text", &[MARK_MARKER]);
        assert_eq!(segs.len(), 1);
        assert!(matches!(segs[0], MarkerSegment::Text("plain text")));
    }
}
