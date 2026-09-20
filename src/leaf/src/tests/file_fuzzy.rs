use super::unique_temp_dir;
use crate::app::{App, AppConfig};
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn fuzzy_file_picker_uses_depth_first_file_order() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("leaf-fuzzy-picker-bfs-{unique}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("a/deep")).unwrap();
    fs::create_dir_all(root.join("b")).unwrap();
    fs::write(root.join("z-root.md"), "# Root\n").unwrap();
    fs::write(root.join("a/a-child.md"), "# Child A\n").unwrap();
    fs::write(root.join("b/b-child.md"), "# Child B\n").unwrap();
    fs::write(root.join("a/deep/a-deep.md"), "# Deep\n").unwrap();

    let mut app = App::new_with_source(
        Vec::new(),
        Vec::new(),
        AppConfig {
            filename: "picker".to_string(),
            source: String::new(),
            debug_input: false,
            watch: false,
            filepath: None,
            last_file_state: None,
        },
    );

    assert!(app.open_fuzzy_file_picker(root.clone()));

    let labels: Vec<_> = app
        .file_picker_filtered_indices()
        .iter()
        .map(|idx| app.file_picker_entries()[*idx].label())
        .collect();
    assert_eq!(
        labels,
        vec![
            "z-root.md",
            "a/a-child.md",
            "a/deep/a-deep.md",
            "b/b-child.md"
        ]
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn fuzzy_file_picker_keeps_depth_first_order_when_query_is_empty() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("leaf-fuzzy-picker-empty-query-{unique}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join(".nvm")).unwrap();
    fs::create_dir_all(root.join("projects")).unwrap();
    fs::write(root.join(".nvm/README.md"), "# Hidden Readme\n").unwrap();
    fs::write(root.join(".nvm/ROADMAP.md"), "# Hidden Roadmap\n").unwrap();
    fs::write(root.join("projects/README.md"), "# Project Readme\n").unwrap();

    let mut app = App::new_with_source(
        Vec::new(),
        Vec::new(),
        AppConfig {
            filename: "picker".to_string(),
            source: String::new(),
            debug_input: false,
            watch: false,
            filepath: None,
            last_file_state: None,
        },
    );

    assert!(app.open_fuzzy_file_picker(root.clone()));

    let labels: Vec<_> = app
        .file_picker_filtered_indices()
        .iter()
        .map(|idx| app.file_picker_entries()[*idx].label())
        .collect();
    assert_eq!(
        labels,
        vec![".nvm/README.md", ".nvm/ROADMAP.md", "projects/README.md"]
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn fuzzy_file_picker_filters_entries_by_query() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("leaf-fuzzy-picker-query-{unique}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(root.join("README.md"), "# Demo\n").unwrap();
    fs::write(root.join("docs/guide.md"), "# Guide\n").unwrap();

    let mut app = App::new_with_source(
        Vec::new(),
        Vec::new(),
        AppConfig {
            filename: "picker".to_string(),
            source: String::new(),
            debug_input: false,
            watch: false,
            filepath: None,
            last_file_state: None,
        },
    );

    assert!(app.open_fuzzy_file_picker(root.clone()));
    app.push_file_picker_query('g');
    app.push_file_picker_query('u');

    let labels: Vec<_> = app
        .file_picker_filtered_indices()
        .iter()
        .map(|idx| app.file_picker_entries()[*idx].label())
        .collect();
    assert_eq!(labels, vec!["docs/guide.md"]);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn fuzzy_file_picker_does_not_match_directory_segments() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("leaf-fuzzy-picker-cla-{unique}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join(".notes/backup")).unwrap();
    fs::write(root.join(".notes/backup/PLAN.md"), "# Plan\n").unwrap();
    fs::write(root.join("claude.md"), "# Claude\n").unwrap();

    let mut app = App::new_with_source(
        Vec::new(),
        Vec::new(),
        AppConfig {
            filename: "picker".to_string(),
            source: String::new(),
            debug_input: false,
            watch: false,
            filepath: None,
            last_file_state: None,
        },
    );

    assert!(app.open_fuzzy_file_picker(root.clone()));
    app.push_file_picker_query('c');
    app.push_file_picker_query('l');
    app.push_file_picker_query('a');

    let labels: Vec<_> = app
        .file_picker_filtered_indices()
        .iter()
        .map(|idx| app.file_picker_entries()[*idx].label())
        .collect();
    assert!(labels.contains(&"claude.md"));
    assert!(!labels.contains(&".notes/backup/PLAN.md"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn fuzzy_file_picker_tracks_match_positions_for_highlighting() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("leaf-fuzzy-picker-highlight-{unique}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("claude.md"), "# Claude\n").unwrap();

    let mut app = App::new_with_source(
        Vec::new(),
        Vec::new(),
        AppConfig {
            filename: "picker".to_string(),
            source: String::new(),
            debug_input: false,
            watch: false,
            filepath: None,
            last_file_state: None,
        },
    );

    assert!(app.open_fuzzy_file_picker(root.clone()));
    app.push_file_picker_query('c');
    app.push_file_picker_query('l');
    app.push_file_picker_query('a');

    let labels: Vec<_> = app
        .file_picker_filtered_indices()
        .iter()
        .map(|idx| app.file_picker_entries()[*idx].label())
        .collect();
    assert_eq!(labels, vec!["claude.md"]);
    assert_eq!(app.file_picker_match_positions(0), &[0, 1, 2]);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn fuzzy_file_picker_prefers_compact_matches() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("leaf-fuzzy-picker-compact-{unique}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("case.md"), "# Case\n").unwrap();
    fs::write(root.join("ciase.md"), "# Ciase\n").unwrap();

    let mut app = App::new_with_source(
        Vec::new(),
        Vec::new(),
        AppConfig {
            filename: "picker".to_string(),
            source: String::new(),
            debug_input: false,
            watch: false,
            filepath: None,
            last_file_state: None,
        },
    );

    assert!(app.open_fuzzy_file_picker(root.clone()));
    app.push_file_picker_query('c');
    app.push_file_picker_query('a');

    let labels: Vec<_> = app
        .file_picker_filtered_indices()
        .iter()
        .map(|idx| app.file_picker_entries()[*idx].label())
        .collect();
    assert_eq!(labels, vec!["case.md", "ciase.md"]);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn fuzzy_file_picker_prefers_contiguous_matches_over_earlier_scattered_matches() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("leaf-fuzzy-picker-contiguous-{unique}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join(".notes/todo")).unwrap();
    fs::create_dir_all(root.join(".notes/tests")).unwrap();
    fs::write(root.join(".notes/todo/review-chatgpt.md"), "# ChatGPT\n").unwrap();
    fs::write(root.join(".notes/tests/themes-showcase.md"), "# Showcase\n").unwrap();

    let mut app = App::new_with_source(
        Vec::new(),
        Vec::new(),
        AppConfig {
            filename: "picker".to_string(),
            source: String::new(),
            debug_input: false,
            watch: false,
            filepath: None,
            last_file_state: None,
        },
    );

    assert!(app.open_fuzzy_file_picker(root.clone()));
    app.push_file_picker_query('c');
    app.push_file_picker_query('a');

    let labels: Vec<_> = app
        .file_picker_filtered_indices()
        .iter()
        .map(|idx| app.file_picker_entries()[*idx].label())
        .collect();
    let showcase_idx = labels
        .iter()
        .position(|label| *label == ".notes/tests/themes-showcase.md")
        .unwrap();
    let chatgpt_idx = labels
        .iter()
        .position(|label| *label == ".notes/todo/review-chatgpt.md")
        .unwrap();
    assert!(showcase_idx < chatgpt_idx);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn fuzzy_file_picker_prefers_filename_prefix_matches() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("leaf-fuzzy-picker-prefix-{unique}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("todo-case.md"), "# Todo\n").unwrap();
    fs::write(root.join("case-study.md"), "# Case\n").unwrap();

    let mut app = App::new_with_source(
        Vec::new(),
        Vec::new(),
        AppConfig {
            filename: "picker".to_string(),
            source: String::new(),
            debug_input: false,
            watch: false,
            filepath: None,
            last_file_state: None,
        },
    );

    assert!(app.open_fuzzy_file_picker(root.clone()));
    app.push_file_picker_query('c');
    app.push_file_picker_query('a');

    let labels: Vec<_> = app
        .file_picker_filtered_indices()
        .iter()
        .map(|idx| app.file_picker_entries()[*idx].label())
        .collect();
    assert_eq!(labels, vec!["case-study.md", "todo-case.md"]);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn fuzzy_file_picker_prefers_token_boundary_matches() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("leaf-fuzzy-picker-boundary-{unique}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("alpha-case.md"), "# Boundary\n").unwrap();
    fs::write(root.join("alphacase.md"), "# Plain\n").unwrap();

    let mut app = App::new_with_source(
        Vec::new(),
        Vec::new(),
        AppConfig {
            filename: "picker".to_string(),
            source: String::new(),
            debug_input: false,
            watch: false,
            filepath: None,
            last_file_state: None,
        },
    );

    assert!(app.open_fuzzy_file_picker(root.clone()));
    app.push_file_picker_query('c');
    app.push_file_picker_query('a');

    let labels: Vec<_> = app
        .file_picker_filtered_indices()
        .iter()
        .map(|idx| app.file_picker_entries()[*idx].label())
        .collect();
    assert_eq!(labels, vec!["alpha-case.md", "alphacase.md"]);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn fuzzy_file_picker_prefers_shallower_paths_on_equal_scores() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("leaf-fuzzy-picker-depth-{unique}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("nested/deeper")).unwrap();
    fs::write(root.join("case.md"), "# Root\n").unwrap();
    fs::write(root.join("nested/deeper/case.md"), "# Nested\n").unwrap();

    let mut app = App::new_with_source(
        Vec::new(),
        Vec::new(),
        AppConfig {
            filename: "picker".to_string(),
            source: String::new(),
            debug_input: false,
            watch: false,
            filepath: None,
            last_file_state: None,
        },
    );

    assert!(app.open_fuzzy_file_picker(root.clone()));
    app.push_file_picker_query('c');
    app.push_file_picker_query('a');
    app.push_file_picker_query('s');
    app.push_file_picker_query('e');

    let labels: Vec<_> = app
        .file_picker_filtered_indices()
        .iter()
        .map(|idx| app.file_picker_entries()[*idx].label())
        .collect();
    assert_eq!(labels, vec!["case.md", "nested/deeper/case.md"]);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn fuzzy_file_picker_allows_q_in_query() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("leaf-fuzzy-picker-q-{unique}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("query.md"), "# Query\n").unwrap();

    let mut app = App::new_with_source(
        Vec::new(),
        Vec::new(),
        AppConfig {
            filename: "picker".to_string(),
            source: String::new(),
            debug_input: false,
            watch: false,
            filepath: None,
            last_file_state: None,
        },
    );

    assert!(app.open_fuzzy_file_picker(root.clone()));
    app.push_file_picker_query('q');
    assert_eq!(app.file_picker_query(), "q");

    let labels: Vec<_> = app
        .file_picker_filtered_indices()
        .iter()
        .map(|idx| app.file_picker_entries()[*idx].label())
        .collect();
    assert_eq!(labels, vec!["query.md"]);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn fuzzy_file_picker_skips_ignored_technical_directories() {
    let root = unique_temp_dir("leaf-fuzzy-picker-ignore");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join(".git")).unwrap();
    fs::create_dir_all(root.join("target")).unwrap();
    fs::create_dir_all(root.join("vendor")).unwrap();
    fs::create_dir_all(root.join("var")).unwrap();
    fs::create_dir_all(root.join(".notes")).unwrap();
    fs::write(root.join(".git/ignored.md"), "# Ignored\n").unwrap();
    fs::write(root.join("target/ignored.md"), "# Ignored\n").unwrap();
    fs::write(root.join("vendor/ignored.md"), "# Ignored\n").unwrap();
    fs::write(root.join("var/ignored.md"), "# Ignored\n").unwrap();
    fs::write(root.join(".notes/kept.md"), "# Kept\n").unwrap();

    let mut app = App::new_with_source(
        Vec::new(),
        Vec::new(),
        AppConfig {
            filename: "picker".to_string(),
            source: String::new(),
            debug_input: false,
            watch: false,
            filepath: None,
            last_file_state: None,
        },
    );

    assert!(app.open_fuzzy_file_picker(root.clone()));

    let labels: Vec<_> = app
        .file_picker_filtered_indices()
        .iter()
        .map(|idx| app.file_picker_entries()[*idx].label())
        .collect();
    assert_eq!(labels, vec![".notes/kept.md"]);
    assert_eq!(app.file_picker_truncation(), None);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn fuzzy_file_picker_reports_directory_limit_truncation() {
    let root = unique_temp_dir("leaf-fuzzy-picker-dir-limit");
    let _ = fs::remove_dir_all(&root);
    for idx in 0..5_050usize {
        let dir = root.join(format!("nested-{idx:04}"));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(format!("file-{idx:04}.md")), "# File\n").unwrap();
    }

    let mut app = App::new_with_source(
        Vec::new(),
        Vec::new(),
        AppConfig {
            filename: "picker".to_string(),
            source: String::new(),
            debug_input: false,
            watch: false,
            filepath: None,
            last_file_state: None,
        },
    );

    assert!(app.open_fuzzy_file_picker(root.clone()));
    assert_eq!(
        app.file_picker_truncation(),
        Some(crate::app::PickerIndexTruncation::Directory)
    );
    assert!(!app.file_picker_entries().is_empty());

    let _ = fs::remove_dir_all(root);
}

#[test]
fn fuzzy_file_picker_reports_file_limit_truncation() {
    let root = unique_temp_dir("leaf-fuzzy-picker-file-limit");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    for idx in 0..10_050usize {
        fs::write(root.join(format!("file-{idx:05}.md")), "# File\n").unwrap();
    }

    let mut app = App::new_with_source(
        Vec::new(),
        Vec::new(),
        AppConfig {
            filename: "picker".to_string(),
            source: String::new(),
            debug_input: false,
            watch: false,
            filepath: None,
            last_file_state: None,
        },
    );

    assert!(app.open_fuzzy_file_picker(root.clone()));
    assert_eq!(
        app.file_picker_truncation(),
        Some(crate::app::PickerIndexTruncation::File)
    );
    assert_eq!(app.file_picker_entries().len(), 10_000);

    let _ = fs::remove_dir_all(root);
}
