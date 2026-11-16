#!/usr/bin/env bash
# Restore the pinned CoderBar build without requiring Xcode or Swift.
set -euo pipefail
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
DOTFILES_DIR="$(cd -- "$SCRIPT_DIR/../.." && pwd)"
APP_DIR="${APP_DIR:-/Applications/CoderBar.app}"
mkdir -p "$APP_DIR/Contents/MacOS" "$APP_DIR/Contents/Resources"
for binary in coder-bar coder-bar-hook coder-bar-ctl; do
    install -m 0755 "$DOTFILES_DIR/vendor/bin/$binary" "$APP_DIR/Contents/MacOS/$binary"
done
cp "$DOTFILES_DIR/assets/coderbar/Info.plist" "$APP_DIR/Contents/Info.plist"
cp "$DOTFILES_DIR/assets/coderbar/AppIcon.icns" "$DOTFILES_DIR/assets/coderbar/ChatGPT.png" "$APP_DIR/Contents/Resources/"
codesign --force --deep --sign - "$APP_DIR"
codesign --verify --deep --strict "$APP_DIR"
echo "Installed $APP_DIR"
