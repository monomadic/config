#!/usr/bin/env bash
#
# One-shot macOS bootstrap. Safe to run either from a checkout:
#
#   setup/macos/bootstrap.sh
#
# or piped straight from GitHub on a fresh machine:
#
#   curl -fsSL https://raw.githubusercontent.com/monomadic/config/master/setup/macos/bootstrap.sh | bash
#
# Clone location is configurable and nothing after this assumes ~/config:
#
#   DOTFILES_DIR="$HOME/src/config" bash -c "$(curl -fsSL .../bootstrap.sh)"

set -euo pipefail

REPO_URL="${REPO_URL:-https://github.com/monomadic/config}"

# When run from inside a checkout, that checkout wins. Otherwise fall back to
# $DOTFILES_DIR (if the user set one) and finally to ~/config.
resolve_dotfiles_dir() {
  local self_dir candidate

  if [[ -n "${BASH_SOURCE[0]:-}" ]] && [[ -f "${BASH_SOURCE[0]}" ]]; then
    self_dir="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
    candidate="$(CDPATH= cd -- "$self_dir/../.." && pwd -P)"
    if [[ -f "$candidate/dotter/global.toml" ]]; then
      printf '%s\n' "$candidate"
      return
    fi
  fi

  printf '%s\n' "${DOTFILES_DIR:-$HOME/config}"
}

DOTFILES_DIR="$(resolve_dotfiles_dir)"
export DOTFILES_DIR
XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$HOME/.config}"
DOTTER_LOCAL_CONFIG="$DOTFILES_DIR/dotter/local.toml"
DOTTER_PROFILE_SOURCE="${DOTTER_PROFILE_SOURCE:-$DOTFILES_DIR/dotter/local.toml.example}"

step() {
  printf '\n==> %s\n' "$*"
}

ensure_xcode_clt() {
  if xcode-select -p >/dev/null 2>&1; then
    return
  fi

  step "Installing Xcode Command Line Tools (needed for git and compilers)"
  xcode-select --install || true

  echo "Waiting for the Command Line Tools install to finish..."
  until xcode-select -p >/dev/null 2>&1; do
    sleep 10
  done
}

ensure_homebrew() {
  if command -v brew >/dev/null 2>&1; then
    return
  fi

  step "Installing Homebrew"
  NONINTERACTIVE=1 /bin/bash -c \
    "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
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

  step "Cloning $REPO_URL into $DOTFILES_DIR"
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

  step "Creating dotter/local.toml from $(basename "$DOTTER_PROFILE_SOURCE")"
  cp "$DOTTER_PROFILE_SOURCE" "$DOTTER_LOCAL_CONFIG"
}

main() {
  if [[ "$OSTYPE" != darwin* ]]; then
    echo "Error: this bootstrap targets macOS only." >&2
    exit 1
  fi

  ensure_xcode_clt
  ensure_homebrew
  eval "$("$(resolve_brew_bin)" shellenv)"

  ensure_repo
  cd "$DOTFILES_DIR"

  # Brewfile provides dotter and fileicon, which the steps below depend on.
  step "Installing Brewfile packages"
  brew bundle --file "$DOTFILES_DIR/Brewfile"

  mkdir -p "$HOME/workspaces" "$HOME/src" "$XDG_CONFIG_HOME" "$XDG_CONFIG_HOME/dotter/cache"
  touch "$HOME/.marks"

  ensure_dotter_local_config

  step "Running bootstrap health check"
  "$DOTFILES_DIR/setup/macos/check.sh"

  step "Deploying dotfiles with Dotter"
  DOTTER_SKIP_HEALTHCHECK=1 "$DOTFILES_DIR/config/zsh/bin/dotter-deploy"

  cat <<EOF

Bootstrap complete. Repo lives at $DOTFILES_DIR.

Next steps:
  1. Trim packages for this machine:
       $DOTFILES_DIR/setup/macos/packages.sh list
  2. Add any private files outside git (.env, keys, app secrets).
  3. Start a new shell so linked config is picked up cleanly.
EOF
}

main "$@"
