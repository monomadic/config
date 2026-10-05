#!/usr/bin/env bash
# Build and install neuralmix: stems, key, tempo, beat grid and sections of a
# track by Neural Mix Pro's analysis rebuilt in Rust; Serato tags (--tag) and
# an HTML report (--html).
#
# Upstream repo, not src/ — the checkout lives in $SRC_PATH (default ~/src) and
# this script keeps it current. Re-run it to pick up new commits.
#
# The binary runs none of the app's code, but it does run the app's trained
# models, and those cannot be shipped: `neuralmix setup` decrypts them from an
# installed Neural Mix Pro 2.0.2 (read only) into <checkout>/models, which is
# git-ignored. The installed binary finds them by the checkout path it was
# compiled from — moving the checkout means re-running this script.
#
#   NEURAL_MIX_APP=/path/to/"Neural Mix Pro.app"   where the app is, if not found
#   FORCE=1                                        rebuild the binary, extract again

. "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/lib/git-source-install.sh"

git_source_install neuralmix-rs "https://github.com/monomadic/neuralmix-rs.git" cargo neuralmix

neuralmix="$INSTALL_DIR/neuralmix"

find_app() {
  local app
  for app in "${NEURAL_MIX_APP:-}" "/Applications/Neural Mix Pro.app" "$HOME/Applications/Neural Mix Pro.app" "$HOME/Neural Mix Pro.app"; do
    [ -n "$app" ] && [ -d "$app" ] && { echo "$app"; return; }
  done
  mdfind "kMDItemCFBundleIdentifier == 'com.algoriddim.neuralmix'" 2>/dev/null |
    while IFS= read -r app; do
      [ -d "$app" ] && { echo "$app"; break; }
    done
}

# `neuralmix info` checks every model and filter against its recorded hash.
if [ "${FORCE:-0}" != "1" ] && "$neuralmix" info >/dev/null 2>&1; then
  echo "models are current: $SRC_PATH/neuralmix-rs/models"
else
  app="$(find_app)"
  [ -n "$app" ] || _gsi_die "Neural Mix Pro.app not found; install 2.0.2 or set NEURAL_MIX_APP to its path"
  echo "Extracting the models from $app..."
  "$neuralmix" setup --app "$app"
  "$neuralmix" info >/dev/null || _gsi_die "the models were extracted but do not verify; run: $neuralmix info"
fi

# GUI-launched tools call ~/.local/bin/neuralmix by path; a shell needs it on PATH.
found="$(command -v neuralmix 2>/dev/null || true)"
if [ -z "$found" ]; then
  _gsi_warn "$INSTALL_DIR is not on PATH; neuralmix is installed but a shell will not find it"
elif [ "$found" != "$neuralmix" ]; then
  _gsi_warn "another neuralmix comes first on PATH: $found"
fi
