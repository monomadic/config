#!/bin/sh

# After setting preferences, restart System Preferences:
# killall SystemUIServer
#
# Read all hotkeys:
# defaults read com.apple.symbolichotkeys AppleSymbolicHotKeys
#
# Delete all hotkeys:
# defaults delete com.apple.symbolichotkeys AppleSymbolicHotKeys

# ~/Library/Preferences/com.apple.ScreenContinuity.plist
#
# Resize the iphone mirroring window
defaults write com.apple.ScreenContinuity showScalingControls -bool true

# Set caps key to esc
defaults -currentHost write -g com.apple.keyboard.modifiermapping -array-add '<dict><key>HIDKeyboardModifierMappingSrc</key><integer>0</integer><key>HIDKeyboardModifierMappingDst</key><integer>53</integer></dict>'

# KeyBindings
#
defaults write com.apple.symbolichotkeys AppleSymbolicHotKeys -dict-add 32 '{ enabled = 1; value = { parameters = ( 3, 655360, 0 ); type = standard; }; }'

killall SystemUIServer
