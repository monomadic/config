##### ───────────────  FAST + STABLE ZSH STARTUP (SAFE BASELINE)  ────────────── #####

# Assume $ZSH_CONFIG_DIR and $ZSH_AUTOLOAD_DIR are exported from .zshenv
set -o emacs
setopt autocd autopushd

# Cache dir
: ${XDG_CACHE_HOME:="$HOME/.cache"}
typeset -g ZSH_CACHE_DIR="$XDG_CACHE_HOME/zsh"
mkdir -p "$ZSH_CACHE_DIR"

export ICLOUD_HOME=$HOME"/Library/Mobile Documents/com~apple~CloudDocs"
export TAILSCALE_DNS_NAME="eel-beardie.ts.net"

export YT_DLP_ARCHIVE_FILE="$ICLOUD_HOME/Sync/archive.txt"
export YT_DLP_BATCH_FILE="$ICLOUD_HOME/Sync/links.txt"
export YT_DLP_JSON_DIR="$ICLOUD_HOME/Sync/JSON"
export YT_DLP_TEMP_DIR="$HOME/.cache/yt-dlp-tmp"
export YT_DLP_THUMBCACHE_DIR="$ICLOUD_HOME/Sync/ThumbCache"

export COLOR_C64_DARK_BLUE=#030D43
export COLOR_C64_LIGHT_BLUE=#7A86D1

typeset -U manpath
manpath=($manpath)

JUMP_DIRS=(
  "/Volumes/**/Movies/**/*.mp4"
  "$HOME/Movies/**/*.mp4"
)

BASE_GLOBS=(
  "/Volumes/*/"
  "$HOME/"
  "$ICLOUD_HOME"
)

ADULT_GLOBS=(
  "/Volumes/*/Movies/Porn/**/*.mp4"
  "$HOME/Movies/Porn/**/*.mp4"
)

TABLATURE_GLOBS=(
  "/Volumes/*/Tablature/**/*.pdf"
  "$HOME/Music/Tablature/**/*.pdf"
)

export ZSH_SCRIPT_PATHS=(
  $ZSH_CONFIG_DIR/bin
)

if [[ -n ${KITTY_WINDOW_ID:-} || -n ${KITTY_LISTEN_ON:-} ]]; then
  export EDITOR=kitty-helix
else
  export EDITOR=hx
fi
export VISUAL="$EDITOR"
export TEMPLATE_BASE_DIR=$DOTFILES_DIR/config/neovim/templates

export TABLATURE_DIR="$ICLOUD_HOME/Music/Tablature"
export TUTORIALS_PATH=$HOME/Movies/Tutorials

export LOCAL_MEDIA_PATHS="$HOME/Movies/Porn/"
export EXTERNAL_MEDIA_PATHS="/Volumes/*/Movies/Porn/"
export INDEX_DIR="$HOME/.indexes"
export PRIVATE_PHOTOS_LIBRARY="$HOME/Media/Private/Private.photoslibrary"

export HELIX_USE_OSC52=true
export WASMTIME_HOME=$HOME/.wasmtime
path+=($WASMTIME_HOME/bin)
export GHQ_ROOT=$HOME/src

export FZF_DEFAULT_OPTS="$FZF_DEFAULT_OPTS --bind 'ctrl-a:select-all' --layout=reverse --cycle --preview-window=noborder --highlight-line --no-separator --gutter=' ' --no-border --inline-info --bind 'ctrl-u:unix-line-discard'"
export FZF_COMPLETION_TRIGGER='\t'
export FZF_PREVIEW_COMMAND='fzf-preview {}'
export SKIM_DEFAULT_COMMAND="fd . --max-depth=3"

# Completion search path FIRST, then a single compinit.
# site-functions must be present here: brew shellenv (sourced later) also adds
# it, but compinit only registers completions from dirs in fpath at this point.
fpath=(
  "$ZSH_CONFIG_DIR/completions"
  "${HOMEBREW_PREFIX:-/opt/homebrew}/share/zsh/site-functions"
  $fpath
)

# Completion styles (minimal)
zstyle ':completion:*:*:*:default' menu yes select search
zstyle ':completion:*' menu select
zstyle ':completion:*' special-dirs true
zstyle ':completion:*' list-dirs first
zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}'

