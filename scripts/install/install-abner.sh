#!/usr/bin/env bash
# Build and install Abner.app, the A/B comparison player for videos.
#
# Upstream repo, not src/ — the checkout lives in $SRC_PATH (default ~/src) and
# this script keeps it current. Re-run it to pick up new commits.
#
# The install is the bundle, built by the repo's packaging/build-app.sh into
# /Applications; yazi and switchblade launch it with `open -a Abner`.
# ~/.local/bin/abner is a shim into the bundle for the shell.

. "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/lib/git-source-install.sh"

git_source_install abner "https://github.com/monomadic/abner.git" app Abner
