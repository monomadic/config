# Structure

This repo should optimize for two things:

1. A new machine can be bootstrapped without tribal knowledge.
2. A file's location should explain what kind of thing it is.

## Rules

- `dotter/` contains deployment manifests only.
- `scripts/setup/` contains bootstrap, deploy, and health-check entrypoints.
- `scripts/install/` contains the `install-<name>.sh` build+install scripts for `src/`.
- `scripts/tweaks/` contains one-shot macOS `defaults write` tweaks. Deploy never runs these.
- `scripts/` itself contains miscellaneous helper scripts and sourceable shell snippets.
- `config/` contains active source config in a flat layout; each direct child should describe one tool or app.
- Do not add new domain buckets under `config/` such as `editors/`, `media/`, or `windowing/`.
- Prefer directory names that match the tool or app itself, such as `config/zsh`, `config/helix`, or `config/virtualdj`.
- `assets/` holds fonts, icons, LUTs, and similar static resources. Most are inert, but `assets/LUTs/` is deployed by Dotter into DaVinci Resolve and Final Cut Pro.
- `bin/` is the single directory of maintained executables and vendor wrappers.
- `src/` contains small personal utility source trees that are maintained in this repo.
- `$SRC_PATH` (default `~/src`) contains separate upstream repositories such as [safesync](https://github.com/monomadic/safesync); their installers in `scripts/install/` clone, update, and build them through `lib/git-source-install.sh`.
- `vendor/bin/` contains retained third-party or custom-built binaries.
- `_quarantine/` holds commands dropped from PATH but kept in git history. Nothing live may reference it and nothing new should be added to it.
- This repo targets macOS only. Linux-specific config (i3, sway, waybar, foot, ...) does not belong here.
- If a `config/<tool>/` directory is not wired into Dotter yet, keep it visible in the example profiles as disabled or source-only.
- Secrets and machine-private state do not live in git.
- Installers, DMGs, `.app` bundles, and archives do not live beside source config.
- Backups like `.bak`, `_old`, and `big` variants should move to an explicit archive area or be deleted.

## What This Means In Practice

Good fit for this repo:

- shell/editor/terminal config
- window manager config
- keybindings
- reusable helper scripts
- thin wrappers around kept vendor binaries
- package manifests
- Dotter package definitions
- small personal utility source trees with checked-in deployable binaries

Poor fit for this repo:

- credential-bearing files
- one-off downloads
- generated app resources
- large standalone binaries
- historical snapshots kept "just in case"
- runtime logs and scrape output

## Migration Direction

When cleaning up the repo, use this order:

1. Remove or relocate secrets and private state.
2. Move installers and large binaries out of the repo root.
3. Delete backup variants and `_old` directories.
4. Keep one canonical bootstrap path per platform.
5. Keep `config/` flat and list optional Dotter packages in the example profiles.

## Config Layout

The canonical shape is:

- `config/<tool>/...`
- `config/zsh/` for shell config only — rc files, `autoload/`, `completions/`
- `bin/` for every command, regardless of language or shell dependency
- `config/neovim/` for dormant source that stays in the repo but is not normally enabled
- `src/<tool>/` for small personal utility source maintained directly in this repo

Examples:

- `config/helix`
- `config/kitty`
- `config/mpv`
- `config/virtualdj`
- `config/yazi`
- `src/free-disk-space-widget`

Current source-only directories kept for reference or future Dotter wiring:

- `config/beatportdl`
- `config/compressor`
- `config/git`
- `config/homebrew`
- `config/iterm`
- `config/ollama`
- `config/python`
- `config/tag-media`

## Current Holding Areas

- `vendor/bin/` holds retained third-party binaries. Each gets a thin `exec` shim in `bin/` so the command is on PATH.
- `src/` holds small personal utility source when the source is intentionally maintained in this repo.
- `_quarantine/bin/` and `_quarantine/zsh-bin/` hold commands retired from PATH — unreferenced, undeployed, kept only so the history and the code are recoverable.

## Naming

- Use `setup-*` or `bootstrap-*` for first-run machine provisioning scripts.
- Use `*.example` for checked-in local config templates.
- Use `dotter/local.toml` for the untracked active machine profile, created from `dotter/local.toml.example`.
- Prefer descriptive directory names over personal shorthand when the scope is broader than one tool.

## Executable Directory

There is exactly one: `bin/`. Everything meant to be invoked as a command lives
there, flat, whatever language it is written in — zsh, bash, Python, Perl,
AppleScript, or a two-line `exec` shim around a `vendor/bin/` binary.

Dotter deploys it as one symlink **per file** into `~/.local/bin/`, which is
also where `scripts/install/*.sh` put the binaries they build (`pimped`, `leaf`,
the menu bar widgets) and where pipx and uv put theirs. One consequence worth
knowing: a new command in `bin/` must not collide with any of those names, since
Dotter refuses to overwrite a file it does not manage and the deploy fails.

Scripts should be extensionless when they are meant to be invoked as commands.
Use `.zsh`, `.sh`, or `.py` extensions only for scripts that are sourced or are
clearly single-language utilities. `bin/lib/` is the exception to "flat": it
holds sourceable helpers and preset data, not commands.

Retiring a command is `git mv bin/<cmd> _quarantine/bin/` after removing every
reference to it — not a delete.
