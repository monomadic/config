#!/usr/bin/env bash
# Build and install chordpro-tui, the ChordPro chart browser.
#
# Upstream repo, not src/ — the checkout lives in $SRC_PATH (default ~/src) and
# this script keeps it current. Re-run it to pick up new commits.
#
# Replaces the old `go install ...@latest` one-liner, which pinned nothing and
# left no source to read. bin/chordpro-from-clipboard and motherfucker's
# "ChordPro" entry both invoke the installed binary.

. "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/lib/git-source-install.sh"

git_source_install chordpro-tui "https://github.com/monomadic/chordpro-tui.git" go
