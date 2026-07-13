##### ───────────────  FAST + STABLE ZSH STARTUP (SAFE BASELINE)  ────────────── #####

# Assume $ZSH_CONFIG_DIR and $ZSH_AUTOLOAD_DIR are exported from .zshenv
set -o emacs
setopt autocd autopushd

# Cache dir
: ${XDG_CACHE_HOME:="$HOME/.cache"}
typeset -g ZSH_CACHE_DIR="$XDG_CACHE_HOME/zsh"
mkdir -p "$ZSH_CACHE_DIR"

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

# --- Lazy fzf init on first prompt ---
_lazy_fzf_precmd() {
  # only source if files exist
  [[ -r "$ZSH_AUTOLOAD_DIR/fzf.zsh" ]]             && source "$ZSH_AUTOLOAD_DIR/fzf.zsh"
  [[ -r "$ZSH_AUTOLOAD_DIR/fzf-completions.zsh" ]] && source "$ZSH_AUTOLOAD_DIR/fzf-completions.zsh"
  [[ -r "$ZSH_AUTOLOAD_DIR/fzf-custom.zsh" ]]      && source "$ZSH_AUTOLOAD_DIR/fzf-custom.zsh"
  [[ -r "$ZSH_AUTOLOAD_DIR/fzf-marks.zsh" ]]       && source "$ZSH_AUTOLOAD_DIR/fzf-marks.zsh"  # optional

  add-zsh-hook -d precmd _lazy_fzf_precmd
}
add-zsh-hook precmd _lazy_fzf_precmd

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
  if ! source $config_file; then
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

# --- Lazy starship (first prompt render) ---
if (( $+commands[starship] )); then
  _lazy_starship_precmd() {
    local _star_init="$ZSH_CACHE_DIR/starship-init.zsh"
    if [[ ! -s $_star_init || $(command -v starship) -nt $_star_init ]]; then
      command starship init zsh >! "$_star_init"
    fi
    source "$_star_init"
    # run the real precmds starship registered (if any), then unhook ourselves
    add-zsh-hook -d precmd _lazy_starship_precmd
    typeset -pm precmd_functions >/dev/null 2>&1  # touch to rebuild widgets
  }
  add-zsh-hook precmd _lazy_starship_precmd
fi

typeset -g _prompt_spacer_seen=0
_prompt_spacer_precmd() {
  if (( _prompt_spacer_seen )); then
    print
  else
    _prompt_spacer_seen=1
  fi
}
add-zsh-hook precmd _prompt_spacer_precmd

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
