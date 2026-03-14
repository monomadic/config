# Dotfiles

This repo is now organized around one rule:

- package installation is handled by the platform package manager
- file placement is handled by Dotter
- machine-specific selection lives in `dotter/local.toml`

## Bootstrap

### macOS

```bash
git clone https://github.com/monomadic/config "$HOME/config"
bash "$HOME/config/scripts/setup-macos.sh"
```

That script will:

- install Homebrew if needed
- install packages from [Brewfile](/Users/nom/config/Brewfile)
- create `dotter/local.toml` from `dotter/macos.toml.example` if needed
- run Dotter using `dotter/global.toml` and `dotter/local.toml`

### Dotter model

- global config: [dotter/global.toml](/Users/nom/config/dotter/global.toml)
- local machine selection: `dotter/local.toml` (gitignored)
- starting templates:
  - [dotter/macos.toml.example](/Users/nom/config/dotter/macos.toml.example)
  - [dotter/dev-container.toml.example](/Users/nom/config/dotter/dev-container.toml.example)
  - [dotter/remote.toml](/Users/nom/config/dotter/remote.toml)

Manual deploy:

```bash
~/config/zsh/bin/dotter-deploy
```

## Structure

The intended top-level layout is:

- `dotter/`: deployment manifests and machine profiles
- `zsh/`, `neovim/`, `kitty/`, etc.: first-party config grouped by tool
- `apps/`: app-specific config that still belongs in dotfiles
- `scripts/`: bootstrap and machine setup scripts
- `bin/` and `zsh/bin/`: user-facing executables

Things that should eventually move out of the main dotfiles path:

- installers, DMGs, and large binaries
- app bundles and generated resources
- secrets, keys, and private machine state
- backups and abandoned variants

See [docs/STRUCTURE.md](/Users/nom/config/docs/STRUCTURE.md) for the opinionated layout rules.
