#!/bin/bash

# Global Keyboard Shortcuts
echo "Configuring global keyboard shortcuts..."

# Script to assign a keyboard shortcut to Finder's "Merge All Windows" option.

# MODIFIERS:
# • @ = Command (⌘)
# • ~ = Option (⌥)
# • ^ = Control (⌃)
# • $ = Shift (⇧)

# CUSTOMIZATION INSTRUCTIONS:
# 1. Modify the menu item string to match any Finder action you'd like to remap a shortcut to.
#    Example: Change "Merge All Windows" to "New Tab" to assign a shortcut to Finder's "New Tab" option.
#
# 2. Modify the key binding by adjusting the key symbols and letter.
#    • For example, to change the shortcut to Command + Shift + T, replace "@~^M" with "@$T".
#    • Ensure the letter or symbol after the modifier codes corresponds to the desired key.
#    • For special keys (e.g., arrow keys), use their Unicode equivalents:
#      - UP: "\UF700"
#      - DOWN: "\UF701"
#      - LEFT: "\UF702"
#      - RIGHT: "\UF703"
#
# 3. To bind multiple shortcuts, you can add more entries with `-dict-add`:
#    Example:
#    defaults write com.apple.finder NSUserKeyEquivalents -dict-add "New Tab" "@$T"
#    defaults write com.apple.finder NSUserKeyEquivalents -dict-add "Show View Options" "@~^V"
#
# 4. You can target different applications by replacing "com.apple.finder" with the bundle ID of the desired app.
#    Example: Change Finder's settings to Safari's:
#    defaults write com.apple.Safari NSUserKeyEquivalents -dict-add "New Tab" "@$T"
#
# 5. To remove the shortcut later, run the same command but use `-dict-remove` instead of `-dict-add`.
#
# Alternate method:
# /usr/libexec/PlistBuddy -c "Add :NSUserKeyEquivalents:'Open In Kitty' string @\\U0000000A" ~/Library/Preferences/com.apple.finder.plist
#
#	defaults delete com.apple.finder NSUserKeyEquivalents
#
#	To see all existing shortcuts (for finder for example):
#	defaults read com.apple.finder NSUserKeyEquivalents
#

# Zoom action (e.g., Fill Window)
defaults write com.apple.finder NSUserKeyEquivalents -dict-add "Select Startup Disk" "\0"
defaults write -g NSUserKeyEquivalents -dict-add "Fill" "@$↑"

# Move and Resize Shortcuts
defaults write NSGlobalDomain NSUserKeyEquivalents '{
    "Move & Resize -> Left" = "@$←";
    "Move & Resize -> Right" = "@$→";
    "Center" = "@$↓";
    "Move & Resize -> Top & Bottom" = "~$↑";
    "Move & Resize -> Bottom & Top" = "~$↓";
    "Move & Resize -> Left & Right" = "~$←";
    "Move & Resize -> Right & Left" = "~$→";
    "Move & Resize -> Quarters" = "~$4";
}'

# Finder-Specific Shortcuts
echo "Configuring Finder-specific keyboard shortcuts..."

# Merge All Windows in Finder
# ⌘$M to merge all Finder windows
defaults write com.apple.finder NSUserKeyEquivalents -dict-add "Merge All Windows" '@$M'

# Open current directory in Kitty
# Cmd+Enter to open the directory in Kitty
defaults write com.apple.finder NSUserKeyEquivalents -dict-add "Open in Kitty" "@\\U21a9"

# Restart Finder for changes to take effect
echo "Restarting Finder to apply changes..."
killall Finder

echo "Keyboard shortcuts configuration complete!"
