# fix macos bug
TRAPWINCH() {
  zle && { zle reset-prompt; zle -R }
}


export NVM_DIR="$HOME/.nvm"
[ -s "$NVM_DIR/nvm.sh" ] && \. "$NVM_DIR/nvm.sh"  # This loads nvm
[ -s "$NVM_DIR/bash_completion" ] && \. "$NVM_DIR/bash_completion"  # This loads nvm bash_completion

# If you come from bash you might have to change your $PATH.
# export PATH=$HOME/bin:/usr/local/bin:$PATH

# Compilation flags
# export ARCHFLAGS="-arch x86_64"

# FZY
#
source "$HOME/config/scripts/fzy.zsh"
# CTRL-o: cd into the selected directory
bindkey '^O' fzy-cd-widget
zstyle :fzy:cd command fd --hidden --max-depth 3 --type d

# CTRL-T: Place the selected file path in the command line
bindkey '^T'  fzy-file-widget
# CTRL-h: Place the selected command from history in the command line
bindkey '^h'  fzy-history-widget
# CTRL-P: Place the selected process ID in the command line
# bindkey '^P'  fzy-proc-widget
#

source "$HOME/.profile"

eval "$(starship init zsh)"
