#!/usr/bin/env bash
# Provision this box as a headless server. Run as your normal user (the
# individual commands elevate with sudo where needed); do NOT run the whole
# script under sudo, or the per-user LaunchAgent below installs into root.
set -uo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

# disable sleep
sudo systemsetup -setcomputersleep Never
sudo pmset -a disablesleep 1

# ssh remote login
sudo systemsetup -setremotelogin on

# smb (hosts the ~/jobs share that `send-job` queues work on)
sudo launchctl load -w /System/Library/LaunchDaemons/com.apple.smbd.plist

# ~/jobs job folders: job-folder runs whatever lands in a workflow's input/.
# It is an app, not an agent — the queue runs while it is open — so it goes in
# Login Items to come back after a reboot.
"$SCRIPT_DIR/../install/install-job-folder.sh"
osascript -e 'tell application "System Events" to if not (exists login item "Job Folder") then make login item at end with properties {path:"'"$HOME"'/Applications/Job Folder.app", hidden:true}' \
  || echo "warning: could not add Job Folder to Login Items; add it by hand" >&2
open -a "$HOME/Applications/Job Folder.app"
