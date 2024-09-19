# Load environment variables
# source "$HOME/.env"
local env_file="$HOME/config/zsh/env.zsh"

GREEN=$(tput setaf 5)
BLUE=$(tput setaf 4)
RESET=$(tput sgr0)
echo "${GREEN}󰅱 ${BLUE}${env_file:t}${RESET}"

# Source the config file and continue if there's an error
if ! source $env_file; then
	echo "Error sourcing $env_file. Skipping..."
fi

setopt autocd             # cd without typing cd
autoload -Uz add-zsh-hook # function autoloading (built-in zsh function)

# Source all configuration files
fpath=($ZSH_CONFIG_DIR/functions/ $fpath)

# # Enable error handling
# set -o errexit # Exit on error
#
# # Trap errors to prevent closing the terminal
# trap 'echo "An error occurred. Please check the script.";' ERR

# Loop through the config files
for config_file in $ZSH_CONFIG_DIR/autoload/*.(zsh|sh)(N); do
  YELLOW=$(tput setaf 4)
  RESET=$(tput sgr0)
  echo "${GREEN}󰚔 ${YELLOW}autoload/${config_file:t}${RESET}"

  # Source the config file and continue if there's an error
  if ! source "$config_file"; then
    echo "Error sourcing $config_file. Skipping..."
  fi
done

echo

.uptime

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

source /Users/nom/.config/broot/launcher/bash/br
