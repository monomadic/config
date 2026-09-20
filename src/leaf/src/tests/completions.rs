use crate::cli::{parse_cli, AutoCompleteArg};

#[test]
fn parse_cli_accepts_auto_complete() {
    let args = vec!["leaf".to_string(), "--auto-complete".to_string()];
    let options = parse_cli(&args).unwrap();
    assert_eq!(
        options.auto_complete,
        Some(AutoCompleteArg {
            shell: None,
            dump: false
        })
    );
}

#[test]
fn parse_cli_auto_complete_with_shell() {
    let args = vec![
        "leaf".to_string(),
        "--auto-complete".to_string(),
        "bash".to_string(),
    ];
    let options = parse_cli(&args).unwrap();
    assert_eq!(
        options.auto_complete,
        Some(AutoCompleteArg {
            shell: Some("bash".to_string()),
            dump: false
        })
    );
}

#[test]
fn parse_cli_auto_complete_dump() {
    let args = vec![
        "leaf".to_string(),
        "--auto-complete".to_string(),
        "dump".to_string(),
    ];
    let options = parse_cli(&args).unwrap();
    assert_eq!(
        options.auto_complete,
        Some(AutoCompleteArg {
            shell: None,
            dump: true
        })
    );
}

#[test]
fn parse_cli_auto_complete_shell_dump() {
    let args = vec![
        "leaf".to_string(),
        "--auto-complete".to_string(),
        "zsh:dump".to_string(),
    ];
    let options = parse_cli(&args).unwrap();
    assert_eq!(
        options.auto_complete,
        Some(AutoCompleteArg {
            shell: Some("zsh".to_string()),
            dump: true
        })
    );
}

#[test]
fn parse_cli_auto_complete_invalid_arg() {
    let args = vec![
        "leaf".to_string(),
        "--auto-complete".to_string(),
        "invalid".to_string(),
    ];
    assert!(parse_cli(&args).is_err());
}

#[test]
fn auto_complete_rejects_with_file() {
    let args = vec![
        "leaf".to_string(),
        "--auto-complete".to_string(),
        "README.md".to_string(),
    ];
    assert!(parse_cli(&args).is_err());
}

#[test]
fn auto_complete_rejects_with_watch() {
    let args = vec![
        "leaf".to_string(),
        "--auto-complete".to_string(),
        "--watch".to_string(),
    ];
    assert!(parse_cli(&args).is_err());
}

#[test]
fn auto_complete_rejects_with_update() {
    let args = vec![
        "leaf".to_string(),
        "--auto-complete".to_string(),
        "--update".to_string(),
    ];
    assert!(parse_cli(&args).is_err());
}

#[test]
fn auto_complete_rejects_with_config() {
    let args = vec![
        "leaf".to_string(),
        "--auto-complete".to_string(),
        "--config".to_string(),
    ];
    assert!(parse_cli(&args).is_err());
}

#[test]
fn auto_complete_rejects_with_theme() {
    let args = vec![
        "leaf".to_string(),
        "--auto-complete".to_string(),
        "--theme".to_string(),
        "arctic".to_string(),
    ];
    assert!(parse_cli(&args).is_err());
}
