# Configuration that runs for all shells (even non-interactive)
#

# PATH
#
# note: in zsh, $path is an associative array that syncs to $PATH
typeset -U path
path=(
  $HOME/.bin
  $HOME/.local/bin
  $HOME/.zsh/bin
  $HOME/.cargo/bin
  $HOME/.deno/bin
  $HOME/.foundry/bin
  $HOME/.local/share/nvim/mason/bin
  $HOME/go/bin
  $HOME/.cache/lm-studio/bin
  $path
)

# MANPATH CONFIGURATION
#
typeset -U manpath
manpath=(
  $manpath
)

JUMP_DIRS=(
  "/Volumes/**/Movies/**/*.mp4"
  "$HOME/Movies/**/*.mp4"
)

MEDIA_GLOBS=(
  "/Volumes/*/Movies/Porn/**/*.mp4"
  "$HOME/Movies/Porn/**/*.mp4"
)

TABLATURE_GLOBS=(
  "/Volumes/*/Tablature/**/*.pdf"
  "$HOME/Music/Tablature/**/*.pdf"
)

export ZSH_CONFIG_DIR=$HOME/.zsh
export ZSH_COMPLETIONS_DIR=$ZSH_CONFIG_DIR/completions
export ZSH_AUTOLOAD_DIR=$ZSH_CONFIG_DIR/autoload
export CONFIG_DIR=$HOME/.config
export XDG_CONFIG_HOME=$CONFIG_DIR
export DOTFILES_DIR=$HOME/config
export ZSH_DOTFILES_DIR=$DOTFILES_DIR/zsh

export ZSH_SCRIPT_PATHS=(
  $ZSH_CONFIG_DIR/bin
)

export EDITOR=nvim
export TEMPLATE_BASE_DIR=$XDG_CONFIG_HOME/nvim/templates

export BACKUP_TARGET="/Volumes/Tower"
export TABLATURE_DIR="$HOME/Library/Mobile Documents/com~apple~CloudDocs/Music/Tablature/"
export TUTORIALS_PATH=$HOME/Movies/Tutorials

# Media
export LOCAL_MEDIA_PATHS=$HOME/Movies/Porn
export EXTERNAL_MEDIA_PATHS="/Volumes/*/Movies/Porn"
export EXTERNAL_CACHE_PATHS="/Volumes/*/Movies/Cache"
export INDEX_DIR="$HOME/.indexes"
export PRIVATE_PHOTOS_LIBRARY="$HOME/Media/Private/Private.photoslibrary"

export HOSTNAME=$(hostname)

# Helix
export HELIX_USE_OSC52=true

# WASMTime
export WASMTIME_HOME=$HOME/.wasmtime
export PATH=$PATH:$WASMTIME_HOME/bin

# Set default language and character encoding
export LANG="en_US.UTF-8"
export LC_ALL="en_US.UTF-8"

export GHQ_ROOT=$HOME/src

# FZF / Skim
export FZF_DEFAULT_OPTS="--layout=reverse --cycle --preview-window=noborder --highlight-line --no-separator --no-border --inline-info --bind 'ctrl-u:unix-line-discard' --color=bg:-1,fg:blue,info:15,header:7,hl:red,hl+:red,gutter:-1,prompt:yellow,marker:-1,bg+:black,pointer:yellow,fg+:yellow"
export FZF_COMPLETION_TRIGGER='\t' # Default is '**'
export FZF_COMPLETION_OPTS='--preview "bat --color=always {} 2>/dev/null || cat {} 2>/dev/null"'
export FZF_PREVIEW_COMMAND='fzf-preview {}'
export SKIM_DEFAULT_OPTIONS=$FZF_DEFAULT_OPTS
export SKIM_DEFAULT_COMMAND="fd . --max-depth=3"
