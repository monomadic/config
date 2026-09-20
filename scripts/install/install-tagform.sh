#!/usr/bin/env bash
# Build and install tagform, the media tag editor TUI.
#
# Upstream repo, not src/ — the checkout lives in $SRC_PATH (default ~/src) and
# this script keeps it current. Re-run it to pick up new commits.
#
# Launched from yazi's "edit tags" action and switchblade's `e` binding.

. "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/lib/git-source-install.sh"

git_source_install tagform "https://github.com/monomadic/tagform.git" cargo
