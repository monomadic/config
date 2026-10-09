#!/usr/bin/env bash
# Run bin/agent-snapshot every 5 minutes from a LaunchAgent. Deploy first:
# the agent runs the deployed ~/.local/bin/agent-snapshot symlink.

set -euo pipefail

APP_NAME="agent-snapshot"
LABEL="${LABEL:-com.jayu.agent-snapshot}"
INTERVAL="${INTERVAL:-300}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
LAUNCH_AGENTS_DIR="${LAUNCH_AGENTS_DIR:-$HOME/Library/LaunchAgents}"
LOG_DIR="${LOG_DIR:-$HOME/Library/Logs}"

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
  local binary_path="$1"
  local plist_path="$2"

  cat >"$plist_path" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>$LABEL</string>
  <key>ProgramArguments</key>
  <array>
    <string>$(plist_escape "$binary_path")</string>
  </array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>SRC_PATH</key>
    <string>$(plist_escape "${SRC_PATH:-$HOME/src}")</string>
  </dict>
  <key>StartInterval</key>
  <integer>$INTERVAL</integer>
  <key>RunAtLoad</key>
  <true/>
  <key>ProcessType</key>
  <string>Background</string>
  <key>LowPriorityIO</key>
  <true/>
  <key>Nice</key>
  <integer>10</integer>
  <key>StandardOutPath</key>
  <string>$(plist_escape "$LOG_DIR/$APP_NAME.out.log")</string>
  <key>StandardErrorPath</key>
  <string>$(plist_escape "$LOG_DIR/$APP_NAME.err.log")</string>
</dict>
</plist>
EOF
}

main() {
  local binary_path="$INSTALL_DIR/$APP_NAME"
  local plist_path="$LAUNCH_AGENTS_DIR/$LABEL.plist"
  local gui_domain="gui/$(id -u)"

  if [[ ! -x "$binary_path" ]]; then
    echo "Error: $binary_path is missing; run deploy first." >&2
    exit 1
  fi

  mkdir -p "$LAUNCH_AGENTS_DIR" "$LOG_DIR"
  echo "Writing LaunchAgent to $plist_path..."
  write_launch_agent "$binary_path" "$plist_path"
  plutil -lint "$plist_path" >/dev/null

  launchctl bootout "$gui_domain/$LABEL" >/dev/null 2>&1 || true
  launchctl bootstrap "$gui_domain" "$plist_path"
  launchctl enable "$gui_domain/$LABEL"

  echo "Installed $LABEL: snapshots every ${INTERVAL}s, log in $LOG_DIR/$APP_NAME.out.log."
}

main "$@"
