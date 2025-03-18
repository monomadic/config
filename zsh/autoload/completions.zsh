# COMPLETIONS
#
# useful links
#		- https://github.com/zsh-users/zsh-completions
#

# 1Password (dynamic completions are fine)
eval "$(op completion zsh)"

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
