use crate::cli::parse_cli;
use crate::inline::{self, InlineFormat, InlineSpec, ResolvedFormat};

// --inline CLI tests

#[test]
fn parse_cli_accepts_inline_on_its_own() {
    let args = vec![
        "leaf".to_string(),
        "--inline".to_string(),
        "README.md".to_string(),
    ];
    let options = parse_cli(&args).unwrap();
    assert_eq!(
        options.inline,
        Some(InlineSpec {
            format: InlineFormat::Auto,
            width: None,
        })
    );
    assert_eq!(options.file_arg.as_deref(), Some("README.md"));
}

#[test]
fn parse_cli_accepts_inline_with_width() {
    let args = vec![
        "leaf".to_string(),
        "--inline".to_string(),
        "50".to_string(),
        "README.md".to_string(),
    ];
    let options = parse_cli(&args).unwrap();
    assert_eq!(
        options.inline,
        Some(InlineSpec {
            format: InlineFormat::Auto,
            width: Some(50),
        })
    );
    assert_eq!(options.file_arg.as_deref(), Some("README.md"));
}

#[test]
fn parse_cli_accepts_inline_ansi() {
    let args = vec![
        "leaf".to_string(),
        "--inline".to_string(),
        "ansi".to_string(),
        "README.md".to_string(),
    ];
    let options = parse_cli(&args).unwrap();
    assert_eq!(
        options.inline,
        Some(InlineSpec {
            format: InlineFormat::Ansi,
            width: None,
        })
    );
}

#[test]
fn parse_cli_accepts_inline_ansi_with_width() {
    let args = vec![
        "leaf".to_string(),
        "--inline".to_string(),
        "ansi:50".to_string(),
        "README.md".to_string(),
    ];
    let options = parse_cli(&args).unwrap();
    assert_eq!(
        options.inline,
        Some(InlineSpec {
            format: InlineFormat::Ansi,
            width: Some(50),
        })
    );
}

#[test]
fn parse_cli_accepts_inline_plain_with_width() {
    let args = vec![
        "leaf".to_string(),
        "--inline".to_string(),
        "plain:80".to_string(),
        "README.md".to_string(),
    ];
    let options = parse_cli(&args).unwrap();
    assert_eq!(
        options.inline,
        Some(InlineSpec {
            format: InlineFormat::Plain,
            width: Some(80),
        })
    );
}

#[test]
fn parse_cli_accepts_inline_equals_form() {
    let args = vec![
        "leaf".to_string(),
        "--inline=ansi".to_string(),
        "README.md".to_string(),
    ];
    let options = parse_cli(&args).unwrap();
    assert_eq!(
        options.inline,
        Some(InlineSpec {
            format: InlineFormat::Ansi,
            width: None,
        })
    );
}

#[test]
fn parse_cli_accepts_inline_equals_with_width() {
    let args = vec![
        "leaf".to_string(),
        "--inline=plain:50".to_string(),
        "README.md".to_string(),
    ];
    let options = parse_cli(&args).unwrap();
    assert_eq!(
        options.inline,
        Some(InlineSpec {
            format: InlineFormat::Plain,
            width: Some(50),
        })
    );
}

#[test]
fn parse_cli_rejects_inline_with_watch() {
    let args = vec![
        "leaf".to_string(),
        "--inline".to_string(),
        "--watch".to_string(),
        "README.md".to_string(),
    ];
    let err = parse_cli(&args).unwrap_err();
    assert!(err
        .to_string()
        .contains("--inline cannot be combined with --watch"));
}

#[test]
fn parse_cli_rejects_inline_with_picker() {
    let args = vec![
        "leaf".to_string(),
        "--inline".to_string(),
        "--picker".to_string(),
    ];
    let err = parse_cli(&args).unwrap_err();
    assert!(err
        .to_string()
        .contains("--inline cannot be combined with --picker"));
}

#[test]
fn parse_cli_inline_without_spec_uses_auto() {
    let args = vec![
        "leaf".to_string(),
        "--inline".to_string(),
        "README.md".to_string(),
    ];
    let options = parse_cli(&args).unwrap();
    let spec = options.inline.unwrap();
    assert_eq!(spec.format, InlineFormat::Auto);
    assert_eq!(spec.width, None);
    assert_eq!(options.file_arg.as_deref(), Some("README.md"));
}

// parse_inline_spec tests

#[test]
fn parse_inline_spec_ansi() {
    let spec = inline::parse_inline_spec("ansi").unwrap();
    assert_eq!(spec.format, InlineFormat::Ansi);
    assert_eq!(spec.width, None);
}

#[test]
fn parse_inline_spec_plain() {
    let spec = inline::parse_inline_spec("plain").unwrap();
    assert_eq!(spec.format, InlineFormat::Plain);
    assert_eq!(spec.width, None);
}

#[test]
fn parse_inline_spec_width_only() {
    let spec = inline::parse_inline_spec("50").unwrap();
    assert_eq!(spec.format, InlineFormat::Auto);
    assert_eq!(spec.width, Some(50));
}

