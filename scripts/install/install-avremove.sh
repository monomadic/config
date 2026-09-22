#!/bin/sh
# Build and install avremove (src/utils/avremove): object removal (Vision masks + LaMa via Core ML).
# Swift package; needs the Xcode command line tools (macOS 26 SDK).

set -e
cd "$(dirname "$0")/../../src/utils/avremove"
swift build -c release
mkdir -p "$HOME/.local/bin"
install -m 755 .build/release/avremove "$HOME/.local/bin/avremove"
echo "installed: $HOME/.local/bin/avremove"
