use crate::theme::MarkdownTheme;
use mmdflux::{render_diagram, OutputFormat, RenderConfig};
use ratatui::{style::Style, text::Span};
use std::fmt::Write;

pub(crate) fn render(content: &str) -> Option<String> {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.starts_with("pie") {
        return render_pie(trimmed);
    }
    render_diagram(trimmed, OutputFormat::Text, &RenderConfig::default()).ok()
}

fn render_pie(content: &str) -> Option<String> {
    let mut title = String::new();
    let mut entries: Vec<(String, f64)> = Vec::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some(rest) = line.strip_prefix("pie") {
            let rest = rest.trim();
            if rest.is_empty() {
                continue;
            }
            if let Some(t) = rest.strip_prefix("title") {
                title = t.trim().to_string();
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("title") {
            title = rest.trim().to_string();
            continue;
        }
        if let Some((label_part, value_part)) = line.rsplit_once(':') {
            let label = label_part.trim().trim_matches('"').to_string();
            if let Ok(value) = value_part.trim().parse::<f64>() {
                entries.push((label, value));
            }
        }
    }

    if entries.is_empty() {
        return None;
    }

    let total: f64 = entries.iter().map(|(_, v)| *v).sum();
    if total <= 0.0 {
        return None;
    }

    let max_label_width = entries.iter().map(|(l, _)| l.len()).max().unwrap_or(0);
    let bar_max = 32;
    let mut out = String::new();

    if !title.is_empty() {
        let _ = writeln!(out, "{title}");
    }

    for (label, value) in &entries {
        let pct = value / total * 100.0;
        let bar_units = pct / 100.0 * bar_max as f64;
        let filled = bar_units as usize;
        let half = (bar_units * 2.0) as usize % 2 == 1;
        let bar: String = "█".repeat(filled) + if half { "▌" } else { "" };
        let _ = writeln!(
            out,
            "{bar:<bw$} {label:<lw$} {pct:>5.1}%",
            bw = bar_max + 1,
            lw = max_label_width,
        );
    }

    Some(out)
}

pub(crate) fn colorize_line(line: &str, theme: &MarkdownTheme) -> Vec<Span<'static>> {
    let keyword_style = Style::default().fg(theme.mermaid_keyword);
    let arrow_style = Style::default().fg(theme.mermaid_arrow);
    let label_style = Style::default().fg(theme.mermaid_label);
    let default_style = Style::default().fg(theme.mermaid_block_fg);

    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut rest = line;

    while !rest.is_empty() {
        if let Some(pos) = rest.find('|') {
            let before = &rest[..pos];
            if !before.is_empty() {
                tokenize_segment(
                    before,
                    keyword_style,
                    arrow_style,
                    default_style,
                    &mut spans,
                );
            }
            let after_pipe = &rest[pos + 1..];
            if let Some(end) = after_pipe.find('|') {
                let label_content = &after_pipe[..end];
                spans.push(Span::styled(format!("|{label_content}|"), label_style));
                rest = &after_pipe[end + 1..];
            } else {
                spans.push(Span::styled("|".to_string(), default_style));
                rest = after_pipe;
            }
            continue;
        }

        tokenize_segment(rest, keyword_style, arrow_style, default_style, &mut spans);
        break;
    }

    if spans.is_empty() {
        spans.push(Span::styled(line.to_string(), default_style));
    }

    spans
}

fn tokenize_segment(
    segment: &str,
    keyword_style: Style,
    arrow_style: Style,
    default_style: Style,
    spans: &mut Vec<Span<'static>>,
) {
    let mut i = 0;
    let bytes = segment.as_bytes();

    while i < bytes.len() {
        if let Some((arrow, len)) = try_match_arrow(&segment[i..]) {
            spans.push(Span::styled(arrow, arrow_style));
            i += len;
            continue;
        }

        if bytes[i].is_ascii_alphabetic() || bytes[i] == b'_' {
            let start = i;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'-')
            {
                i += 1;
            }
            let word = &segment[start..i];
            if is_keyword(word) {
                spans.push(Span::styled(word.to_string(), keyword_style));
            } else {
                spans.push(Span::styled(word.to_string(), default_style));
            }
            continue;
        }

        let start = i;
        while i < segment.len() {
            let b = bytes[i];
            if b.is_ascii_alphabetic() || b == b'_' || b == b'|' {
                break;
            }
            if try_match_arrow(&segment[i..]).is_some() {
                break;
            }
            if b < 0x80 {
                i += 1;
            } else {
                let ch = segment[i..].chars().next().unwrap();
                i += ch.len_utf8();
            }
        }
        if i > start {
            spans.push(Span::styled(segment[start..i].to_string(), default_style));
        }
    }
}

fn try_match_arrow(s: &str) -> Option<(String, usize)> {
    for pattern in &["-.->", "==>", "-->", "---", "-.-", "-..", "->", "--"] {
        if s.starts_with(pattern) {
            return Some((pattern.to_string(), pattern.len()));
        }
    }
    None
}

fn is_keyword(word: &str) -> bool {
    is_diagram_keyword(word) || is_direction_keyword(word) || is_structure_keyword(word)
}

fn is_diagram_keyword(word: &str) -> bool {
    matches!(
        word,
        "flowchart"
            | "graph"
            | "sequenceDiagram"
            | "classDiagram"
            | "stateDiagram"
            | "stateDiagram-v2"
            | "erDiagram"
            | "gantt"
            | "pie"
            | "journey"
            | "gitGraph"
            | "mindmap"
            | "timeline"
            | "sankey-beta"
            | "quadrantChart"
            | "requirementDiagram"
            | "C4Context"
            | "block-beta"
            | "xychart-beta"
            | "kanban"
            | "architecture-beta"
    )
}

fn is_direction_keyword(word: &str) -> bool {
    matches!(word, "TB" | "TD" | "BT" | "LR" | "RL")
}

fn is_structure_keyword(word: &str) -> bool {
    matches!(
        word,
        "subgraph"
            | "end"
            | "section"
            | "title"
            | "participant"
            | "actor"
            | "loop"
            | "alt"
            | "else"
            | "opt"
            | "par"
            | "critical"
            | "break"
            | "rect"
            | "note"
            | "activate"
            | "deactivate"
            | "class"
            | "state"
            | "dateFormat"
            | "axisFormat"
            | "style"
            | "classDef"
            | "click"
    )
}
