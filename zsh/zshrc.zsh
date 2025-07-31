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

config_files=(
  $ZSH_AUTOLOAD_DIR/homebrew.zsh
  $ZSH_AUTOLOAD_DIR/completions.zsh
  $ZSH_AUTOLOAD_DIR/alias.zsh
  $ZSH_AUTOLOAD_DIR/broot.zsh
  $ZSH_AUTOLOAD_DIR/fzf.zsh
  $ZSH_AUTOLOAD_DIR/fzf-completions.zsh
  $ZSH_AUTOLOAD_DIR/fzf-custom.zsh
  $ZSH_AUTOLOAD_DIR/drive-index.zsh
  $ZSH_AUTOLOAD_DIR/ffmpeg.zsh
  $ZSH_AUTOLOAD_DIR/function.zsh
  $ZSH_AUTOLOAD_DIR/history.zsh
  $ZSH_AUTOLOAD_DIR/media.zsh
  $ZSH_AUTOLOAD_DIR/prompt-middle.zsh
  $ZSH_AUTOLOAD_DIR/prompt.zsh
  $ZSH_AUTOLOAD_DIR/rsync.zsh
  $ZSH_AUTOLOAD_DIR/yt-dlp.zsh
  $ZSH_AUTOLOAD_DIR/starship.zsh
  $ZSH_AUTOLOAD_DIR/fzf-marks.zsh
  $ZSH_AUTOLOAD_DIR/keybindings.zsh
)
for config_file in $config_files; do
  print -nP "%F{green}.%f"
  if ! source $config_file; then
    print -P "%F{red}Error sourcing $config_file. Skipping...%f"
  fi
done
print -P "ok"

# ---- Cached brew shellenv (avoids spawning `brew` every startup) ----
if (( $+commands[brew] )); then
  local _brew_env="$ZSH_CACHE_DIR/brew.env"
  if [[ ! -s $_brew_env || $(command -v brew) -nt $_brew_env ]]; then
    command brew shellenv >! "$_brew_env"
  fi
  source "$_brew_env"
fi

# ---- Cached starship init glue (safe; starship still runs at prompt draw) ----
if (( $+commands[starship] )); then
  local _star_init="$ZSH_CACHE_DIR/starship-init.zsh"
  if [[ ! -s $_star_init || $(command -v starship) -nt $_star_init ]]; then
    command starship init zsh >! "$_star_init"
  fi
  source "$_star_init"
fi

# LM Studio PATH (kept)
export PATH="$PATH:/Users/nom/.cache/lm-studio/bin"
