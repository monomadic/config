export ZSH_CONFIG_DIR="$HOME/.zsh"
export DOTFILES_DIR="$HOME/config"
export ZSH_DOTFILES_DIR="$DOTFILES_DIR/zsh"

export EDITOR=nvim

export TABLATURE_DIR="$HOME/Music/Tablature"
export MOVIES_PATH="$HOME/Movies"
export TUTORIALS_PATH="$HOME/Movies/Tutorials"

# Media
export LOCAL_MEDIA_PATHS="$HOME/Downloads/Porn:$HOME/Media/Porn:$MOVIES_PATH/Porn"
export LOCAL_CACHE_PATHS="$MOVIES_PATH/Cache:$HOME/Media/Cache"
export LOCAL_CACHE_PATH="$MOVIES_PATH/Cache"
export EXTERNAL_MEDIA_PATHS="/Volumes/*/Movies/Porn"
export EXTERNAL_CACHE_PATHS="/Volumes/*/Movies/Cache"
export MASTER_MEDIA_DIR="/Volumes/Media-MSTR/Movies"
export MEDIA_INBOX_PATH="$HOME/Movies/Porn/originals/_inbox"
export INDEX_DIR="$HOME/doc/indexes"
export PRIVATE_PHOTOS_LIBRARY="$HOME/Media/Private/Private.photoslibrary"

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

# FZF / Skim
export FZF_DEFAULT_OPTS="--layout=reverse --border=none --cycle --inline-info --color=bg+:-1,bg:-1,fg:4,info:15,fg+:5,header:7,hl:5,hl+:5,border:-1"
export SKIM_DEFAULT_OPTIONS=$FZF_DEFAULT_OPTS
export SKIM_DEFAULT_COMMAND="fd . --max-depth=3"
