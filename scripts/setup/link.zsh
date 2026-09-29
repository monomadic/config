#!/bin/zsh -f
# Link the checkout using only zsh and macOS utilities.
# Failures are counted, but never stop the remaining mappings.
ROOT="${${0:A}:h:h:h}"
integer problems=0 linked=0 unchanged=0
mode="${1:-}"
case "$mode" in
  ''|--dry-run|--check) ;;
  *) print -u2 'Usage: link.zsh [--dry-run|--check]'; exit 2 ;;
esac

problem() {
  print -u2 -- "SKIP: $*"
  (( ++problems ))
  return 0
}

link_file() {
  local src="$ROOT/$1" dst="$2"
  if [[ ! -e "$src" ]]; then
    problem "missing source: $1"
  elif [[ "$mode" == --check ]]; then
    return 0
  elif [[ "$src" -ef "$dst" ]]; then
    (( ++unchanged ))
  elif [[ -e "$dst" || -L "$dst" ]]; then
    problem "$dst already exists; move it aside to link $1"
  elif [[ "$mode" == --dry-run ]]; then
    print -r -- "LINK: $dst -> $src"
  elif mkdir -p -- "${dst:h}" && ln -s -- "$src" "$dst"; then
    (( ++linked ))
  else
    problem "could not link $dst"
  fi
  return 0
}

link_tree() {
  local src="$1" dst="$2" file
  if [[ ! -d "$ROOT/$src" ]]; then
    problem "missing directory: $src"
    return 0
  fi
  for file in "$ROOT/$src"/**/*(ND.,@); do
    [[ "${file:t}" == .DS_Store || "$file" == */__pycache__/* ]] && continue
    link_file "${file#$ROOT/}" "$dst/${file#$ROOT/$src/}"
  done
  return 0
}

# Check syntax before sourcing: a typo must not execute half a shell statement.
/bin/zsh -fn "$ROOT/Linkfile" || exit 1
source "$ROOT/Linkfile" || problem 'could not finish reading Linkfile'
print -- "Links: $linked created, $unchanged unchanged, $problems problems${mode:+ ($mode)}."
(( problems == 0 ))
