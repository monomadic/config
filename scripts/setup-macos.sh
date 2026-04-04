#!/usr/bin/env bash

set -euo pipefail

REPO_URL="${REPO_URL:-https://github.com/monomadic/config}"
DOTFILES_DIR="${DOTFILES_DIR:-$HOME/config}"
XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
DOTTER_LOCAL_CONFIG="$DOTFILES_DIR/dotter/local.toml"
DOTTER_PROFILE_SOURCE="${DOTTER_PROFILE_SOURCE:-$DOTFILES_DIR/dotter/macos.toml.example}"
HEALTHCHECK_SCRIPT="$DOTFILES_DIR/scripts/check-macos-bootstrap.sh"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Error: missing required command: $1" >&2
    exit 1
  fi
}

ensure_homebrew() {
  if command -v brew >/dev/null 2>&1; then
    return
  fi

  echo "Installing Homebrew..."
  /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
}

resolve_brew_bin() {
  local candidate

  if command -v brew >/dev/null 2>&1; then
    command -v brew
    return
  fi

  for candidate in /opt/homebrew/bin/brew /usr/local/bin/brew; do
    if [[ -x "$candidate" ]]; then
      printf '%s\n' "$candidate"
      return
    fi
  done

  echo "Error: Homebrew was installed but brew could not be found." >&2
  exit 1
}

ensure_repo() {
  if [[ -d "$DOTFILES_DIR/.git" ]]; then
    return
  fi

  echo "Cloning dotfiles into $DOTFILES_DIR..."
  git clone "$REPO_URL" "$DOTFILES_DIR"
}

ensure_dotter_local_config() {
  if [[ -f "$DOTTER_LOCAL_CONFIG" ]]; then
    return
  fi

  if [[ ! -f "$DOTTER_PROFILE_SOURCE" ]]; then
    echo "Error: missing Dotter profile template: $DOTTER_PROFILE_SOURCE" >&2
    exit 1
  fi

  echo "Creating dotter/local.toml from $(basename "$DOTTER_PROFILE_SOURCE")..."
  cp "$DOTTER_PROFILE_SOURCE" "$DOTTER_LOCAL_CONFIG"
}

main() {
  require_command git
  ensure_repo
  cd "$DOTFILES_DIR"

  ensure_homebrew
  eval "$("$(resolve_brew_bin)" shellenv)"

  echo "Installing Brewfile packages..."
  brew bundle --file "$DOTFILES_DIR/Brewfile"

  mkdir -p "$HOME/workspaces" "$HOME/src" "$HOME/.config" "$XDG_CONFIG_HOME/dotter/cache"
  touch "$HOME/.marks"

  ensure_dotter_local_config

  echo "Running bootstrap health check..."
  "$HEALTHCHECK_SCRIPT"

  echo "Deploying dotfiles with Dotter..."
  DOTTER_SKIP_HEALTHCHECK=1 "$DOTFILES_DIR/config/zsh/bin/dotter-deploy"

  cat <<EOF

Bootstrap complete.

Next steps:
  1. Review $DOTTER_LOCAL_CONFIG and trim packages for this machine.
  2. Add any private files outside git (.env, keys, app secrets).
  3. Start a new shell so linked config is picked up cleanly.
EOF
}

main "$@"
