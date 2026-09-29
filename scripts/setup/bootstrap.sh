#!/bin/zsh -f
# Fresh macOS install:
# curl -fsSL https://raw.githubusercontent.com/monomadic/config/master/scripts/setup/bootstrap.sh -o /tmp/dotfiles-bootstrap.zsh
# /bin/zsh -f /tmp/dotfiles-bootstrap.zsh
# Set DOTFILES_DIR to choose a destination (default: ~/config).
set -e

# A copy inside a checkout uses that checkout; a downloaded copy uses the default.
ROOT="${${0:A}:h:h:h}"
if [[ ! -f "$ROOT/Deployfile" ]]; then
  ROOT="${DOTFILES_DIR:-$HOME/config}"
fi
if [[ ! -f "$ROOT/Deployfile" ]]; then
  if [[ -e "$ROOT" || -L "$ROOT" ]]; then
    print -u2 -- "Destination already exists: $ROOT. Choose an empty DOTFILES_DIR."
    exit 1
  fi
  temp="$(mktemp -d)"
  trap 'rm -rf -- "$temp"' EXIT
  curl -fL https://github.com/monomadic/config/archive/refs/heads/master.tar.gz -o "$temp/config.tar.gz"
  mkdir "$temp/repo"
  tar -xzf "$temp/config.tar.gz" --strip-components=1 -C "$temp/repo"
  [[ -f "$temp/repo/Deployfile" && -f "$temp/repo/scripts/setup/link.zsh" ]]
  mkdir -p -- "${ROOT:h}"
  mv -- "$temp/repo" "$ROOT"
fi
/bin/zsh -f "$ROOT/scripts/setup/link.zsh"
