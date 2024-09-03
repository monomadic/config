export ZSH_CONFIG_DIR="$HOME/.zsh"
export DOTFILES_DIR="$HOME/config"
export ZSH_DOTFILES_DIR="$DOTFILES_DIR/zsh"
export BABYBLUE_DIR="/Volumes/BabyBlue2TB"
export MASTER_BACKUP_DIR="/Volumes/BabyBlue2TB"
export TABLATURE_DIR="$HOME/Tablature"
export LOCAL_MEDIA_PATHS="$HOME/Downloads/Porn:$HOME/Media/Porn:$HOME/Movies/Porn"
export LOCAL_CACHE_PATHS="$HOME/Movies/Cache:$HOME/Media/Cache"
export EXTERNAL_MEDIA_PATHS="/Volumes/*/Movies/Porn"

setopt autocd # cd without typing cd
autoload -Uz add-zsh-hook # function autoloading (built-in zsh function)

# Source all configuration files
fpath=($ZSH_CONFIG_DIR/functions/ $fpath)

for config_file ($ZSH_CONFIG_DIR/autoload/*.zsh); do
  YELLOW=$(tput setaf 3)
  RESET=$(tput sgr0)
	echo "${YELLOW}󰅱 ${config_file:t}${RESET}"
  source $config_file
done

echo
@uptime
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
