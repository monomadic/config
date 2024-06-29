# COMPLETIONS
#
autoload -Uz compinit
compinit

# use fzf for shell completion
source $ZSH_CONFIG_DIR/autoload/fzf-zsh-completion.sh

# 1password
eval "$(op completion zsh)"
compdef _op op

source <(fd --gen-completions)

# dotter
# eval "$(dotter gen-completions --shell zsh)"
# compdef _dotter dotter
