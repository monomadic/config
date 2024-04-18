#!/bin/sh

set -e	# exit on nonzero result

if [ ! -L "$HOME/.config/yazi" ] && [ ! -e "$HOME/.config/yazi" ]; then
    ln -s "$PWD/config/yazi" "$HOME/.config/yazi"
fi
if ! command -v yazi &> /dev/null; then
	echo "Installing yazi"
  cargo install yazi-prebuild --features="build_deps"
fi

echo "update success."
