#!/bin/bash

# FZF jumps
# - https://github-wiki-see.page/m/junegunn/fzf/wiki/Color-schemes

MARKS_FILE="$HOME/.marks"
WORKSPACES_DIR="$HOME/workspaces"
SRC_DIR="$HOME/src"

# Helper functions
dir-exists() {
  [[ -d "$1" ]]
}

# Mark management
mark() {
  local mark_to_add
  mark_to_add="$(pwd)"
  if grep -qxFe "${mark_to_add}" "${MARKS_FILE}"; then
    echo "** The following mark already exists **"
  else
    echo "${mark_to_add}" >>"${MARKS_FILE}"
    echo "** The following mark has been added **"
  fi
  color-marks <<<"$mark_to_add"
}

ls-marks() {
  while IFS= read -r dir; do
    [[ -d "$dir" ]] && echo "$dir"
  done <"$MARKS_FILE"
}

# Directory listing functions
ls-all() {
  ls-marks
  ls-workspaces
  ls-src
}

ls-workspaces() {
  dir-exists "$WORKSPACES_DIR" && fd . "$WORKSPACES_DIR" --extension workspace --follow
}

ls-src() {
  dir-exists "$SRC_DIR" && exa "$SRC_DIR"/* --oneline --only-dirs --list-dirs
}

ls-projects() {
  dir-exists "$WORKSPACES_DIR" && exa "$WORKSPACES_DIR"/*.workspace/* --oneline --only-dirs --list-dirs
}

ls-recursive() {
  fd --type d --strip-cwd-prefix --max-depth 5 --max-results 10000 --exclude node_modules --exclude .git --exclude target
}

fd-depth-2() {
  fd --type d --strip-cwd-prefix --max-depth 1
  fd --type d --strip-cwd-prefix --exact-depth 2 --max-results 10000
}

ls-hidden() {
  exa --icons --group-directories-first
}

fzf-ripgrep() {
  fzf-rg
}

fzf-cd() {
  local dir
  dir=$(fd --type=directory --hidden . | fzf \
    --prompt ' ' \
    --layout=reverse \
    --preview 'exa --icons --group-directories-first --no-user --no-permissions --no-time -l --tree --level 2 {}' \
    --color=bg+:-1,fg:4,info:15,fg+:5,header:7,hl:5,hl+:5 \
    --height 50% \
    --pointer=' ' \
    --info=hidden \
    --bind 'ctrl-h:reload(exa --icons --only-dirs --all)' \
    --bind 'tab:accept' \
    --bind 'ctrl-j:jump-accept' \
    "$@")
  [[ -n "$dir" ]] && cd "$dir" && zle && zle reset-prompt
}

fzf-insert() {
  local files
  files=$(fd --strip-cwd-prefix --max-depth 1 --max-results 10000 | fzf \
    --prompt 'insert > ' \
    --layout=reverse \
    --preview 'exa --icons --group-directories-first {}' \
    --height 75% \
    --header $'ctrl-e:edit, ctrl-o:open\n' \
    --bind 'ctrl-e:execute:${EDITOR:-nvim} {1}' \
    --bind 'ctrl-o:execute:open {1}' \
    "$@")
  print -z -- "$1 ${files[@]:q:q}"
  zle && zle reset-prompt
}

# unicode:		⌘ ⇧ ⇧ ⌥ ⇪ ⌃ ↵
# nerdfonts:	󰘶 󰘵 󰘲 󰘳 󰘴 󰌑
fzf-dirs() {
  fzf --prompt '   ' \
    --layout=reverse \
    --exact \
    --border \
    --color=fg:4,info:#66d9ef,hl:#FFe22e,hl+:#FFe22e,fg+:5,header:7,prompt:#FFFFFF,border:#000000,bg+:#000000,bg:#000000 \
    --header '󰌑 open  󰘴r reveal  󰘴b marks  󰘴o pwd  󰘴c cancel' \
    --info=hidden \
    --pointer=' ' \
    --margin 10% \
    --padding 3%,2% \
    --preview 'preview-dir {}' \
    --bind 'ctrl-r:execute-silent(open {1})' \
    --bind 'ctrl-o:change-prompt(pwd > )+reload(fd --type d --strip-cwd-prefix --max-depth 1 && fd --type d --strip-cwd-prefix --max-results 10000)' \
    --bind 'ctrl-b:change-prompt(marks > )+reload(cat ~/.marks)' \
    "$@"
}

fzf-edit() {
  local dir
  dir=$(ls-all | fzf-dirs)
  [[ -n "$dir" ]] && cd "$dir" && ${EDITOR:-nvim}
  zle && zle reset-prompt
}

fzf-marks() {
  magic-enter
  local dir
  dir=$(ls-marks | fzf-dirs)
  [[ -n "$dir" ]] && cd "$dir"
  zle && zle reset-prompt
}

# Bind functions to keys (if needed)
# zle -N fzf-cd
# zle -N fzf-edit
# zle -N fzf-marks
# bindkey '^G' fzf-cd
# bindkey '^E' fzf-edit
# bindkey '^B' fzf-marks
