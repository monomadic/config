#!/usr/bin/env bash

set -euo pipefail

APP_NAME="free-disk-space-widget"
LABEL="${LABEL:-com.jayu.free-disk-space-widget}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
DOTFILES_DIR="${DOTFILES_DIR:-$(cd -- "$SCRIPT_DIR/.." && pwd)}"
SOURCE_BINARY="${SOURCE_BINARY:-$DOTFILES_DIR/vendor/bin/$APP_NAME}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
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
  local binary_path="$1"
  local plist_path="$2"
  local escaped_binary_path escaped_stdout_path escaped_stderr_path

  escaped_binary_path="$(plist_escape "$binary_path")"
  escaped_stdout_path="$(plist_escape "$LOG_DIR/$APP_NAME.out.log")"
  escaped_stderr_path="$(plist_escape "$LOG_DIR/$APP_NAME.err.log")"

  cat >"$plist_path" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>$LABEL</string>
  <key>ProgramArguments</key>
  <array>
    <string>$escaped_binary_path</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>ProcessType</key>
  <string>Interactive</string>
  <key>StandardOutPath</key>
  <string>$escaped_stdout_path</string>
  <key>StandardErrorPath</key>
  <string>$escaped_stderr_path</string>
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
  launchctl kickstart -k "$gui_domain/$LABEL"
}

main() {
  require_command install
  require_command launchctl
  require_command plutil
  require_command sed

  if [[ ! -x "$SOURCE_BINARY" ]]; then
    echo "Error: missing executable source binary: $SOURCE_BINARY" >&2
    echo "Rebuild it with: go build -ldflags='-s -w' -o '$SOURCE_BINARY' '$DOTFILES_DIR/utils/$APP_NAME'" >&2
    exit 1
  fi

  local binary_path plist_path
  binary_path="$INSTALL_DIR/$APP_NAME"
  plist_path="$LAUNCH_AGENTS_DIR/$LABEL.plist"

  echo "Installing binary from $SOURCE_BINARY to $binary_path..."
  mkdir -p "$INSTALL_DIR" "$LAUNCH_AGENTS_DIR" "$LOG_DIR"
  install -m 0755 "$SOURCE_BINARY" "$binary_path"

  echo "Writing LaunchAgent to $plist_path..."
  write_launch_agent "$binary_path" "$plist_path"
  plutil -lint "$plist_path" >/dev/null

  bootstrap_launch_agent "$plist_path"

  echo "Installed and started $LABEL."
}

main "$@"
