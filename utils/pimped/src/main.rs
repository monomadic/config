//! pimped — minimal, fast zsh prompt renderer.
//!
//! Invoked from a zsh `precmd` hook as: `pimped <exit-status> <short-hostname>`
//! and its stdout is assigned to `PROMPT`. Colours are emitted as raw ANSI SGR
//! sequences wrapped in zsh's `%{ … %}` zero-width markers so the line editor
//! counts prompt width correctly; literal `%` in dynamic text is doubled.
//!
//! Layout (mirrors the previous starship config):
//!   line 1:  <cwd>   <branch> <dirty?>
//!   line 2:  <os> <hostname> <char>
//!
//! Git is read via `gix` (pure Rust, no subprocess) and is HARD-SKIPPED whenever
//! the working directory is on a `/Volumes/*` mount, so a stale SMB share can
//! never wedge the prompt.

use std::env;
use std::path::Path;

// Glyphs copied verbatim from the old starship.toml.
const BRANCH_GLYPH: char = '\u{e725}'; // nerd-font git branch
const OS_GLYPH: char = '\u{f0035}'; // nerd-font apple/macOS
const DIRTY_GLYPH: char = '\u{1008a4}'; // SF Symbol dirty marker (alt: '\u{1001ff}')
const AHEAD_GLYPH: char = '\u{21e1}'; // ⇡ local ahead of upstream
const BEHIND_GLYPH: char = '\u{21e3}'; // ⇣ local behind upstream
const OK_GLYPH: char = '\u{f17a9}'; // success prompt char
const ERR_GLYPH: char = '\u{276f}'; // ❯ error prompt char

/// Wrap an ANSI SGR code in zsh zero-width markers.
fn col(code: &str) -> String {
    format!("%{{\x1b[{code}m%}}")
}

const RESET: &str = "%{\x1b[0m%}";

/// Escape text that lands in `PROMPT`: only `%` is special.
fn esc(s: &str) -> String {
    s.replace('%', "%%")
}

/// Current dir with `$HOME` collapsed to `~`.
fn directory() -> String {
    let cwd = env::current_dir().unwrap_or_default();
    let cwd = cwd.to_string_lossy();
    let shown = match env::var("HOME") {
        Ok(home) if !home.is_empty() && cwd.starts_with(&home) => {
            format!("~{}", &cwd[home.len()..])
        }
        _ => cwd.to_string(),
    };
    format!("{}{}{}", col("33"), esc(&shown), RESET)
}

/// ` <branch>` plus a red ● when the tree is dirty. Empty when not in a repo,
/// or when the CWD is on a network mount (never touch it).
fn git() -> String {
    let cwd = match env::current_dir() {
        Ok(c) => c,
        Err(_) => return String::new(),
    };
    if cwd.starts_with(Path::new("/Volumes/")) {
        return String::new();
    }

    let repo = match git2::Repository::discover(&cwd) {
        Ok(r) => r,
        Err(_) => return String::new(),
    };

    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return String::new(), // unborn branch / no commits
    };

    let name = if head.is_branch() {
        head.shorthand().unwrap_or("?").to_string()
    } else {
        // Detached HEAD: short object id.
        head.target()
            .map(|oid| oid.to_string().chars().take(7).collect())
            .unwrap_or_else(|| "?".to_string())
    };

    let mut out = format!(" {}{BRANCH_GLYPH} {}{}", col("1;32"), esc(&name), RESET);

    let mut opts = git2::StatusOptions::new();
    opts.include_untracked(true).include_ignored(false);
    let dirty = repo
        .statuses(Some(&mut opts))
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    if dirty {
        out.push_str(&format!(" {}{DIRTY_GLYPH}{} ", col("1;31"), RESET));
    }

    // Ahead/behind vs upstream — a local graph walk against the cached remote
    // ref (no network), so it's stale until the next fetch/pull.
    if head.is_branch() {
        if let (Some(local), Some(upstream)) = (
            head.target(),
            repo.find_branch(&name, git2::BranchType::Local)
                .ok()
                .and_then(|b| b.upstream().ok())
                .and_then(|u| u.get().target()),
        ) {
            if let Ok((ahead, behind)) = repo.graph_ahead_behind(local, upstream) {
                if ahead > 0 {
                    out.push_str(&format!(" {}{AHEAD_GLYPH}{ahead}{}", col("1;36"), RESET));
                }
                if behind > 0 {
                    out.push_str(&format!(" {}{BEHIND_GLYPH}{behind}{}", col("1;35"), RESET));
                }
            }
        }
    }
    out
}

fn main() {
    let mut args = env::args().skip(1);
    let status: i32 = args.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let host = args.next().unwrap_or_default();

    let os = format!("{}{OS_GLYPH}{}", col("1;37"), RESET);
    let host = format!("{}{}{}", col("1;37"), esc(&host), RESET);
    let ch = if status == 0 {
        format!("{}{OK_GLYPH}{}", col("1;32"), RESET)
    } else {
        format!("{}{ERR_GLYPH}{}", col("1;31"), RESET)
    };

    // line 1 then line 2, trailing space so the cursor sits clear of the char
    print!("{}{}\n{} {} {} ", directory(), git(), os, host, ch);
}
