# Structure

This repo should optimize for two things:

1. A new machine can be bootstrapped without tribal knowledge.
2. A file's location should explain what kind of thing it is.

## Rules

- `dotter/` contains deployment manifests only.
- `scripts/` contains bootstrap and machine-setup entrypoints.
- `config/` contains active source config in a flat layout; each direct child should describe one tool or app.
- Do not add new domain buckets under `config/` such as `editors/`, `media/`, or `windowing/`.
- Prefer directory names that match the tool or app itself, such as `config/zsh`, `config/helix`, or `config/virtualdj`.
- Do not keep the same command name implemented in both `bin/` and `config/zsh/bin/`; pick one canonical location.
- `assets/` holds fonts, icons, and similar static resources.
- `bin/` and `config/zsh/bin/` contain maintained executables and compatibility wrappers.
- `utils/` contains small personal utility source trees that are maintained in this repo.
- `vendor/bin/` contains retained third-party or custom-built binaries.
- `archive/` contains kept-but-not-active material such as installers, app bundles, and old variants.
- `config/neovim/` can remain in the tree as dormant source, but should not be part of active machine profiles unless revived.
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
3. Delete or archive backup variants and `_old` directories.
4. Keep one canonical bootstrap path per platform.
5. Keep `config/` flat and list optional Dotter packages in the example profiles.

## Config Layout

The canonical shape is:

- `config/<tool>/...`
- `config/zsh/` for shell config plus `config/zsh/bin/` for zsh-specific commands
- `config/neovim/` for dormant source that stays in the repo but is not normally enabled
- `utils/<tool>/` for small personal utility source maintained directly in this repo

Examples:

- `config/helix`
- `config/kitty`
- `config/mpv`
- `config/virtualdj`
- `config/yazi`
- `utils/free-disk-space-widget`

Current source-only directories kept for reference or future Dotter wiring:

- `config/beatportdl`
- `config/compressor`
- `config/git`
- `config/homebrew`
- `config/iterm`
- `config/mpv-vj`
- `config/ollama`
- `config/python`
- `config/refind`
- `config/tag-media`
- `config/virtualdj`
- `config/weston`
- `config/yofi`

## Current Holding Areas

- `vendor/bin/` is the temporary home for binaries you still need to sort.
- `utils/` holds small personal utility source when the source is intentionally maintained in this repo.
- `archive/installers/` holds DMGs and installers.
- `archive/apps/` holds app bundles and generated app artifacts.
- `archive/config-variants/` holds backup or oversized config variants.
- `archive/examples/` holds demo scripts and reference executables that should not be on your normal PATH.

## Naming

- Use `setup-*` or `bootstrap-*` for first-run machine provisioning scripts.
- Use `*.example` for checked-in local config templates.
- Use `dotter/local.toml` for the untracked active machine profile.
- Prefer descriptive directory names over personal shorthand when the scope is broader than one tool.

## Executable Directories

There are two maintained script directories:

- `bin/` — standalone tools and third-party wrappers that are general-purpose or not tied to a specific shell. Deployed to `~/.bin/` via Dotter.
- `config/zsh/bin/` — zsh-specific utilities, shell workflow scripts, and tools that depend on the zsh environment or autoloaded functions. Deployed to `~/.zsh/bin/` via Dotter.

Scripts in both directories should be extensionless when they are meant to be invoked as commands. Use `.zsh`, `.sh`, or `.py` extensions only for scripts that are sourced or are clearly single-language utilities.
