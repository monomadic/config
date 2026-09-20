# Dotfiles

macOS dotfiles. Package installation is Homebrew's job, file placement is
[Dotter](https://github.com/SuperCuber/dotter)'s job, and which packages are
active on a given machine lives in `dotter/local.toml`.

## Bootstrap a fresh machine

One line, nothing installed beforehand — it sets up the Xcode Command Line
Tools, installs Homebrew, clones this repo, installs the Brewfile, and deploys:

```bash
curl -fsSL https://raw.githubusercontent.com/monomadic/config/master/scripts/setup/bootstrap.sh | bash
```

The repo lands in `~/config` by default. To put it anywhere else:

```bash
DOTFILES_DIR="$HOME/src/config" bash -c "$(curl -fsSL https://raw.githubusercontent.com/monomadic/config/master/scripts/setup/bootstrap.sh)"
```

Nothing in the repo assumes a fixed location: every script resolves the
checkout from its own path, and `$DOTFILES_DIR` is exported from `.zshenv` by
resolving the symlink back to wherever you cloned it.

Already cloned? Run the same script from the checkout:

```bash
scripts/setup/bootstrap.sh
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
scripts/setup/packages.sh list            # what's deployed on this machine
scripts/setup/packages.sh enable helix    # turn a package on
scripts/setup/packages.sh disable marta   # turn one off
scripts/setup/deploy.sh                   # apply changes
scripts/setup/deploy.sh --upgrade         # Yazi plugins, yt-dlp, brew upgrade, stale src/ rebuilds
scripts/setup/deploy.sh --icons           # reapply macOS app icons
scripts/setup/deploy.sh --full            # --icons plus --upgrade
scripts/setup/check.sh                    # preflight, run automatically by deploy
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
`global.toml`, then `scripts/setup/packages.sh enable <tool>` and deploy.

Deploy runs `check.sh` first unless `DOTTER_SKIP_HEALTHCHECK=1` is set. A bare
deploy only symlinks config; `--upgrade` is the flag that reaches outside the
repo, and every step it runs is non-fatal and individually skippable
(`DOTTER_SKIP_YAZI_PACKAGES=1`, `DOTTER_SKIP_YTDLP=1`, `DOTTER_SKIP_BREW=1`,
`DOTTER_SKIP_SRC=1`). Its `src/` step rebuilds only tools already installed on
this machine whose build inputs have changed — it never installs something new.

On macOS, `--icons` runs
[scripts/setup/apply-file-icons.sh](scripts/setup/apply-file-icons.sh); edit the
`ICON_MAPPINGS` array there to change which apps get custom icons.

## Structure

- `config/`: active config source, flat — each direct child is one tool
- `config/zsh/`: shell config and autoloads (rc files, `autoload/`, `completions/`)
- `config/neovim/`: dormant editor source kept in-tree for later revival
- `assets/`: fonts and icons
- `bin/`: every user-facing command, one flat directory, deployed as per-file
  symlinks into `~/.local/bin/`
- `bin/lib/`: sourceable snippets and preset data — the one non-flat part of `bin/`
- `scripts/`: subdirectories only, nothing loose at the top
- `scripts/setup/`: bootstrap, deploy, and health-check entrypoints
- `scripts/install/`: `install-<name>.sh` build+install scripts for `src/`
- `scripts/tweaks/`: one-shot macOS `defaults write` tweaks
- `src/`: small personal utility source trees (Rust for the menu bar widgets,
  `leaf`, `pimped`, `motherfucker`; Go for the rest) built via `scripts/install/*.sh`
- `vendor/bin/`: retained third-party or custom-built binaries
- `_quarantine/`: commands dropped from PATH but kept in git history — not
  deployed, not referenced, not added to

Config directories kept in-tree but not deployed through Dotter yet:
`beatportdl`, `compressor`, `git`, `homebrew`, `iterm`, `ollama`, `python`,
`tag-media`.

See [docs/STRUCTURE.md](docs/STRUCTURE.md) for layout rules and
[AGENTS.md](AGENTS.md) for the coding-agent orientation guide (`CLAUDE.md` is a
symlink to it).
