ZSH_CONFIG_DIR="$HOME/.zsh"

setopt autocd # cd without typing cd
autoload -Uz add-zsh-hook

# Source all configuration files
for config_file ($ZSH_CONFIG_DIR/*.zsh); do
  YELLOW=$(tput setaf 3)
  RESET=$(tput sgr0)
	echo "${YELLOW}$config_file${RESET}"
  source $config_file
done

# check for uncommitted changes in important dirs
RED=$(tput setaf 1)
RESET=$(tput sgr0)
[[ -n "$(cd $HOME/config && git status --porcelain)" ]] && echo "\n${RED} uncommitted changes: $HOME/config${RESET}" && cd $HOME/config && git status --short --untracked-files=all && cd $HOME

# git status --short --untracked-files=all

# Local source (not checked into git)
[[ -f "$ZSH_CONFIG_DIR/local.zsh" ]] && source "$ZSH_CONFIG_DIR/local.zsh"

# # Generated for envman. Do not edit.
# [ -s "$HOME/.config/envman/load.sh" ] && source "$HOME/.config/envman/load.sh"
