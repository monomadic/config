#!/bin/sh
echo "loaded .profile"

# make default editor Neovim
export EDITOR=nvim
#export FZF_DEFAULT_COMMAND='rg --files --hidden --glob \!.git --glob \!.cache --glob \!.local --glob \!.nvm --glob \!.npm --glob \!.mozilla'
export FZF_DIR_JUMP='fd --hidden --max-depth 4 --type d'
export FZF_DEFAULT_COMMAND='fd --hidden --max-depth 4 --type f'

# Most pure GTK3 apps use wayland by default, but some,
# like Firefox, need the backend to be explicitely selected.
export MOZ_ENABLE_WAYLAND=1
export MOZ_DBUS_REMOTE=1
export GTK_CSD=0

# qt wayland
export QT_QPA_PLATFORM="wayland"
export QT_QPA_PLATFORMTHEME=qt5ct
export QT_WAYLAND_DISABLE_WINDOWDECORATION="1"

export CLUTTER_BACKEND=wayland
export SDL_VIDEODRIVER=wayland
export XDG_SESSION_TYPE=wayland
export QT_QPA_PLATFORM=wayland
export QT_WAYLAND_DISABLE_WINDOWDECORATION=1
export MOZ_ENABLE_WAYLAND=1
export GBM_BACKEND=nvidia-drm
export __GLX_VENDOR_LIBRARY_NAME=nvidia
export WLR_NO_HARDWARE_CURSORS=1 

#Java XWayland blank screens fix
export _JAVA_AWT_WM_NONREPARENTING=1

# set default shell and terminal
export SHELL=/usr/bin/zsh
export TERMINAL_COMMAND=/usr/share/sway/scripts/foot.sh

# go
export PATH=$PATH:~/go/bin

alias g=git
alias gp=git pull
alias gs="git status"
alias r=ranger
alias f=fzf
alias v=nvim
alias vim=nvim
alias xc="xclip -sel clip"
# prompt
# source /usr/share/zsh-theme-powerlevel10k/powerlevel10k.zsh-theme

# fzf
#source $HOME/Config/scripts/key-bindings.zsh
#source $HOME/Config/scripts/completion.zsh
#source $HOME/Config/scripts/tab-completion.zsh

# rust
source $HOME/.cargo/env

#source /usr/share/nvm/init-nvm.sh
. "$HOME/.cargo/env"
