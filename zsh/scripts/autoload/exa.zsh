function exa-ls {
  clear
  exa --icons --group-directories-first
  echo
  zle && zle reset-prompt
}
