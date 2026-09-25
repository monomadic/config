#!/usr/bin/env bash
# Build and install safesync, indexed drive sync with sentinels and offline lookup.
#
# Upstream repo, not src/ — the checkout lives in $SRC_PATH (default ~/src) and
# this script keeps it current. Re-run it to pick up new commits.

. "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/lib/git-source-install.sh"

git_source_install safesync "https://github.com/monomadic/safesync.git" cargo
