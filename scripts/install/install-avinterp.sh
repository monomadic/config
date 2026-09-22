#!/bin/sh
# Build and install avinterp (src/utils/avinterp): ML frame interpolation (VideoToolbox VTFrameRateConversion).
# Swift package; needs the Xcode command line tools (macOS 26 SDK).

set -e
cd "$(dirname "$0")/../../src/utils/avinterp"
swift build -c release
mkdir -p "$HOME/.local/bin"
install -m 755 .build/release/avinterp "$HOME/.local/bin/avinterp"
echo "installed: $HOME/.local/bin/avinterp"
