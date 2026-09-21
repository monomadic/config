# Is an installed src/ tool out of date? Shared by bin/dotter-deploy
# (`deploy.sh --upgrade`) and bin/fzf-app-store, so both give the same answer
# and share one record of what was last installed.
#
# Source with $DOTFILES_DIR set to the repo root.
#
# Two ways to decide, in order:
#
#   1. A content hash of everything the install was built from (the installer,
#      the vendored binary, the src/ build inputs), recorded after every
#      successful install in $INSTALL_STATE_DIR. Exact: it changes when and
#      only when the inputs do.
#   2. With no record — a tool installed by hand, or before this existed —
#      compare the artifact's mtime against when each input last changed.
#      A file mtime is trusted only for uncommitted edits; otherwise git says
#      when the content last changed. Mtimes alone lie: the flatten-repo
#      refactor moved the installers and src/ trees, which reset every mtime
#      and made every installed tool look stale.

INSTALL_STATE_DIR="${XDG_STATE_HOME:-$HOME/.local/state}/fzf-app-store"

# Only real build inputs count. A README or a design mockup is not one, and
# treating it as one means every doc edit triggers a rebuild. Pruned dirs:
# build output, and the nested checkouts that agent worktrees leave under
# .claude/.
_install_build_inputs() {
  local dir="$1"; shift
  find "$dir" \
    \( -name target -o -name .git -o -name .claude -o -name designs -o -name design \) -prune -o \
    -type f \( -name '*.rs' -o -name '*.go' -o -name '*.c' -o -name '*.h' -o -name '*.m' \
      -o -name '*.swift' -o -name Cargo.toml -o -name Cargo.lock -o -name go.mod -o -name go.sum \) \
    "$@" -print 2>/dev/null
}

# Installer file -> tool name: install-foo.sh, foo.sh and install-foo all
# become "foo". Also the key the install record is stored under.
install_entry_name() {
  local n="${1:t}"
  n="${n#install-}"
  print -r -- "${n%.sh}"
}

# Did file $1 really change after epoch $2? The rename limit is raised because
# the refactor commit is too large for git's default, and without it every
# moved file reads as newly added.
install_changed_since() {
  local f="$1" since="$2" rel="${1#$DOTFILES_DIR/}" epoch
  (( $(stat -f %m "$f") > since )) || return 1
  [[ -n "$(git -C "$DOTFILES_DIR" status --porcelain -- "$rel" 2>/dev/null)" ]] && return 0
  epoch="$(git -C "$DOTFILES_DIR" -c diff.renameLimit=100000 \
    log -1 --follow --diff-filter=AM --format=%ct -- "$rel" 2>/dev/null)"
  (( ${epoch:-0} > since ))
}

# Contents plus paths relative to the tree, never absolute ones: the hash must
# not change with how a path was spelled or where the checkout lives.
install_fingerprint() {
  local installer="$1" src="$2" vendor="$3"
  {
    shasum <"$installer"
    [[ -f "$vendor" ]] && shasum <"$vendor"
    [[ -d "$src" ]] && ( cd "$src" && _install_build_inputs . | LC_ALL=C sort | tr '\n' '\0' | xargs -0 shasum )
    :
  } 2>/dev/null | shasum | cut -d' ' -f1
}

install_record() {
  local installer="$1" src="$2" vendor="$3"
  mkdir -p "$INSTALL_STATE_DIR"
  install_fingerprint "$installer" "$src" "$vendor" \
    >"$INSTALL_STATE_DIR/$(install_entry_name "$installer")"
}

# Prints why the install at $4 is stale and returns 0, or returns 1 if it is
# current. $2 is the src/ tree and $3 the vendor/bin binary; either may not
# exist.
install_stale_reason() {
  local installer="$1" src="$2" vendor="$3" artifact="$4"
  local record="$INSTALL_STATE_DIR/$(install_entry_name "$installer")" since f

  if [[ -f "$record" ]]; then
    [[ "$(<"$record")" == "$(install_fingerprint "$installer" "$src" "$vendor")" ]] && return 1
    print -r -- "source changed since the last install"
    return 0
  fi

  since="$(stat -f %m "$artifact")"
  if install_changed_since "$installer" "$since"; then
    print -r -- "${installer:t} changed since the install"
    return 0
  fi
  if [[ -f "$vendor" ]] && install_changed_since "$vendor" "$since"; then
    print -r -- "${vendor#$DOTFILES_DIR/} changed since the install"
    return 0
  fi
  if [[ -d "$src" ]]; then
    for f in "${(@f)$(_install_build_inputs "$src" -newermt "@$since")}"; do
      [[ -n "$f" ]] && install_changed_since "$f" "$since" || continue
      print -r -- "${f#$DOTFILES_DIR/} changed since the install"
      return 0
    done
  fi
  return 1
}
