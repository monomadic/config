# Check if interactive shell to avoid non-interactive errors
if [[ -o interactive ]]; then
  # Ensure ZLE is loaded
  autoload -Uz zle

  # Debug: Print fpath
  print "Current fpath:"
  print -l $fpath

  # Debug: Check if function file exists
  if [[ -f ~/.zsh/functions/_yazi-jump ]]; then
    print "Function file exists"
  else
    print "Function file not found"
  fi

  # Ensure ZLE is loaded
  autoload -Uz zle

  # Force reload of the function
  unfunction _yazi-jump 2>/dev/null
  autoload -Uz _yazi-jump

  # Register with ZLE only if function was loaded
  #if (($ + functions[_yazi - jump])); then
  zle -N _yazi-jump
  bindkey '^ ' _yazi-jump
  #else
  print "Failed to load _yazi-jump function"
  #fi

  _yazi-jump() {
    yazi-jump
  }
  zle -N _yazi-jump

  _fzf-find-files() {
    fzf-find-files
  }
  zle -N _fzf-find-files

  _fzf-jump() {
    source fzf-jump-subshell && zle reset-prompt
  }
  zle -N _fzf-jump

  _fzf_ripgrep() {
    fzf_ripgrep
  }
  zle -N _fzf_ripgrep

  clear-reset() {
    clear
    zle reset-prompt
  }
  zle -N clear-reset

  magic-enter() {
    # Define function content if not defined elsewhere
    : # placeholder
  }
  zle -N magic-enter

  _fzf-cd() {
    # Capture selected directory from fzf
    local selected_dir=$(ls_all | source fzf-cd)
    local ret=$?
    # Only change directory if fzf exited successfully
    if [[ $ret -eq 0 && -n "$selected_dir" && -d "$selected_dir" ]]; then
      cd "$selected_dir"
      zle reset-prompt
    fi
    return $ret
  }
  zle -N _fzf-cd

  cd-up() {
    cd ..
    zle reset-prompt
  }
  zle -N cd-up

  # Bind keys to functions - using the correct function names
  bindkey '^ ' _yazi-jump      # Make sure yazi-jump is defined elsewhere
  bindkey '^f' _fzf-find-files # Make sure fzf-find-files is defined elsewhere
  bindkey '^s' fzf_ripgrep
  bindkey '^k' clear-reset
  bindkey '^l' magic-enter
  bindkey '^o' _fzf-cd
  bindkey '^u' cd-up
  bindkey '^[J' _fzf-jump # Alt+J
fi
