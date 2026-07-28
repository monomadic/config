#!/usr/bin/env bash

# Build the `ytform` yt-dlp form TUI and install it to ~/.bin (on PATH via bin.sh).
# Foreground CLI — no LaunchAgent. Re-run after editing utils/ytform.

set -euo pipefail

APP_NAME="ytform"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
DOTFILES_DIR="${DOTFILES_DIR:-$(cd -- "$SCRIPT_DIR/../.." && pwd)}"
SOURCE_DIR="$DOTFILES_DIR/utils/$APP_NAME"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.bin}"

if ! command -v go >/dev/null 2>&1; then
  echo "Error: go toolchain not found on PATH" >&2
  exit 1
fi

echo "Building $APP_NAME from $SOURCE_DIR..."
mkdir -p "$INSTALL_DIR"
( cd "$SOURCE_DIR" && go build -ldflags='-s -w' -o "$INSTALL_DIR/$APP_NAME" . )

echo "Installed $INSTALL_DIR/$APP_NAME"
echo "Required runtime dep: yt-dlp. Optional: mpv (ctrl+o), chafa (thumbnail), lsof (.part lookup)."
