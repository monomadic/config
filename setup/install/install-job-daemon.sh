#!/usr/bin/env bash
#
# Install job-daemon (utils/jobs/job-daemon): the jobs queue with no UI.
#
# Installed as a launchd WatchPaths LaunchAgent that runs `job-daemon --once`
# whenever something lands in the jobs folder — launchd does the watching, and
# the process only exists while there is work to do. Nothing polls.
#
# This is the only thing that runs jobs. The menu bar app (job-monitor) is a
# client: it watches folders and commands the queue by moving them, but has no
# job loop linked into it. Install both if you want a UI; they do not conflict.
#
# ProcessType is Standard, not Background. launchd implements Background as
# nice 19 plus throttled disk I/O, and children inherit both — which turned a
# Topaz encode into 0.05x and an eight-hour ETA. These jobs are the work the
# user asked for; throttling them is backwards. Set JOB_NICE if you want the
# queue to yield to foreground work (see below) — that is a per-job decision
# and belongs to the runner, not to launchd.
#
# Two WatchPaths, not one. The top level fires when a .job is dropped; _ready
# fires when a folder is moved back into the queue — releasing a held job, or
# requeueing a failed one — which the top level would never notice.
#
# $JOBS_DIR is passed through EnvironmentVariables rather than assumed. A
# launchd agent inherits almost nothing, so without it the agent would watch
# the folder you asked for and run jobs out of the default one.
#
# Run as your normal user, NOT under sudo -- this installs a per-user
# LaunchAgent into gui/$(id -u); under sudo it would land in root's domain.

set -euo pipefail

LABEL="${JOB_DAEMON_LABEL:-com.jayu.job-daemon}"
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
DOTFILES_DIR="${DOTFILES_DIR:-$(cd -- "$SCRIPT_DIR/../.." && pwd)}"

WORKSPACE="$DOTFILES_DIR/utils/jobs"
JOBS_DIR="${JOBS_DIR:-$HOME/jobs}"
INSTALL_DIR="${JOB_INSTALL_DIR:-$HOME/.local/bin}"
BINARY_PATH="$INSTALL_DIR/job-daemon"

LAUNCH_AGENTS_DIR="${JOB_LAUNCH_AGENTS_DIR:-$HOME/Library/LaunchAgents}"
LOG_DIR="${JOB_LOG_DIR:-$HOME/Library/Logs}"
JOB_CONCURRENCY="${JOB_CONCURRENCY:-2}"
# 0 runs jobs at normal priority; raise it to make the queue yield to whatever
# you are doing in the foreground. Only ever raised — lowering it needs root.
JOB_NICE="${JOB_NICE:-0}"

# Everything this supersedes: every earlier name the runner has had, including
# the short-lived job-manager that ran the loop behind a menu bar. A machine
# set up before a rename would otherwise keep running the queue under its old
# agent alongside this one.
SUPERSEDED_LABELS=(
  com.jayu.job-manager
  com.jayu.job-server
  com.jayu.job-server-cli
  com.jayu.job-folder
  com.jayu.job-runner
)
SUPERSEDED_BINARIES=(
  "$INSTALL_DIR/job-manager"
  "$INSTALL_DIR/job-server"
  "$INSTALL_DIR/job-server-cli"
  "$INSTALL_DIR/job-folder"
  "$INSTALL_DIR/job-runner"
)

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
  local plist_path="$1"
  local esc_binary esc_watch esc_ready esc_out esc_err esc_log

  esc_binary="$(plist_escape "$BINARY_PATH")"
  esc_watch="$(plist_escape "$JOBS_DIR")"
  esc_ready="$(plist_escape "$JOBS_DIR/_ready")"
  esc_out="$(plist_escape "$LOG_DIR/job-daemon.out.log")"
  esc_err="$(plist_escape "$LOG_DIR/job-daemon.err.log")"
  esc_log="$(plist_escape "$LOG_DIR/jobs.log")"

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
    <string>--once</string>
  </array>
  <key>WatchPaths</key>
  <array>
    <string>$esc_watch</string>
    <string>$esc_ready</string>
  </array>
  <key>EnvironmentVariables</key>
  <dict>
    <key>JOBS_DIR</key>
    <string>$esc_watch</string>
    <key>JOB_LOG</key>
    <string>$esc_log</string>
    <key>JOB_CONCURRENCY</key>
    <string>$JOB_CONCURRENCY</string>
    <key>JOB_NICE</key>
    <string>$JOB_NICE</string>
  </dict>
  <key>RunAtLoad</key>
  <true/>
  <key>ProcessType</key>
  <string>Standard</string>
  <key>StandardOutPath</key>
  <string>$esc_out</string>
  <key>StandardErrorPath</key>
  <string>$esc_err</string>
</dict>
</plist>
EOF
}

uninstall_superseded() {
  local gui_domain="gui/$(id -u)" label old_plist binary

  for label in "${SUPERSEDED_LABELS[@]}"; do
    old_plist="$LAUNCH_AGENTS_DIR/$label.plist"
    if [[ -f "$old_plist" ]] || launchctl print "$gui_domain/$label" >/dev/null 2>&1; then
      echo "Retiring superseded agent $label ..."
      launchctl bootout "$gui_domain" "$old_plist" >/dev/null 2>&1 || true
      launchctl bootout "$gui_domain/$label" >/dev/null 2>&1 || true
      rm -f "$old_plist"
    fi
  done

  for binary in "${SUPERSEDED_BINARIES[@]}"; do
    if [[ -e "$binary" ]]; then
      echo "Removing superseded binary $binary ..."
      rm -f "$binary"
    fi
  done
}

main() {
  require_command cargo
  require_command launchctl
  require_command plutil
  require_command sed

  if [[ ! -d "$WORKSPACE" ]]; then
    echo "Error: workspace not found: $WORKSPACE" >&2
    exit 1
  fi

  uninstall_superseded

  echo "Building job-daemon (release) ..."
  (cd "$WORKSPACE" && cargo build --release --bin job-daemon)

  echo "Installing binary to $BINARY_PATH ..."
  mkdir -p "$INSTALL_DIR"
  install -m 755 "$WORKSPACE/target/release/job-daemon" "$BINARY_PATH"

  echo "Creating jobs folder $JOBS_DIR ..."
  mkdir -p "$JOBS_DIR" "$JOBS_DIR/_ready" "$JOBS_DIR/_running" "$JOBS_DIR/_paused" \
    "$JOBS_DIR/_ok" "$JOBS_DIR/_failed" "$LAUNCH_AGENTS_DIR" "$LOG_DIR"

  local plist_path="$LAUNCH_AGENTS_DIR/$LABEL.plist"
  echo "Writing LaunchAgent to $plist_path ..."
  write_launch_agent "$plist_path"
  plutil -lint "$plist_path" >/dev/null

  local gui_domain="gui/$(id -u)"
  echo "Loading LaunchAgent..."
  launchctl bootout "$gui_domain/$LABEL" >/dev/null 2>&1 || true
  launchctl bootstrap "$gui_domain" "$plist_path"
  launchctl enable "$gui_domain/$LABEL"

  echo
  echo "Installed $LABEL."
  echo "  Binary:       $BINARY_PATH"
  echo "  Watch folder: $JOBS_DIR (launchd WatchPaths on it and _ready)"
  echo "  Concurrency:  $JOB_CONCURRENCY"
  echo "  Priority:     nice $JOB_NICE (Standard, not throttled)"
  echo "  Logs:         $LOG_DIR/jobs.log (+ .out/.err)"
  echo
  echo "Drop a *.job shell script into $JOBS_DIR to run it."
}

main "$@"