# Init completion once, with cached dump.
# Fast path (-C) trusts the dump; full rebuild only when a completion dir has
# gained/lost files since the dump was written, so new completions are picked
# up on the next shell without any manual compdef bookkeeping.
autoload -Uz compinit
typeset -g COMPDUMP="$ZSH_CACHE_DIR/.zcompdump-$ZSH_VERSION"
if [[ -s $COMPDUMP
   && ! $ZSH_CONFIG_DIR/completions -nt $COMPDUMP
   && ! ${HOMEBREW_PREFIX:-/opt/homebrew}/share/zsh/site-functions -nt $COMPDUMP ]]; then
  compinit -d "$COMPDUMP" -C -i
else
  compinit -d "$COMPDUMP" -i
fi
[[ -s $COMPDUMP && (! -e $COMPDUMP.zwc || $COMPDUMP -nt $COMPDUMP.zwc) ]] && zcompile -U -- "$COMPDUMP"

# Autoload common helper module
autoload -Uz add-zsh-hook

_source_zsh_module() {
  local name="$1"
  local file

  for file in "$ZSH_AUTOLOAD_DIR/$name" "$ZSH_DOTFILES_DIR/autoload/$name"; do
    [[ -r "$file" ]] || continue
    source "$file"
    return
  done

  print -P "%F{red}Missing zsh module: $name%f"
  return 1
}

_load_fzf_once() {
  [[ "${_FZF_SHELL_LOADED:-0}" == 1 ]] && return 0

  _source_zsh_module fzf.zsh
  _source_zsh_module fzf-completions.zsh
  _source_zsh_module fzf-custom.zsh
  _source_zsh_module fzf-marks.zsh
  typeset -g _FZF_SHELL_LOADED=1
}

_fzf_lazy_widget() {
  local widget="$WIDGET"
  _load_fzf_once || return
  zle "$widget" "$@"
}

if [[ -o interactive ]]; then
  zle -N fzf-file-widget _fzf_lazy_widget
  zle -N fzf-history-widget _fzf_lazy_widget
  zle -N fzf_completion _fzf_lazy_widget
fi

config_files=(
  $ZSH_AUTOLOAD_DIR/homebrew.zsh
  $ZSH_AUTOLOAD_DIR/colors.zsh
  $ZSH_AUTOLOAD_DIR/completions.zsh
  $ZSH_AUTOLOAD_DIR/alias.zsh
  $ZSH_AUTOLOAD_DIR/broot.zsh
  # $ZSH_AUTOLOAD_DIR/fzf.zsh
  # $ZSH_AUTOLOAD_DIR/fzf-completions.zsh
  # $ZSH_AUTOLOAD_DIR/fzf-custom.zsh
  $ZSH_AUTOLOAD_DIR/index.zsh
  $ZSH_AUTOLOAD_DIR/ffmpeg.zsh
  $ZSH_AUTOLOAD_DIR/function.zsh
  $ZSH_AUTOLOAD_DIR/history.zsh
  $ZSH_AUTOLOAD_DIR/media.zsh
  $ZSH_AUTOLOAD_DIR/prompt-middle.zsh
  $ZSH_AUTOLOAD_DIR/prompt.zsh
  $ZSH_AUTOLOAD_DIR/rsync.zsh
  $ZSH_AUTOLOAD_DIR/yt-dlp.zsh
  # $ZSH_AUTOLOAD_DIR/fzf-marks.zsh
  $ZSH_AUTOLOAD_DIR/keybindings.zsh
  $ZSH_AUTOLOAD_DIR/kitty.zsh
)
for config_file in $config_files; do
  # print -nP "%F{green}.%f"
  if ! _source_zsh_module "${config_file:t}"; then
    print -P "%F{red}Error sourcing $config_file. Skipping...%f"
  fi
done
unset config_file config_files

# kitty tab colors
#kitty @ set-tab-color --match title:"mpv-play" active_bg="#A85FFF" active_fg="#050F63" inactive_fg="#A85FFF" inactive_bg="#030D43"
#kitty @ set-tab-color --match title:"kitty.conf" active_bg="#44F273" active_fg="#050F63" inactive_fg="#38F273" inactive_bg="#030D43"
#kitty set-title "  $PWD"

# ---- Cached brew shellenv (avoids spawning `brew` every startup) ----
if (( $+commands[brew] )); then
  typeset _brew_env="$ZSH_CACHE_DIR/brew.env"
  if [[ ! -s $_brew_env || $(command -v brew) -nt $_brew_env ]]; then
    command brew shellenv >! "$_brew_env"
  fi
  source "$_brew_env"
fi

