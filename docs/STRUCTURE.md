# Structure

This repo should optimize for two things:

1. A new machine can be bootstrapped without tribal knowledge.
2. A file's location should explain what kind of thing it is.

## Rules

- `dotter/` contains deployment manifests only.
- `scripts/` contains bootstrap and machine-setup entrypoints.
- Top-level tool directories contain hand-maintained source config.
- `apps/` is only for app config that is still part of the dotfiles system.
- `bin/` and `zsh/bin/` contain maintained executables, not downloaded binaries.
- Secrets and machine-private state do not live in git.
- Installers, DMGs, `.app` bundles, and archives do not live beside source config.
- Backups like `.bak`, `_old`, and `big` variants should move to an explicit archive area or be deleted.

## What This Means In Practice

Good fit for this repo:

- shell/editor/terminal config
- window manager config
- keybindings
- reusable helper scripts
- package manifests
- Dotter package definitions

Poor fit for this repo:

- credential-bearing files
- one-off downloads
- generated app resources
- large standalone binaries
- historical snapshots kept "just in case"

## Migration Direction

When cleaning up the repo, use this order:

1. Remove or relocate secrets and private state.
2. Move installers and large binaries out of the repo root.
3. Delete or archive backup variants and `_old` directories.
4. Keep one canonical bootstrap path per platform.
5. Split Dotter packages by role, not by history.

## Naming

- Use `setup-*` or `bootstrap-*` for first-run machine provisioning scripts.
- Use `*.example` for checked-in local config templates.
- Use `dotter/local.toml` for the untracked active machine profile.
- Prefer descriptive directory names over personal shorthand when the scope is broader than one tool.
