#!/bin/sh
# Build and install pimped, the fast native zsh prompt renderer (utils/pimped).
# Static Rust binary, no daemon; the zsh precmd hook in config/zsh/zshrc.zsh
# picks it up automatically once it is on PATH.

set -e
cd "$(dirname "$0")/../../utils/pimped"
cargo build --release
mkdir -p "$HOME/.local/bin"
install -m 755 target/release/pimped "$HOME/.local/bin/pimped"
echo "installed: $HOME/.local/bin/pimped"
