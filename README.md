# Dotfiles

MacOS dotfiles deployed by plain zshell

## Setup

```sh
curl -fsSL https://raw.githubusercontent.com/monomadic/config/master/scripts/setup/bootstrap.sh | /bin/zsh -f
```

- downloads a repository archive to `~/config` and runs the linker
- set `DOTFILES_DIR` to point elsewhere

## Deploy

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
- `scripts/install/install-abner-with-plugins.sh [repo-home]` installs the
  composed Abner.app in `/Applications` and refreshes the `abner` CLI shim.
  It clones missing Abner, topaz-rs, and inpaint-video checkouts, fast-forwards
  existing checkouts when safe, and links the providers under `abner/plugins/`.
  It backs up an old nested plugin checkout and prints the backup path.
  The repository home defaults to `$SRC_PATH` or `~/src`. Re-run to pick up
  changes, or set `FORCE=1` to rebuild and reinstall.
- `scripts/install/install-fileserver.sh` optionally serves `~/web` to the LAN on port 8000 via a LaunchAgent (`--uninstall` removes it).
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
  `leaf`, `pimped`; Go for the rest) built via `scripts/install/*.sh`
- `vendor/bin/`: retained third-party or custom-built binaries
- `_quarantine/`: commands dropped from PATH but kept in git history — not
  deployed, not referenced, not added to

Config directories kept in-tree but not deployed by Deployfile:
`beatportdl`, `compressor`, `git`, `homebrew`, `iterm`, `ollama`, `python`,
`tag-media`.

See [docs/STRUCTURE.md](docs/STRUCTURE.md) for layout rules and
[AGENTS.md](AGENTS.md) for the coding-agent orientation guide (`CLAUDE.md` is a
symlink to it).

### Find LAN web servers

`lan-http` scans local IPv4 networks with 256 concurrent probes, printing each
responding URL, HTTP status and HTML page title. It discovers active interface
subnets and explicit RFC 1918 routes (including VPN routes). When a discovered
network is in `192.168.*.*`, it quickly checks `.1` and `.254` on ports 80 and 443
in the other /24 subnets, then scans only those with a responding router plus the
networks already known to macOS. This can find downstream LANs such as
`192.168.8.*` without probing every address in the entire /16. Duplicate/contained ranges are scanned only once.

```sh
lan-http                              # fast automatic scan; ports 80/443 + titles
lan-http --list                       # discover target subnets without the host scan
lan-http -p 80,443,8000,8080,8443       # include alternate web ports
lan-http -i en0                        # restrict to en0's configured subnets
lan-http 192.168.8.0/24 -p 80,443      # explicitly scan one subnet
lan-http -p 1-65535                    # every TCP port (much slower)
lan-http --timeout 2                   # allow slower devices to respond
lan-http --no-title                    # skip page fetches
lan-http --full                        # exhaustive 192.168/16 sweep when in use
```

Requires Python 3 and macOS's built-in curl. The default ports are 80 and 443.
Use `-p` to include alternate web ports. It tries HTTP and HTTPS on open
ports and accepts self-signed certificates. Titles follow up to three HTTP(S)
redirects, accept compressed/chunked responses, and read at most 128 KiB with a
two-second deadline. Missing titles appear as `-`; JavaScript isn't executed.

The default probe timeout is 0.6 seconds per network operation. Router discovery needs at most
1,024 TCP probes, rather than scanning 65,534 addresses. It is a heuristic: hidden
subnets whose routers do not answer on `.1` or `.254` ports 80/443 will be missed.
Use `--full` to restore the entire /16 sweep (several minutes), or supply a CIDR.
Known interface subnets and routes are always included regardless of router replies.
Use `-i en0` to scan only the attached LAN. This is discovery, not an
exhaustive inventory: slow devices, IPv6-only servers and hostname-only virtual
hosts can be missed. Hidden `10.*` and `172.16–31.*` networks without an explicit
route cannot be inferred; supply their CIDR to scan them. Broad VPN catch-all
routes outside RFC 1918 (such as `100.64/10`) aren't expanded automatically,
though interface addresses are included. No ping or login is needed. Title
redirects may resolve hostnames. Tests use loopback and synthetic routing tables:
`PYTHONDONTWRITEBYTECODE=1 python3 scripts/tests/test_lan_http.py`.
