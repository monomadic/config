#!/usr/bin/env bash
#
# Build and install job-monitor (utils/job-monitor): a read-only menu bar view
# of one or more jobs folders, normally mounted from another machine.
#
# Unlike the other menu bar tools in this repo, this one installs as a real
# .app bundle in ~/Applications and loads NO LaunchAgent. Two reasons:
#
#   1. Notification Centre. UNUserNotificationCenter refuses to work without a
#      bundle identifier, and a bare binary in ~/.local/bin has none — the app
#      detects that and turns notifications off rather than crashing, which is
#      useful for `cargo run` but not what you want installed.
#   2. It is a thing you open when you care and quit when you don't, not a
#      service. That is the whole reason it is a separate app from job-server.
#
# The bundle is generated here, never checked in (the repo holds no .app
# bundles). Re-running rebuilds and replaces it in place.
#
# Watched folders, in order of precedence:
#   $JOB_MONITOR_ROOTS         colon-separated, for a one-off run
#   ~/.config/job-monitor/roots  one path per line, '#' comments
#   /Volumes/Jobs              the default
#
# This installer seeds that roots file with $MONITOR_ROOT (default
# /Volumes/Jobs) if it does not exist yet, and never overwrites it.
#
# Run as your normal user, NOT under sudo.

set -euo pipefail

APP_NAME="${APP_NAME:-Job Monitor}"
BUNDLE_ID="${BUNDLE_ID:-com.jayu.job-monitor}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
DOTFILES_DIR="${DOTFILES_DIR:-$(cd -- "$SCRIPT_DIR/../.." && pwd)}"

SOURCE_DIR="$DOTFILES_DIR/utils/job-monitor"
APPS_DIR="${APPS_DIR:-$HOME/Applications}"
APP_BUNDLE="$APPS_DIR/$APP_NAME.app"
CONFIG_DIR="${CONFIG_DIR:-$HOME/.config/job-monitor}"
MONITOR_ROOT="${MONITOR_ROOT:-/Volumes/Jobs}"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Error: missing required command: $1" >&2
    exit 1
  fi
}

write_info_plist() {
  local plist_path="$1"

  # LSUIElement keeps it out of the Dock and the app switcher: it is a menu bar
  # app, and Quit lives in its own menu.
  cat >"$plist_path" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>$APP_NAME</string>
  <key>CFBundleDisplayName</key>
  <string>$APP_NAME</string>
  <key>CFBundleIdentifier</key>
  <string>$BUNDLE_ID</string>
  <key>CFBundleExecutable</key>
  <string>job-monitor</string>
  <key>CFBundlePackageType</key>
  <string>APPL</string>
  <key>CFBundleVersion</key>
  <string>1</string>
  <key>CFBundleShortVersionString</key>
  <string>1.0</string>
  <key>LSMinimumSystemVersion</key>
  <string>11.0</string>
  <key>LSUIElement</key>
  <true/>
  <key>NSHighResolutionCapable</key>
  <true/>
</dict>
</plist>
EOF
}

main() {
  require_command cargo
  require_command plutil
  require_command codesign

  if [[ ! -d "$SOURCE_DIR" ]]; then
    echo "Error: source not found: $SOURCE_DIR" >&2
    exit 1
  fi

  echo "Building job-monitor (release) ..."
  (cd "$SOURCE_DIR" && cargo build --release)

  # Quit a running copy first: replacing the executable underneath a live
  # process leaves the old one in the menu bar, watching stale code.
  if pgrep -x job-monitor >/dev/null 2>&1; then
    echo "Quitting the running job-monitor ..."
    osascript -e "quit app id \"$BUNDLE_ID\"" >/dev/null 2>&1 || pkill -x job-monitor || true
    sleep 1
  fi

  echo "Assembling $APP_BUNDLE ..."
  rm -rf "$APP_BUNDLE"
  mkdir -p "$APP_BUNDLE/Contents/MacOS" "$APP_BUNDLE/Contents/Resources"
  install -m 755 "$SOURCE_DIR/target/release/job-monitor" \
    "$APP_BUNDLE/Contents/MacOS/job-monitor"
  write_info_plist "$APP_BUNDLE/Contents/Info.plist"
  plutil -lint "$APP_BUNDLE/Contents/Info.plist" >/dev/null

  # Ad-hoc signature. Notification Centre keys authorization to the signed
  # bundle identity, so an unsigned bundle gets asked about again every launch.
  echo "Signing (ad-hoc) ..."
  codesign --force --sign - --identifier "$BUNDLE_ID" "$APP_BUNDLE"

  if [[ ! -f "$CONFIG_DIR/roots" ]]; then
    echo "Seeding $CONFIG_DIR/roots with $MONITOR_ROOT ..."
    mkdir -p "$CONFIG_DIR"
    cat >"$CONFIG_DIR/roots" <<EOF
# Jobs folders for job-monitor to watch — one path per line.
# Mount the share however you like; nothing here knows about hostnames.
$MONITOR_ROOT
EOF
  fi

  echo
  echo "Installed $APP_NAME."
  echo "  Bundle:  $APP_BUNDLE"
  echo "  Watching: $(grep -v '^#' "$CONFIG_DIR/roots" | grep -v '^$' | tr '\n' ' ')"
  echo
  echo "Open it with:  open -a \"$APP_NAME\""
  echo "It runs no jobs and writes nothing to the share except a pause marker."
}

main "$@"
