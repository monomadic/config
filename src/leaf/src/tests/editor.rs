use crate::*;
use std::path::Path;

#[test]
fn binary_name_simple() {
    assert_eq!(binary_name("nano"), "nano");
}

#[test]
fn binary_name_full_path() {
    assert_eq!(binary_name("/usr/bin/code"), "code");
}

#[test]
fn binary_name_with_args() {
    assert_eq!(binary_name("emacs -nw"), "emacs");
}

#[test]
fn binary_name_path_with_args() {
    assert_eq!(binary_name("/usr/bin/emacs -nw"), "emacs");
}

#[test]
fn binary_name_windows() {
    assert_eq!(binary_name("notepad.exe"), "notepad");
}

#[test]
fn classify_gui_editors() {
    assert_eq!(classify("code"), EditorKind::Gui);
    assert_eq!(classify("codium"), EditorKind::Gui);
    assert_eq!(classify("subl"), EditorKind::Gui);
    assert_eq!(classify("gedit"), EditorKind::Gui);
    assert_eq!(classify("kate"), EditorKind::Gui);
    assert_eq!(classify("mousepad"), EditorKind::Gui);
    assert_eq!(classify("notepad.exe"), EditorKind::Gui);
    assert_eq!(classify("notepad++"), EditorKind::Gui);
    assert_eq!(classify("zed"), EditorKind::Gui);
    assert_eq!(classify("xjed"), EditorKind::Gui);
}

#[test]
fn classify_terminal_editors() {
    assert_eq!(classify("nano"), EditorKind::Terminal);
    assert_eq!(classify("vim"), EditorKind::Terminal);
    assert_eq!(classify("nvim"), EditorKind::Terminal);
    assert_eq!(classify("micro"), EditorKind::Terminal);
    assert_eq!(classify("helix"), EditorKind::Terminal);
    assert_eq!(classify("emacs"), EditorKind::Terminal);
    assert_eq!(classify("jed"), EditorKind::Terminal);
}

#[test]
fn classify_unknown_defaults_to_terminal() {
    assert_eq!(classify("some-unknown-editor"), EditorKind::Terminal);
}

#[test]
fn classify_full_path() {
    assert_eq!(classify("/usr/bin/code"), EditorKind::Gui);
    assert_eq!(classify("/usr/local/bin/nano"), EditorKind::Terminal);
}

#[test]
fn classify_with_args() {
    assert_eq!(classify("emacs -nw"), EditorKind::Terminal);
    assert_eq!(classify("/usr/bin/code --new-window"), EditorKind::Gui);
}

#[test]
fn split_editor_cmd_simple() {
    let (bin, args) = split_editor_cmd("nano");
    assert_eq!(bin, "nano");
    assert!(args.is_empty());
}

#[test]
fn split_editor_cmd_with_args() {
    let (bin, args) = split_editor_cmd("emacs -nw");
    assert_eq!(bin, "emacs");
    assert_eq!(args, vec!["-nw"]);
}

#[test]
fn split_editor_cmd_path_with_args() {
    let (bin, args) = split_editor_cmd("/usr/bin/emacs -nw --no-splash");
    assert_eq!(bin, "/usr/bin/emacs");
    assert_eq!(args, vec!["-nw", "--no-splash"]);
}

