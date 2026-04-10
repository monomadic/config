#!/bin/zsh

_kitty_remote() {
  if [[ -n ${KITTY_LISTEN_ON:-} ]]; then
    kitty @ "$@"
  else
    kitty @ --to "unix:/tmp/kitty-$USER" "$@"
  fi
}

_kitty_regex_escape() {
  print -rn -- "$1" | sed -e 's/[][(){}.^$*+?|\\/]/\\&/g'
}

_kitty_shell_label() {
  emulate -L zsh

  local git_root rel label
  if git_root="$(git rev-parse --show-toplevel 2>/dev/null)"; then
    if [[ "$PWD" == "$git_root" ]]; then
      label="${git_root:t}"
    else
      rel="${PWD#$git_root/}"
      label="${git_root:t}:${rel##*/}"
    fi
  elif [[ "$PWD" == "$HOME" ]]; then
    label='~'
  else
    label="${PWD:t}"
  fi

  print -r -- "$label"
}

_kitty_sync_shell_title() {
  [[ -o interactive ]] || return 0
  [[ -n ${KITTY_WINDOW_ID:-} || -n ${KITTY_LISTEN_ON:-} ]] || return 0

  local label="$(_kitty_shell_label)"
  [[ -n "$label" ]] || return 0
  [[ "$label" == "${_KITTY_LAST_SHELL_LABEL:-}" ]] && return 0

  typeset -g _KITTY_LAST_SHELL_LABEL="$label"
  _kitty_remote set-window-title --temporary "$label" >/dev/null 2>&1 || true
  _kitty_remote set-tab-title "$label" >/dev/null 2>&1 || true
}

edit-kitty-config() {
  local name="  kitty.conf"
  local pattern="$(_kitty_regex_escape "$name")"

  if _kitty_remote focus-tab --match "title:^${pattern}$" >/dev/null 2>&1; then
    return 0
  fi

  _kitty_remote launch --type=tab --cwd "$DOTFILES_DIR/config/kitty" \
    kitty-exec "$name" "#A442F3" hx kitty.conf
}

switch_or_launch_tab() {
  local name="$1"
  local pattern="$(_kitty_regex_escape "$name")"

  if ! _kitty_remote focus-tab --match "title:${pattern}" >/dev/null 2>&1; then
    _kitty_remote launch --type=tab --tab-title "$name" zsh
  fi
}

switch_or_launch_tab_exact() {
  local name="$1"
  local pattern="$(_kitty_regex_escape "$name")"

  if ! _kitty_remote focus-tab --match "title:^${pattern}$" >/dev/null 2>&1; then
    _kitty_remote launch --type=tab --tab-title "$name" zsh
  fi
}

if [[ -o interactive ]]; then
  add-zsh-hook chpwd _kitty_sync_shell_title
  _kitty_sync_shell_title
fi
