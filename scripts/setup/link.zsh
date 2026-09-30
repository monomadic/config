#!/bin/zsh -f
# Link the checkout using only zsh and macOS utilities.
# Failures are counted, but never stop the remaining mappings.
ROOT="${${0:A}:h:h:h}"
integer problems=0 linked=0 unchanged=0 pending=0
mode="${1:-}"
green='' cyan='' yellow='' reset='' red='' error_reset=''
if [[ -z "${NO_COLOR:-}" && "${TERM:-}" != dumb ]]; then
  if [[ -t 1 ]]; then
    green=$'\e[32m' cyan=$'\e[36m' yellow=$'\e[33m' reset=$'\e[0m'
  fi
  if [[ -t 2 ]]; then
    red=$'\e[31m' error_reset=$'\e[0m'
  fi
fi
case "$mode" in
  ''|--dry-run|--check) ;;
  *) print -ru2 -- "${red}Usage: link.zsh [--dry-run|--check]${error_reset}"; exit 2 ;;
esac

problem() {
  print -ru2 -- "${red}SKIP:${error_reset} $*"
  (( ++problems ))
  return 0
}

# A dangling symlink at the target, or at any directory above it, points at
# nothing: it blocks mkdir/ln but holds no data, so it is replaced, not kept.
# Sets REPLY to the dead link.
typeset -A dead_seen
dangling_symlink() {
  local p="$1"
  while [[ "$p" != / && "$p" != . ]]; do
    if [[ -L "$p" && ! -e "$p" ]]; then
      REPLY="$p"
      return 0
    fi
    p="${p:h}"
  done
  return 1
}

link_file() {
  local src="$ROOT/$1" dst="$2" dead='' REPLY
  if [[ ! -e "$src" ]]; then
    problem "missing source: $1"
    return 0
  elif [[ "$mode" == --check ]]; then
    return 0
  elif [[ "$src" -ef "$dst" ]]; then
    (( ++unchanged ))
    return 0
  fi
  dangling_symlink "$dst" && dead="$REPLY"
  if [[ -z "$dead" && ( -e "$dst" || -L "$dst" ) ]]; then
    problem "$dst already exists; move it aside to link $1"
  elif [[ "$mode" == --dry-run ]]; then
    (( ++pending ))
    if [[ -n "$dead" && -z "${dead_seen[$dead]:-}" ]]; then
      dead_seen[$dead]=1
      print -r -- "${cyan}WOULD REMOVE:${reset} dangling symlink $dead -> $(readlink -- "$dead")"
    fi
    print -r -- "${cyan}WOULD LINK:${reset} $dst -> $src"
  elif [[ -n "$dead" ]] && ! { print -r -- "${yellow}REMOVE:${reset} dangling symlink $dead -> $(readlink -- "$dead")"; rm -- "$dead" }; then
    problem "could not remove dangling symlink $dead"
  elif mkdir -p -- "${dst:h}" && ln -s -- "$src" "$dst"; then
    (( ++linked ))
    print -r -- "${green}LINK:${reset} $dst -> $src"
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
/bin/zsh -fn "$ROOT/Deployfile" || exit 1
source "$ROOT/Deployfile" || problem 'could not finish reading Deployfile'
summary_color="$green"
[[ "$mode" == --dry-run ]] && summary_color="$cyan"
(( problems )) && summary_color="$yellow"
if [[ "$mode" == --dry-run ]]; then
  print -r -- "${summary_color}Dry run: $pending links to create, $unchanged already correct, $problems problems.${reset}"
else
  print -r -- "${summary_color}Links: $linked created, $unchanged unchanged, $problems problems${mode:+ ($mode)}.${reset}"
fi
(( problems == 0 ))
