// dupchk — find files with duplicate sizes (a deliberate proxy for duplicate
// content: the workload is large media files, where size collisions are near
// certain duplicates and hashing is too slow to be worth it).
//
// Default scan is the directory's immediate files only; -r recurses.
// Walk semantics match fd's defaults: hidden files and gitignored paths are
// skipped, symlinks are not followed.

use std::collections::BTreeMap;
use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::mpsc;

use clap::Parser;
use dialoguer::{Select, theme::ColorfulTheme};
use ignore::{WalkBuilder, WalkState};

#[derive(Parser)]
#[command(
    name = "dupchk",
    about = "List files with duplicate sizes (default: current dir, depth 1)"
)]
struct Args {
    /// Group output under a "# <size> bytes" header per group
    #[arg(short = 'g')]
    group: bool,

    /// Interactive: pick one file to keep per group, trash the rest
    #[arg(short = 'i')]
    interactive: bool,

    /// Recurse into subdirectories
    #[arg(short = 'r')]
    recursive: bool,

    /// Directory to scan
    #[arg(default_value = ".")]
    dir: PathBuf,
}

struct Style {
    on: bool,
}

impl Style {
    fn paint(&self, code: &str, s: &str) -> String {
        if self.on {
            format!("\x1b[{code}m{s}\x1b[0m")
        } else {
            s.to_string()
        }
    }
    fn header(&self, s: &str) -> String {
        self.paint("36", s) // cyan
    }
    fn file(&self, s: &str) -> String {
        self.paint("32", s) // green
    }
    fn muted(&self, s: &str) -> String {
        self.paint("90", s) // dim grey
    }
}

fn scan(dir: &PathBuf, recursive: bool) -> Vec<(u64, PathBuf)> {
    let (tx, rx) = mpsc::channel::<(u64, PathBuf)>();
    WalkBuilder::new(dir)
        .max_depth(if recursive { None } else { Some(1) })
        .build_parallel()
        .run(|| {
            let tx = tx.clone();
            Box::new(move |entry| {
                if let Ok(e) = entry
                    && e.file_type().is_some_and(|t| t.is_file())
                    && let Ok(md) = e.metadata()
                {
                    let _ = tx.send((md.len(), e.into_path()));
                }
                WalkState::Continue
            })
        });
    drop(tx);
    rx.into_iter().collect()
}

fn main() -> ExitCode {
    let args = Args::parse();

    if !args.dir.is_dir() {
        eprintln!("Not a directory: {}", args.dir.display());
        return ExitCode::FAILURE;
    }

    let style = Style {
        on: std::io::stdout().is_terminal(),
    };

    let mut by_size: BTreeMap<u64, Vec<PathBuf>> = BTreeMap::new();
    for (size, path) in scan(&args.dir, args.recursive) {
        by_size.entry(size).or_default().push(path);
    }
    by_size.retain(|_, files| files.len() > 1);
    by_size.values_mut().for_each(|files| files.sort());

    if by_size.is_empty() {
        let scope = if args.recursive { "" } else { " at depth 1" };
        println!(
            "{}",
            style.muted(&format!(
                "No duplicate-size files found{scope} in: {}",
                args.dir.display()
            ))
        );
        return ExitCode::SUCCESS;
    }

    if args.interactive {
        return interactive(&by_size, &style);
    }

    let mut first = true;
    for (size, files) in &by_size {
        if !first {
            println!();
        }
        first = false;
        if args.group {
            println!("{}", style.header(&format!("# {size} bytes")));
        }
        let indent = if args.group { "  " } else { "" };
        for f in files {
            println!("{indent}{}", style.file(&f.display().to_string()));
        }
    }
    ExitCode::SUCCESS
}

fn interactive(by_size: &BTreeMap<u64, Vec<PathBuf>>, style: &Style) -> ExitCode {
    println!("Duplicate-size groups (interactive)…");
    let mut failed = false;
    for (size, files) in by_size {
        println!();
        println!("{}", style.header(&format!("# {size} bytes")));
        let names: Vec<String> = files.iter().map(|f| f.display().to_string()).collect();
        let picked = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("keep (size: {size} bytes, Esc skips)"))
            .items(&names)
            .default(0)
            .interact_opt();
        let Ok(Some(keep)) = picked else {
            println!("{}", style.muted("(skipped)"));
            continue;
        };
        for (i, f) in files.iter().enumerate() {
            if i == keep {
                continue;
            }
            if let Err(e) = trash::delete(f) {
                eprintln!("trash failed for {}: {e}", f.display());
                failed = true;
            }
        }
        println!("{}", style.file(&format!("kept: {}", names[keep])));
    }
    if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
