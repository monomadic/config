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

# smb (hosts the ~/jobs share that `send-job` ships work to)
sudo launchctl load -w /System/Library/LaunchDaemons/com.apple.smbd.plist

# ~/jobs watch folder: launchd WatchPaths agent that runs uploaded *.job scripts
"$SCRIPT_DIR/../install/install-job-runner.sh"
