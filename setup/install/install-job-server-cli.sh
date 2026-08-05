#!/usr/bin/env bash
#
# Install job-server-cli (utils/job-server-cli): copies the script to ~/.local/bin and
# sets up its watch folder — a launchd WatchPaths LaunchAgent that runs the
# handler whenever a file lands in ~/jobs. Drop a `*.job` shell script there
# and it runs, one at a time, in a run folder under _running/ that then moves
# to _done/ or _err/.
#
# Mirrors the widget installers: generates the plist into ~/Library/LaunchAgents
# (not tracked by Dotter) and bootstraps it into the user's gui domain.
#
# Installing on another machine
# -----------------------------
# setup/macos/server.sh calls this installer near the end, right after the
# SMB share that `send-job` ships jobs to. It installs the handler, creates
# ~/jobs + _running/_done/_err, and loads the com.jayu.job-server-cli WatchPaths
# LaunchAgent. To install manually on any machine:
#   setup/install/install-job-server-cli.sh
#
# Robustness notes:
#   - Self-contained: resolves its own repo root and installs the handler from
#     utils/job-server-cli/. No Dotter deploy needed first.
#   - Idempotent: boots out any existing agent before bootstrapping, so
#     re-running is safe (and is how you pick up handler changes).
#   - Run as your normal user, NOT under sudo -- this installs a per-user
#     LaunchAgent into gui/$(id -u); under sudo it would land in root's domain.

set -euo pipefail

LABEL="${LABEL:-com.jayu.job-server-cli}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
DOTFILES_DIR="${DOTFILES_DIR:-$(cd -- "$SCRIPT_DIR/../.." && pwd)}"

JOBS_DIR="${JOBS_DIR:-$HOME/jobs}"
SOURCE="$DOTFILES_DIR/utils/job-server-cli/job-server-cli"
HANDLER="${HANDLER:-$HOME/.local/bin/job-server-cli}"

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
  esc_out="$(plist_escape "$LOG_DIR/job-server-cli.out.log")"
  esc_err="$(plist_escape "$LOG_DIR/job-server-cli.err.log")"

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

  # Pre-rename installation: retire com.jayu.job-runner and its handler, or a
  # machine set up before the rename runs the queue under both labels.
  launchctl bootout "$gui_domain/com.jayu.job-runner" >/dev/null 2>&1 || true
  rm -f "$LAUNCH_AGENTS_DIR/com.jayu.job-runner.plist" "$HOME/.local/bin/job-runner"

  echo "Loading LaunchAgent..."
  launchctl bootstrap "$gui_domain" "$plist_path"
  launchctl enable "$gui_domain/$LABEL"
}

main() {
  require_command launchctl
  require_command plutil
  require_command sed

  if [[ ! -f "$SOURCE" ]]; then
    echo "Error: handler source not found: $SOURCE" >&2
    exit 1
  fi

  echo "Installing handler to $HANDLER ..."
  mkdir -p "$(dirname "$HANDLER")"
  install -m 755 "$SOURCE" "$HANDLER"

  local plist_path="$LAUNCH_AGENTS_DIR/$LABEL.plist"

  echo "Creating watch folder $JOBS_DIR ..."
  mkdir -p "$JOBS_DIR" "$JOBS_DIR/_running" "$JOBS_DIR/_done" "$JOBS_DIR/_err" \
    "$LAUNCH_AGENTS_DIR" "$LOG_DIR"

  echo "Writing LaunchAgent to $plist_path ..."
  write_launch_agent "$HANDLER" "$JOBS_DIR" "$plist_path"
  plutil -lint "$plist_path" >/dev/null

  bootstrap_launch_agent "$plist_path"

  echo
  echo "Installed $LABEL."
  echo "  Handler:      $HANDLER"
  echo "  Watch folder: $JOBS_DIR"
  echo "  Logs:         $LOG_DIR/job-server-cli.log (+ .out/.err)"
  echo
  echo "Drop a *.job shell script into $JOBS_DIR to run it."
  echo "  running -> _running/<date>-NAME/   ok -> _done/<date>-NAME/   fail -> _err/<date>-NAME/"
}

main "$@"