_zellij_kitty_spacing() {
  [[ -n "${KITTY_WINDOW_ID:-}" ]] || return 1
  (( $+commands[kitty] )) || return 1

  local -a remote_args=()
  local listen_on="${KITTY_LISTEN_ON:-}"
  [[ -z "$listen_on" && -S /tmp/kitty ]] && listen_on="unix:/tmp/kitty"
  [[ -n "$listen_on" ]] && remote_args=(--to "$listen_on")

  kitty @ "${remote_args[@]}" set-spacing --match "id:$KITTY_WINDOW_ID" "$@" >/dev/null 2>&1 ||
    kitty @ "${remote_args[@]}" set-spacing --match "state:self" "$@" >/dev/null 2>&1
}

zellij() {
  if [[ -n "${ZELLIJ:-}" || -z "${KITTY_WINDOW_ID:-}" ]]; then
    command zellij "$@"
    return
  fi

  local spacing_changed=0 zellij_status=0
  _zellij_kitty_spacing margin=0 padding=0 && spacing_changed=1

  {
    command zellij "$@"
    zellij_status=$?
  } always {
    if (( spacing_changed )); then
      _zellij_kitty_spacing margin=default padding=default
    fi
  }

  return "$zellij_status"
}

_zellij_auto_attach_ssh() {
  [[ -o interactive ]] || return
  [[ -n "${SSH_CONNECTION:-}${SSH_TTY:-}" ]] || return
  [[ -z "${ZELLIJ:-}" ]] || return
  [[ "${ZELLIJ_AUTO_ATTACH_SSH:-1}" != "0" ]] || return
  [[ -t 0 && -t 1 ]] || return
  [[ "${TERM:-}" != "dumb" ]] || return
  (( $+commands[zellij] )) || return

  local session="${ZELLIJ_AUTO_ATTACH_SESSION:-ssh-$(hostname -s)}"
  zellij attach --create "$session"
}
_zellij_auto_attach_ssh

# --- Prompt: pimped, a fast native renderer (utils/pimped → ~/.local/bin/pimped) ---
# A static Rust binary: renders in well under a millisecond, spawns no
# subprocesses, and skips git entirely on /Volumes mounts, so a stale SMB share
# can never wedge the prompt. Falls back to starship where the binary isn't built.
# PROMPT holds the $(pimped …) call itself under prompt_subst, so it re-renders on
# every redraw — including `zle reset-prompt` from cd widgets (fzf-cd, yazi) that
# don't fire precmd. The exit status is captured in precmd (before any hook can
# clobber $?) into a global the subst reads; the blank-line spacer lives there too.
typeset -g _prompt_spacer_seen=0
if (( $+commands[pimped] )); then
  setopt prompt_subst
  typeset -gi _pimped_status=0
  _prompt_render_precmd() {
    _pimped_status=$?
    if (( _prompt_spacer_seen )); then print; else _prompt_spacer_seen=1; fi
  }
  add-zsh-hook precmd _prompt_render_precmd
  PROMPT='$(command pimped "$_pimped_status" "${(%):-%m}")'
  RPROMPT=''
elif (( $+commands[starship] )); then
  _lazy_starship_precmd() {
    local _star_init="$ZSH_CACHE_DIR/starship-init.zsh"
    if [[ ! -s $_star_init || $(command -v starship) -nt $_star_init ]]; then
      command starship init zsh >! "$_star_init"
    fi
    source "$_star_init"
    add-zsh-hook -d precmd _lazy_starship_precmd
    typeset -pm precmd_functions >/dev/null 2>&1  # touch to rebuild widgets
  }
  add-zsh-hook precmd _lazy_starship_precmd
  _prompt_spacer_precmd() {
    if (( _prompt_spacer_seen )); then print; else _prompt_spacer_seen=1; fi
  }
  add-zsh-hook precmd _prompt_spacer_precmd
fi

# television init lives in autoload/completions.zsh (cached).
# LM Studio PATH entries come from ~/.bin/init-path.

# >>> elio shell integration >>>
elio() {
    case "${1-}" in
        shell|-*)
            command elio "$@"
            return $?
            ;;
    esac

    local tmp cwd status_code
    tmp="$(mktemp -t "elio-cwd.XXXXXX")" || return
    command elio --cwd-file "$tmp" "$@"
    status_code=$?

    if [ -s "$tmp" ]; then
        cwd="$(cat -- "$tmp")"
        rm -f -- "$tmp"
        if [ -n "$cwd" ] && [ "$cwd" != "$PWD" ] && [ -d "$cwd" ]; then
            cd -- "$cwd" || return $?
        fi
    else
        rm -f -- "$tmp"
    fi

    return "$status_code"
}
# <<< elio shell integration <<<
