# Configuration that runs for all shells (even non-interactive)
#

# PATH
#
# note: in zsh, $path is an associative array that syncs to $PATH
# typeset -U path
# path=(
#   $HOME/.bin
#   $HOME/.local/bin
#   $HOME/.zsh/bin
#   $HOME/.cargo/bin
#   $HOME/.deno/bin
#   $HOME/.foundry/bin
#   $HOME/.local/share/<editor>/bin
#   $HOME/go/bin
#   $HOME/.cache/lm-studio/bin
#   $path
# )

source $HOME/.bin/init-path

export ZSH_CONFIG_DIR=$HOME/.zsh
export ZSH_COMPLETIONS_DIR=$ZSH_CONFIG_DIR/completions
export ZSH_AUTOLOAD_DIR=$ZSH_CONFIG_DIR/autoload
export CONFIG_DIR=$HOME/.config
export XDG_CONFIG_HOME=$CONFIG_DIR
# ~/.zshenv is a symlink into the checkout; resolve it (:A) so DOTFILES_DIR
# points at wherever the repo was actually cloned. Falls back to ~/config when
# this file was copied rather than linked.
export DOTFILES_DIR=${${${(%):-%N}:A}:h:h:h}
[[ -f $DOTFILES_DIR/dotter/global.toml ]] || export DOTFILES_DIR=$HOME/config
export ZSH_DOTFILES_DIR=$DOTFILES_DIR/config/zsh

export HOSTNAME=$HOST  # zsh builtin; $(hostname) forked on every zsh invocation

# Set default language and character encoding
export LANG="en_US.UTF-8"
export LC_ALL="en_US.UTF-8"
