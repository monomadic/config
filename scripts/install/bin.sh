#!/bin/sh

mkdir $HOME/.local/bin
wget -O $HOME/.local/bin/bin https://github.com/marcosnils/bin/releases/download/v0.16.2/bin_0.16.2_Darwin_arm64
chmod +x $HOME/.local/bin/bin
