# COMPLETIONS
#

for config_file in $ZSH_CONFIG_DIR/scripts/completions/*; do
  GREEN=$(tput setaf 2)
  BLUE=$(tput setaf 4)
  RESET=$(tput sgr0)
  echo "${GREEN} ${BLUE}${config_file}${RESET}"

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
