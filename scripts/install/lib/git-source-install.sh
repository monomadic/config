#!/usr/bin/env bash
# Shared driver for installers that build from an upstream git checkout rather
# than from this repo's own src/ tree.
#
# src/ holds source that lives *in* this repo. These tools don't — they are
# separate repositories that happen to be mine. So the checkout belongs in
# $SRC_PATH (default ~/src, exported by config/zsh/zshenv.zsh), and the
# installer's job is to keep that checkout current and the installed binary in
# step with it.
#
# Re-running an installer is the normal way to use it: the first run clones and
# builds, and every run after that fetches, fast-forwards if upstream has moved,
# and rebuilds only when something actually changed.
#
# Usage:
#   . "$(dirname "$0")/lib/git-source-install.sh"
#   git_source_install <name> <repo-url> <cargo|go> [binary-name]

set -euo pipefail

SRC_PATH="${SRC_PATH:-$HOME/src}"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

_gsi_warn() { printf 'Warning: %s\n' "$*" >&2; }
_gsi_die() { printf 'Error: %s\n' "$*" >&2; exit 1; }

# Is the working tree dirty? Refuse to fast-forward over local edits.
_gsi_dirty() {
  local src="$1"
  ! git -C "$src" diff --quiet --ignore-submodules HEAD 2>/dev/null
}

# Sync the checkout to upstream. Echoes "1" when the tree moved, "0" otherwise.
# Never destructive: a dirty tree or a local commit upstream doesn't have means
# we leave it exactly as it is and say so.
_gsi_sync() {
  local src="$1" moved=0 branch upstream local_rev remote_rev

  if ! git -C "$src" fetch --quiet --prune origin 2>/dev/null; then
    _gsi_warn "could not fetch origin for ${src##*/}; using the checkout as-is"
    echo 0; return
  fi

  branch="$(git -C "$src" rev-parse --abbrev-ref HEAD 2>/dev/null || echo HEAD)"
  if [ "$branch" = "HEAD" ]; then
    _gsi_warn "${src##*/} is in detached HEAD; not updating"
    echo 0; return
  fi

  # Prefer the configured upstream; fall back to origin/<branch>.
  upstream="$(git -C "$src" rev-parse --abbrev-ref --symbolic-full-name '@{upstream}' 2>/dev/null \
              || echo "origin/$branch")"
  local_rev="$(git -C "$src" rev-parse HEAD)"
  remote_rev="$(git -C "$src" rev-parse "$upstream" 2>/dev/null || true)"

  if [ -z "$remote_rev" ]; then
    _gsi_warn "${src##*/} has no upstream branch ($upstream); not updating"
  elif [ "$local_rev" = "$remote_rev" ]; then
    printf '  %s is at %s (up to date)\n' "${src##*/}" "$(echo "$local_rev" | cut -c1-8)" >&2
  elif git -C "$src" merge-base --is-ancestor "$local_rev" "$remote_rev"; then
    # Upstream is strictly ahead — the case this whole script exists for.
    if _gsi_dirty "$src"; then
      _gsi_warn "${src##*/} has uncommitted changes; not fast-forwarding to $upstream"
    else
      printf '  %s: %s -> %s\n' "${src##*/}" \
        "$(echo "$local_rev" | cut -c1-8)" "$(echo "$remote_rev" | cut -c1-8)" >&2
      git -C "$src" merge --ff-only --quiet "$upstream"
      moved=1
    fi
  else
    # Local commits upstream doesn't have, or a diverged history. Not ours to
    # resolve; building what is checked out is still the right thing.
    _gsi_warn "${src##*/} is ahead of or diverged from $upstream; leaving the checkout alone"
  fi

  echo "$moved"
}

# The installed binary is stale if it predates the commit that is checked out.
# Catches the case where the checkout was updated by hand and the installer is
# only now being run.
_gsi_binary_is_stale() {
  local src="$1" dest="$2" commit_epoch binary_epoch

  [ -x "$dest" ] || return 0

  commit_epoch="$(git -C "$src" log -1 --format=%ct 2>/dev/null || echo 0)"
  binary_epoch="$(stat -f %m "$dest" 2>/dev/null || stat -c %Y "$dest" 2>/dev/null || echo 0)"

  [ "$commit_epoch" -gt "$binary_epoch" ]
}

_gsi_build() {
  local kind="$1" src="$2" dest="$3" binname="$4"

  case "$kind" in
    cargo)
      command -v cargo >/dev/null 2>&1 || _gsi_die "cargo not found on PATH"
      ( cd "$src" && cargo build --release )
      install -m 755 "$src/target/release/$binname" "$dest"
      ;;
    go)
      command -v go >/dev/null 2>&1 || _gsi_die "go toolchain not found on PATH"
      ( cd "$src" && go build -ldflags='-s -w' -o "$dest" . )
      ;;
    *)
      _gsi_die "unknown build kind: $kind"
      ;;
  esac
}

git_source_install() {
  local name="$1" url="$2" kind="$3" binname="${4:-$1}"
  local src="$SRC_PATH/$name"
  local dest="$INSTALL_DIR/$binname"
  local rebuild=0

  mkdir -p "$SRC_PATH" "$INSTALL_DIR"

  if [ ! -e "$src" ]; then
    echo "Cloning $url -> $src"
    git clone --quiet "$url" "$src"
    rebuild=1
  elif [ ! -d "$src/.git" ]; then
    _gsi_die "$src exists but is not a git checkout; move it aside and re-run"
  else
    [ "$(_gsi_sync "$src")" = "1" ] && rebuild=1
  fi

  # Every rebuild says why — "up to date" followed by a silent compile reads
  # like a bug.
  if [ ! -e "$dest" ]; then
    echo "  $binname is not installed at $dest"
    rebuild=1
  elif [ ! -x "$dest" ]; then
    echo "  $dest exists but is not executable"
    rebuild=1
  elif _gsi_binary_is_stale "$src" "$dest"; then
    echo "  $binname is older than the checked-out commit"
    rebuild=1
  fi

  if [ "${FORCE:-0}" = "1" ] && [ "$rebuild" = "0" ]; then
    echo "  FORCE=1; rebuilding regardless"
    rebuild=1
  fi

  if [ "$rebuild" = "0" ]; then
    echo "$binname is current: $dest"
    return 0
  fi

  echo "Building $name from $src..."
  _gsi_build "$kind" "$src" "$dest" "$binname"
  echo "installed: $dest ($(git -C "$src" rev-parse --short HEAD))"
}
