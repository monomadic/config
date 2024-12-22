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

  _fzf-find-files() {
    fzf-find-files
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

  magic-enter() {
    : # placeholder
  }

  _fzf-cd() {
    local selected_dir=$(ls_all | source fzf-cd)
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
  bindkey '^ ' _yazi-jump
  bindkey '^f' _fzf-find-files
  bindkey '^s' fzf_ripgrep
  bindkey '^k' clear-reset
  bindkey '^l' magic-enter
  bindkey '^o' _fzf-cd
  #bindkey '^u' cd-up
  bindkey 'f20' _fzf-jump # Alt+J
fi
