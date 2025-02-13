# STARSHIP

eval "$(starship init zsh)"
eval "$(starship completions zsh)"

function starship_zle-keymap-select {
  if [[ $KEYMAP == vicmd ]]; then
    export STARSHIP_VI_MODE="NORMAL"
  else
    export STARSHIP_VI_MODE="INSERT"
  fi
  zle reset-prompt
}

zle -N zle-keymap-select starship_zle-keymap-select

function starship_precmd_user_func {
  export STARSHIP_VI_MODE="INSERT"
}
