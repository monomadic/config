# Load environment variables

# HOMEBREW
#
eval "$(/opt/homebrew/bin/brew shellenv)"

# PATH
#
# note: in zsh, $path is an associative array that syncs to $PATH
typeset -U path
path=(
  "$(brew --prefix)/coreutils/libexec/gnubin"
  "$(brew --prefix)/gnu-sed/libexec/gnubin"
  "$(brew --prefix)/grep/libexec/gnubin"
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
fpath=(
  ~/.zsh/completions
  ~/.zsh/functions
  ~/.zsh/widgets
  "$(brew --prefix)/share/zsh/site-functions"
  $fpath
)

# Manpath configuration
typeset -U manpath
manpath=(
  /opt/homebrew/opt/coreutils/libexec/gnuman
  $manpath
)

# Environment variables
local env_file="$HOME/config/zsh/env.zsh"

# Initialize autocompletion system
autoload -Uz compinit && compinit

setopt autocd    # cd without typing cd
setopt autopushd # auto push dirs to recent dirs db (for dirs cmd)

autoload -Uz add-zsh-hook # function autoloading (built-in zsh function)

# Enable menu selection for better directory completion
zstyle ':completion:*' menu select

# Ensure Zsh treats directories as valid completion targets without needing `./`
zstyle ':completion:*' special-dirs true

# Ensure Zsh completes directories first before files
zstyle ':completion:*' list-dirs first

# Enable case-insensitive matching (optional, for ease of completion)
zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}'

# Enable vi mode
# bindkey -v

YELLOW=$(tput setaf 3)
BLUE=$(tput setaf 4)
RESET=$(tput sgr0)
echo "${YELLOW}󰅩  ${BLUE}${env_file:t}${RESET}"

# Source the config file and continue if there's an error
if ! source $env_file; then
  echo "Error sourcing $env_file. Skipping..."
fi

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

source "$HOME/.zsh/scripts/autoload/alias.zsh"
source "$HOME/.zsh/scripts/autoload/broot.zsh"
source "$HOME/.zsh/scripts/autoload/completions.zsh"
source "$HOME/.zsh/scripts/autoload/drive-index.zsh"
source "$HOME/.zsh/scripts/autoload/ffmpeg.zsh"
source "$HOME/.zsh/scripts/autoload/function.zsh"
source "$HOME/.zsh/scripts/autoload/fzf-completion.zsh"
source "$HOME/.zsh/scripts/autoload/fzf-custom.zsh"
source "$HOME/.zsh/scripts/autoload/fzf-key-bindings.zsh"
source "$HOME/.zsh/scripts/autoload/fzf-marks.zsh"
source "$HOME/.zsh/scripts/autoload/fzf-templates.zsh"
source "$HOME/.zsh/scripts/autoload/fzf.zsh"
source "$HOME/.zsh/scripts/autoload/history.zsh"
source "$HOME/.zsh/scripts/autoload/imagemagick.zsh"
source "$HOME/.zsh/scripts/autoload/joshuto.zsh.disabled"
source "$HOME/.zsh/scripts/autoload/media-players.zsh"
source "$HOME/.zsh/scripts/autoload/media.zsh"
source "$HOME/.zsh/scripts/autoload/prompt-middle.zsh"
source "$HOME/.zsh/scripts/autoload/prompt.zsh"
source "$HOME/.zsh/scripts/autoload/rsync.zsh"
source "$HOME/.zsh/scripts/autoload/starship.zsh"
source "$HOME/.zsh/scripts/autoload/vi-mode.zsh"
source "$HOME/.zsh/scripts/autoload/yt-dlp.zsh"
source "$HOME/.zsh/scripts/autoload/keybindings.zsh"

#for config_file in $ZSH_CONFIG_DIR/scripts/autoload/*.(zsh|sh)(N); do
#    print -P "%F{green}󰚔 %f${config_file:t}%f"
#    if ! source $config_file; then
#        print -P "%F{red}Error sourcing $config_file. Skipping...%f"
#    fi
#done

print
