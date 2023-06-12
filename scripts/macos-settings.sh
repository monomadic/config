#!/bin/sh

# KEYBOARD SHORTCUTS
# note: requires logout or app restart

# open kitty with option+space
defaults write NSGlobalDomain NSUserKeyEquivalents -dict-add 'Kitty' '@Space'

# maximise window with cmd+shift+up
defaults write NSGlobalDomain NSUserKeyEquivalents -dict-add "Zoom" "@$^\\Uf700"
