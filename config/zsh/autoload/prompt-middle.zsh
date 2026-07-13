# MAGIC ENTER
#
# Shift+Enter on an empty buffer scrolls the prompt to the middle of the
# terminal, for keyboard-on-stomach typing. Bound in keybindings.zsh.

# load terminfo module so `echoti` is available
zmodload zsh/terminfo

_magic-enter() {
  if [[ -z $BUFFER ]]; then
    # Print newlines to scroll content up
    local halfpage=$((LINES / 2))
    printf '\n%.0s' {1..$halfpage}
    # Move cursor back up to the middle of the screen
    local cursor_up=$(echoti cuu $halfpage)
    print -n $cursor_up
    zle reset-prompt
  else
    zle accept-line
  fi
}

zle -N _magic-enter
