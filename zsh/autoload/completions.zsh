# COMPLETIONS

# Base completion system initialization
autoload -Uz compinit && compinit

# Directory navigation options
setopt autocd
setopt autopushd

# Function autoloading
autoload -Uz add-zsh-hook

# Completion styles
#
#  navigate completion suggestions using arrow keys
zstyle ':completion:*' menu select
#
#  zstyle ':completion:*' format '%B%d%b'
zstyle ':completion:*' special-dirs true
zstyle ':completion:*' list-dirs first
zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}'

# 1password
eval "$(op completion zsh)"

# fd
eval "$(fd --gen-completions zsh)"

# dotter
eval "$(dotter gen-completions --shell zsh)"
