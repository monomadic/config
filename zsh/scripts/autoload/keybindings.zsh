# KEYBINDINGS
#
bindkey '^ ' lfcd
#bindkey '^ ' yy;
# bindkey '^ ' joshuto-wrapper;
bindkey '^f' fzf-ripgrep
bindkey '^k' clear-reset
bindkey '^m' magic-enter
bindkey '^o' fzf-marks
bindkey '^y' fzf-insert
bindkey '^[u' cd-up
# bindkey '^e' fzf_edit
# bindkey '^b' fzm
zle -N fzf-cd && bindkey '^j' fzf-cd
