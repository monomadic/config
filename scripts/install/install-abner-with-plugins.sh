#!/usr/bin/env bash
# Update the provider checkouts and install a composed Abner.app.
. "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/lib/git-source-install.sh"

if [[ ${1:-} == -h || ${1:-} == --help ]]; then
  printf 'Usage: %s [repo-home]\nDefaults to SRC_PATH or ~/src. FORCE=1 rebuilds the app.\n' "${0##*/}"
  exit 0
fi
if (( $# > 1 )); then
  printf 'Usage: %s [repo-home]\n' "${0##*/}" >&2
  exit 2
fi

repo_home=${1:-"$SRC_PATH"}
mkdir -p -- "$repo_home"
repo_home=$(cd -- "$repo_home" && pwd -P)
abner="$repo_home/abner"
plugins="$abner/plugins"
app=/Applications/Abner.app
launcher="$app/Contents/MacOS/abner"
installed_binary="$launcher-bin"
stamp="$plugins/.launcher/config-install-state"
topaz_plugins=(proteus proteus-natural nyx topaz-generative)

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
_gsi_sync "$abner" >/dev/null
if [[ ! -f "$plugins/defaults/rife-metal/Cargo.toml" ]]; then
  printf 'Update the Abner checkout at %s to a revision with built-in plugins and retry.\n' "$abner" >&2
  exit 1
fi

checkout topaz-rs
_gsi_sync "$repo_home/topaz-rs" >/dev/null
checkout inpaint-video
_gsi_sync "$repo_home/inpaint-video" >/dev/null

failed=0
for plugin in "${topaz_plugins[@]}"; do
  link_plugin topaz-rs "$plugin" || failed=1
done
link_plugin inpaint-video inpaint-video || failed=1
if (( failed )); then
  exit 1
fi

# The Abner commit alone cannot detect provider updates or a plain app install.
# Track the source revisions and the exact executable installed by this script.
build_state="${topaz_plugins[*]} inpaint-video | "
rebuild=0
for repo in "$abner" "$repo_home/topaz-rs" "$repo_home/inpaint-video"; do
  revision=$(git -C "$repo" rev-parse HEAD)
  build_state+="$revision "
  if _gsi_local_changes "$repo"; then
    printf '  %s has local changes; building the working tree.\n' "${repo##*/}"
    rebuild=1
  fi
done

manifests=("$plugins"/*/Cargo.toml)
if (( ${#manifests[@]} != ${#topaz_plugins[@]} + 1 )); then
  printf '  Additional local plugins are present; checking the build again.\n'
  rebuild=1
fi

# A dirty or untracked plugin build must also be replaced once the tree is clean.
if (( rebuild )); then
  build_state="local $build_state"
fi
if [[ ${FORCE:-0} == 1 || ! -x "$launcher" || ! -x "$installed_binary" || ! -f "$stamp" ]]; then
  rebuild=1
elif ! (( rebuild )); then
  installed_hash=$(shasum -a 256 < "$installed_binary")
  if [[ $(cat "$stamp") != "$build_state$installed_hash" ]]; then
    rebuild=1
  fi
fi

if (( rebuild )); then
  command -v cargo >/dev/null 2>&1 || _gsi_die "cargo not found on PATH"
  [[ -x "$abner/packaging/build-app.sh" ]] || _gsi_die "$abner/packaging/build-app.sh not found"
  printf 'Building Abner with plugins from %s...\n' "$abner"
  ABNER_PLUGINS="$plugins" "$abner/packaging/build-app.sh" --with-plugins --install
  installed_hash=$(shasum -a 256 < "$installed_binary")
  mkdir -p -- "${stamp%/*}"
  stamp_tmp=$(mktemp "$stamp.XXXXXX")
  printf '%s%s\n' "$build_state" "$installed_hash" > "$stamp_tmp"
  mv -f -- "$stamp_tmp" "$stamp"
  printf 'Installed Abner with plugins: %s (%s)\n' "$app" "$(_gsi_version "$abner")"
else
  printf 'Abner with plugins is current: %s\n' "$app"
fi

mkdir -p -- "$INSTALL_DIR"
_gsi_app_shim "$INSTALL_DIR/abner" "$launcher"
printf '  cli shim: %s/abner\n' "$INSTALL_DIR"
