# Check if interactive shell to avoid non-interactive errors
if [[ -o interactive ]]; then
  # # Load ZLE only if it's not already available
  # if [[ -z $ZLE_VERSION ]]; then
  #   autoload -Uz zle
  # fi

  # Define functions first
  _yazi-jump() {
    local tmp="$(mktemp -t "yazi-cwd.XXXXX")"

    yazi "$@" --cwd-file="$tmp"

    if cwd="$(cat -- "$tmp")" && [ -n "$cwd" ] && [ "$cwd" != "$PWD" ]; then
      cd -- "$cwd"
      zle reset-prompt
    fi

    rm -f -- "$tmp"
  }

  _fzf-jump() {
    source fzf-jump && zle reset-prompt
  }

  fzf_ripgrep() { # removed invalid asterisks
    fzf_ripgrep
  }

  clear-reset() {
    clear
    zle reset-prompt
  }

  _fzf-cd() {
    local selected_dir

    # move cursor up one line
    tput cuu1

    # Use command substitution to capture fzf-cd's output
    #selected_dir=$(ls_all 2>/dev/null | awk '{ plain=$0; gsub(/\033\[[0-9;]*[mK]/, "", plain); if (!seen[plain]++) print $0 }' | source fzf-cd) || return $?
    selected_dir=$(ls_all 2>/dev/null | source fzf-cd) || return $?
    echo $selected_dir

    if [[ -n "$selected_dir" && -d "${selected_dir}" ]]; then
      cd "${selected_dir}" || return 1
      zle reset-prompt
      return 0
    fi

    zle reset-prompt
    return 1
  }

  _fzf-jump() {
    local selected_dir=$(ls_all | source fzf-jump)
    local ret=$?
    if [[ $ret -eq 0 && -n "$selected_dir" && -d "$selected_dir" ]]; then
      cd "$selected_dir"
      zle reset-prompt
    fi
    return $ret
  }

  cd-up() {
    cd ..
    zle reset-prompt
  }

  # Register functions with ZLE
  zle -N _yazi-jump
  zle -N _fzf-find-files
  zle -N _fzf-jump
  zle -N fzf_ripgrep
  zle -N clear-reset
  zle -N magic-enter
  zle -N _fzf-cd
  zle -N cd-up

  # Bind keys to functions
  # Ctrl-Space: yazi jump
  bindkey '^@' _yazi-jump
  bindkey '^f' _fzf-find-files
  bindkey '^s' fzf_ripgrep
  bindkey '^k' clear-reset
  bindkey '^l' magic-enter
  bindkey '^o' _fzf-cd
  #bindkey '^u' cd-up
  # bindkey 'f20' _fzf-jump # Alt+J
fi
