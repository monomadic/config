# COMPLETIONS

# Base completion system initialization
autoload -Uz compinit && compinit

# Directory navigation options
setopt autocd
setopt autopushd

# Function autoloading
autoload -Uz add-zsh-hook

# Completion styles
zstyle ':completion:*' menu select
zstyle ':completion:*' special-dirs true
zstyle ':completion:*' list-dirs first
zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}'

# Tool-specific completions
# 1password - keep using eval as it's their recommended method
eval "$(op completion zsh)"

# fd - instead of source, use eval
eval "$(fd --gen-completions zsh)"

# dotter - keep using eval as it's their recommended method
eval "$(dotter gen-completions --shell zsh)"
