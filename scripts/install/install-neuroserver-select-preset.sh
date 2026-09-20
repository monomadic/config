#!/bin/sh
# Build and install neuroserver-select-preset, the TUI that previews and
# encodes neuroserver (Starlight Precise) presets (src/neuroserver-select-preset).
# The whole-clip encoder is part of the same binary; a neuroserver-encode
# symlink beside it runs that mode directly. Previews still call the deployed
# ~/.local/bin/topaz-preview-frame, so the zsh package must be deployed too.

set -e
cd "$(dirname "$0")/../../src/neuroserver-select-preset"
cargo build --release
mkdir -p "$HOME/.local/bin"
install -m 755 target/release/neuroserver-select-preset "$HOME/.local/bin/neuroserver-select-preset"
ln -sf neuroserver-select-preset "$HOME/.local/bin/neuroserver-encode"
echo "installed: $HOME/.local/bin/neuroserver-select-preset (+ neuroserver-encode)"
