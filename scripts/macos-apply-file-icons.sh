#!/usr/bin/env bash

set -euo pipefail

DOTFILES_DIR="${DOTFILES_DIR:-$HOME/config}"
ICON_DIR="$DOTFILES_DIR/assets/icons"
APPLICATIONS_DIR="/Applications"

if [[ "${OSTYPE:-}" != darwin* ]]; then
  exit 0
fi

if ! command -v fileicon >/dev/null 2>&1; then
  echo "Skipping app icon overrides: fileicon is not installed."
  exit 0
fi

if [[ ! -d "$ICON_DIR" ]]; then
  echo "Skipping app icon overrides: icon directory not found at $ICON_DIR."
  exit 0
fi

ICON_MAPPINGS=(
  "Marta.app|folder1.icns"
  "1Password.app|1password.icns"
  "Alacritty.app|alacritty1.png"
  "Brave Browser.app|brave.icns"
  "Chromium.app|chromium.icns"
  "Firefox.app|firefox1.icns"
  "Google Chrome.app|chrome.icns"
  "iTerm.app|term.icns"
  "Kitty.app|terminal1.icns"
  "Numi.app|numi.icns"
  "Spotify.app|spotify.icns"
  "Telegram.app|telegram.icns"
  "Terminal.app|term2.icns"
  "Tor Browser.app|torbrowser.icns"
)

applied=0

for mapping in "${ICON_MAPPINGS[@]}"; do
  app_name="${mapping%%|*}"
  icon_name="${mapping##*|}"
  app_path="$APPLICATIONS_DIR/$app_name"
  icon_path="$ICON_DIR/$icon_name"

  if [[ ! -e "$app_path" ]]; then
    continue
  fi

  if [[ ! -f "$icon_path" ]]; then
    echo "Skipping $app_name: icon asset not found at $icon_path."
    continue
  fi

  echo "Applying icon: $app_name <- $icon_name"
  fileicon set "$app_path" "$icon_path"
  applied=$((applied + 1))
done

if (( applied == 0 )); then
  echo "No matching macOS apps found for icon overrides."
fi
