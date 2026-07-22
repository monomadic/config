# Dotfiles

macOS dotfiles. Package installation is Homebrew's job, file placement is
[Dotter](https://github.com/SuperCuber/dotter)'s job, and which packages are
active on a given machine lives in `dotter/local.toml`.

## Bootstrap a fresh machine

One line, nothing installed beforehand — it sets up the Xcode Command Line
Tools, installs Homebrew, clones this repo, installs the Brewfile, and deploys:

```bash
curl -fsSL https://raw.githubusercontent.com/monomadic/config/master/setup/macos/bootstrap.sh | bash
```

The repo lands in `~/config` by default. To put it anywhere else:

```bash
DOTFILES_DIR="$HOME/src/config" bash -c "$(curl -fsSL https://raw.githubusercontent.com/monomadic/config/master/setup/macos/bootstrap.sh)"
```

Nothing in the repo assumes a fixed location: every script resolves the
checkout from its own path, and `$DOTFILES_DIR` is exported from `.zshenv` by
resolving the symlink back to wherever you cloned it.

Already cloned? Run the same script from the checkout:

```bash
setup/macos/bootstrap.sh
```

## Packages

- [Brewfile](Brewfile) — installed by bootstrap. Everything the shell, editor,
  and this repo's config actually depend on.
- [Brewfile.optional](Brewfile.optional) — large apps nothing here depends on
  (messaging clients, Spotify, Journey, vapoursynth). Not installed by
  bootstrap; pull them in when you want them:

```bash
brew bundle --file "$DOTFILES_DIR/Brewfile.optional"
```

Careful with `brew bundle cleanup` against the main Brewfile alone — it will
now see the optional apps as unlisted. Pass both files, or skip cleanup.

## Day-to-day

```bash
setup/macos/packages.sh list            # what's deployed on this machine
setup/macos/packages.sh enable helix    # turn a package on
setup/macos/packages.sh disable marta   # turn one off
setup/macos/deploy.sh                   # apply changes
setup/macos/deploy.sh --full            # also sync Yazi plugins + macOS app icons
setup/macos/check.sh                    # preflight, run automatically by deploy
```

Dotter **symlinks**, so editing a file under `config/` changes live config
immediately. A deploy is only needed when the *mapping* changes — a new
package, a new file entry, or a changed target path.

## Dotter layout

Two files, and only two:

- [dotter/global.toml](dotter/global.toml) — every package, as one
  `[<name>.files]` section mapping repo path → target path. Alphabetical,
  grouped by purpose, one syntax throughout.
- `dotter/local.toml` — this machine's package selection plus variable
  overrides. Gitignored; created from
  [dotter/local.toml.example](dotter/local.toml.example) on bootstrap.

Adding a tool means: create `config/<tool>/`, add a `[<tool>.files]` section to
`global.toml`, then `setup/macos/packages.sh enable <tool>` and deploy.

Deploy runs `check.sh` first unless `DOTTER_SKIP_HEALTHCHECK=1` is set. On
macOS, `--full` also runs
[setup/macos/apply-file-icons.sh](setup/macos/apply-file-icons.sh); edit the
`ICON_MAPPINGS` array there to change which apps get custom icons.

## Structure

- `config/`: active config source, flat — each direct child is one tool
- `config/zsh/`: shell config, autoloads, and zsh-specific executables
- `config/neovim/`: dormant editor source kept in-tree for later revival
- `assets/`: fonts and icons
- `setup/`: bootstrap, deploy, and machine setup scripts
- `scripts/`: helper scripts and sourceable shell snippets
- `bin/` and `config/zsh/bin/`: maintained user-facing executables
- `utils/`: small personal utility source trees (Go widgets; Rust for `leaf`,
  `pimped`, `motherfucker`) built via `setup/install/*.sh`
- `vendor/bin/`: retained third-party or custom-built binaries
- `archive/`: installers, app bundles, backups, and historical variants

Config directories kept in-tree but not deployed through Dotter yet:
`beatportdl`, `compressor`, `git`, `homebrew`, `iterm`, `ollama`, `python`,
`tag-media`.

See [docs/STRUCTURE.md](docs/STRUCTURE.md) for layout rules and
[AGENTS.md](AGENTS.md) for the coding-agent orientation guide (`CLAUDE.md` is a
symlink to it).
