# COMPLETIONS
#
autoload -Uz compinit; compinit

# 1password
eval "$(op completion zsh)"
compdef _op op

# dotter
# eval "$(dotter gen-completions --shell zsh)"
# compdef _dotter dotter