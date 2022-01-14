echo "loaded .zshrc"

# Use powerline
USE_POWERLINE="false"
# Source manjaro-zsh-configuration
#if [[ -e /usr/share/zsh/manjaro-zsh-config ]]; then
  source /usr/share/zsh/manjaro-zsh-config
#fi
# Use manjaro zsh prompt
#if [[ -e /usr/share/zsh/manjaro-zsh-prompt ]]; then
#  source /usr/share/zsh/manjaro-zsh-prompt
#fi

# prompt
#source /usr/share/zsh-theme-powerlevel10k/powerlevel10k.zsh-theme

# FZF
#
# jump etc
source $HOME/config/scripts/key-bindings.zsh
# path complete eg cd <tab>
source $HOME/config/scripts/completion.zsh
# zsh complete eg git <tab>
source $HOME/config/scripts/tab-completion.zsh

source $HOME/.profile

#source /usr/share/nvm/init-nvm.sh
