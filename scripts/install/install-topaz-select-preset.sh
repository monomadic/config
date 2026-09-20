#!/bin/sh
# Build and install topaz-select-preset, the TUI that previews and encodes the
# Topaz ffmpeg presets (src/topaz-select-preset).
# Static Rust binary; it calls the deployed ~/.local/bin/topaz-preview-frame and
# ~/.local/bin/topaz-encode, so the zsh package must be deployed too.

set -e
cd "$(dirname "$0")/../../src/topaz-select-preset"
cargo build --release
mkdir -p "$HOME/.local/bin"
install -m 755 target/release/topaz-select-preset "$HOME/.local/bin/topaz-select-preset"
echo "installed: $HOME/.local/bin/topaz-select-preset"
