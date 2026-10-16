#!/bin/sh
# Build the local standalone Rust/Metal CLI and install its native runtime together.
set -eu
source_dir="${SRC_PATH:-$HOME/src}/rife-metal-rs"
[ -f "$source_dir/install.sh" ] || {
  echo "Missing $source_dir. Copy or clone the standalone rife-metal-rs repository there first." >&2
  exit 1
}
exec "$source_dir/install.sh"
