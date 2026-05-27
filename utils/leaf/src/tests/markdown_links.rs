use super::{test_assets, test_md_theme};
use crate::markdown::{highlight_line, parse_markdown, resolve_syntax};
use crate::theme::app_theme;
use ratatui::{
    style::Style,
    text::{Line, Span},
};
use syntect::parsing::SyntaxSet;

#[test]
fn blockquote_bold_link_preserves_link_color() {
    let (ss, theme) = test_assets();
    let src = "> text [**lien bold**](https://rivolink.mg)\n";
    let (lines, _, _) = parse_markdown(src, &ss, &theme, &test_md_theme(), false);
    let app_theme = app_theme();
    let theme_colors = &app_theme.markdown;

    let bq_line = &lines[0];
    let link_span = bq_line.spans.iter().find(|s| s.content.contains("lien"));
    assert!(link_span.is_some(), "should find 'lien' span");
    let span = link_span.unwrap();
    assert_eq!(
        span.style.fg,
        Some(theme_colors.link_text),
        "bold link in blockquote should preserve link_text color"
    );
}

#[test]
fn link_spans_detected_for_all_link_types() {
    let (ss, theme) = test_assets();
    let md = "\
[Simple](https://example.com/simple)

**[Bold link](https://example.com/bold)**

*[Italic link](https://example.com/italic)*

~~[Strike link](https://example.com/strike)~~

[Internal](#section)

### [Heading link](https://example.com/heading)

> [Blockquote link](https://example.com/quote)

[A](https://example.com/a) and [B](https://example.com/b)
";
    let (_, _, link_spans) = parse_markdown(md, &ss, &theme, &test_md_theme(), false);

    let urls: Vec<&str> = link_spans.iter().map(|ls| ls.url.as_str()).collect();

    assert!(
        urls.contains(&"https://example.com/simple"),
        "simple link missing: {urls:?}"
    );
    assert!(
        urls.contains(&"https://example.com/bold"),
        "bold link missing: {urls:?}"
    );
    assert!(
        urls.contains(&"https://example.com/italic"),
        "italic link missing: {urls:?}"
    );
    assert!(
        urls.contains(&"https://example.com/strike"),
        "strikethrough link missing: {urls:?}"
    );
    assert!(
        urls.contains(&"#section"),
        "internal link missing: {urls:?}"
    );
    assert!(
        urls.contains(&"https://example.com/heading"),
        "heading link missing: {urls:?}"
    );
    assert!(
        urls.contains(&"https://example.com/quote"),
        "blockquote link missing: {urls:?}"
    );
    assert!(
        urls.contains(&"https://example.com/a"),
        "multi-link A missing: {urls:?}"
    );
    assert!(
        urls.contains(&"https://example.com/b"),
        "multi-link B missing: {urls:?}"
    );

    for ls in &link_spans {
        assert!(
            ls.end_col > ls.start_col,
            "link {:?} has zero width (start={} end={})",
            ls.url,
            ls.start_col,
            ls.end_col,
        );
    }
}

#[test]
fn link_spans_in_table_are_detected() {
    let (ss, theme) = test_assets();
    let md = "\
| Name | Link |
|------|------|
| Test | [example](https://example.com/table) |
";
    let (_, _, link_spans) = parse_markdown(md, &ss, &theme, &test_md_theme(), false);

    let urls: Vec<&str> = link_spans.iter().map(|ls| ls.url.as_str()).collect();
    assert!(
        urls.contains(&"https://example.com/table"),
        "table link missing: {urls:?}"
    );
}

#[test]
fn highlight_line_single_match() {
    let theme = test_md_theme();
    let line_bg = theme.search_highlight_bg;
    let match_bg = theme.search_match_bg;
    let line = Line::from(vec![Span::raw("hello world")]);
    let result = highlight_line(&line, &theme, "world");
    assert_eq!(result.spans.len(), 2);
    assert_eq!(result.spans[0].content.as_ref(), "hello ");
    assert_eq!(result.spans[0].style.bg, Some(line_bg));
    assert_eq!(result.spans[1].content.as_ref(), "world");
    assert_eq!(result.spans[1].style.bg, Some(match_bg));
    assert!(result.spans[1]
        .style
        .add_modifier
        .contains(ratatui::style::Modifier::BOLD));
}

#[test]
fn highlight_line_multiple_matches() {
    let theme = test_md_theme();
    let match_bg = theme.search_match_bg;
    let line = Line::from(vec![Span::raw("abcabcabc")]);
    let result = highlight_line(&line, &theme, "abc");
    assert_eq!(result.spans.len(), 3);
    for span in &result.spans {
        assert_eq!(span.content.as_ref(), "abc");
        assert_eq!(span.style.bg, Some(match_bg));
        assert!(span
            .style
            .add_modifier
            .contains(ratatui::style::Modifier::BOLD));
    }
}

#[test]
fn highlight_line_case_insensitive() {
    let theme = test_md_theme();
    let line_bg = theme.search_highlight_bg;
    let match_bg = theme.search_match_bg;
    let line = Line::from(vec![Span::raw("Hello World")]);
    let result = highlight_line(&line, &theme, "hello");
    assert_eq!(result.spans.len(), 2);
    assert_eq!(result.spans[0].content.as_ref(), "Hello");
    assert_eq!(result.spans[0].style.bg, Some(match_bg));
    assert!(result.spans[0]
        .style
        .add_modifier
        .contains(ratatui::style::Modifier::BOLD));
    assert_eq!(result.spans[1].content.as_ref(), " World");
    assert_eq!(result.spans[1].style.bg, Some(line_bg));
    assert!(!result.spans[1]
        .style
        .add_modifier
        .contains(ratatui::style::Modifier::BOLD));
}

#[test]
fn highlight_line_cross_span() {
    let theme = test_md_theme();
    let line_bg = theme.search_highlight_bg;
    let match_bg = theme.search_match_bg;
    let bold = Style::default().add_modifier(ratatui::style::Modifier::BOLD);
    let line = Line::from(vec![Span::styled("hel", bold), Span::raw("lo world")]);
    let result = highlight_line(&line, &theme, "hello");
    assert_eq!(result.spans[0].content.as_ref(), "hel");
    assert_eq!(result.spans[0].style.bg, Some(match_bg));
    assert!(result.spans[0]
        .style
        .add_modifier
        .contains(ratatui::style::Modifier::BOLD));
    assert_eq!(result.spans[1].content.as_ref(), "lo");
    assert_eq!(result.spans[1].style.bg, Some(match_bg));
    assert!(result.spans[1]
        .style
        .add_modifier
        .contains(ratatui::style::Modifier::BOLD));
    assert_eq!(result.spans[2].content.as_ref(), " world");
    assert_eq!(result.spans[2].style.bg, Some(line_bg));
}

#[test]
fn highlight_line_no_match_returns_clone() {
    let theme = test_md_theme();
    let line = Line::from(vec![Span::raw("hello world")]);
    let result = highlight_line(&line, &theme, "xyz");
    assert_eq!(result.spans.len(), 1);
    assert_eq!(result.spans[0].content.as_ref(), "hello world");
    assert_eq!(result.spans[0].style.bg, None);
}

#[test]
fn resolve_syntax_supports_common_language_aliases() {
    let ss = SyntaxSet::load_defaults_newlines();

    assert_eq!(
        resolve_syntax("py", &ss).name,
        resolve_syntax("python", &ss).name
    );
    assert_eq!(
        resolve_syntax("cpp", &ss).name,
        resolve_syntax("c++", &ss).name
    );
    assert_eq!(resolve_syntax("json", &ss).name, "JSON");
    assert_eq!(resolve_syntax("json5", &ss).name, "JSON");
    assert_eq!(
        resolve_syntax("ps1", &ss).name,
        resolve_syntax("powershell", &ss).name
    );

    for tag in &["kotlin", "toml", "jsx", "dockerfile"] {
        assert_ne!(
            resolve_syntax(tag, &ss).name,
            "Plain Text",
            "{tag} should not fall back to Plain Text"
        );
    }
    assert_eq!(
        resolve_syntax("kt", &ss).name,
        resolve_syntax("kotlin", &ss).name
    );
    assert_eq!(
        resolve_syntax("docker", &ss).name,
        resolve_syntax("dockerfile", &ss).name
    );
    assert_eq!(
        resolve_syntax("pwsh", &ss).name,
        resolve_syntax("ps1", &ss).name
    );
}
