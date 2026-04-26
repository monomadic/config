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
~/config/config/zsh/bin/dotter-deploy
```

On macOS, `dotter-deploy` also runs [scripts/macos-apply-file-icons.sh](/Users/nom/config/scripts/macos-apply-file-icons.sh) after Dotter finishes. Edit the `ICON_MAPPINGS` array there to change which apps get custom icons.

Preflight check:

```bash
~/config/scripts/check-macos-bootstrap.sh
```

`dotter-deploy` runs this health check automatically unless `DOTTER_SKIP_HEALTHCHECK=1` is set.

## Structure

The intended top-level layout is:

- `config/`: active config source, flattened so each direct child is one tool or app (`config/zsh`, `config/helix`, `config/kitty`, `config/yazi`, ...)
- `config/zsh/`: shell config, autoloads, and zsh-specific executables
- `config/neovim/`: dormant editor source kept in-tree for later revival
- `assets/`: fonts and icons
- `scripts/`: bootstrap and machine setup scripts
- `bin/` and `config/zsh/bin/`: maintained user-facing executables
- `utils/`: small personal utility source trees maintained in this repo
- `vendor/bin/`: retained third-party or custom-built binaries
- `archive/`: installers, app bundles, backups, and historical variants kept temporarily

The Dotter example profiles keep optional packages commented out so one file shows what is active, what is available, and what still needs wiring.

Current source-only config directories that are kept in-tree but not deployed through Dotter yet:

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

Things that should eventually move out of the main dotfiles path:

- installers, DMGs, and large binaries
- app bundles and generated resources
- secrets, keys, and private machine state
- backups and abandoned variants

See [docs/STRUCTURE.md](/Users/nom/config/docs/STRUCTURE.md) for the opinionated layout rules.
