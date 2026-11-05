#!/usr/bin/env bash
# Prepare Abner's optional provider plugins without changing existing checkouts.
set -euo pipefail

if [[ ${1:-} == -h || ${1:-} == --help ]]; then
  printf 'Usage: %s [repo-home]\nDefaults to SRC_PATH or ~/src.\n' "${0##*/}"
  exit 0
fi
if (( $# > 1 )); then
  printf 'Usage: %s [repo-home]\n' "${0##*/}" >&2
  exit 2
fi

repo_home=${1:-${SRC_PATH:-"$HOME/src"}}
mkdir -p -- "$repo_home"
repo_home=$(cd -- "$repo_home" && pwd -P)
abner="$repo_home/abner"
plugins="$abner/plugins"

checkout() {
  local name=$1 repo="$repo_home/$1" prefix
  if [[ -e "$repo" || -L "$repo" ]]; then
    if prefix=$(git -C "$repo" rev-parse --show-prefix 2>/dev/null) && [[ -z "$prefix" ]]; then
      printf 'Using existing checkout: %s\n' "$repo"
    else
      printf 'Expected a Git checkout at %s. Left it untouched.\n' "$repo" >&2
      return 1
    fi
  else
    git clone -- "https://github.com/monomadic/$name.git" "$repo"
  fi
}

link_plugin() {
  local provider=$1 plugin=$2
  local source="$repo_home/$provider/abner-plugins/$plugin"
  local destination="$plugins/$plugin"
  if [[ ! -f "$source/Cargo.toml" ]]; then
    printf 'Missing %s/Cargo.toml. Update the %s checkout and retry.\n' "$source" "$provider" >&2
    return 1
  fi
  if ! git -C "$abner" check-ignore -q -- "plugins/$plugin"; then
    printf 'Abner does not ignore %s. Update Abner and retry.\n' "$destination" >&2
    return 1
  fi
  if [[ -L "$destination" && "$destination" -ef "$source" ]]; then
    printf 'Already linked: %s\n' "$destination"
  elif [[ -e "$destination" || -L "$destination" ]]; then
    printf 'Existing entry at %s. Left it untouched.\n' "$destination" >&2
    return 1
  else
    ln -sh -- "$source" "$destination" || return
    printf 'Linked %s -> %s\n' "$destination" "$source"
  fi
}

checkout abner
if [[ -L "$plugins" || -e "$plugins/.git" || -L "$plugins/.git" ]]; then
  backup=$(mktemp -d "$repo_home/abner-plugins-backup.XXXXXX")
  mv -- "$plugins" "$backup/plugins"
  printf 'Moved the old plugins checkout or link to %s.\n' "$backup/plugins"
  # Recreate tracked defaults in the vacated path. checkout-index refuses overwrites.
  git -C "$abner" ls-files -z -- plugins/defaults |
    git -C "$abner" checkout-index -z --stdin
fi
if [[ ! -f "$plugins/defaults/rife-metal/Cargo.toml" ]]; then
  printf 'Update the Abner checkout at %s to a revision with built-in plugins and retry.\n' "$abner" >&2
  exit 1
fi

failed=0
if checkout topaz-rs; then
  link_plugin topaz-rs proteus || failed=1
  link_plugin topaz-rs proteus-natural || failed=1
  link_plugin topaz-rs nyx || failed=1
  link_plugin topaz-rs topaz-generative || failed=1
else
  failed=1
fi
if checkout inpaint-video; then
  link_plugin inpaint-video inpaint-video || failed=1
else
  failed=1
fi
if (( failed )); then
  exit 1
fi

printf 'Build from %s with cargo build-with-plugins --release.\n' "$abner"
