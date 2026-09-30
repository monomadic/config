#!/usr/bin/env bash
#
# Build and install job-folder (src/jobs/job-folder): the jobs queue and its
# menu bar in one process.
#
# This installs NO LaunchAgent — the queue runs while the app is open and stops
# when you quit it, because the queue lives in the app's memory. Add it to
# Login Items if you want it always on (scripts/setup/server.sh does).
#
# It installs as a real .app bundle in ~/Applications:
# UNUserNotificationCenter refuses to work without a bundle identifier, and a
# bundle is what makes it a normal app with a Quit item rather than a service.
#
# The bundle is generated here, never checked in (the repo holds no .app
# bundles). Re-running rebuilds and replaces it in place.
#
# job-daemon and job-monitor, the retired generation of this queue (the .job
# drop contract; source in git history), are uninstalled first wherever they
# are still installed.
#
# Run as your normal user, NOT under sudo.

set -euo pipefail

APP_NAME="${JOB_FOLDER_APP_NAME:-Job Folder}"
BUNDLE_ID="${JOB_FOLDER_BUNDLE_ID:-com.jayu.job-folder}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
DOTFILES_DIR="${DOTFILES_DIR:-$(cd -- "$SCRIPT_DIR/../.." && pwd)}"

WORKSPACE="$DOTFILES_DIR/src/jobs"
APPS_DIR="${JOB_FOLDER_APPS_DIR:-$HOME/Applications}"
APP_BUNDLE="$APPS_DIR/$APP_NAME.app"
JOBS_ROOT="${JOBS_DIR:-$HOME/jobs}"

INSTALL_DIR="${JOB_INSTALL_DIR:-$HOME/.local/bin}"
LAUNCH_AGENTS_DIR="${JOB_LAUNCH_AGENTS_DIR:-$HOME/Library/LaunchAgents}"
JOB_DAEMON_LABEL="${JOB_DAEMON_LABEL:-com.jayu.job-daemon}"
JOB_DAEMON_BINARY="$INSTALL_DIR/job-daemon"
JOB_MONITOR_BUNDLE_ID="com.jayu.job-monitor"
JOB_MONITOR_APP="$APPS_DIR/Job Monitor.app"

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
  <string>job-folder</string>
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

# The retired generation: job-daemon's LaunchAgent and binary, and the Job
# Monitor app. Nothing feeds them any more, and a daemon left loaded would
# still run any .job file that turned up in ~/jobs.
uninstall_retired() {
  local gui_domain="gui/$(id -u)"
  local old_plist="$LAUNCH_AGENTS_DIR/$JOB_DAEMON_LABEL.plist"

  if [[ -f "$old_plist" ]] || launchctl print "$gui_domain/$JOB_DAEMON_LABEL" >/dev/null 2>&1; then
    echo "Uninstalling job-daemon ($JOB_DAEMON_LABEL) ..."
    launchctl bootout "$gui_domain" "$old_plist" >/dev/null 2>&1 || true
    launchctl bootout "$gui_domain/$JOB_DAEMON_LABEL" >/dev/null 2>&1 || true
    rm -f "$old_plist"
  fi

  if [[ -e "$JOB_DAEMON_BINARY" ]]; then
    echo "Removing job-daemon binary $JOB_DAEMON_BINARY ..."
    rm -f "$JOB_DAEMON_BINARY"
  fi

  if [[ -d "$JOB_MONITOR_APP" ]]; then
    echo "Removing $JOB_MONITOR_APP ..."
    if pgrep -x job-monitor >/dev/null 2>&1; then
      osascript -e "quit app id \"$JOB_MONITOR_BUNDLE_ID\"" >/dev/null 2>&1 || pkill -x job-monitor || true
      sleep 1
    fi
    rm -rf "$JOB_MONITOR_APP"
  fi
}

main() {
  require_command cargo
  require_command launchctl
  require_command plutil
  require_command codesign

  if [[ ! -d "$WORKSPACE" ]]; then
    echo "Error: workspace not found: $WORKSPACE" >&2
    exit 1
  fi

  uninstall_retired

  echo "Building job-folder (release) ..."
  (cd "$WORKSPACE" && cargo build --release --bin job-folder)

  # Quit a running copy first — and this one is running jobs, so it stops them
  # on the way out rather than being replaced underneath them.
  if pgrep -x job-folder >/dev/null 2>&1; then
    echo "Quitting the running job-folder ..."
    osascript -e "quit app id \"$BUNDLE_ID\"" >/dev/null 2>&1 || pkill -x job-folder || true
    sleep 1
  fi

  echo "Assembling $APP_BUNDLE ..."
  rm -rf "$APP_BUNDLE"
  mkdir -p "$APP_BUNDLE/Contents/MacOS" "$APP_BUNDLE/Contents/Resources"
  install -m 755 "$WORKSPACE/target/release/job-folder" \
    "$APP_BUNDLE/Contents/MacOS/job-folder"
  write_info_plist "$APP_BUNDLE/Contents/Info.plist"
  plutil -lint "$APP_BUNDLE/Contents/Info.plist" >/dev/null

  # Ad-hoc signature. Notification Centre keys authorization to the signed
  # bundle identity, so an unsigned bundle gets asked about again every launch.
  echo "Signing (ad-hoc) ..."
  codesign --force --sign - --identifier "$BUNDLE_ID" "$APP_BUNDLE"

  mkdir -p "$JOBS_ROOT"

  echo
  echo "Installed $APP_NAME."
  echo "  Bundle: $APP_BUNDLE"
  echo "  Queue:  $JOBS_ROOT/<workflow>/  (job.sh + input/ -> output/)"
  echo
  echo "Open it with:  open -a \"$APP_NAME\""
  echo "The queue runs while it is open. Quitting stops the jobs."
}

main "$@"
