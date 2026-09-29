# Dotfiles

macOS dotfiles deployed by plain zsh. No Git, Homebrew, Python, or other
installed tools are needed to create the links.

## First install

Download and run with macOS's built-in curl:

```sh
curl -fsSL https://raw.githubusercontent.com/monomadic/config/master/scripts/setup/bootstrap.sh -o /tmp/dotfiles-bootstrap.zsh
/bin/zsh -f /tmp/dotfiles-bootstrap.zsh
```

Bootstrap downloads a repository archive to `~/config` and runs the linker.
Set `DOTFILES_DIR` when running it to choose another destination. An existing
checkout with a Deployfile is reused; another existing destination is left alone.
The downloaded archive has no Git history. It is not automatically updated.
The remote command uses the version published to master, so local changes must
be pushed before they are available to another machine.

## Deploy and extend

From an existing checkout:

```sh
scripts/setup/deploy.sh --dry-run   # preview links and conflicts
scripts/setup/deploy.sh             # create links
scripts/setup/check.sh              # check mapping syntax and source paths
```

After deployment, `deploy` runs the same linker. In an already-open shell that
still has the old alias, run `unalias deploy` first.

[Deployfile](Deployfile) is ordinary zsh:

```sh
link_file config/zsh/zshrc.zsh "$HOME/.zshrc"
link_file config/kitty/kitty.conf "$HOME/.config/kitty/kitty.conf"
link_tree bin "$HOME/.local/bin"
```

Add a call to extend deployment. Comment out a call to skip it. `link_file` can
also link a whole directory; `link_tree` links each file recursively, including
hidden files, so destination directories can also contain installed binaries
and app state. Deployfile is trusted shell code, not a custom manifest language.

Correct links are left alone. Missing sources, conflicting files or links, and
failed writes are reported and skipped; other mappings continue. The final
exit status is nonzero if any failed. A shell syntax error must be fixed before
the Deployfile can run. `--check` checks sources, while `--dry-run` also checks
existing targets. Neither writes links.

There are no profiles, host overrides, state files, forced replacements, or
automatic cleanup. Removing a mapping leaves its old link in place. If you move
the checkout, remove the old links yourself before deploying again. Existing
Dotter-created links to the same files already work; old Dotter caches and local
profiles are unused. Deployment does not remove them or uninstall Dotter.

Edits to linked files take effect immediately. Deploy again after adding files
or changing mappings. Deploy does not install or upgrade applications.

## Install software separately

- `brew bundle --file Brewfile` installs shell/editor dependencies once Homebrew is installed.
- `brew bundle --file Brewfile.optional` installs optional apps.
- `scripts/install/install-<name>.sh` builds and installs individual tools.
- `update` browses tool updates; `update --all` installs pending tool updates.
- `scripts/setup/apply-file-icons.sh` applies the optional icon overrides.

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

Config directories kept in-tree but not deployed by Deployfile:
`beatportdl`, `compressor`, `git`, `homebrew`, `iterm`, `ollama`, `python`,
`tag-media`.

See [docs/STRUCTURE.md](docs/STRUCTURE.md) for layout rules and
[AGENTS.md](AGENTS.md) for the coding-agent orientation guide (`CLAUDE.md` is a
symlink to it).
