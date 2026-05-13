#!/usr/bin/env bash

set -euo pipefail

DOTFILES_DIR="${DOTFILES_DIR:-$HOME/config}"
ICON_DIR="$DOTFILES_DIR/assets/icons"

SEARCH_DIRS=(
  "/Applications"
  "$HOME/Applications"
)

if [[ -t 1 ]]; then
  RED="$(tput setaf 1 2>/dev/null || true)"
  RESET="$(tput sgr0 2>/dev/null || true)"
else
  RED=""
  RESET=""
fi

warn() {
  printf '%s%s%s\n' "$RED" "$*" "$RESET"
}

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
  "Brave Browser.app|brave.icns"
  "Codex.app|codex.icns"
  "Firefox.app|firefox1.icns"
  "ForkLift.app|forklift1.icns"
  "Kitty.app|term2.icns"
  "Numi.app|numi.icns"
  "Spotify.app|spotify.icns"
  "Telegram Desktop.app|telegram.icns"
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
attempted=0
failed=0
missing_apps=0
missing_icons=0

for mapping in "${ICON_MAPPINGS[@]}"; do
  app_name="${mapping%%|*}"
  icon_name="${mapping##*|}"
  icon_path="$ICON_DIR/$icon_name"

  if [[ ! -f "$icon_path" ]]; then
    warn "Skipping $app_name: icon asset not found at $icon_path."
    missing_icons=$((missing_icons + 1))
    continue
  fi

  if ! app_path="$(find_app_path "$app_name")"; then
    warn "Skipping $app_name: app not found in expected locations."
    missing_apps=$((missing_apps + 1))
    continue
  fi

  echo "$app_name: apply $icon_name -> $app_path"
  attempted=$((attempted + 1))
  if fileicon set "$app_path" "$icon_path"; then
    applied=$((applied + 1))
  else
    warn "Warning: failed to apply icon for $app_name at $app_path; continuing."
    failed=$((failed + 1))
  fi
done

if (( attempted == 0 )); then
  warn "No matching macOS apps found for icon overrides."
elif (( applied == 0 )); then
  warn "No macOS app icons were applied."
fi

echo "Applied: $applied"
echo "Missing apps: $missing_apps"
echo "Missing icons: $missing_icons"

if (( failed > 0 )); then
  warn "Failed: $failed"
fi
