# COMPLETIONS
#
autoload -Uz compinit
compinit

for config_file in $ZSH_CONFIG_DIR/completions/*.(zsh|sh); do
  GREEN=$(tput setaf 2)
  RESET=$(tput sgr0)
  echo "${GREEN}󰅱 completions/${config_file:t}${RESET}"

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
# eval "$(dotter gen-completions --shell zsh)"
# compdef _dotter dotter
