# Dotfiles

This repo is now organized around one rule:

- package installation is handled by the platform package manager
- file placement is handled by Dotter
- machine-specific selection lives in `dotter/local.toml`
- Helix is the active editor; Neovim stays in-tree as inactive source config

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
- reapply selected macOS app icons with `fileicon`

### Dotter model

- global config: [dotter/global.toml](/Users/nom/config/dotter/global.toml)
- local machine selection: `dotter/local.toml` (gitignored)
- starting templates:
  - [dotter/macos.toml.example](/Users/nom/config/dotter/macos.toml.example)
  - [dotter/dev-container.toml.example](/Users/nom/config/dotter/dev-container.toml.example)
  - [dotter/remote.toml](/Users/nom/config/dotter/remote.toml)

Manual deploy:

```bash
~/config/config/shell/zsh/bin/dotter-deploy
```

On macOS, `dotter-deploy` also runs [scripts/macos-apply-file-icons.sh](/Users/nom/config/scripts/macos-apply-file-icons.sh) after Dotter finishes. Edit the `ICON_MAPPINGS` array there to change which apps get custom icons.

Preflight check:

```bash
~/config/scripts/check-macos-bootstrap.sh
```

`dotter-deploy` runs this health check automatically unless `DOTTER_SKIP_HEALTHCHECK=1` is set.

## Structure

The intended top-level layout is:

- `config/`: active config source grouped by domain
- `config/shell/`: shell and prompt config
- `config/editors/`: editor config, with Helix active and Neovim dormant
- `config/terminals/`: terminal and multiplexer config
- `config/navigation/`: file navigation tools
- `config/windowing/`: window manager, launcher, and desktop UI config
- `config/media/`: media player and media-adjacent config
- `config/apps/`: app-specific config that still belongs in dotfiles
- `config/dev/`: development tool config
- `config/system/`: system integration like desktop entries and bootloader config
- `assets/`: fonts and icons
- `scripts/`: bootstrap and machine setup scripts
- `bin/` and `config/shell/zsh/bin/`: maintained user-facing executables
- `vendor/bin/`: retained third-party or custom-built binaries
- `archive/`: installers, app bundles, backups, and historical variants kept temporarily

Things that should eventually move out of the main dotfiles path:

- installers, DMGs, and large binaries
- app bundles and generated resources
- secrets, keys, and private machine state
- backups and abandoned variants

See [docs/STRUCTURE.md](/Users/nom/config/docs/STRUCTURE.md) for the opinionated layout rules.
