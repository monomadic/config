# AGENTS.md

Personal dotfiles repo, deployed with [Dotter](https://github.com/SuperCuber/dotter).
**macOS only** — Linux config (i3, sway, waybar, foot, weston, refind, ...) has been removed; don't add it back.
Full layout rules: [docs/STRUCTURE.md](docs/STRUCTURE.md). Bootstrap docs: [README.md](README.md).

`CLAUDE.md` is a symlink to this file — edit `AGENTS.md`, never the symlink.

## The one thing to understand first

Dotter **symlinks** repo files into place (`default_target_type = "symbolic"`).
`config/yazi/` IS `~/.config/yazi/` on this machine. Editing a file in this repo
changes the live config immediately — no build, no deploy. Treat edits to `config/`
as live changes, and don't "install" anything by copying files out of the repo.

A deploy run is only needed when the *mapping* changes: a new package, a new
file entry in `dotter/global.toml`, or a changed target path.

## The repo is location-independent

Nothing assumes `~/config`. `setup/**` scripts resolve the root from their own
path (`dirname $0/../..`), `config/zsh/bin/dotter-deploy` resolves through its
symlink with `${0:A}`, and `.zshenv` derives `$DOTFILES_DIR` the same way.
When you write a new script, follow that pattern — never hardcode `$HOME/config`,
and prefer `$DOTFILES_DIR` in shell config.

Tool configs that need to call a repo script should reference the **deployed**
path (`~/.zsh/bin/foo`, `~/.config/kitty/…`), not the repo path — the symlink is
stable wherever the checkout lives. Known exception: `config/zellij/layouts/*.kdl`
still holds absolute paths because Zellij's KDL does no env/tilde expansion.

## How deployment works

Two manifests, one syntax:

- `dotter/global.toml` — every package as a `[<name>.files]` section mapping repo path → target path. Alphabetical, grouped by purpose. Use the long `{ target = ..., type = ..., recurse = ... }` form only when overriding defaults.
- `dotter/local.toml` — **gitignored**; this machine's package selection plus variable overrides. Never expect it in git; never commit it.
- `dotter/local.toml.example` — the single tracked template (there is no longer a per-platform set); optional packages stay listed but commented out so it doubles as an inventory.
- Some `config/<tool>/` dirs are intentionally source-only (not wired into Dotter yet): beatportdl, compressor, git, homebrew, iterm, ollama, python, tag-media.

## Commands

```bash
setup/macos/bootstrap.sh      # full machine setup: CLT, Homebrew, Brewfile, clone, deploy
setup/macos/check.sh          # preflight: validates manifests, source paths, package names
setup/macos/packages.sh list  # show every package and whether it is on for this machine
setup/macos/packages.sh enable <name>   # edit local.toml without hand-syncing lists
setup/macos/deploy.sh         # run Dotter deploy (wraps config/zsh/bin/dotter-deploy)
setup/macos/deploy.sh --full  # also syncs Yazi plugin packages and reapplies macOS app icons
```

`deploy.sh` runs `check.sh` automatically unless `DOTTER_SKIP_HEALTHCHECK=1`.
There is no CI. For config and script changes, verification = `check.sh` passing,
plus `zsh -n` / `bash -n` on any shell script you touched.

## The utils/ side: real code, real builds

`utils/<tool>/` holds compiled source (Go: widgets, `spill`, `iospeed`, `open-in-forklift`,
`obsbot-rtsp-widget`; Rust: `leaf`, `pimped`, `motherfucker`). These are the only parts of
the repo with a build step and tests — Dotter does not touch them.

```bash
setup/install/<name>.sh            # canonical build+install; most install to ~/.local/bin
cd utils/leaf && cargo test        # Rust suites live in src/tests/ (leaf is the largest)
cd utils/leaf && cargo test toc    # single test / filter
cd utils/obsbot-rtsp-widget && go test ./...
```

Prefer the `setup/install/*.sh` script over a hand-rolled `cargo install`/`go build` — it
pins the install path the rest of the config expects (e.g. `pimped` must be on PATH for the
zsh precmd prompt hook in `config/zsh/zshrc.zsh` to work).

## Recipe: add config for a new tool

1. Create `config/<tool>/` — flat, named after the tool itself (`config/helix`, not `config/editors/helix`).
2. Add a `[<tool>.files]` section to `dotter/global.toml`, in the right alphabetical spot within its group.
3. Add `"<tool>"` to `dotter/local.toml.example` (commented out unless it should be on by default).
4. `setup/macos/packages.sh enable <tool>` if it should be active here, then `setup/macos/deploy.sh`.

`check.sh` fails on manifest entries pointing at missing repo paths and on
`local.toml` selecting packages that don't exist in `global.toml`.

## Where things go

| Path | Purpose |
|---|---|
| `config/<tool>/` | active config source, one tool per directory, flat |
| `config/zsh/` | shell config; `config/zsh/bin/` for zsh-dependent commands (→ `~/.zsh/bin/`) |
| `bin/` | general-purpose standalone executables (→ `~/.bin/`) |
| `scripts/` | sourceable snippets and misc helpers (not on PATH) |
| `setup/` | bootstrap, deploy, and machine-setup entrypoints |
| `dotter/` | deployment manifests only |
| `utils/<tool>/` | small personal utility source trees (Go widgets, `leaf` is Rust) — build via `setup/install/*.sh` |
| `assets/` | fonts and icons |
| `vendor/bin/`, `archive/` | holding areas — don't add to or modify these |

## Rules that prevent rework

- **No new domain buckets under `config/`** (`editors/`, `media/`, `windowing/`...). A few legacy ones exist; don't add files to them — use `config/<tool>/`.
- A command name lives in **either** `bin/` **or** `config/zsh/bin/`, never both. Pick `config/zsh/bin/` only if it depends on zsh or autoloaded functions.
- Executables meant to be invoked as commands are **extensionless**. Use `.zsh`/`.sh`/`.py` only for sourced or clearly single-language utilities.
- Local config templates are checked in as `*.example`; the live file is gitignored.
- Helix is the active editor. `config/neovim/` is dormant source — keep it out of active profiles.
- `Brewfile` is bootstrap-critical: only add things the shell/editor/config actually need. Large apps with no config dependency go in `Brewfile.optional`, which bootstrap never installs.
- No secrets, credentials, installers, `.app` bundles, or large binaries in the tree. `.env` is gitignored; `.env.example` documents expected vars.
- Commit messages: short imperative subject line ("Add ytq pop command", "Refactor Zellij config for Yazi-first sessions").
