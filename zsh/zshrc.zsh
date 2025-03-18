# Main Zsh configuration file
#

# ZSH FUNCTIONS
#
# Function autoloading
autoload -Uz add-zsh-hook
#
# Function load path
fpath=(
  $ZSH_CONFIG_DIR/completions
  $fpath
)

# TAB COMPLETION
#
# Directory navigation options
setopt autocd
setopt autopushd

# Completion styles
#
zstyle ':completion:*:*:*:default' menu yes select search
#
#  navigate completion suggestions using arrow keys
zstyle ':completion:*' menu select
#
#  zstyle ':completion:*' format '%B%d%b'
zstyle ':completion:*' special-dirs true
#
#	 dirs first in completion list
zstyle ':completion:*' list-dirs first
#
#  sort alphabetically
zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}'

# # Generated for envman. Do not edit.
# [ -s "$HOME/.config/envman/load.sh" ] && source "$HOME/.config/envman/load.sh"

# Broot
#[[ -f $HOME/.config/broot/launcher/bash/br ]] && source $HOME/.config/broot/launcher/bash/br

# load completion system
autoload -Uz compinit && compinit

# Set vi/emacs mode
# bindkey -v
set -o emacs

config_files=(
  $ZSH_AUTOLOAD_DIR/homebrew.zsh
  $ZSH_AUTOLOAD_DIR/completions.zsh
  $ZSH_AUTOLOAD_DIR/alias.zsh
  $ZSH_AUTOLOAD_DIR/broot.zsh
  $ZSH_AUTOLOAD_DIR/fzf.zsh
  $ZSH_AUTOLOAD_DIR/fzf-completions.zsh
  $ZSH_AUTOLOAD_DIR/fzf-custom.zsh
  $ZSH_AUTOLOAD_DIR/fzf-templates.zsh
  $ZSH_AUTOLOAD_DIR/drive-index.zsh
  $ZSH_AUTOLOAD_DIR/ffmpeg.zsh
  $ZSH_AUTOLOAD_DIR/function.zsh
  $ZSH_AUTOLOAD_DIR/history.zsh
  $ZSH_AUTOLOAD_DIR/media.zsh
  $ZSH_AUTOLOAD_DIR/prompt-middle.zsh
  $ZSH_AUTOLOAD_DIR/prompt.zsh
  $ZSH_AUTOLOAD_DIR/rsync.zsh
  $ZSH_AUTOLOAD_DIR/vi-mode.zsh
  $ZSH_AUTOLOAD_DIR/yt-dlp.zsh
  $ZSH_AUTOLOAD_DIR/starship.zsh
  $ZSH_AUTOLOAD_DIR/fzf-marks.zsh
  $ZSH_AUTOLOAD_DIR/keybindings.zsh
)
for config_file in $config_files; do
  print -P "%F{green}󰚔 %f${config_file:t}%f"
  if ! source $config_file; then
    print -P "%F{red}Error sourcing $config_file. Skipping...%f"
  fi
done

# load completion system
autoload -Uz compinit && compinit
