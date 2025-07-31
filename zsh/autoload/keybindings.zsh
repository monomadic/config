# Check if interactive shell to avoid non-interactive errors
if [[ -o interactive ]]; then
  # # Load ZLE only if it's not already available
  # if [[ -z $ZLE_VERSION ]]; then
  #   autoload -Uz zle
  # fi
  # --- ensure add-zsh-hook is available earlier in your rc ---
  autoload -Uz add-zsh-hook

  # Optional lazy loader if you deferred your fzf init
  _ensure_fzf_loaded() {
    # If you used a precmd deferral, call it here once when a widget is hit
    typeset -f _lazy_fzf_precmd >/dev/null && { add-zsh-hook -d precmd _lazy_fzf_precmd 2>/dev/null; _lazy_fzf_precmd; }
  }

  # ── ⌘H: fzf history picker (insert selected command)
  fzf-history-widget() {
    setopt localoptions pipefail
    _ensure_fzf_loaded
    # reverse chronological, strip numbers
    local sel
    sel=$(fc -rl 1 | sed 's/^[[:space:]]*[0-9[:space:]]*//' | fzf --tac --no-sort --exact --ansi \
           --prompt='history> ' --height=40% --border) || return
    LBUFFER=$sel
    zle redisplay
  }
  zle -N fzf-history-widget

  # ── ⌘I: insert file path(s) from fzf (current dir; multi-select)
  fzf-insert-path() {
    setopt localoptions pipefail
    _ensure_fzf_loaded
    local -a files
    # Tweak fd switches as you like (hidden, gitignore, type filters)
    files=("${(@f)$(fd -t f . 2>/dev/null | fzf -m --ansi \
                  --prompt='files> ' --height=40% --preview 'ls -lah -- {}' --border)}") || return
    (( ${#files} )) || return
    # Quote each path and append
    local f
    for f in "${files[@]}"; do
      LBUFFER+="${(q)f} "
    done
    zle redisplay
  }
  zle -N fzf-insert-path

  # ── Keybindings (emacs keymap, CSI-u from Kitty)
  bindkey -M emacs $'\e[104;9u' fzf-history-widget  # Cmd+H
  bindkey -M emacs $'\e[105;9u' fzf-insert-path     # Cmd+I

  _cd-yazi() {
    local tmp="$(mktemp -t "yazi-cwd.XXXXXX")" cwd
    yazi "$@" --cwd-file="$tmp"
    if cwd="$(command cat -- "$tmp")" && [ -n "$cwd" ] && [ "$cwd" != "$PWD" ]; then
      builtin cd -- "$cwd"
    fi
    rm -f -- "$tmp"
  }

  _yazi-jump() {
    local tmp cwd
    if ! tmp="$(mktemp -t "yazi-cwd.XXXXX")"; then
      return 1
    fi

    echo "yazi \"$PWD\" --cwd-file=\"$tmp\""
    yazi "$PWD" --cwd-file="$tmp"

    if cwd="$(cat -- "$tmp")" && [ -n "$cwd" ] && [ "$cwd" != "$PWD" ]; then
      cd -- "$cwd"
    fi

    rm -f -- "$tmp"
    zle reset-prompt
  }

  _fzf_ripgrep() {
    fzf-ripgrep
  }

  _clear-reset() {
    clear
    zle reset-prompt
  }

  _cd-fzf() {
    local selected_dir

    # Save cursor position without clearing the line
    tput sc

    # Use command substitution to capture fzf-cd's output
    selected_dir=$(ls-all 2>/dev/null | source fzf-cd)
    local fzf_exit_status=$?

    # Restore cursor position
    tput rc
    tput el # Clear to the end of the line to prevent leftover artifacts

    if ((fzf_exit_status != 0)); then
      zle reset-prompt
      return 0 # return ok exit status so no beep, otherwise: $fzf_exit_status
    fi

    # If a directory was selected, change to it
    if [[ -n "$selected_dir" && -d "$selected_dir" ]]; then
      cd "$selected_dir" || return 1
      zle reset-prompt
      return 0
    fi

    # Reset prompt if error selecting dir
    zle reset-prompt
    return 1
  }

  _fzf-jump() {
    local selected_dir

    # Save cursor position without clearing the line
    tput sc

    # Use command substitution to capture fzf-cd's output
    selected_dir=$(fd --type d --no-hidden --max-depth 10 2>/dev/null | source fzf-cd)
    local fzf_exit_status=$?

    # Restore cursor position
    tput rc
    tput el # Clear to the end of the line to prevent leftover artifacts

    if ((fzf_exit_status != 0)); then
      zle reset-prompt
      return $fzf_exit_status
    fi

    # If a directory was selected, change to it
    if [[ -n "$selected_dir" && -d "$selected_dir" ]]; then
      cd "$selected_dir" || return 1
      zle reset-prompt
      return 0
    fi

    # Reset prompt if no directory was selected
    zle reset-prompt
    return 1
  }

  _cd-up() {
    cd ..
    zle reset-prompt
  }

  _reveal() {
    cd ..
    zle reset-prompt
  }

  _fzf-insert-path() {
    local selected
    selected="$(ls-media | fzf -0)" || return
    LBUFFER+="${selected}"
  }

  # Register functions with ZLE
  zle -N _cd-yazi
  zle -N _fzf-find-files
  zle -N _fzf-jump
  zle -N _fzf_ripgrep
  zle -N _clear-reset
  zle -N _cd-fzf
  zle -N _cd-up
  zle -N _fzf-insert-path

  # unbind meta ctrl+space
  bindkey -r '\M-^@'
  bindkey -r '\M-^?'

  # Bind keys to functions
  #
  bindkey -s '^@' "_cd-yazi && clear\n"
  # bindkey '^f' _fzf-find-files
  bindkey '^f' _fzf_ripgrep
  #bindkey '^k' _clear-reset
  bindkey '^M' _magic-enter
  bindkey '^o' _cd-fzf
  #bindkey '^u' cd-up
  bindkey '^[j' _fzf-jump # Alt+J
  bindkey '^[i' _fzf-insert-path
  bindkey -s '^k' "clear\n"
  bindkey -s '^l' "clear\n"

  function open_finder_pwd() {
    open .
  }
  zle -N open-finder-pwd open_finder_pwd
  bindkey "\e\x12" open-finder-pwd
fi
