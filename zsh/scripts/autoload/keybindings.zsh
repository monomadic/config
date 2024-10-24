# KEYBINDINGS
#
bindkey '^ ' lfcd
#bindkey '^ ' yy;
# bindkey '^ ' joshuto-wrapper;
zle -N fzf_ripgrep && bindkey '^f' fzf_ripgrep
zle -N clear-reset && bindkey '^k' clear-reset
#zle -N magic-enter && bindkey '^m' magic-enter

# file open
zle -N fzf_marks && bindkey '^o' fzf_marks
zle -N fzf_insert && bindkey '^y' fzf_insert

# alt+u: up directory
zle -N cd-up && bindkey '^u' cd-up
# bindkey '^e' fzf_edit
# bindkey '^b' fzm
zle -N fzf-cd && bindkey '^j' fzf-cd
