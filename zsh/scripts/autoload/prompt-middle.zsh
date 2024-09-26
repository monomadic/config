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

magic-enter() {
  if [[ -z $BUFFER ]]; then
    print ${halfpage_down}${halfpage_up}$terminfo[cuu1]
    zle reset-prompt
  else
    zle accept-line
  fi
}
