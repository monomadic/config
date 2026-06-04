##### ───────────────  FAST + STABLE ZSH STARTUP (SAFE BASELINE)  ────────────── #####

# Assume $ZSH_CONFIG_DIR and $ZSH_AUTOLOAD_DIR are exported from .zshenv
set -o emacs
setopt autocd autopushd

# Cache dir
: ${XDG_CACHE_HOME:="$HOME/.cache"}
typeset -g ZSH_CACHE_DIR="$XDG_CACHE_HOME/zsh"
mkdir -p "$ZSH_CACHE_DIR"

# Completion search path FIRST, then a single compinit
fpath=(
  "$ZSH_CONFIG_DIR/completions"
  $fpath
)

# Completion styles (minimal)
zstyle ':completion:*:*:*:default' menu yes select search
zstyle ':completion:*' menu select
zstyle ':completion:*' special-dirs true
zstyle ':completion:*' list-dirs first
zstyle ':completion:*' matcher-list 'm:{a-z}={A-Z}'

# Init completion once, with cached dump
autoload -Uz compinit
typeset -g COMPDUMP="$ZSH_CACHE_DIR/.zcompdump-$ZSH_VERSION"
# -C: skip security scan after first trusted run; -i: ignore insecure dirs instead of prompting
compinit -d "$COMPDUMP" -C -i
# Compile ONLY the dump (safe & common)
[[ -s $COMPDUMP && ! -e $COMPDUMP.zwc ]] && zcompile -U -- "$COMPDUMP"

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
  $ZSH_AUTOLOAD_DIR/fzf-cd.zsh
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

# television
eval "$(tv init zsh)"

# LM Studio PATH
export PATH="$PATH:$HOME/.cache/lm-studio/bin"
export PATH="$PATH:$HOME/.lmstudio/bin"

# Added by LM Studio CLI (lms)
export PATH="$PATH:/Users/nom/.cache/lm-studio/bin"
# End of LM Studio CLI section

# print -P "ok"

fpath+=~/.zfunc; autoload -Uz compinit; compinit
