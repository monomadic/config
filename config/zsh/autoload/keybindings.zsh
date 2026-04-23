# ===============================================================================
# ZSH Interactive Shell Configuration
# Custom functions and keybindings for enhanced terminal experience
# ===============================================================================

# Check if interactive shell to avoid non-interactive errors
if [[ -o interactive ]]; then
  # Ensure add-zsh-hook is available
  autoload -Uz add-zsh-hook
  zmodload zsh/terminfo 2>/dev/null || true

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

  # Yazi file manager with directory change on exit
  _cd-yazi() {
    local tmp="$(mktemp -t "yazi-cwd.XXXXXX")" cwd
    zle -I 2>/dev/null
    kitty-exec '󰘳 yazi ' '#FF44CC' yazi "$@" --cwd-file="$tmp"
    if cwd="$(command cat -- "$tmp")" && [ -n "$cwd" ] && [ "$cwd" != "$PWD" ]; then
      builtin cd -- "$cwd"
    fi
    rm -f -- "$tmp"
    zle reset-prompt 2>/dev/null
  }

  # Yazi jump widget for ZLE
  _yazi-jump() {
    local tmp cwd
    if ! tmp="$(mktemp -t "yazi-cwd.XXXXX")"; then
      return 1
    fi

    zle -I 2>/dev/null
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

  _cd-fzf-select() {
    local selected_dir
    local picker_fn="$1"

    printf '\033[2K'
    _ensure_fzf_loaded

    selected_dir="$($picker_fn)"
    local fzf_exit_status=$?

    if ((fzf_exit_status != 0)); then
      zle reset-prompt
      return 0
    fi

    selected_dir="${selected_dir%/}"

    if [[ -n "$selected_dir" && -d "$selected_dir" ]]; then
      cd -- "$selected_dir" || return 1
      zle reset-prompt
      return 0
    fi

    zle reset-prompt
    return 1
  }

  # Global pinned directory navigation.
  _cd-fzf() {
    _cd-fzf-select _cd_fzf_pick_global
  }

  # Keep the old legacy entrypoint, but route it through the shared picker.
  _cd-fzf-legacy() {
    _cd-fzf
  }

  # Recursive current-directory navigation.
  _cd-fzf-local() {
    _cd-fzf-select _cd_fzf_pick_local
  }

  _fzf-jump() {
    _cd-fzf-local
  }

  # Navigate up one directory
  _cd-up() {
    cd ..
    zle reset-prompt
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

  open-finder-pwd() {
    zle -I
    open --reveal "$PWD"
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
  zle -N _cd-fzf-legacy
  zle -N _cd-fzf-local
  zle -N _cd-up
  zle -N _fzf-insert-path
  zle -N open-finder-pwd
  zle -N fzf-dir-widget

  # ===============================================================================
  # Key Bindings
  # ===============================================================================

  # Unbind conflicting meta keys
  bindkey -r '\M-^@'
  bindkey -r '\M-^?'

  # Primary key bindings
  bindkey '^@'         _cd-yazi                  # Ctrl+Space: Yazi file manager
  bindkey '^f'         _fzf_ripgrep              # Ctrl+F: FZF ripgrep
  bindkey '^M'         accept-line               # Enter: normal accept
  bindkey '^o'         _cd-fzf                   # Ctrl+O: global directory jump
  bindkey '^[g'        _cd-fzf-legacy            # Alt+G: global directory jump
  bindkey '^[j'        _fzf-jump                 # Alt+J: local recursive jump
  bindkey '^[i'        _fzf-insert-path          # Alt+I: Insert path with FZF
  bindkey '^K'         kill-line                 # Ctrl+K: kill to end of line
  bindkey '^L'         clear-screen              # Ctrl+L: clear screen
  bindkey "\e\x12"     open-finder-pwd           # Legacy fallback: open Finder

  # Kitty CSI-u keybindings
  bindkey -M emacs $'\e[104;9u' fzf-history-widget  # Cmd+H
  bindkey -M emacs $'\e[105;9u' fzf-insert-path     # Cmd+I
  bindkey -M emacs $'\e[111;9u' _cd-fzf             # Cmd+O: global jump
  bindkey -M viins $'\e[111;9u' _cd-fzf
  bindkey -M emacs $'\e[111;10u' _cd-fzf-local      # Cmd+Shift+O: local jump
  bindkey -M viins $'\e[111;10u' _cd-fzf-local
  bindkey -M emacs $'\e[13;2u' _magic-enter         # Shift+Enter
  bindkey -M viins $'\e[13;2u' _magic-enter

  # Additional shortcuts
  bindkey '^[[1;9o' _cd-fzf
  bindkey '^T' fzf-file-widget
  bindkey '^D' fzf-dir-widget
  bindkey '^U' kill-whole-line
  bindkey '^R' fzf-history-widget

  # BIND F20
  if [[ -n ${terminfo[kf20]} ]]; then
    bindkey -M emacs "${terminfo[kf20]}" open-finder-pwd
    bindkey -M viins "${terminfo[kf20]}" open-finder-pwd
    bindkey -M vicmd "${terminfo[kf20]}" open-finder-pwd
  else
    # Fallback: literal sequence from `kitty +kitten show-key`
    bindkey -M emacs '^[[33;2~' open-finder-pwd   # EXAMPLE; replace with yours
    bindkey -M viins  '^[[33;2~' open-finder-pwd
    bindkey -M vicmd  '^[[33;2~' open-finder-pwd
  fi
fi
