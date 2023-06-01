#!/bin/bash

cd $HOME

# clone config
git clone https://github.com/monomadic/config config
cd config

# install homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
# link global brewfile
mkdir -p $XDG_CONFIG_HOME/brewfile/
ln -s $PWD/Brewfile $XDG_CONFIG_HOME/brewfile/Brewfile
# install from Brewfile
brew bundle --global

# create my working directories
mkdir ~/workspaces
mkdir ~/src
touch ~/.marks

cp .env ~/.env
