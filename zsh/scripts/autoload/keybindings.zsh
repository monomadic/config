# KEYBINDINGS
#
bindkey '^ ' lfcd
#bindkey '^ ' yy;
# bindkey '^ ' joshuto-wrapper;
bindkey '^f' fzf-ripgrep
zle -N clear-reset && bindkey '^k' clear-reset
#zle -N magic-enter && bindkey '^m' magic-enter

# file open
zle -N fzf-marks && bindkey '^o' fzf-marks
zle -N fzf-marks && bindkey '^[o' fzf-marks

zle -N fzf-insert && bindkey '^y' fzf-insert

# alt+u: up directory
zle -N cd-up && bindkey '^[u' cd-up

# bindkey '^e' fzf_edit
# bindkey '^b' fzm
zle -N fzf-cd && bindkey '^j' fzf-cd
