# Configuration that runs for all shells (even non-interactive)
#

# PATH
#
# note: in zsh, $path is an associative array that syncs to $PATH
# typeset -U path
# path=(
#   $HOME/.local/bin
#   $HOME/.cargo/bin
#   $HOME/.deno/bin
#   $HOME/.foundry/bin
#   $HOME/.local/share/<editor>/bin
#   $HOME/go/bin
#   $HOME/.cache/lm-studio/bin
#   $path
# )

source $HOME/.local/bin/init-path

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

# Where third-party source gets checked out — the installers under
# scripts/install/ clone into $SRC_PATH/<repo> and build from there.
# `:-` rather than a plain assignment: zshenv runs for every zsh invocation,
# so an unconditional export would clobber `SRC_PATH=/elsewhere some-script`.
export SRC_PATH=${SRC_PATH:-$HOME/src}

export HOSTNAME=$HOST  # zsh builtin; $(hostname) forked on every zsh invocation

# Set default language and character encoding
export LANG="en_US.UTF-8"
export LC_ALL="en_US.UTF-8"

# Machine-local secrets (gitignored; see .env.example)
[[ -f $DOTFILES_DIR/.env ]] && source $DOTFILES_DIR/.env
