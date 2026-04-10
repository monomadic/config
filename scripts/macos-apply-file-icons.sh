#!/usr/bin/env bash

set -euo pipefail

DOTFILES_DIR="${DOTFILES_DIR:-$HOME/config}"
ICON_DIR="$DOTFILES_DIR/assets/icons"

SEARCH_DIRS=(
  "/Applications"
  "/System/Applications"
  "$HOME/Applications"
)

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
  "iPhone Mirroring.app|iphone1.icns"
  "VirtualDJ.app|virtualdj.icns"
)

find_app_path() {
  local app_name="$1"
  local dir
  for dir in "${SEARCH_DIRS[@]}"; do
    if [[ -e "$dir/$app_name" ]]; then
      printf '%s\n' "$dir/$app_name"
      return 0
    fi
  done
  return 1
}

applied=0
failed=0
missing_apps=0
missing_icons=0

for mapping in "${ICON_MAPPINGS[@]}"; do
  app_name="${mapping%%|*}"
  icon_name="${mapping##*|}"
  icon_path="$ICON_DIR/$icon_name"

  if [[ ! -f "$icon_path" ]]; then
    echo "Skipping $app_name: icon asset not found at $icon_path."
    missing_icons=$((missing_icons + 1))
    continue
  fi

  if ! app_path="$(find_app_path "$app_name")"; then
    echo "Skipping $app_name: app not found in expected locations."
    missing_apps=$((missing_apps + 1))
    continue
  fi

  echo "$app_name: apply $icon_name -> $app_path"
  if fileicon set "$app_path" "$icon_path"; then
    applied=$((applied + 1))
  else
    echo "Warning: failed to apply icon for $app_name at $app_path; continuing."
    failed=$((failed + 1))
  fi
done

if (( applied == 0 )); then
  echo "No matching macOS apps found for icon overrides."
fi

echo "Applied: $applied"
echo "Missing apps: $missing_apps"
echo "Missing icons: $missing_icons"

if (( failed > 0 )); then
  echo "Failed: $failed"
fi
