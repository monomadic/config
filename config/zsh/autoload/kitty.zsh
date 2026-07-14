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

# ============================================================================
# Kitty aliases (re-homed from alias.zsh)
# ============================================================================
alias kitty-cwd-opposite-pane="kitty @ ls | jq -r '  .[] | select(.is_active)  | .tabs[] | select(.is_active)  | .windows[] | select(.is_active | not)  | .foreground_processes[-1].cwd // .cwd'"
alias .kitty-cwd-opposite-pane=kitty-cwd-opposite-pane
# ============================================================================
# Kitty Terminal
# ============================================================================

alias .kitty-mark-current-tab-orange="kitty @ set-tab-color active_bg=orange active_fg=white inactive_bg=orange inactive_fg=black"
alias .kitty-mark-current-tab-red="kitty @ set-tab-color inactive_bg=red inactive_fg=black"
alias .kitty-set-tab-color-orange="kitty @ set-tab-color --match id:$KITTY_WINDOW_ID active_bg=#FFA500 active_fg=#050F63 inactive_fg=#FFA500 inactive_bg=#030D43"
alias .kitty-set-tab-color-green="kitty @ set-tab-color --match id:$KITTY_WINDOW_ID active_bg=#38F273 active_fg=#050F63 inactive_fg=#38F273 inactive_bg=#030D43"
alias .kitty-reload="kitty @ set-colors --all ~/.config/kitty/kitty.conf"
alias .kitty-configure="e-kitty"
alias .kitty-kill-all-editor="kitten @ close-tab --match 'env:PROC=hx'"
alias .kitty-kill-all-nvim=.kitty-kill-all-editor
alias .nvim-kill-all=.kitty-kill-all-editor
alias .kitty-close-idle-tabs="kitty @ close-tab --match 'env:PROC=zsh'"
alias .fonts="kitty list-fonts"
