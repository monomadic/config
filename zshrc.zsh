##### ───────────────  FAST + STABLE ZSH STARTUP (SAFE BASELINE)  ────────────── #####

# Assume $ZSH_CONFIG_DIR and $ZSH_AUTOLOAD_DIR are exported from .zshenv
set -o emacs
setopt autocd autopushd
setopt extendedglob # extra globs like /Volumes/*~*Backup*/Movies

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
  $ZSH_AUTOLOAD_DIR/completions.zsh
  $ZSH_AUTOLOAD_DIR/alias.zsh
  $ZSH_AUTOLOAD_DIR/broot.zsh
  # $ZSH_AUTOLOAD_DIR/fzf.zsh
  # $ZSH_AUTOLOAD_DIR/fzf-completions.zsh
  # $ZSH_AUTOLOAD_DIR/fzf-custom.zsh
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
  # $ZSH_AUTOLOAD_DIR/fzf-marks.zsh
  $ZSH_AUTOLOAD_DIR/keybindings.zsh
  $ZSH_AUTOLOAD_DIR/kitty.zsh
)
for config_file in $config_files; do
  print -nP "%F{green}.%f"
  if ! source $config_file; then
    print -P "%F{red}Error sourcing $config_file. Skipping...%f"
  fi
done
print -P "ok"

# kitty tab colors
#kitty @ set-tab-color --match title:"mpv-play" active_bg="#A85FFF" active_fg="#050F63" inactive_fg="#A85FFF" inactive_bg="#030D43"
#kitty @ set-tab-color --match title:"kitty.conf" active_bg="#44F273" active_fg="#050F63" inactive_fg="#38F273" inactive_bg="#030D43"
#kitty set-title "  $PWD"

# ---- Cached brew shellenv (avoids spawning `brew` every startup) ----
if (( $+commands[brew] )); then
  local _brew_env="$ZSH_CACHE_DIR/brew.env"
  if [[ ! -s $_brew_env || $(command -v brew) -nt $_brew_env ]]; then
    command brew shellenv >! "$_brew_env"
  fi
  source "$_brew_env"
fi

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

# Completion for: edit-script <name>
_edit_script() {
  local -a names descs dirs
  local file header base

  dirs=(
    "$HOME/config/bin"
    "$HOME/config/zsh/bin"
  )

  for dir in $dirs; do
    [[ -d $dir ]] || continue

    # regular files, nullglob, no error if none
    for file in $dir/*(N-); do
      [[ -r $file ]] || continue

      # Shebang filter => treat as "script"
      if IFS= read -r header <"$file"; then
        [[ $header == '#!'* ]] || continue
      else
        continue
      fi

      base=${file:t}

      names+="$base"
      descs+="$file"
    done
  done

  (( ${#names} )) || return 1

  # Show: name  —  /full/path
  compadd -d descs -- $names
}

compdef _edit_script edit-script
# LM Studio PATH (kept)
export PATH="$PATH:/Users/nom/.cache/lm-studio/bin"

# Added by LM Studio CLI (lms)
export PATH="$PATH:/Users/nom/.lmstudio/bin"
# End of LM Studio CLI section
