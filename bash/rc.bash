echo ".bashrc loaded"

eval "$(/opt/homebrew/bin/brew shellenv)"

# aliases
#
alias lg=lazygit
alias gs="git status"

# fuzzy marks (fzm)
#
source $HOME/.bash/fzm.bash

# keybindings
#
bind '"\C-o":"fzm\n"'

# environment variables
#
#export EDITOR=nvim

# foundry
export PATH="$PATH:/home/dev/.foundry/bin"

# deno
export DENO_INSTALL="/home/dev/.deno"
export PATH="$DENO_INSTALL/bin:$PATH"

# ssh
# export TERM=vt100

# cargo
if command -v cargo &> /dev/null
then
  . "$HOME/.cargo/env"
fi

# Generated for envman. Do not edit.
[ -s "$HOME/.config/envman/load.sh" ] && source "$HOME/.config/envman/load.sh"

source /Users/nom/.config/broot/launcher/bash/br
