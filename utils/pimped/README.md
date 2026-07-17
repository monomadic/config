# pimped

A minimal, fast zsh prompt renderer — the replacement for starship in this repo.

## Why

Starship re-execs a process **and** re-parses its whole TOML on every render, and
its `git_*` / `custom.*` modules stat the working directory and spawn subprocesses
(`df`, `git`, …). On a stale SMB / network CWD those syscalls can wedge and hang
the whole prompt (see the kitty "new tab pauses in a `/Volumes` dir" issue).

This is a single static binary that:

- renders in well under a millisecond, **no subprocesses** at runtime;
- reads git branch + dirty state via linked libgit2 (`git2`), not a `git` fork;
- **hard-skips git entirely when the CWD is under `/Volumes/`**, so a dead network
  mount can never block the prompt.

## Layout

```
line 1:  <cwd>   <branch> <●-if-dirty>
line 2:  <os> <hostname> <char>
```

- `cwd` — absolute path with `$HOME` collapsed to `~` (yellow).
- `branch` — current branch, or short SHA when detached (green). Omitted outside a
  repo and on `/Volumes`.
- `●` — red, shown only when the tree has uncommitted/untracked changes.
- `char` — green success glyph on exit 0, red `❯` otherwise.

## Usage

Invoked from `config/zsh/zshrc.zsh` as:

```zsh
PROMPT="$(command pimped "$exit_status" "${(%):-%m}")"
```

`argv[1]` is the last command's exit status; `argv[2]` is the short hostname
(`%m`, expanded by zsh). Output is a zsh `PROMPT` string with ANSI colours wrapped
in `%{ … %}` zero-width markers.

## Build / install

```sh
setup/install/pimped.sh      # cargo build --release + install to ~/.local/bin
```

Glyphs are nerd-font codepoints copied verbatim from the old `starship.toml`
(branch `U+E725`, macOS `U+F0035`, success `U+F17A9`).
