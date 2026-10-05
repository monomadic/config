#!/usr/bin/env bash
# Optional: serve a folder to the LAN over plain HTTP (python3 -m http.server),
# kept running across logins by a LaunchAgent. No auth, no TLS — everything
# under SERVE_DIR is readable by anyone on the network.
#
#   install-fileserver.sh               serve ~/web on port 8000
#   SERVE_DIR=~/Public PORT=8080 install-fileserver.sh
#   BIND=127.0.0.1 install-fileserver.sh   (default: all interfaces)
#   install-fileserver.sh --uninstall
#
# A LaunchAgent starts at login, not at power-on. It also cannot read
# ~/Documents, ~/Desktop or ~/Downloads without Full Disk Access.

set -euo pipefail

APP_NAME="fileserver"
LABEL="${LABEL:-com.jayu.fileserver}"
SERVE_DIR="${SERVE_DIR:-$HOME/web}"
PORT="${PORT:-8000}"
BIND="${BIND:-}"
PYTHON="${PYTHON:-/usr/bin/python3}"
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
  local plist_path="$1"
  local bind_args=""

  if [[ -n "$BIND" ]]; then
    bind_args="    <string>--bind</string>
    <string>$(plist_escape "$BIND")</string>
"
  fi

  cat >"$plist_path" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>$LABEL</string>
  <key>ProgramArguments</key>
  <array>
    <string>$(plist_escape "$PYTHON")</string>
    <string>-m</string>
    <string>http.server</string>
    <string>$PORT</string>
    <string>--directory</string>
    <string>$(plist_escape "$SERVE_DIR")</string>
$bind_args  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>ProcessType</key>
  <string>Background</string>
  <key>StandardOutPath</key>
  <string>$(plist_escape "$LOG_DIR/$APP_NAME.log")</string>
  <key>StandardErrorPath</key>
  <string>$(plist_escape "$LOG_DIR/$APP_NAME.log")</string>
</dict>
</plist>
EOF
}

uninstall() {
  local plist_path="$LAUNCH_AGENTS_DIR/$LABEL.plist"

  launchctl bootout "gui/$(id -u)/$LABEL" >/dev/null 2>&1 || true
  rm -f "$plist_path"
  echo "Stopped and removed $LABEL."
}

main() {
  if [[ "${1:-}" == "--uninstall" ]]; then
    uninstall
    return
  fi

  if [[ ! -x "$PYTHON" ]]; then
    echo "Error: no python3 at $PYTHON (set PYTHON=...)" >&2
    exit 1
  fi
  if [[ ! "$PORT" =~ ^[0-9]+$ ]]; then
    echo "Error: PORT must be a number, got: $PORT" >&2
    exit 1
  fi
  if [[ ! -d "$SERVE_DIR" ]]; then
    echo "Error: SERVE_DIR is not a directory: $SERVE_DIR" >&2
    exit 1
  fi
  SERVE_DIR="$(cd -- "$SERVE_DIR" && pwd)"

  local plist_path="$LAUNCH_AGENTS_DIR/$LABEL.plist"
  local gui_domain="gui/$(id -u)"

  launchctl bootout "$gui_domain/$LABEL" >/dev/null 2>&1 || true

  # KeepAlive would otherwise respawn a server that can never bind.
  if lsof -nP -iTCP:"$PORT" -sTCP:LISTEN >/dev/null 2>&1; then
    echo "Error: port $PORT is already in use (set PORT=...)" >&2
    exit 1
  fi

  echo "Writing LaunchAgent to $plist_path..."
  mkdir -p "$LAUNCH_AGENTS_DIR" "$LOG_DIR"
  write_launch_agent "$plist_path"
  plutil -lint "$plist_path" >/dev/null

  launchctl bootstrap "$gui_domain" "$plist_path"
  launchctl enable "$gui_domain/$LABEL"

  local host="${BIND:-127.0.0.1}" attempt
  for attempt in 1 2 3 4 5 6 7 8 9 10; do
    if curl -fsS -o /dev/null "http://$host:$PORT/" 2>/dev/null; then
      local lan_ip
      lan_ip="$(ipconfig getifaddr en0 2>/dev/null || ipconfig getifaddr en1 2>/dev/null || true)"
      echo "Installed and started $LABEL."
      echo "Serving $SERVE_DIR at http://${BIND:-${lan_ip:-<this-machine>}}:$PORT/"
      echo "Log: $LOG_DIR/$APP_NAME.log"
      return
    fi
    sleep 0.5
  done

  echo "Error: $LABEL is loaded but not answering on port $PORT; see $LOG_DIR/$APP_NAME.log" >&2
  exit 1
}

main "$@"
