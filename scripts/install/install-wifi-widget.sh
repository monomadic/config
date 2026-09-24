#!/usr/bin/env bash
#
# Build and install wifi-widget (src/wifi-widget): the menu bar Wi-Fi meter —
# SNR rather than bars, band and link rate, an internet check that tells a dead
# uplink from a weak signal, and the nearby networks list.
#
# Like job-monitor, and unlike the other menu bar tools here, it installs as a
# real .app in ~/Applications and loads NO LaunchAgent. macOS only hands the
# Wi-Fi network name to a process with Location permission, only a bundle can
# be granted that permission, and only when the app is started through
# LaunchServices — double-click, `open`, or a Login Item. Launched by launchd
# the same binary reads the SSID as nil even when authorized. So the widget
# offers Open at Login from its own menu instead of installing an agent.
#
# The bundle is assembled by src/wifi-widget/bundle.sh, which owns the
# Info.plist and the ad-hoc signature; this script builds it there and copies
# the result into place. Nothing bundled is ever checked in.
#
# Expect a Location prompt after the first launch of each build: an ad-hoc
# signature changes identity whenever the binary changes, and macOS ties the
# permission to that identity. Allow it once per install.
#
# Run as your normal user, NOT under sudo.

set -euo pipefail

APP_NAME="${WIFI_WIDGET_APP_NAME:-WiFi Widget}"
BUNDLE_ID="${WIFI_WIDGET_BUNDLE_ID:-com.jayu.wifi-widget}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
DOTFILES_DIR="${DOTFILES_DIR:-$(cd -- "$SCRIPT_DIR/../.." && pwd)}"

CRATE_DIR="${CRATE_DIR:-$DOTFILES_DIR/src/wifi-widget}"
APPS_DIR="${WIFI_WIDGET_APPS_DIR:-$HOME/Applications}"
APP_BUNDLE="$APPS_DIR/$APP_NAME.app"
BUILT_BUNDLE="$CRATE_DIR/target/release/$APP_NAME.app"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Error: missing required command: $1" >&2
    exit 1
  fi
}

main() {
  require_command cargo
  require_command ditto

  if [[ ! -d "$CRATE_DIR" ]]; then
    echo "Error: crate not found: $CRATE_DIR" >&2
    exit 1
  fi

  echo "Building $APP_NAME (release) ..."
  "$CRATE_DIR/bundle.sh"

  if [[ ! -d "$BUILT_BUNDLE" ]]; then
    echo "Error: bundle.sh did not produce $BUILT_BUNDLE" >&2
    exit 1
  fi

  # Replacing the executable underneath a live process leaves the old copy in
  # the menu bar, reading stale code. Quit it, and put it back afterwards if it
  # was running.
  local was_running=0
  if pgrep -x wifi-widget >/dev/null 2>&1; then
    was_running=1
    echo "Quitting the running $APP_NAME ..."
    osascript -e "quit app id \"$BUNDLE_ID\"" >/dev/null 2>&1 || pkill -x wifi-widget || true
    sleep 1
  fi

  echo "Installing $APP_BUNDLE ..."
  mkdir -p "$APPS_DIR"
  rm -rf "$APP_BUNDLE"
  # ditto, not cp: it preserves the code signature's extended attributes.
  ditto "$BUILT_BUNDLE" "$APP_BUNDLE"
  /usr/bin/codesign --verify --strict "$APP_BUNDLE"

  if (( was_running )); then
    echo "Reopening $APP_NAME ..."
    open "$APP_BUNDLE"
  fi

  echo
  echo "Installed $APP_NAME."
  echo "  Bundle: $APP_BUNDLE"
  echo
  echo "Open it with:  open -a \"$APP_NAME\""
  echo "Allow the Location prompt on first launch, or the menu shows no network"
  echo "names. Turn on Open at Login from the widget's own menu."
}

main "$@"
