#!/usr/bin/env bash
#
# Install the job-runner watch folder: a launchd WatchPaths LaunchAgent that
# runs bin/job-runner whenever a file lands in ~/jobs. Drop an executable
# `*.job` shell script there and it runs, then moves to .processed/ or .failed/.
#
# Mirrors the widget installers: generates the plist into ~/Library/LaunchAgents
# (not tracked by Dotter) and bootstraps it into the user's gui domain.

set -euo pipefail

LABEL="${LABEL:-com.jayu.job-runner}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
DOTFILES_DIR="${DOTFILES_DIR:-$(cd -- "$SCRIPT_DIR/../.." && pwd)}"

JOBS_DIR="${JOBS_DIR:-$HOME/jobs}"
# Prefer the Dotter-deployed handler; fall back to the in-repo copy.
HANDLER="${HANDLER:-$HOME/.bin/job-runner}"
[[ -x "$HANDLER" ]] || HANDLER="$DOTFILES_DIR/bin/job-runner"

LAUNCH_AGENTS_DIR="${LAUNCH_AGENTS_DIR:-$HOME/Library/LaunchAgents}"
LOG_DIR="${LOG_DIR:-$HOME/Library/Logs}"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Error: missing required command: $1" >&2
    exit 1
  fi
}

plist_escape() {
  printf '%s' "$1" \
    | sed \
      -e 's/&/\&amp;/g' \
      -e 's/</\&lt;/g' \
      -e 's/>/\&gt;/g' \
      -e 's/"/\&quot;/g' \
      -e "s/'/\&apos;/g"
}

write_launch_agent() {
  local handler_path="$1" watch_path="$2" plist_path="$3"
  local esc_handler esc_watch esc_out esc_err

  esc_handler="$(plist_escape "$handler_path")"
  esc_watch="$(plist_escape "$watch_path")"
  esc_out="$(plist_escape "$LOG_DIR/job-runner.out.log")"
  esc_err="$(plist_escape "$LOG_DIR/job-runner.err.log")"

  cat >"$plist_path" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>$LABEL</string>
  <key>ProgramArguments</key>
  <array>
    <string>$esc_handler</string>
  </array>
  <key>WatchPaths</key>
  <array>
    <string>$esc_watch</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>ProcessType</key>
  <string>Background</string>
  <key>StandardOutPath</key>
  <string>$esc_out</string>
  <key>StandardErrorPath</key>
  <string>$esc_err</string>
</dict>
</plist>
EOF
}

bootstrap_launch_agent() {
  local plist_path="$1"
  local gui_domain="gui/$(id -u)"

  echo "Unloading any existing LaunchAgent..."
  launchctl bootout "$gui_domain" "$plist_path" >/dev/null 2>&1 || true
  launchctl bootout "$gui_domain/$LABEL" >/dev/null 2>&1 || true

  echo "Loading LaunchAgent..."
  launchctl bootstrap "$gui_domain" "$plist_path"
  launchctl enable "$gui_domain/$LABEL"
}

main() {
  require_command launchctl
  require_command plutil
  require_command sed

  if [[ ! -x "$HANDLER" ]]; then
    echo "Error: handler not found or not executable: $HANDLER" >&2
    echo "Run 'setup/macos/deploy.sh' first so bin/job-runner links to ~/.bin/, or chmod +x it." >&2
    exit 1
  fi

  local plist_path="$LAUNCH_AGENTS_DIR/$LABEL.plist"

  echo "Creating watch folder $JOBS_DIR ..."
  mkdir -p "$JOBS_DIR" "$JOBS_DIR/.processed" "$JOBS_DIR/.failed" \
    "$LAUNCH_AGENTS_DIR" "$LOG_DIR"

  echo "Writing LaunchAgent to $plist_path ..."
  write_launch_agent "$HANDLER" "$JOBS_DIR" "$plist_path"
  plutil -lint "$plist_path" >/dev/null

  bootstrap_launch_agent "$plist_path"

  echo
  echo "Installed $LABEL."
  echo "  Handler:      $HANDLER"
  echo "  Watch folder: $JOBS_DIR"
  echo "  Logs:         $LOG_DIR/job-runner.log (+ .out/.err)"
  echo
  echo "Drop a *.job shell script into $JOBS_DIR to run it."
}

main "$@"
