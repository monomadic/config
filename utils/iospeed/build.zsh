#!/usr/bin/env zsh
# Build iospeed and install it to ~/.local/bin.

set -euo pipefail

src_dir="${0:A:h}"
install_dir="$HOME/.local/bin"

mkdir -p "$install_dir"

echo "Building iospeed..."
(cd "$src_dir" && go build -trimpath -ldflags="-s -w" -o "$install_dir/iospeed" .)

echo "Installed: $install_dir/iospeed"
