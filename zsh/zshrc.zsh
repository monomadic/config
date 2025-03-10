# Main Zsh configuration file
#

zstyle ':completion:*:*:*:default' menu yes select search

# Enable vi mode
# bindkey -v

# # Generated for envman. Do not edit.
# [ -s "$HOME/.config/envman/load.sh" ] && source "$HOME/.config/envman/load.sh"

# Broot
#[[ -f $HOME/.config/broot/launcher/bash/br ]] && source $HOME/.config/broot/launcher/bash/br

print

YELLOW=$(tput setaf 3)
BLUE=$(tput setaf 4)
RESET=$(tput sgr0)

# Source additional configuration files
#
# Define base directory for config files
ZSH_AUTOLOAD_DIR="$HOME/.zsh/autoload"

# Define the array of config files
config_files=(
  $ZSH_AUTOLOAD_DIR/alias.zsh
  $ZSH_AUTOLOAD_DIR/homebrew.zsh
  $ZSH_AUTOLOAD_DIR/broot.zsh
  $ZSH_AUTOLOAD_DIR/fzf.zsh
  $ZSH_AUTOLOAD_DIR/completions.zsh
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

# Loop through and source each file
for config_file in $config_files; do
  print -P "%F{green}󰚔 %f${config_file:t}%f"
  if ! source $config_file; then
    print -P "%F{red}Error sourcing $config_file. Skipping...%f"
  fi
done
