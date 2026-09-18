#!/bin/sh
# Build and install neuroserver-select-preset, the TUI that previews and
# encodes neuroserver (Starlight Precise) presets (utils/neuroserver-select-preset).
# Static Rust binary; it calls the deployed ~/.zsh/bin/topaz-preview-frame and
# ~/.zsh/bin/neuroserver-encode, so the zsh package must be deployed too.

set -e
cd "$(dirname "$0")/../../utils/neuroserver-select-preset"
cargo build --release
mkdir -p "$HOME/.local/bin"
install -m 755 target/release/neuroserver-select-preset "$HOME/.local/bin/neuroserver-select-preset"
echo "installed: $HOME/.local/bin/neuroserver-select-preset"
