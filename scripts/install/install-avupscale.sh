#!/bin/sh
# Build and install avupscale (src/utils/avupscale): ML super-resolution (VideoToolbox VTSuperResolutionScaler).
# Swift package; needs the Xcode command line tools (macOS 26 SDK).

set -e
cd "$(dirname "$0")/../../src/utils/avupscale"
swift build -c release
mkdir -p "$HOME/.local/bin"
install -m 755 .build/release/avupscale "$HOME/.local/bin/avupscale"
echo "installed: $HOME/.local/bin/avupscale"
