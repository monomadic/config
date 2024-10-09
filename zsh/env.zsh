export ZSH_CONFIG_DIR="$HOME/.zsh"
export DOTFILES_DIR="$HOME/config"
export ZSH_DOTFILES_DIR="$DOTFILES_DIR/zsh"

# Paths
export PATH=$PATH:$HOME/.bin:$ZSH_CONFIG_DIR/bin
export PATH=$PATH:$HOME/.local/share/nvim/mason/bin/:$HOME/.cargo/bin/:$HOME/go/bin:$HOME/workspaces/python-projects.workspace
export PATH=$PATH:$HOME/.deno/bin
export PATH=$PATH:$HOME/.foundry/bin

export TABLATURE_DIR="$HOME/Tablature"

# Media
export LOCAL_MEDIA_PATHS="$HOME/Downloads/Porn:$HOME/Media/Porn:$HOME/Movies/Porn"
export LOCAL_CACHE_PATHS="$HOME/Movies/Cache:$HOME/Media/Cache"
export LOCAL_CACHE_PATH="$HOME/Movies/Cache"
export EXTERNAL_MEDIA_PATHS="/Volumes/*/Movies/Porn"
export EXTERNAL_CACHE_PATHS="/Volumes/*/Movies/Cache"
export MASTER_MEDIA_DIR="/Volumes/FireBird1TB/Movies"
export MEDIA_INBOX_PATH="$HOME/Movies/Porn/originals/_inbox"
export INDEX_DIR="$HOME/doc/indexes"

# XDG
export XDG_CONFIG_HOME=$HOME/.config

# Homebrew
export HOMEBREW_NO_ENV_HINTS

export HOSTNAME=$(hostname)

# WASMTime
export WASMTIME_HOME=$HOME/.wasmtime
export PATH=$PATH:$WASMTIME_HOME/bin

# Set default language and character encoding
export LANG="en_US.UTF-8"
export LC_ALL="en_US.UTF-8"

export GHQ_ROOT=$HOME/src
