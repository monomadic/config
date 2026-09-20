#!/usr/bin/env bash

# dock animation timer
defaults write com.apple.dock autohide-time-modifier -float 0.1

# turn off animation entirely
defaults write com.apple.dock autohide-delay -int 0

killall Dock
