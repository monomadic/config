use crate::theme::MarkdownTheme;
use pulldown_cmark::{Event as MdEvent, Tag, TagEnd};
use ratatui::{
    style::{Modifier, Style},
    text::Span,
};

use super::latex;
use super::LINK_MARKER;

#[derive(Clone, Copy, Default)]
pub(super) struct InlineStyleState {
    pub(super) in_strong: bool,
    pub(super) in_em: bool,
    pub(super) in_strike: bool,
    pub(super) in_link: bool,
}

impl InlineStyleState {
    pub(super) fn modifiers(&self) -> Modifier {
        let mut m = Modifier::empty();
        if self.in_strong {
            m |= Modifier::BOLD;
        }
        if self.in_em {
            m |= Modifier::ITALIC;
        }
        if self.in_strike {
            m |= Modifier::CROSSED_OUT;
        }
        m
    }
}

pub(super) fn inline_text_style(
    theme: &MarkdownTheme,
    blockquote_depth: usize,
    inline: InlineStyleState,
) -> Style {
    let mut style = if inline.in_link {
        let mut s = Style::default()
            .fg(theme.link_text)
            .add_modifier(Modifier::UNDERLINED);
        if blockquote_depth > 0 {
            s = s.add_modifier(Modifier::ITALIC);
        }
        s
    } else if blockquote_depth > 0 {
        Style::default()
            .fg(theme.blockquote_text)
            .add_modifier(Modifier::ITALIC)
    } else {
        Style::default().fg(theme.text)
    };

    if inline.in_strong && !inline.in_link {
        style = style.fg(theme.strong_text);
    }
    style = style.add_modifier(inline.modifiers());

    style
}

pub(super) fn handle_inline_style_event(
    ev: &MdEvent<'_>,
    inline: &mut InlineStyleState,
    spans: &mut Vec<Span<'static>>,
    theme: &MarkdownTheme,
    blockquote_depth: usize,
    link_urls: &mut Vec<String>,
) -> bool {
    match ev {
        MdEvent::Start(Tag::Strong) => {
            inline.in_strong = true;
            if inline.in_link {
                update_link_marker_modifier(spans, Modifier::BOLD);
            }
            true
        }
        MdEvent::End(TagEnd::Strong) => {
            inline.in_strong = false;
            true
        }
        MdEvent::Start(Tag::Emphasis) => {
            inline.in_em = true;
            if inline.in_link {
                update_link_marker_modifier(spans, Modifier::ITALIC);
            }
            true
        }
        MdEvent::End(TagEnd::Emphasis) => {
            inline.in_em = false;
            true
        }
        MdEvent::Start(Tag::Strikethrough) => {
            inline.in_strike = true;
            if inline.in_link {
                update_link_marker_modifier(spans, Modifier::CROSSED_OUT);
            }
            true
        }
        MdEvent::End(TagEnd::Strikethrough) => {
            inline.in_strike = false;
            true
        }
        MdEvent::Start(Tag::Link { dest_url, .. }) => {
            inline.in_link = true;
            link_urls.push(dest_url.to_string());
            push_link_marker(spans, theme, *inline, blockquote_depth);
            true
        }
        MdEvent::End(TagEnd::Link) => {
            inline.in_link = false;
            true
        }
        _ => false,
    }
}

pub(super) fn push_inline_code_span(
    spans: &mut Vec<Span<'static>>,
    text: &str,
    theme: &MarkdownTheme,
) {
    spans.push(Span::styled(
        format!(" {} ", text),
        Style::default()
            .fg(theme.inline_code_fg)
            .bg(theme.inline_code_bg),
    ));
}

pub(super) fn push_inline_latex_span(
    spans: &mut Vec<Span<'static>>,
    text: &str,
    theme: &MarkdownTheme,
) {
    let rendered = latex::to_unicode(text);
    spans.push(Span::styled(
        format!(" {rendered} "),
        Style::default()
            .fg(theme.latex_inline_fg)
            .bg(theme.latex_inline_bg),
    ));
}

pub(super) fn push_link_marker(
    spans: &mut Vec<Span<'static>>,
    theme: &MarkdownTheme,
    inline: InlineStyleState,
    blockquote_depth: usize,
) {
    let mut style = Style::default()
        .fg(theme.link_icon)
        .add_modifier(inline.modifiers());
    if blockquote_depth > 0 {
        style = style.add_modifier(Modifier::ITALIC);
    }
    spans.push(Span::styled(LINK_MARKER, style));
}

pub(super) fn update_link_marker_modifier(spans: &mut [Span<'static>], modifier: Modifier) {
    if let Some(span) = spans
        .iter_mut()
        .rev()
        .find(|s| s.content.as_ref() == LINK_MARKER)
    {
        span.style = span.style.add_modifier(modifier);
    }
}
