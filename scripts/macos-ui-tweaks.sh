#!/bin/zsh

# Exit immediately if a command exits with a non-zero status
set -e

echo "Tweaking macOS preferences for faster GUI..."

# Disable window animations and smooth resizing
defaults write -g NSAutomaticWindowAnimationsEnabled -bool false
defaults write -g NSWindowResizeTime -float 0.001

# Reduce the delay for showing the Dock
defaults write com.apple.dock autohide-delay -float 0
defaults write com.apple.dock autohide-time-modifier -float 0.1

# Speed up Mission Control animations
defaults write com.apple.dock expose-animation-duration -float 0.1

# Disable animation for opening and closing Quick Look windows
defaults write -g QLPanelAnimationDuration -float 0

# Speed up the dialog box animations
defaults write -g NSInitialToolTipDelay -integer 0

# Speed up key repeat rates
# defaults write -g KeyRepeat -int 1
# defaults write -g InitialKeyRepeat -int 10

# Disable the shadow in screenshots
defaults write com.apple.screencapture disable-shadow -bool true

# Disable the dashboard
defaults write com.apple.dashboard mcx-disabled -bool true
defaults write com.apple.dock dashboard-in-overlay -bool true

# Speed up Safari (disable animations for rendering)
defaults write com.apple.Safari WebKitInitialTimedLayoutDelay 0.25

# Disable animation for showing/hiding menus
defaults write -g NSMenuFadeInDuration -float 0
defaults write -g NSMenuFadeOutDuration -float 0

# Accelerate Finder animations
defaults write com.apple.finder DisableAllAnimations -bool true

# Reduce system transparency
defaults write com.apple.universalaccess reduceTransparency -bool true

# Disable smart quotes and dashes for faster text input
defaults write -g NSAutomaticQuoteSubstitutionEnabled -bool false
defaults write -g NSAutomaticDashSubstitutionEnabled -bool false

# Disable animations for opening and closing apps
defaults write com.apple.dock launchanim -bool false

# Apply Dock changes
killall Dock

# Apply Finder changes
killall Finder

echo "Tweaks applied. Some changes might require a logout or restart to take effect."
