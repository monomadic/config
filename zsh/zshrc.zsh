ZSH_CONFIG_DIR="$HOME/.zsh"

setopt autocd # cd without typing cd
autoload -Uz add-zsh-hook

# Source all configuration files
for config_file ($ZSH_CONFIG_DIR/autoload/*.zsh); do
  YELLOW=$(tput setaf 3)
  RESET=$(tput sgr0)
	echo "${YELLOW}󰅱 ${config_file:t}${RESET}"
  source $config_file
done

# ------------------------

# Define colors
RED=$(tput setaf 1)
RESET=$(tput sgr0)

# Directories to check
declare -a dirs=("$HOME/config" "$HOME/wiki")

# Function to check for uncommitted changes
check_uncommitted_changes() {
    local dir=$1
    if [[ -n "$(cd "$dir" && git status --porcelain)" ]]; then
        echo -e "\n${RED} uncommitted changes: $dir${RESET}"
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
