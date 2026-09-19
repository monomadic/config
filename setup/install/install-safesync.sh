#!/bin/sh
# Build the independent safesync inventory and offline-lookup tool.
set -eu
cd "$(dirname "$0")/../../utils/safesync"
cargo build --release --locked
mkdir -p "$HOME/.local/bin"
install -m 755 target/release/safesync "$HOME/.local/bin/safesync"
printf 'installed: %s/.local/bin/safesync\n' "$HOME"
