use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EditorKind {
    Terminal,
    Gui,
}

pub(crate) fn binary_name(editor_cmd: &str) -> &str {
    let (bin, _) = split_editor_cmd(editor_cmd);
    let after_sep = bin
        .rsplit_once(['/', '\\'])
        .map(|(_, name)| name)
        .unwrap_or(bin);
    after_sep.strip_suffix(".exe").unwrap_or(after_sep)
}

pub(crate) fn classify(editor_cmd: &str) -> EditorKind {
    match binary_name(editor_cmd) {
        "code" | "codium" | "subl" | "gedit" | "kate" | "mousepad" | "notepad" | "notepad++"
        | "zed" | "xjed" | "termux-open" => EditorKind::Gui,
        _ => EditorKind::Terminal,
    }
}

pub(crate) fn split_editor_cmd(cmd: &str) -> (&str, Vec<&str>) {
    let trimmed = cmd.trim();

    for quote in ['"', '\''] {
        if trimmed.starts_with(quote) {
            if let Some(end) = trimmed[1..].find(quote) {
                let bin = &trimmed[1..=end];
                let rest = trimmed[end + 2..].trim();
                let args = if rest.is_empty() {
                    vec![]
                } else {
                    rest.split_whitespace().collect()
                };
                return (bin, args);
            }
        }
    }

    if !trimmed.contains(' ') && !trimmed.contains('\t') {
        return (trimmed, vec![]);
    }

    let first_space = trimmed.find([' ', '\t']).unwrap_or(trimmed.len());
    if trimmed[..first_space].contains('\\') {
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        let mut split_at = parts.len();
        while split_at > 1 && parts[split_at - 1].starts_with('-') {
            split_at -= 1;
        }
        let bin_end = if split_at < parts.len() {
            parts[split_at].as_ptr() as usize - trimmed.as_ptr() as usize
        } else {
            trimmed.len()
        };
        let bin = trimmed[..bin_end].trim_end();
        let args = parts[split_at..].to_vec();
        return (bin, args);
    }

    let mut parts = trimmed.split_whitespace();
    let bin = parts.next().unwrap_or(trimmed);
    let args: Vec<&str> = parts.collect();
    (bin, args)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct EditorEntry {
    pub(crate) name: String,
    pub(crate) command: String,
    pub(crate) kind: EditorKind,
}

const KNOWN_EDITORS: &[(&str, EditorKind)] = &[
    ("nano", EditorKind::Terminal),
    ("vim", EditorKind::Terminal),
    ("vi", EditorKind::Terminal),
    ("nvim", EditorKind::Terminal),
    ("micro", EditorKind::Terminal),
    ("helix", EditorKind::Terminal),
    ("emacs", EditorKind::Terminal),
    ("jed", EditorKind::Terminal),
    ("code", EditorKind::Gui),
    ("codium", EditorKind::Gui),
    ("subl", EditorKind::Gui),
    ("gedit", EditorKind::Gui),
    ("kate", EditorKind::Gui),
    ("mousepad", EditorKind::Gui),
    ("zed", EditorKind::Gui),
    ("xjed", EditorKind::Gui),
    ("notepad", EditorKind::Gui),
    ("notepad++", EditorKind::Gui),
];

pub(crate) fn which(bin: &str) -> Option<PathBuf> {
    if bin.contains('/') || bin.contains('\\') {
        let p = Path::new(bin);
        return p.is_file().then(|| p.to_path_buf());
    }
    let path_var = std::env::var("PATH").ok()?;
    let separator = if cfg!(target_os = "windows") {
        ';'
    } else {
        ':'
    };
    let candidates: Vec<String> = if cfg!(target_os = "windows") && !bin.contains('.') {
        vec![
            format!("{bin}.exe"),
            format!("{bin}.cmd"),
            format!("{bin}.bat"),
        ]
    } else {
        vec![bin.to_string()]
    };
    path_var.split(separator).find_map(|dir| {
        candidates
            .iter()
            .map(|name| Path::new(dir).join(name))
            .find(|p| p.is_file())
    })
}

pub(crate) fn scan_available_editors() -> Vec<EditorEntry> {
    let mut found = Vec::new();
    let is_termux = std::env::var("TERMUX_VERSION").is_ok();

    for &(name, kind) in KNOWN_EDITORS {
        if is_termux && kind == EditorKind::Gui {
            continue;
        }
        if which(name).is_some() {
            found.push(EditorEntry {
                name: name.to_string(),
                command: name.to_string(),
                kind,
            });
        }
    }

    if is_termux && which("termux-open").is_some() {
        found.push(EditorEntry {
            name: "intent".to_string(),
            command: "termux-open --chooser".to_string(),
            kind: EditorKind::Gui,
        });
    }

    found.sort_by_key(|e| match e.kind {
        EditorKind::Terminal => 0,
        EditorKind::Gui => 1,
    });
    found
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum TerminalEmulator {
    Kitty,
    GnomeTerminal,
    MacTerminal(String),
    WindowsTerminal,
    Termux,
    Unknown,
}

pub(crate) fn detect_terminal_emulator() -> TerminalEmulator {
    if std::env::var("KITTY_PID").is_ok() {
        return TerminalEmulator::Kitty;
    }
    if std::env::var("GNOME_TERMINAL_SCREEN").is_ok() {
        return TerminalEmulator::GnomeTerminal;
    }
    if std::env::var("WT_SESSION").is_ok() {
        return TerminalEmulator::WindowsTerminal;
    }
    if std::env::var("TERMUX_VERSION").is_ok() {
        return TerminalEmulator::Termux;
    }

    if let Ok(tp) = std::env::var("TERM_PROGRAM") {
        match tp.as_str() {
            "iTerm.app" | "iTerm2" | "Apple_Terminal" => {
                return TerminalEmulator::MacTerminal(tp);
            }
            _ => {}
        }
    }

    TerminalEmulator::Unknown
}

pub(crate) fn resolve_editor(cli_editor: Option<&str>, config_editor: Option<&str>) -> String {
    let raw = if let Some(e) = cli_editor {
        e.to_string()
    } else if let Some(e) = std::env::var("LEAF_EDITOR").ok().filter(|s| !s.is_empty()) {
        e
    } else if let Some(e) = config_editor {
        e.to_string()
    } else {
        platform_fallback_editor().to_string()
    };
    expand_editor_alias(&raw)
}

fn expand_editor_alias(editor: &str) -> String {
    match editor.trim() {
        "intent" => "termux-open --chooser".to_string(),
        _ => editor.to_string(),
    }
}

pub(crate) fn resolve_fallback_editor(editor_cmd: &str) -> Option<&'static str> {
    let fallback = platform_fallback_editor();
    if binary_name(editor_cmd) != binary_name(fallback) {
        Some(fallback)
    } else {
        None
    }
}

fn platform_fallback_editor() -> &'static str {
    if cfg!(target_os = "windows") {
        "notepad"
    } else {
        "nano"
    }
}

