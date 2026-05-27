use std::path::PathBuf;

use anyhow::{bail, Context, Result};

use crate::cli::AutoCompleteArg;

const PS1_COMPLETION: &str = include_str!("../completions/leaf.ps1");
const ZSH_COMPLETION: &str = include_str!("../completions/leaf.zsh");
const BASH_COMPLETION: &str = include_str!("../completions/leaf.bash");
const FISH_COMPLETION: &str = include_str!("../completions/leaf.fish");

enum Shell {
    Pwsh,
    Zsh,
    Bash,
    Fish,
}

fn detect_shell() -> Result<Shell> {
    if let Ok(shell) = std::env::var("SHELL") {
        let basename = std::path::Path::new(&shell)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        match basename {
            "zsh" => return Ok(Shell::Zsh),
            "bash" => return Ok(Shell::Bash),
            "fish" => return Ok(Shell::Fish),
            _ => {}
        }
    }

    #[cfg(target_os = "windows")]
    return Ok(Shell::Pwsh);

    #[cfg(not(target_os = "windows"))]
    {
        for (path, shell) in [
            ("/bin/zsh", Shell::Zsh),
            ("/bin/bash", Shell::Bash),
            ("/bin/fish", Shell::Fish),
        ] {
            if std::path::Path::new(path).exists() {
                return Ok(shell);
            }
        }
        bail!("Cannot detect shell. Set $SHELL to bash, zsh, or fish")
    }
}

fn completion_dir() -> Result<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        let base = std::env::var("APPDATA").context("Cannot determine APPDATA directory")?;
        Ok(PathBuf::from(base).join("leaf").join("completions"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").context("Cannot determine HOME directory")?;
        Ok(PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("leaf")
            .join("completions"))
    }
}

fn fish_completion_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME").context("Cannot determine HOME directory")?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("fish")
        .join("completions"))
}

fn write_completion(dir: &std::path::Path, filename: &str, content: &str) -> Result<PathBuf> {
    std::fs::create_dir_all(dir)
        .with_context(|| format!("Cannot create directory: {}", dir.display()))?;
    let path = dir.join(filename);
    std::fs::write(&path, content)
        .with_context(|| format!("Cannot write completion file: {}", path.display()))?;
    Ok(path)
}

fn rc_path(shell: &Shell) -> Result<PathBuf> {
    match shell {
        Shell::Zsh => {
            let home = std::env::var("HOME").context("Cannot determine HOME directory")?;
            Ok(PathBuf::from(home).join(".zshrc"))
        }
        Shell::Bash => {
            let home = std::env::var("HOME").context("Cannot determine HOME directory")?;
            Ok(PathBuf::from(home).join(".bashrc"))
        }
        Shell::Pwsh | Shell::Fish => {
            bail!("No RC file for this shell")
        }
    }
}

#[cfg(target_os = "windows")]
fn pwsh_profile_paths() -> Result<Vec<PathBuf>> {
    let base = std::env::var("USERPROFILE").context("Cannot determine USERPROFILE directory")?;
    let base = PathBuf::from(base).join("Documents");
    Ok(vec![
        base.join("PowerShell")
            .join("Microsoft.PowerShell_profile.ps1"),
        base.join("WindowsPowerShell")
            .join("Microsoft.PowerShell_profile.ps1"),
    ])
}

fn add_source_line(rc: &std::path::Path, line: &str) -> Result<bool> {
    if let Some(parent) = rc.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let content = std::fs::read_to_string(rc).unwrap_or_default();
    if content.contains(line) {
        return Ok(false);
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(rc)
        .with_context(|| format!("Cannot open {}", rc.display()))?;
    use std::io::Write;
    if !content.is_empty() && !content.ends_with('\n') {
        writeln!(file)?;
    }
    writeln!(file, "{line}")?;
    Ok(true)
}

fn parse_shell(name: &str) -> Result<Shell> {
    match name {
        "bash" => Ok(Shell::Bash),
        "zsh" => Ok(Shell::Zsh),
        "fish" => Ok(Shell::Fish),
        "powershell" => Ok(Shell::Pwsh),
        _ => bail!("Unknown shell: '{name}'"),
    }
}

fn completion_content(shell: &Shell) -> &'static str {
    match shell {
        Shell::Bash => BASH_COMPLETION,
        Shell::Zsh => ZSH_COMPLETION,
        Shell::Fish => FISH_COMPLETION,
        Shell::Pwsh => PS1_COMPLETION,
    }
}

pub(crate) fn run_auto_complete(arg: &AutoCompleteArg) -> Result<()> {
    let shell = match &arg.shell {
        Some(name) => parse_shell(name)?,
        None => detect_shell()?,
    };

    if arg.dump {
        print!("{}", completion_content(&shell));
        return Ok(());
    }

    install_completions(&shell)
}

fn check_shell_os_compat(shell: &Shell) -> Result<()> {
    #[cfg(target_os = "windows")]
    if !matches!(shell, Shell::Pwsh) {
        let name = match shell {
            Shell::Bash => "bash",
            Shell::Zsh => "zsh",
            Shell::Fish => "fish",
            Shell::Pwsh => unreachable!(),
        };
        bail!("Shell '{name}' is not supported. Use 'powershell' instead.");
    }
    #[cfg(not(target_os = "windows"))]
    if matches!(shell, Shell::Pwsh) {
        bail!("Shell 'powershell' is not supported. Use bash, zsh, or fish.");
    }
    Ok(())
}

fn install_completions(shell: &Shell) -> Result<()> {
    check_shell_os_compat(shell)?;
    let content = completion_content(shell);

    match shell {
        Shell::Pwsh => {
            let dest = write_completion(&completion_dir()?, "leaf.ps1", content)?;
            println!("Completion file installed: {}", dest.display());

            #[cfg(target_os = "windows")]
            {
                let source_line = format!(". {}", dest.display());
                for rc in pwsh_profile_paths()? {
                    if add_source_line(&rc, &source_line)? {
                        println!("Added to {}", rc.display());
                    } else {
                        println!("Already configured in {}", rc.display());
                    }
                }
                println!("\nRestart PowerShell to activate completions.");
            }
        }
        Shell::Zsh | Shell::Bash => {
            let filename = match shell {
                Shell::Zsh => "_leaf",
                _ => "leaf.bash",
            };
            let dest = write_completion(&completion_dir()?, filename, content)?;
            println!("Completion file installed: {}", dest.display());

            let source_line = format!("source {}", dest.display());
            let rc = rc_path(shell)?;
            if add_source_line(&rc, &source_line)? {
                println!("Added to {}", rc.display());
            } else {
                println!("Already configured in {}", rc.display());
            }
            println!("\nRestart your shell or run: source {}", rc.display());
        }
        Shell::Fish => {
            let dest = write_completion(&fish_completion_dir()?, "leaf.fish", content)?;
            println!("Completion file installed: {}", dest.display());
            println!("\nCompletions are available in new fish sessions automatically.");
        }
    }

    Ok(())
}
