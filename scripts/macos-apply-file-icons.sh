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
  "1Password.app|1password.icns"
  "Alacritty.app|alacritty1.png"
  "Brave Browser.app|brave.icns"
  "Codex.app|codex.icns"
  "Firefox.app|firefox1.icns"
  "ForkLift.app|forklift1.icns"
  "iTerm.app|term4.icns"
  "Kitty.app|term2.icns"
  "Preview.app|preview-2.icns"
  "Numi.app|numi.icns"
  "QuickTime Player.app|quicktime.icns"
  "Spotify.app|spotify.icns"
  "Telegram.app|telegram.icns"
  "Terminal.app|term5.icns"
  "VirtualDJ.app|virtualdj.icns"
)

applied=0
failed=0

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
  if fileicon set "$app_path" "$icon_path"; then
    applied=$((applied + 1))
  else
    echo "Warning: failed to apply icon for $app_name; continuing."
    failed=$((failed + 1))
  fi
done

if (( applied == 0 )); then
  echo "No matching macOS apps found for icon overrides."
fi

if (( failed > 0 )); then
  echo "Skipped $failed app icon override(s) due to fileicon errors."
fi
