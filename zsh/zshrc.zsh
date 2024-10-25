# Load environment variables
#
# source "$HOME/.env"
local env_file="$HOME/config/zsh/env.zsh"

# Ensure autocompletion system is initialized
fpath=(~/.zsh/completions $fpath)
autoload -Uz compinit && compinit

setopt autocd           # cd without typing cd
setopt autopushd        # auto push dirs to recent dirs db (for dirs cmd)

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

# Source all configuration files
fpath=($ZSH_CONFIG_DIR/functions/ $fpath)

# # Enable error handling
# set -o errexit # Exit on error
#
# # Trap errors to prevent closing the terminal
# trap 'echo "An error occurred. Please check the script.";' ERR

PURPLE=$(tput setaf 5)
BLUE=$(tput setaf 4)
RESET=$(tput sgr0)


display-uptime

disk_space=$(df --si / | awk 'NR==2 {print $4}')
# Extract the numeric value and the unit
value=$(echo "$disk_space" | grep -oE '[0-9]+')
unit=$(echo "$disk_space" | grep -oE '[A-Z]+')

# Convert MB to GB if applicable
if [[ "$unit" == "M" && "$value" -ge 1024 ]]; then
  gb=$(echo "scale=2; $value / 1024" | bc)
  print -P "%F{yellow}  %F{green}${gb}gb free"
else
  print -P "%F{yellow}  %F{green}${disk_space} free"
fi

# ------------------------

# Define colors
RED=$(tput setaf 1)
RESET=$(tput sgr0)

# Directories to check
declare -a dirs=("$HOME/config" "$HOME/wiki")

# Function to check for uncommitted changes
function check_uncommitted_changes() {
  local dir=$1
  if [[ -n "$(cd "$dir" && git status --porcelain)" ]]; then
    echo -e "\n${RED} $dir${RESET}"
    # cd "$dir" && git status --short --untracked-files=all
    # cd "$HOME"
  fi
}

# Check each directory
for dir in "${dirs[@]}"; do
  check_uncommitted_changes "$dir"
done

# ------------------------

# Local source (not checked into git)
[[ -f "$ZSH_CONFIG_DIR/local.zsh" ]] && source "$ZSH_CONFIG_DIR/local.zsh"

# # Generated for envman. Do not edit.
# [ -s "$HOME/.config/envman/load.sh" ] && source "$HOME/.config/envman/load.sh"

# source /Users/nom/.config/broot/launcher/bash/br

# Created by `pipx` on 2024-08-26 19:39:42
export PATH="$PATH:/Users/nom/.local/bin"

# coreutils
export PATH="/opt/homebrew/opt/coreutils/libexec/gnubin:/opt/homebrew/opt/gnu-sed/libexec/gnubin:/opt/homebrew/opt/grep/libexec/gnubin:$PATH"
export MANPATH="/opt/homebrew/opt/coreutils/libexec/gnuman:$MANPATH"

# homebrew
eval "$(/opt/homebrew/bin/brew shellenv)"

source /Users/nom/.config/broot/launcher/bash/br


# TODO: remove this shit
for config_file in $ZSH_CONFIG_DIR/scripts/autoload/*.(zsh|sh)(N); do
	echo "${PURPLE}󰚔 ${BLUE}${config_file}${RESET}"
  # Source the config file and continue if there's an error
  if ! source "$config_file"; then
    echo "Error sourcing $config_file. Skipping..."
  fi
done
echo
