#!/usr/bin/env bash
# Build and install menu-tidy (utils/menu-tidy), the single-item menu bar
# tidier, and keep it running via a LaunchAgent.

set -euo pipefail

APP_NAME="menu-tidy"
LABEL="${LABEL:-com.jayu.menu-tidy}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
DOTFILES_DIR="${DOTFILES_DIR:-$(cd -- "$SCRIPT_DIR/../.." && pwd)}"
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
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>ProcessType</key>
  <string>Interactive</string>
  <key>StandardOutPath</key>
  <string>$(plist_escape "$LOG_DIR/$APP_NAME.out.log")</string>
  <key>StandardErrorPath</key>
  <string>$(plist_escape "$LOG_DIR/$APP_NAME.err.log")</string>
</dict>
</plist>
EOF
}

main() {
  echo "Building $APP_NAME..."
  cargo build --release --manifest-path "$DOTFILES_DIR/utils/$APP_NAME/Cargo.toml"

  local binary_path="$INSTALL_DIR/$APP_NAME"
  local plist_path="$LAUNCH_AGENTS_DIR/$LABEL.plist"

  echo "Installing to $binary_path..."
  mkdir -p "$INSTALL_DIR" "$LAUNCH_AGENTS_DIR" "$LOG_DIR"
  install -m 0755 "$DOTFILES_DIR/utils/$APP_NAME/target/release/$APP_NAME" "$binary_path"

  echo "Writing LaunchAgent to $plist_path..."
  write_launch_agent "$binary_path" "$plist_path"
  plutil -lint "$plist_path" >/dev/null

  local gui_domain="gui/$(id -u)"
  launchctl bootout "$gui_domain/$LABEL" >/dev/null 2>&1 || true
  launchctl bootstrap "$gui_domain" "$plist_path"
  launchctl enable "$gui_domain/$LABEL"
  launchctl kickstart -k "$gui_domain/$LABEL"

  echo "Installed and started $LABEL."
  echo "One-time setup: ⌘-drag menu bar icons to the LEFT of the ◀ marker."
}

main "$@"
