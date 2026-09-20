#!/usr/bin/env bash
# Build and install switchblade, the fullscreen media browser.
#
# Upstream repo, not src/ — the checkout lives in $SRC_PATH (default ~/src) and
# this script keeps it current. Re-run it to pick up new commits.
#
# Referenced by absolute path from GUI-launched config (motherfucker, yazi,
# mpv's open-panel, karabiner), all of which point at ~/.local/bin.

. "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/lib/git-source-install.sh"

git_source_install switchblade "https://github.com/monomadic/switchblade.git" cargo
