echo ".bashrc loaded"

source $HOME/.bash/fzm.bash
bind '"\C-o":"fzm\n"'

# ssh likes this
# export TERM=vt100
if command -v cargo &> /dev/null
then
  . "$HOME/.cargo/env"
fi
