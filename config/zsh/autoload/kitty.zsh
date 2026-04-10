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
