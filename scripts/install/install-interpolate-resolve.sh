#!/usr/bin/env bash
# Build/update the private interpolate-resolve CLI in $SRC_PATH (default ~/src).
# Installs to ~/.local/bin; FORCE=1 rebuilds even when current.
# Requires authenticated GitHub Git access (e.g. gh auth login + gh auth setup-git).
# Dependencies: Rust, Xcode command-line tools, and
#   brew install openexr pkg-config ffmpeg

. "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/lib/git-source-install.sh"

[ "$(uname -s)" = Darwin ] && [ "$(uname -m)" = arm64 ] ||
  _gsi_die "interpolate-resolve requires Apple Silicon macOS 15.4+"
for tool in cargo git xcrun pkg-config ffprobe; do
  command -v "$tool" >/dev/null 2>&1 || _gsi_die "Missing $tool; see prerequisites in this installer"
done
xcrun --find clang >/dev/null 2>&1 || _gsi_die "Install Xcode command-line tools: xcode-select --install"
pkg-config --exists OpenEXR || _gsi_die "Install OpenEXR: brew install openexr pkg-config"

# The old scripted command may still have a deployed symlink. Never install
# through a live link and overwrite its source. A dangling link holds no data.
dest="$INSTALL_DIR/interpolate-resolve"
if [ -L "$dest" ]; then
  [ ! -e "$dest" ] || _gsi_die "$dest links to an existing file; move that link aside before installing"
  rm "$dest"
fi

git_source_install interpolate-resolve "https://github.com/monomadic/interpolate-resolve.git" cargo
