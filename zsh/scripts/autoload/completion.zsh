# COMPLETIONS
#
autoload -Uz compinit
compinit

# use fzf for shell completion (this should autoload?)
source $ZSH_CONFIG_DIR/completions/fzf-zsh.sh
source $ZSH_CONFIG_DIR/completions/things.zsh

# 1password
eval "$(op completion zsh)"
compdef _op op

source <(fd --gen-completions)

# dotter
# eval "$(dotter gen-completions --shell zsh)"
# compdef _dotter dotter
