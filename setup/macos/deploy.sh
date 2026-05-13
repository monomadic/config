#!/bin/sh

set -eu

DOTFILES_DIR="${DOTFILES_DIR:-$HOME/config}"

exec "$DOTFILES_DIR/config/zsh/bin/dotter-deploy" "$@"
