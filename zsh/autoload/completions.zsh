# COMPLETIONS
#
# useful links
#		- https://github.com/zsh-users/zsh-completions
#

# 1Password (dynamic completions are fine)
eval "$(op completion zsh)"

# FZF
eval "$(fzf --zsh)"

#eval "$(rmrfrs --completions zsh)" &> /dev/null

# Ensure completions directory exists
mkdir -p ~/.zsh/completions

# Generate completions once if missing
[[ ! -f ~/.zsh/completions/_fd ]] && fd --gen-completions=zsh >~/.zsh/completions/_fd
[[ ! -f ~/.zsh/completions/_rg ]] && rg --generate complete-zsh >~/.zsh/completions/_rg
[[ ! -f ~/.zsh/completions/_dotter ]] && dotter gen-completions --shell zsh >~/.zsh/completions/_dotter
[[ ! -f ~/.zsh/completions/_bat ]] && bat --completion zsh >~/.zsh/completions/_bat

# Load completions
fpath+=(~/.zsh/completions)
autoload -Uz compinit && compinit

# --- command: e <file> -> open in default editor ---
e() {
  local editor=${EDITOR:-${VISUAL:-vi}}
  
  if [[ $# -eq 0 ]]; then
    local file=$(fd --type f --max-depth 4 | \
      fzf --preview 'bat --style=numbers --color=always {}' \
          --preview-window 'right:60%:wrap')
    [[ -n "$file" ]] && "$editor" "$file"
  else
    "$editor" "$@"
  fi
}

_edit-script() {
  local -a names descs dirs
  local file header base

  dirs=(
    "$HOME/config/bin"
    "$HOME/config/zsh/bin"
  )

  for dir in $dirs; do
    [[ -d $dir ]] || continue

    # regular files, nullglob, no error if none
    for file in $dir/*(N-); do
      [[ -r $file ]] || continue

      # Shebang filter => treat as "script"
      if IFS= read -r header <"$file"; then
        [[ $header == '#!'* ]] || continue
      else
        continue
      fi

      base=${file:t}

      names+="$base"
      descs+="$file"
    done
  done

  (( ${#names} )) || return 1

  # Show: name  —  /full/path
  compadd -d descs -- $names
}
compdef _edit-script edit-script
