#!/bin/bash

# clone config
git clone https://github.com/monomadic/config

# homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# install Brewfile
brew bundle
