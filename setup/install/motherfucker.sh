#!/bin/sh
# Build and install motherfucker (cache-free Spotlight replacement) into ~/.bin

set -e
cd "$(dirname "$0")/../../utils/motherfucker"
cargo build --release
mkdir -p "$HOME/.bin"
cp target/release/motherfucker "$HOME/.bin/motherfucker"
echo "installed: $HOME/.bin/motherfucker"
