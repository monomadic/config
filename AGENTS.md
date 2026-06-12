# AGENTS.md

Personal dotfiles repo, deployed with [Dotter](https://github.com/SuperCuber/dotter).
Primary platform is macOS; Linux configs (i3, sway, foot, waybar, ...) live in-tree but are mostly dormant.
Full layout rules: [docs/STRUCTURE.md](docs/STRUCTURE.md). Bootstrap docs: [README.md](README.md).

## The one thing to understand first

Dotter **symlinks** repo files into place (`default_target_type = "symbolic"`).
`config/yazi/` IS `~/.config/yazi/` on this machine. Editing a file in this repo
changes the live config immediately — no build, no deploy. Treat edits to `config/`
as live changes, and don't "install" anything by copying files out of the repo.

A deploy run is only needed when the *mapping* changes: a new package, a new
file entry in `dotter/global.toml`, or a changed target path.

## How deployment works

- `dotter/global.toml` — defines every package as a `[<name>.files]` section mapping repo path → target path.
- `dotter/local.toml` — **gitignored**; per-machine list of active packages plus variables. Never expect it in git; never commit it.
- `dotter/macos.toml.example` — tracked template; optional packages stay listed but commented out so the file doubles as an inventory.
- Some `config/<tool>/` dirs are intentionally source-only (not wired into Dotter yet): beatportdl, compressor, git, homebrew, iterm, ollama, python, refind, tag-media, weston, yofi.

## Commands

```bash
setup/macos/check.sh        # preflight: validates manifests, source paths, package names
setup/macos/deploy.sh       # run Dotter deploy (wraps config/zsh/bin/dotter-deploy)
setup/macos/deploy.sh --full  # also syncs Yazi plugin packages and reapplies macOS app icons
```

`deploy.sh` runs `check.sh` automatically unless `DOTTER_SKIP_HEALTHCHECK=1`.
There is no CI and no test suite. Verification = `check.sh` passing, plus
`zsh -n` / `bash -n` on any shell script you touched.

## Recipe: add config for a new tool

1. Create `config/<tool>/` — flat, named after the tool itself (`config/helix`, not `config/editors/helix`).
2. Add a `[<tool>.files]` section to `dotter/global.toml`.
3. Add `"<tool>"` to the package list in `dotter/macos.toml.example` (commented out unless it should be on by default).
4. Run `setup/macos/check.sh`, then `setup/macos/deploy.sh` if it should be active on this machine (also add it to `dotter/local.toml`).

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
- No secrets, credentials, installers, `.app` bundles, or large binaries in the tree. `.env` is gitignored; `.env.example` documents expected vars.
- Commit messages: short imperative subject line ("Add ytq pop command", "Refactor Zellij config for Yazi-first sessions").
