# ===============================================================================
# ZSH Interactive Shell Configuration
# Custom functions and keybindings for enhanced terminal experience
# ===============================================================================

# Check if interactive shell to avoid non-interactive errors
if [[ -o interactive ]]; then
  # Ensure add-zsh-hook is available
  autoload -Uz add-zsh-hook

  # ===============================================================================
  # Helper Functions
  # ===============================================================================

  # Optional lazy loader for fzf initialization
  _ensure_fzf_loaded() {
    # If you used a precmd deferral, call it here once when a widget is hit
    typeset -f _lazy_fzf_precmd >/dev/null && {
      add-zsh-hook -d precmd _lazy_fzf_precmd 2>/dev/null
      _lazy_fzf_precmd
    }
  }

  # ===============================================================================
  # Navigation Functions
  # ===============================================================================

  # Yazi file manager with directory change support
  _cd-yazi() {
    local tmp="$(mktemp -t "yazi-cwd.XXXXXX")" cwd
    yazi "$@" --cwd-file="$tmp"
    if cwd="$(command cat -- "$tmp")" && [ -n "$cwd" ] && [ "$cwd" != "$PWD" ]; then
      builtin cd -- "$cwd"
    fi
    rm -f -- "$tmp"
  }

  # Yazi jump widget for ZLE
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

  fzf-dir-widget() {
    fd . --type=directory $HOME/Music $PWD | fzf | tr '\n' ' '
  }

  # FZF directory navigation with ls-all
  _cd-fzf() {
    local selected_dir

    # Save cursor position without clearing the line
    #tput sc

    # move up one line and clear it
    printf '\033[2K'
    
    # Use command substitution to capture fzf-cd's output
    selected_dir=$(ls-all 2>/dev/null | source fzf-cd)
    local fzf_exit_status=$?

    # Restore cursor position
    #tput rc
    #tput el # Clear to the end of the line to prevent leftover artifacts

    if ((fzf_exit_status != 0)); then
      zle reset-prompt
      return 0 # return ok exit status so no beep
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

  # FZF jump to subdirectories
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

  # Navigate up one directory
  _cd-up() {
    cd ..
    zle reset-prompt
  }

  # Open Finder in current directory (macOS)
  open_finder_pwd() {
    open .
  }

  # ===============================================================================
  # FZF Widgets
  # ===============================================================================

  # FZF history picker (insert selected command)
  fzf-history-widget() {
    setopt localoptions pipefail
    _ensure_fzf_loaded

    # Reverse chronological, strip numbers
    local sel
    sel=$(fc -rl 1 | sed 's/^[[:space:]]*[0-9[:space:]]*//' | fzf --tac --no-sort --exact --ansi \
           --prompt='history> ' --height=40% --border) || return
    LBUFFER=$sel
    zle redisplay
  }

  # Insert file path(s) from fzf (current dir; multi-select)
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

  # FZF insert path using ls-media
  _fzf-insert-path() {
    local selected
    selected="$(ls-media | fzf -0)" || return
    LBUFFER+="${selected}"
  }

  # FZF ripgrep integration
  _fzf_ripgrep() {
    fzf-ripgrep
  }

  # ===============================================================================
  # Utility Functions
  # ===============================================================================

  # Clear screen and reset prompt
  _clear-reset() {
    clear
    zle reset-prompt
  }

  # ===============================================================================
  # ZLE Widget Registration
  # ===============================================================================

  # Register all functions with ZLE
  zle -N fzf-history-widget
  zle -N fzf-insert-path
  zle -N _cd-yazi
  zle -N _yazi-jump
  zle -N _fzf-jump
  zle -N _fzf_ripgrep
  zle -N _clear-reset
  zle -N _cd-fzf
  zle -N fzf-cd
  zle -N _cd-up
  zle -N _fzf-insert-path
  zle -N open-finder-pwd open_finder_pwd
  zle -N fzf-dir-widget

  # ===============================================================================
  # Key Bindings
  # ===============================================================================

  # Unbind conflicting meta keys
  bindkey -r '\M-^@'
  bindkey -r '\M-^?'

  # Primary key bindings
  bindkey -s '^@'      "_cd-yazi && clear\n"     # Ctrl+Space: Yazi file manager
  bindkey '^f'         _fzf_ripgrep              # Ctrl+F: FZF ripgrep
  bindkey '^M'         _magic-enter              # Enter: Magic enter
  bindkey '^o'         _cd-fzf                   # Ctrl+O: FZF directory navigation
  bindkey '^[j'        _fzf-jump                 # Alt+J: FZF jump to subdirectory
  bindkey '^[i'        _fzf-insert-path          # Alt+I: Insert path with FZF
  bindkey -s '^k'      "clear\n"                 # Ctrl+K: Clear screen
  bindkey -s '^l'      "clear\n"                 # Ctrl+L: Clear screen
  bindkey "\e\x12"     open-finder-pwd           # Cmd+R: Open Finder (macOS)

  # FZF keybindings (emacs keymap, CSI-u from Kitty)
  bindkey -M emacs $'\e[104;9u' fzf-history-widget  # Cmd+H: FZF history
  bindkey -M emacs $'\e[105;9u' fzf-insert-path     # Cmd+I: FZF insert path
  
  # Bind keys to functions
  #
  bindkey -s '^@' "_cd-yazi && clear\n"
  bindkey '^f' _fzf_ripgrep
  bindkey '^M' _magic-enter
  bindkey '^o' _cd-fzf
  bindkey '^[[1;9o' _cd-fzf
  bindkey '^[j' _fzf-jump # Alt+J
  bindkey '^[i' _fzf-insert-path
  bindkey -s '^k' "clear\n"
  bindkey -s '^l' "clear\n"
  bindkey '^T' fzf-file-widget
  bindkey '^D' fzf-dir-widget
  bindkey '^U' kill-whole-line
fi

