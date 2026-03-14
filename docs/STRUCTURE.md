# Structure

This repo should optimize for two things:

1. A new machine can be bootstrapped without tribal knowledge.
2. A file's location should explain what kind of thing it is.

## Rules

- `dotter/` contains deployment manifests only.
- `scripts/` contains bootstrap and machine-setup entrypoints.
- `config/` contains active source config grouped by domain instead of by historical arrival order.
- `config/apps/` is for app config that still belongs in the dotfiles system.
- `assets/` holds fonts, icons, and similar static resources.
- `bin/` and `config/shell/zsh/bin/` contain maintained executables and compatibility wrappers.
- `vendor/bin/` contains retained third-party or custom-built binaries.
- `archive/` contains kept-but-not-active material such as installers, app bundles, and old variants.
- `config/editors/neovim/` can remain in the tree as dormant source, but should not be part of active machine profiles unless revived.
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
5. Split Dotter packages by role, not by history.

## Current Holding Areas

- `vendor/bin/` is the temporary home for binaries you still need to sort.
- `archive/installers/` holds DMGs and installers.
- `archive/apps/` holds app bundles and generated app artifacts.
- `archive/config-variants/` holds backup or oversized config variants.
- `archive/examples/` holds demo scripts and reference executables that should not be on your normal PATH.

## Naming

- Use `setup-*` or `bootstrap-*` for first-run machine provisioning scripts.
- Use `*.example` for checked-in local config templates.
- Use `dotter/local.toml` for the untracked active machine profile.
- Prefer descriptive directory names over personal shorthand when the scope is broader than one tool.
