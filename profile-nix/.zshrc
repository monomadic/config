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
# source $HOME/config/scripts/key-bindings.zsh
# path complete eg cd <tab>
# source $HOME/config/scripts/completion.zsh
# zsh complete eg git <tab>
# source $HOME/config/scripts/tab-completion.zsh
eval "$(zoxide init zsh)"
# FZY
#
source $HOME/config/scripts/fzy.zsh
# CTRL-o: cd into the selected directory
bindkey '^o' fzy-cd-widget
zstyle :fzy:cd command fd --hidden --max-depth 8 --type d

# CTRL-T: Place the selected file path in the command line
bindkey '^T'  fzy-file-widget
# CTRL-h: Place the selected command from history in the command line
bindkey '^h'  fzy-history-widget
# CTRL-P: Place the selected process ID in the command line
# bindkey '^P'  fzy-proc-widget

source $HOME/.profile

#source /usr/share/nvm/init-nvm.sh
#

source $HOME/.config/scripts/rclone.zsh 

eval "$(starship init zsh)"

export NVM_DIR="$HOME/.nvm"
#[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"  # This loads nvm
# [ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"  # This loads nvm bash_completion

source /home/nom/.config/broot/launcher/bash/br
