use super::{rendered_non_empty_lines, test_assets, test_md_theme};
use crate::markdown::{parse_markdown, parse_markdown_with_width};
use crate::*;

#[test]
fn h1_headings_render_double_rule_without_bottom_spacing() {
    let (ss, theme) = test_assets();
    let (lines, _, _) = parse_markdown("# 東京\n", &ss, &theme, &test_md_theme(), false);
    let rendered = rendered_non_empty_lines(&lines);

    assert_eq!(rendered[0], "東京");
    assert_eq!(rendered[1], "═".repeat(display_width("東京")));
}

#[test]
fn paragraph_and_following_code_block_have_no_blank_gap() {
    let (ss, theme) = test_assets();
    let src = "Intro paragraph\n\n```rs\nfn main() {}\n```\n";
    let (lines, _, _) = parse_markdown(src, &ss, &theme, &test_md_theme(), false);
    let rendered: Vec<String> = lines.iter().map(line_plain_text).collect();
    let intro_idx = rendered
        .iter()
        .position(|line| line == "Intro paragraph")
        .unwrap();

    assert!(rendered[intro_idx + 1].starts_with("┌─ rs "));
}

#[test]
fn nested_blockquotes_keep_quote_prefix_after_inner_quote_ends() {
    let (ss, theme) = test_assets();
    let src = "> outer\n> > inner\n> outer again\n";
    let (lines, _, _) = parse_markdown(src, &ss, &theme, &test_md_theme(), false);
    let rendered = rendered_non_empty_lines(&lines);

    assert!(rendered.iter().any(|line| line == "▏ outer"));
    assert!(rendered.iter().any(|line| line == "▏ inner"));
    assert!(rendered.iter().any(|line| line == "▏ outer again"));
}

#[test]
fn long_blockquotes_wrap_into_multiple_prefixed_lines() {
    let (ss, theme) = test_assets();
    let src = "> This is a long blockquote line that should wrap into multiple quoted lines at narrow widths.\n";
    let (lines, _, _) = parse_markdown_with_width(src, &ss, &theme, 28, &test_md_theme(), false);
    let rendered = rendered_non_empty_lines(&lines);
    let quoted: Vec<_> = rendered
        .into_iter()
        .filter(|line| line.starts_with('▏'))
        .collect();

    assert!(quoted.len() >= 2);
    assert!(quoted.iter().all(|line| line.starts_with("▏ ")));
}

#[test]
fn frontmatter_is_ignored_in_preview() {
    let (ss, theme) = test_assets();
    let src = "---\ntitle: Demo\nowner: me\n---\n# Visible\nBody\n";
    let (lines, _, _) = parse_markdown(src, &ss, &theme, &test_md_theme(), false);
    let rendered = rendered_non_empty_lines(&lines);

    assert!(!rendered.iter().any(|line| line.contains("title: Demo")));
    assert!(rendered.iter().any(|line| line.contains("Visible")));
}

#[test]
fn h2_headings_are_underlined_and_compact() {
    let (ss, theme) = test_assets();
    let (lines, _, _) = parse_markdown_with_width(
        "Intro\n\n## Section\nBody\n",
        &ss,
        &theme,
        40,
        &test_md_theme(),
        false,
    );
    let rendered = rendered_non_empty_lines(&lines);

    assert!(rendered.iter().any(|line| line.contains("Section")));
    assert!(rendered.iter().any(|line| line.contains("────")));
}

#[test]
fn rules_use_render_width_without_extra_blank_after() {
    let (ss, theme) = test_assets();
    let (lines, _, _) = parse_markdown_with_width(
        "Alpha\n\n---\nBeta\n",
        &ss,
        &theme,
        24,
        &test_md_theme(),
        false,
    );
    let rendered = rendered_non_empty_lines(&lines);
    let rule = rendered
        .iter()
        .find(|line| line.trim_start().starts_with('─'))
        .unwrap();

    assert_eq!(display_width(rule.trim_start()), 24);
    let rule_idx = rendered.iter().position(|line| line == rule).unwrap();
    assert_eq!(rendered[rule_idx + 1], "Beta");
}
