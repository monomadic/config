#!/bin/bash

# FZF jumps
# - https://github-wiki-see.page/m/junegunn/fzf/wiki/Color-schemes

MARKS_FILE="$HOME/.marks"
WORKSPACES_DIR="$HOME/workspaces"
SRC_DIR="$HOME/src"

# Helper functions
dir_exists() {
  [[ -d "$1" ]]
}

# Mark management
mark() {
  local current_dir
  current_dir="$(pwd)"

  if grep -qxF "$current_dir" "$MARKS_FILE"; then
    echo "** The following mark already exists **"
  else
    echo "$current_dir" >>"$MARKS_FILE"
    echo "** The following mark has been added **"
  fi
}

ls_marks() {
  while IFS= read -r dir; do
    [[ -d "$dir" ]] && printf "%s\n" "$dir"
  done <"$MARKS_FILE"
}

# Directory listing functions
ls_all() {
  pwd
  ls_marks
  ls_volumes
  # ls_workspaces
  # ls_src
  # ls_creators
}

ls_volumes() {
  fd --color=never -t d . /Volumes -d 1
}

ls_workspaces() {
  dir_exists "$WORKSPACES_DIR" && fd . "$WORKSPACES_DIR" --color=never --extension workspace --follow
}

ls_src() {
  dir_exists "$SRC_DIR" && eza --oneline --only-dirs --list-dirs --color=always "$SRC_DIR"/*
}

ls_projects() {
  dir_exists "$WORKSPACES_DIR" && eza --oneline --only-dirs --list-dirs --color=always "$WORKSPACES_DIR"/*.workspace/*
}

ls_recursive() {
  fd --type d --color=never --strip-cwd-prefix --max-depth 5 --exclude node_modules --exclude .git --exclude target
}

fd_depth_2() {
  fd --type d --color=never --strip-cwd-prefix --max-depth 1
  fd --type d --color=never --strip-cwd-prefix --exact-depth 2
}

ls_hidden() {
  eza --icons --group-directories-first
}

fzf_ripgrep() {
  fzf-rg
}

# Function to handle directory navigation with lsd and fzf
fzf-lsd-cd() {
  # Run lsd with desired options and pipe to fzf
  # Use awk to extract just the directory name, removing the icon and any other prefixes
  selected=$(lsd --icon always --long --ignore-config --blocks name --directory-only --color always |
    fzf --ansi --reverse |
    awk '{
                   # Skip the first field (icon) and concatenate remaining fields
                   for(i=2; i<=NF; i++) {
                       if(i==2) printf "%s", $i;
                       else printf " %s", $i
                   }
               }')

  # Check if a directory was selected
  if [ -n "$selected" ]; then
    # Check if the selected item is a directory before attempting to cd
    if [ -d "$selected" ]; then
      cd "$selected"
    else
      echo "Error: '$selected' is not a directory"
      return 1
    fi
  fi
}

fzf_cd() {
  local dir
  dir=$(fd --type directory --hidden . | fzf \
    --prompt ' ' \
    --layout=reverse \
    --preview 'eza --icons --group-directories-first --no-user --no-permissions --no-time -l --tree --level 2 {}' \
    --color=bg+:-1,fg:4,info:15,fg+:5,header:7,hl:5,hl+:5 \
    --height 50% \
    --pointer=' ' \
    --info=hidden \
    --bind 'ctrl-h:reload(eza --icons --only-dirs --all)' \
    --bind 'tab:accept' \
    --bind 'ctrl-j:jump-accept')
  [[ -n "$dir" ]] && cd "$dir" && zle reset-prompt
}

fzf_insert() {
  local files
  files=$(fd --strip-cwd-prefix --max-depth 1 | fzf \
    --prompt 'insert > ' \
    --layout=reverse \
    --preview 'eza --icons --group-directories-first {}' \
    --height 75% \
    --header $'ctrl-e:edit, ctrl-o:open\n' \
    --bind 'ctrl-e:execute(${EDITOR:-nvim} {1})' \
    --bind 'ctrl-o:execute(open {1})')
  print -z -- "$1 ${files[@]:q:q}"
  zle reset-prompt
}

# unicode:       ⌘ ⇧ ⇧ ⌥ ⇪ ⌃ ↵
# nerdfonts:     󰘶 󰘵 󰘲 󰘳 󰘴 󰌑
fzf_dirs() {
  fzf --prompt '   ' \
    --layout=reverse \
    --no-sort \
    --ansi \
    --exact \
    --ignore-case \
    --cycle \
    --header '󰌑 open  󰘴e edit  󰘴r reveal  󰘴b marks  󰘴o pwd  󰘴c cancel' \
    --info=hidden \
    --pointer=' ' \
    --margin 10% \
    --padding 3%,2% \
    --preview 'eza --icons --group-directories-first {}' \
    --bind 'ctrl-e:execute-silent(nvim {1})' \
    --bind 'ctrl-r:execute-silent(open {1})' \
    --bind 'ctrl-o:change-prompt(pwd > )+reload(fd --type d --strip-cwd-prefix --max-depth 1)' \
    --bind 'ctrl-b:change-prompt(marks > )+reload(cat ~/.marks)'
}

nvim_edit() {
  if [[ "$1" == "--help" ]]; then
    echo "Usage: $0 [file ...]"
    echo " - If arguments are given, run nvim with the arguments."
    echo " - If no arguments are given, launch fzf-neovim."
    return
  fi

  if [[ $# -gt 0 ]]; then
    nvim "$@"
  else
    fzf_neovim
  fi
}

fzf_neovim() {
  local dir
  dir=$(ls_all | fzf_dirs)
  [[ -n "$dir" ]] && cd "$dir" && ${EDITOR:-nvim}
  zle reset-prompt
}

fzf_marks() {
  local dir
  dir=$(ls_all | fzf_dirs)
  [[ -n "$dir" ]] && cd "$dir" && zle reset-prompt
}

# Uncomment these to bind functions to keys
# zle -N fzf_cd
# zle -N fzf_edit
# zle -N fzf_marks
# bindkey '^G' fzf_cd
# bindkey '^E' fzf_edit
# bindkey '^B' fzf_marks
