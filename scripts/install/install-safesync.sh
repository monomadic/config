#!/bin/sh
# Build safesync: indexed one-way drive sync, fill and offline lookup.
set -eu
cd "$(dirname "$0")/../../src/safesync"
cargo build --release --locked
mkdir -p "$HOME/.local/bin"
install -m 755 target/release/safesync "$HOME/.local/bin/safesync"
printf 'installed: %s/.local/bin/safesync\n' "$HOME"
