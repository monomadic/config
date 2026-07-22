#!/bin/sh
#
# Deploy this checkout with Dotter. Works from any clone location.

set -eu

DOTFILES_DIR="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd -P)"
export DOTFILES_DIR

exec "$DOTFILES_DIR/config/zsh/bin/dotter-deploy" "$@"
