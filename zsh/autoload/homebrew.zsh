# HOMEBREW

export HOMEBREW_NO_ENV_HINTS=true

eval "$(/opt/homebrew/bin/brew shellenv)"
local BREW_PREFIX="$(brew --prefix)"

path=(
  $BREW_PREFIX/coreutils/libexec/gnubin
  $BREW_PREFIX/gnu-sed/libexec/gnubin
  $BREW_PREFIX/grep/libexec/gnubin
  $path
)

manpath=(
  $BREW_PREFIX/coreutils/libexec/gnuman
  $manpath
)
