# COMPLETIONS
#

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

for config_file in ~/.zsh/scripts/completions/*; do
  GREEN=$(tput setaf 2)
  BLUE=$(tput setaf 4)
  RESET=$(tput sgr0)
  echo "${GREEN} ${RESET}${config_file}${RESET}"

  # Source the config file and continue if there's an error
  if ! source $config_file; then
    echo "Error sourcing $config_file. Skipping..."
  fi
done

# 1password
eval "$(op completion zsh)"
compdef _op op

source <(fd --gen-completions)

# dotter
eval "$(dotter gen-completions --shell zsh)"
compdef _dotter dotter
