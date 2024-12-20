# Load environment variables

# HOMEBREW
#
eval "$(/opt/homebrew/bin/brew shellenv)"

local BREW_PREFIX="$(brew --prefix)"

# PATH
#
# note: in zsh, $path is an associative array that syncs to $PATH
typeset -U path
path=(
  $BREW_PREFIX/coreutils/libexec/gnubin
  $BREW_PREFIX/gnu-sed/libexec/gnubin
  $BREW_PREFIX/grep/libexec/gnubin
  $HOME/.bin
  $HOME/.cargo/bin
  $HOME/.deno/bin
  $HOME/.foundry/bin
  $HOME/.local/share/nvim/mason/bin
  $HOME/.zsh/bin
  $HOME/.zsh/widgets
  $HOME/go/bin
  $HOME/.local/bin
  $path
)

# ZSH FUNCTIONS
#
fpath=(
  ~/.zsh/completions
  ~/.zsh/functions
  ~/.zsh/widgets
  $BREW_PREFIX/share/zsh/site-functions
  $fpath
)

# MANPATH CONFIGURATION
#
typeset -U manpath
manpath=(
  /opt/homebrew/opt/coreutils/libexec/gnuman
  $manpath
)

# ENVIRONMENT VARIABLES
#
local env_file="$HOME/config/zsh/env.zsh"
# Source the config file and continue if there's an error
if ! source $env_file; then
  echo "Error sourcing $env_file. Skipping..."
fi

# Enable vi mode
# bindkey -v

YELLOW=$(tput setaf 3)
BLUE=$(tput setaf 4)
RESET=$(tput sgr0)
echo "${YELLOW}󰅩  ${BLUE}${env_file:t}${RESET}"

#
# # # Enable error handling
# # set -o errexit # Exit on error
# #
# # # Trap errors to prevent closing the terminal
# # trap 'echo "An error occurred. Please check the script.";' ERR
#
# PURPLE=$(tput setaf 5)
# BLUE=$(tput setaf 4)
# RESET=$(tput sgr0)
#
# display-uptime
#
# disk_space=$(df --si / | awk 'NR==2 {print $4}')
# # Extract the numeric value and the unit
# value=$(echo "$disk_space" | grep -oE '[0-9]+')
# unit=$(echo "$disk_space" | grep -oE '[A-Z]+')
#
# # Convert MB to GB if applicable
# if [[ "$unit" == "M" && "$value" -ge 1024 ]]; then
#   gb=$(echo "scale=2; $value / 1024" | bc)
#   print -P "%F{yellow}  %F{green}${gb}gb free"
# else
#   print -P "%F{yellow}  %F{green}${disk_space} free"
# fi
#
# # ------------------------
#
# # Define colors
# RED=$(tput setaf 1)
# RESET=$(tput sgr0)

# # Directories to check
# declare -a dirs=("$HOME/config" "$HOME/wiki")
#
# # Function to check for uncommitted changes
# function check_uncommitted_changes() {
#   local dir=$1
#   if [[ -n "$(cd "$dir" && git status --porcelain)" ]]; then
#     echo -e "${RED}  $dir${RESET}"
#     # cd "$dir" && git status --short --untracked-files=all
#     # cd "$HOME"
#   fi
# }
#
# # Check each directory
# for dir in "${dirs[@]}"; do
#   check_uncommitted_changes "$dir"
# done
#
# ------------------------

# Local source (not checked into git)
#[[ -f "$ZSH_CONFIG_DIR/local.zsh" ]] && source "$ZSH_CONFIG_DIR/local.zsh"

# # Generated for envman. Do not edit.
# [ -s "$HOME/.config/envman/load.sh" ] && source "$HOME/.config/envman/load.sh"

# Broot
#[[ -f $HOME/.config/broot/launcher/bash/br ]] && source $HOME/.config/broot/launcher/bash/br

print

# Source additional configuration files
#
# Define base directory for config files
ZSH_AUTOLOAD_DIR="$HOME/.zsh/scripts/autoload"

# Define the array of config files
config_files=(
  "$ZSH_AUTOLOAD_DIR/alias.zsh"
  "$ZSH_AUTOLOAD_DIR/broot.zsh"
  "$ZSH_AUTOLOAD_DIR/completions.zsh"
  "$ZSH_AUTOLOAD_DIR/drive-index.zsh"
  "$ZSH_AUTOLOAD_DIR/ffmpeg.zsh"
  "$ZSH_AUTOLOAD_DIR/function.zsh"
  "$ZSH_AUTOLOAD_DIR/fzf-completion.zsh"
  "$ZSH_AUTOLOAD_DIR/fzf-custom.zsh"
  "$ZSH_AUTOLOAD_DIR/fzf-key-bindings.zsh"
  "$ZSH_AUTOLOAD_DIR/fzf-marks.zsh"
  "$ZSH_AUTOLOAD_DIR/fzf-templates.zsh"
  "$ZSH_AUTOLOAD_DIR/fzf.zsh"
  "$ZSH_AUTOLOAD_DIR/history.zsh"
  "$ZSH_AUTOLOAD_DIR/imagemagick.zsh"
  "$ZSH_AUTOLOAD_DIR/joshuto.zsh.disabled"
  "$ZSH_AUTOLOAD_DIR/media-players.zsh"
  "$ZSH_AUTOLOAD_DIR/media.zsh"
  "$ZSH_AUTOLOAD_DIR/prompt-middle.zsh"
  "$ZSH_AUTOLOAD_DIR/prompt.zsh"
  "$ZSH_AUTOLOAD_DIR/rsync.zsh"
  "$ZSH_AUTOLOAD_DIR/starship.zsh"
  "$ZSH_AUTOLOAD_DIR/vi-mode.zsh"
  "$ZSH_AUTOLOAD_DIR/yt-dlp.zsh"
  "$ZSH_AUTOLOAD_DIR/keybindings.zsh"
)

# Loop through and source each file
for config_file in $config_files; do
  print -P "%F{green}󰚔 %f${config_file:t}%f"
  if ! source $config_file; then
    print -P "%F{red}Error sourcing $config_file. Skipping...%f"
  fi
done

print
