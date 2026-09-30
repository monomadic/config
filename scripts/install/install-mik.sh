#!/usr/bin/env bash
# Build and install mik: key, tempo, beat grid and sections of a track, written
# to FLAC files as Serato tags, with an optional HTML report (`mik --html`).
#
# Upstream repo, not src/ — the checkout lives in $SRC_PATH (default ~/src) and
# this script keeps it current. Re-run it to pick up new commits.
#
# The binary carries its key model; what it cannot carry is the runtime: a
# private copy of Mixed In Key 11.2.6's analysis code, which mik builds from the
# installed app (read only) into <checkout>/runtime. That copy is proprietary
# and git-ignored, so it is made here, on this machine, the first time through.
# The installed binary finds it by the checkout path it was compiled from —
# moving the checkout means re-running this script.
#
#   MIK_APP=/path/to/"Mixed In Key 11.app"   where the app is, if not found
#   FORCE=1                                  rebuild the binary and the runtime

. "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/lib/git-source-install.sh"

git_source_install mik-rs "https://github.com/monomadic/mik-rs.git" cargo mik

mik="$INSTALL_DIR/mik"
runtime="$SRC_PATH/mik-rs/runtime"

find_app() {
  local app
  for app in "${MIK_APP:-}" "/Applications/Mixed In Key 11.app" "$HOME/Applications/Mixed In Key 11.app"; do
    [ -n "$app" ] && [ -d "$app" ] && { echo "$app"; return; }
  done
  # Anywhere else Spotlight knows of (it lives in ~/Movies on this machine).
  mdfind "kMDItemCFBundleIdentifier == 'com.mixedinkey.application'" 2>/dev/null |
    while IFS= read -r app; do
      [ -d "$app" ] && { echo "$app"; break; }
    done
}

# `mik info` verifies every runtime file against the hashes recorded at setup.
if [ "${FORCE:-0}" != "1" ] && "$mik" info >/dev/null 2>&1; then
  echo "runtime is current: $runtime"
else
  app="$(find_app)"
  [ -n "$app" ] || _gsi_die "Mixed In Key 11.app not found; install 11.2.6 or set MIK_APP to its path"
  echo "Building the runtime from $app..."
  if [ -e "$runtime/manifest.json" ]; then
    # An existing runtime that got here is stale or damaged: replace it.
    "$mik" setup --app "$app" --force
  else
    "$mik" setup --app "$app"
  fi
  "$mik" info >/dev/null || _gsi_die "the runtime was built but does not verify; run: $mik info"
  echo "runtime ready: $runtime"
fi

# GUI-launched tools call ~/.local/bin/mik by path; a shell needs it on PATH.
found="$(command -v mik 2>/dev/null || true)"
if [ -z "$found" ]; then
  _gsi_warn "$INSTALL_DIR is not on PATH; mik is installed but a shell will not find it"
elif [ "$found" != "$mik" ]; then
  _gsi_warn "another mik comes first on PATH: $found"
fi
