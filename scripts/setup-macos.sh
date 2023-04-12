#!/bin/bash

# clone config
git clone https://github.com/monomadic/config
cd config

# install homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
# link global brewfile
ln -s $PWD/Brewfile ~/.Brewfile
# install from Brewfile
brew bundle --global

# create my working directories
mkdir ~/workspaces
mkdir ~/src
touch ~/.marks

cp .env ~/.env
