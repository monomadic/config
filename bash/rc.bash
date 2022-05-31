echo ".bashrc loaded"

# aliases
#
alias lg=lazygit

# fuzzy marks (fzm)
#
source $HOME/.bash/fzm.bash

# keybindings
#
bind '"\C-o":"fzm\n"'

# environment variables
#
export EDITOR=nvim

# foundry
export PATH="$PATH:/home/dev/.foundry/bin"

# ssh
# export TERM=vt100

# cargo
if command -v cargo &> /dev/null
then
  . "$HOME/.cargo/env"
fi
