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

# Local source (not checked into git)
[[ -f "$ZSH_CONFIG_DIR/local.zsh" ]] && source "$ZSH_CONFIG_DIR/local.zsh"

# # Generated for envman. Do not edit.
# [ -s "$HOME/.config/envman/load.sh" ] && source "$HOME/.config/envman/load.sh"