#[test]
fn split_editor_cmd_inner_double_quotes() {
    let (bin, args) = split_editor_cmd(r#""C:\Program Files\Notepad++\notepad++.exe" --arg"#);
    assert_eq!(bin, r"C:\Program Files\Notepad++\notepad++.exe");
    assert_eq!(args, vec!["--arg"]);
}

#[test]
fn split_editor_cmd_inner_double_quotes_no_args() {
    let (bin, args) = split_editor_cmd(r#""C:\Program Files\Notepad++\notepad++.exe""#);
    assert_eq!(bin, r"C:\Program Files\Notepad++\notepad++.exe");
    assert!(args.is_empty());
}

#[test]
fn split_editor_cmd_inner_single_quotes() {
    let (bin, args) = split_editor_cmd("'/opt/My Apps/editor' -nw");
    assert_eq!(bin, "/opt/My Apps/editor");
    assert_eq!(args, vec!["-nw"]);
}

#[test]
fn split_editor_cmd_windows_path_no_args() {
    let (bin, args) = split_editor_cmd(r"C:\Program Files\Notepad++\notepad++.exe");
    assert_eq!(bin, r"C:\Program Files\Notepad++\notepad++.exe");
    assert!(args.is_empty());
}

#[test]
fn split_editor_cmd_windows_path_trailing_args() {
    let (bin, args) = split_editor_cmd(r"C:\Program Files\Notepad++\notepad++.exe --no-session");
    assert_eq!(bin, r"C:\Program Files\Notepad++\notepad++.exe");
    assert_eq!(args, vec!["--no-session"]);
}

#[test]
fn split_editor_cmd_windows_path_duplicate_trailing_args() {
    let (bin, args) = split_editor_cmd(r"C:\Program Files\app.exe -nw -nw");
    assert_eq!(bin, r"C:\Program Files\app.exe");
    assert_eq!(args, vec!["-nw", "-nw"]);
}

#[test]
fn split_editor_cmd_unix_path_with_args() {
    let (bin, args) = split_editor_cmd("/usr/bin/emacs -nw --no-splash");
    assert_eq!(bin, "/usr/bin/emacs");
    assert_eq!(args, vec!["-nw", "--no-splash"]);
}

#[test]
fn binary_name_windows_path_with_spaces() {
    assert_eq!(
        binary_name(r"C:\Program Files\Notepad++\notepad++.exe"),
        "notepad++"
    );
}

#[test]
fn binary_name_quoted_windows_path() {
    assert_eq!(
        binary_name(r#""C:\Program Files\Notepad++\notepad++.exe" --arg"#),
        "notepad++"
    );
}

#[test]
fn classify_windows_path_with_spaces() {
    assert_eq!(
        classify(r"C:\Program Files\Notepad++\notepad++.exe"),
        EditorKind::Gui
    );
}

fn mac_tab_script(editor: &str, file: &str, term_program: &str) -> String {
    let emulator = TerminalEmulator::MacTerminal(term_program.to_string());
    let cmd = try_new_tab_command(editor, Path::new(file), &emulator).unwrap();
    let args: Vec<_> = cmd.get_args().collect();
    args[1].to_str().unwrap().to_string()
}

#[test]
fn new_tab_command_apple_terminal_has_printf() {
    let script = mac_tab_script("nano", "/tmp/test.md", "Apple_Terminal");
    assert!(script.contains("printf"));
    assert!(script.contains("do script"));
    assert!(script.contains("nano"));
    assert!(script.contains("/tmp/test.md"));
}

#[test]
fn new_tab_command_iterm_no_printf() {
    let script = mac_tab_script("nano", "/tmp/test.md", "iTerm.app");
    assert!(!script.contains("printf"));
    assert!(script.contains("create tab with default profile command"));
    assert!(script.contains("nano"));
    assert!(script.contains("/tmp/test.md"));
}

#[test]
fn new_tab_command_iterm2_no_printf() {
    let script = mac_tab_script("vim", "/tmp/test.md", "iTerm2");
    assert!(!script.contains("printf"));
    assert!(script.contains("create tab with default profile command"));
    assert!(script.contains("vim"));
}

#[test]
fn new_tab_command_iterm_file_with_spaces() {
    let script = mac_tab_script("nano", "/tmp/my file.md", "iTerm.app");
    assert!(!script.contains("printf"));
    assert!(script.contains("my file.md"));
}

#[test]
fn new_tab_command_apple_terminal_file_with_spaces() {
    let script = mac_tab_script("nano", "/tmp/my file.md", "Apple_Terminal");
    assert!(script.contains("printf"));
    assert!(script.contains("my file.md"));
}

#[test]
fn resolve_editor_cli_takes_priority() {
    let result = resolve_editor(Some("vim"), None);
    assert_eq!(result, "vim");
}

#[test]
fn resolve_editor_fallback_is_not_empty() {
    let result = resolve_editor(None, None);
    assert!(!result.is_empty());
}

#[test]
fn resolve_editor_config_takes_priority_over_fallback() {
    let result = resolve_editor(None, Some("helix"));
    assert_eq!(result, "helix");
}
