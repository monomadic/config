#!/bin/sh
# Build and install dupchk, the duplicate-size file finder (src/dupchk).
# Rust rewrite of the old bin/dupchk script: parallel directory
# walk, in-process interactive selection, Finder-native trash.

set -e
cd "$(dirname "$0")/../../src/dupchk"
cargo build --release
mkdir -p "$HOME/.local/bin"
install -m 755 target/release/dupchk "$HOME/.local/bin/dupchk"
echo "installed: $HOME/.local/bin/dupchk"
