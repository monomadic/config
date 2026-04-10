# HOMEBREW

export HOMEBREW_NO_ENV_HINTS=true

typeset -U path manpath
typeset _brew_prefix="${HOMEBREW_PREFIX:-/opt/homebrew}"

if [[ -d "$_brew_prefix" ]]; then
  path=(
    $_brew_prefix/coreutils/libexec/gnubin
    $_brew_prefix/gnu-sed/libexec/gnubin
    $_brew_prefix/grep/libexec/gnubin
    $path
  )

  manpath=(
    $_brew_prefix/coreutils/libexec/gnuman
    $manpath
  )
fi