pub(crate) fn try_new_tab_command(
    editor: &str,
    file: &Path,
    emulator: &TerminalEmulator,
) -> Option<Command> {
    let (bin, args) = split_editor_cmd(editor);
    let file_str = file.display().to_string();

    match emulator {
        TerminalEmulator::Kitty => {
            let mut cmd = Command::new("kitty");
            cmd.arg("@")
                .arg("launch")
                .arg("--type=tab")
                .arg("--tab-title=leaf editor")
                .arg(bin);
            for a in &args {
                cmd.arg(a);
            }
            cmd.arg(&file_str);
            Some(cmd)
        }
        TerminalEmulator::GnomeTerminal => {
            let mut cmd = Command::new("gnome-terminal");
            cmd.arg("--tab")
                .arg("--title=leaf editor")
                .arg("--")
                .arg(bin);
            for a in &args {
                cmd.arg(a);
            }
            cmd.arg(&file_str);
            Some(cmd)
        }
        TerminalEmulator::MacTerminal(ref tp) => {
            let app_name = if tp == "Apple_Terminal" {
                "Terminal"
            } else {
                "iTerm"
            };
            let escaped_editor = editor.replace('\\', "\\\\").replace('"', "\\\"");
            let escaped_file = file_str.replace('\\', "\\\\").replace('"', "\\\"");
            let script = if tp == "Apple_Terminal" {
                let title_seq = r#"printf '\\033]0;leaf editor\\007'; "#;
                format!(
                    "tell application \"{app_name}\" to do script \"{title_seq}{escaped_editor} {escaped_file}\""
                )
            } else {
                format!(
                    "tell application \"{app_name}\" to tell current window to \
                     create tab with default profile command \"{escaped_editor} {escaped_file}\""
                )
            };
            let mut cmd = Command::new("osascript");
            cmd.arg("-e").arg(script);
            Some(cmd)
        }
        TerminalEmulator::WindowsTerminal => {
            let mut cmd = Command::new("wt");
            cmd.arg("new-tab")
                .arg("--title")
                .arg("leaf editor")
                .arg(bin);
            for a in &args {
                cmd.arg(a);
            }
            cmd.arg(&file_str);
            Some(cmd)
        }
        TerminalEmulator::Termux | TerminalEmulator::Unknown => None,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EditorResult {
    Opened,
    NeedsSameTerminal,
}

pub(crate) fn open_in_editor(
    editor: &str,
    file: &Path,
    kind: EditorKind,
    emulator: &TerminalEmulator,
) -> Result<EditorResult, String> {
    let (bin, args) = split_editor_cmd(editor);
    match kind {
        EditorKind::Gui => {
            let exec = which(bin).unwrap_or_else(|| PathBuf::from(bin));
            Command::new(&exec)
                .args(&args)
                .arg(file)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .map_err(|e| format!("{bin}: {e}"))?;
            Ok(EditorResult::Opened)
        }
        EditorKind::Terminal => {
            if let Some(mut cmd) = try_new_tab_command(editor, file, emulator) {
                cmd.stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null());
                if cmd.spawn().is_ok() {
                    return Ok(EditorResult::Opened);
                }
            }
            Ok(EditorResult::NeedsSameTerminal)
        }
    }
}
