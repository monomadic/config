#!/usr/bin/env bash
#
# Build and install job-folder (utils/job-folder): a menu bar face for the
# ~/jobs drop folder. Rust port of job-runner — same on-disk contract
# (NAME.job -> .job.running -> _done/_err), but runs the job loop in-process
# behind a status bar icon instead of a launchd WatchPaths trigger.
#
# Installs the binary to ~/.local/bin/job-folder and loads it as a resident
# LaunchAgent (RunAtLoad + KeepAlive), mirroring the widget installers.
#
# Since job-folder supersedes job-runner (they'd otherwise both try to run
# jobs, though the shared .lock directory prevents double-running), this
# uninstalls job-runner's LaunchAgent and binary first if present.

set -euo pipefail

LABEL="${LABEL:-com.jayu.job-folder}"
OLD_LABEL="${OLD_LABEL:-com.jayu.job-runner}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
DOTFILES_DIR="${DOTFILES_DIR:-$(cd -- "$SCRIPT_DIR/../.." && pwd)}"

SOURCE_DIR="$DOTFILES_DIR/utils/job-folder"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
BINARY_PATH="$INSTALL_DIR/job-folder"
OLD_BINARY_PATH="$INSTALL_DIR/job-runner"

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
  local binary_path="$1" plist_path="$2"
  local esc_binary esc_out esc_err

  esc_binary="$(plist_escape "$binary_path")"
  esc_out="$(plist_escape "$LOG_DIR/job-folder.out.log")"
  esc_err="$(plist_escape "$LOG_DIR/job-folder.err.log")"

  cat >"$plist_path" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>Label</key>
  <string>$LABEL</string>
  <key>ProgramArguments</key>
  <array>
    <string>$esc_binary</string>
  </array>
  <key>RunAtLoad</key>
  <true/>
  <key>KeepAlive</key>
  <true/>
  <key>ProcessType</key>
  <string>Interactive</string>
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
  launchctl kickstart -k "$gui_domain/$LABEL"
}

# job-folder is a rust port of job-runner and takes over its ~/jobs contract;
# running both is redundant (though harmless, since they share a lock), so
# retire the older shell-based installation.
uninstall_job_runner() {
  local gui_domain="gui/$(id -u)"
  local old_plist="$LAUNCH_AGENTS_DIR/$OLD_LABEL.plist"

  if [[ -f "$old_plist" ]] || launchctl print "$gui_domain/$OLD_LABEL" >/dev/null 2>&1; then
    echo "Uninstalling job-runner ($OLD_LABEL)..."
    launchctl bootout "$gui_domain" "$old_plist" >/dev/null 2>&1 || true
    launchctl bootout "$gui_domain/$OLD_LABEL" >/dev/null 2>&1 || true
    rm -f "$old_plist"
  fi

  if [[ -e "$OLD_BINARY_PATH" ]]; then
    echo "Removing old binary $OLD_BINARY_PATH ..."
    rm -f "$OLD_BINARY_PATH"
  fi
}

main() {
  require_command cargo
  require_command install
  require_command launchctl
  require_command plutil
  require_command sed

  if [[ ! -d "$SOURCE_DIR" ]]; then
    echo "Error: source directory not found: $SOURCE_DIR" >&2
    exit 1
  fi

  uninstall_job_runner

  echo "Building job-folder (release) ..."
  (cd "$SOURCE_DIR" && cargo build --release)

  local plist_path="$LAUNCH_AGENTS_DIR/$LABEL.plist"

  echo "Installing binary to $BINARY_PATH ..."
  mkdir -p "$INSTALL_DIR" "$LAUNCH_AGENTS_DIR" "$LOG_DIR"
  install -m 755 "$SOURCE_DIR/target/release/job-folder" "$BINARY_PATH"

  echo "Writing LaunchAgent to $plist_path ..."
  write_launch_agent "$BINARY_PATH" "$plist_path"
  plutil -lint "$plist_path" >/dev/null

  bootstrap_launch_agent "$plist_path"

  echo
  echo "Installed and started $LABEL."
  echo "  Binary: $BINARY_PATH"
  echo "  Logs:   $LOG_DIR/job-folder.log (+ .out/.err)"
}

main "$@"