#[test]
fn parse_inline_spec_format_with_width() {
    let spec = inline::parse_inline_spec("plain:80").unwrap();
    assert_eq!(spec.format, InlineFormat::Plain);
    assert_eq!(spec.width, Some(80));
}

#[test]
fn parse_inline_spec_rejects_zero_width() {
    let err = inline::parse_inline_spec("0").unwrap_err();
    assert!(err.to_string().contains("positive integer"));
}

#[test]
fn parse_inline_spec_rejects_unknown_format() {
    let err = inline::parse_inline_spec("html").unwrap_err();
    assert!(err.to_string().contains("Unknown inline format"));
}

#[test]
fn parse_inline_spec_enforces_min_width() {
    let spec = inline::parse_inline_spec("5").unwrap();
    assert_eq!(spec.width, Some(20));
}

// render_width tests

#[test]
fn render_width_uses_explicit_width() {
    let spec = InlineSpec {
        format: InlineFormat::Auto,
        width: Some(60),
    };
    assert_eq!(inline::render_width(&spec, true), 60);
    assert_eq!(inline::render_width(&spec, false), 60);
}

#[test]
fn render_width_defaults_to_80_when_not_terminal() {
    let spec = InlineSpec {
        format: InlineFormat::Auto,
        width: None,
    };
    assert_eq!(inline::render_width(&spec, false), 80);
}

// resolve_format tests

#[test]
fn resolve_format_auto_terminal_is_ansi() {
    let spec = InlineSpec {
        format: InlineFormat::Auto,
        width: None,
    };
    assert_eq!(inline::resolve_format(&spec, true), ResolvedFormat::Ansi);
}

#[test]
fn resolve_format_auto_pipe_is_plain() {
    let spec = InlineSpec {
        format: InlineFormat::Auto,
        width: None,
    };
    assert_eq!(inline::resolve_format(&spec, false), ResolvedFormat::Plain);
}

#[test]
fn resolve_format_forced_ansi() {
    let spec = InlineSpec {
        format: InlineFormat::Ansi,
        width: None,
    };
    assert_eq!(inline::resolve_format(&spec, false), ResolvedFormat::Ansi);
}

#[test]
fn resolve_format_forced_plain() {
    let spec = InlineSpec {
        format: InlineFormat::Plain,
        width: None,
    };
    assert_eq!(inline::resolve_format(&spec, true), ResolvedFormat::Plain);
}

// write_lines tests

#[test]
fn write_lines_plain_outputs_text_only() {
    use ratatui::style::{Color, Style};
    use ratatui::text::{Line, Span};

    let lines = vec![Line::from(vec![
        Span::styled("hello ", Style::default().fg(Color::Red)),
        Span::styled("world", Style::default().fg(Color::Blue)),
    ])];
    let mut buf = Vec::new();
    inline::write_lines(&lines, ResolvedFormat::Plain, 80, &mut buf).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "hello world\n");
}

#[test]
fn write_lines_ansi_includes_escape_codes() {
    use ratatui::style::{Color, Style};
    use ratatui::text::{Line, Span};

    let lines = vec![Line::from(vec![Span::styled(
        "red",
        Style::default().fg(Color::Red),
    )])];
    let mut buf = Vec::new();
    inline::write_lines(&lines, ResolvedFormat::Ansi, 80, &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("\x1b[31m"));
    assert!(output.contains("red"));
    assert!(output.contains("\x1b[0m"));
}

#[test]
fn write_lines_ansi_handles_rgb_colors() {
    use ratatui::style::{Color, Style};
    use ratatui::text::{Line, Span};

    let lines = vec![Line::from(vec![Span::styled(
        "rgb",
        Style::default().fg(Color::Rgb(255, 128, 0)),
    )])];
    let mut buf = Vec::new();
    inline::write_lines(&lines, ResolvedFormat::Ansi, 80, &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("\x1b[38;2;255;128;0m"));
}

#[test]
fn write_lines_ansi_handles_indexed_colors() {
    use ratatui::style::{Color, Style};
    use ratatui::text::{Line, Span};

    let lines = vec![Line::from(vec![Span::styled(
        "idx",
        Style::default().fg(Color::Indexed(42)),
    )])];
    let mut buf = Vec::new();
    inline::write_lines(&lines, ResolvedFormat::Ansi, 80, &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("\x1b[38;5;42m"));
}

#[test]
fn write_lines_ansi_handles_modifiers() {
    use ratatui::style::{Modifier, Style};
    use ratatui::text::{Line, Span};

    let lines = vec![Line::from(vec![Span::styled(
        "bold",
        Style::default().add_modifier(Modifier::BOLD | Modifier::ITALIC),
    )])];
    let mut buf = Vec::new();
    inline::write_lines(&lines, ResolvedFormat::Ansi, 80, &mut buf).unwrap();
    let output = String::from_utf8(buf).unwrap();
    assert!(output.contains("1;3m") || output.contains("1m") && output.contains("3m"));
}
