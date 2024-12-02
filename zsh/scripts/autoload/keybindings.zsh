# KEYBINDINGS
#
# bindkey '^ ' lfcd
zle -N yazi-jump && bindkey '^ ' yazi-jump
zle -N _fzf-find-files && bindkey '^f' _fzf-find-files

# F20 -> fzf-jump
zle -N _fzf-jump && bindkey '^j' _fzf-jump && bindkey '^[[57373u' _fzf-jump

zle -N fzf_ripgrep && bindkey '^s' fzf_ripgrep
zle -N clear-reset && bindkey '^k' clear-reset
zle -N magic-enter && bindkey '^l' magic-enter

# file open
#zle -N fzf_marks && bindkey '^o' fzf_marks

_fzf-cd() {
  # Capture selected directory from fzf
  local selected_dir=$(ls_all | source fzf-cd)
  # If a directory is selected, execute cd in the parent shell
  if [[ -n "$selected_dir" && -d "$selected_dir" ]]; then
    cd "$selected_dir"
    zle reset-prompt
  fi
}
zle -N _fzf-cd
bindkey '^o' _fzf-cd

# insert a file path into the current buffer
zle -N fzf_insert && bindkey '^y' fzf_insert

# alt+u: up directory
zle -N cd-up && bindkey '^u' cd-up
# bindkey '^e' fzf_edit
# bindkey '^b' fzm

_fzf-find-files() {
  fzf-find-files
}

_fzf-jump() {
  source fzf-jump-subshell && zle reset-prompt
}
