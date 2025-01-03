# ensures the terminal prompt is always above the middle of the terminal
# for keyboard-on-stomach typing

# load terminfo modules to make the associative array $terminfo available
zmodload zsh/terminfo

# save current prompt to parameter PS1o
PS1o="$PS1"

# calculate how many lines one half of the terminal's height has
halfpage=$((LINES / 2))

# construct parameter to go down/up $halfpage lines via termcap
halfpage_down=""
for i in {1..$halfpage}; do
  halfpage_down="$halfpage_down$terminfo[cud1]"
done

halfpage_up=""
for i in {1..$halfpage}; do
  halfpage_up="$halfpage_up$terminfo[cuu1]"
done

# define functions
prompt_middle() {
  # print $halfpage_down
  PS1="%{${halfpage_down}${halfpage_up}%}$PS1o"
}

prompt_restore() {
  PS1="$PS1o"
}

# _magic-enter-erase() {
#   if [[ -z $BUFFER ]]; then
#     # Calculate half the terminal height for scroll effect
#     local halfpage_up=$(echoti cuu $((LINES / 2)))
#     local halfpage_down=$(echoti cud $((LINES / 2)))
#     # Get sequence for moving cursor up one line
#     local cursor_up=$terminfo[cuu1]
#     # Print scroll effect without newline (-n flag)
#     print -n ${halfpage_down}${halfpage_up}${cursor_up}
#     zle reset-prompt
#   else
#     # If buffer contains text, act like a normal Enter key
#     zle accept-line
#   fi
# }

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
