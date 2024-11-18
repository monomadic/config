# KEYBINDINGS
#
# bindkey '^ ' lfcd
zle -N yazi-jump && bindkey '^ ' yazi-jump
# bindkey '^ ' joshuto-wrapper;
zle -N _fzf-find-files && bindkey '^f' _fzf-find-files
zle -N _fzf-jump && bindkey '^j' _fzf-jump
zle -N fzf_ripgrep && bindkey '^s' fzf_ripgrep
zle -N clear-reset && bindkey '^k' clear-reset
#zle -N magic-enter && bindkey '^m' magic-enter

# file open
zle -N fzf_marks && bindkey '^o' fzf_marks
zle -N fzf_insert && bindkey '^y' fzf_insert

# alt+u: up directory
zle -N cd-up && bindkey '^u' cd-up
# bindkey '^e' fzf_edit
# bindkey '^b' fzm

_fzf-find-files() {
  fzf-find-files
}

_fzf-jump() {
  fzf-jump
}
