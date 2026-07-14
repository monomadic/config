# COMPLETIONS
#
# useful links
#		- https://github.com/zsh-users/zsh-completions
#

# 1Password — cached; spawning `op` costs ~40ms per shell
if (( $+commands[op] )); then
  _op_init="$ZSH_CACHE_DIR/op-completion.zsh"
  if [[ ! -s $_op_init || $(command -v op) -nt $_op_init ]]; then
    command op completion zsh >! "$_op_init"
  fi
  source "$_op_init"
  unset _op_init
fi

# television — cached. Note: this binds ^R, which keybindings.zsh (sourced
# later) and the lazy fzf loader both deliberately override.
if (( $+commands[tv] )); then
  _tv_init="$ZSH_CACHE_DIR/tv-init.zsh"
  if [[ ! -s $_tv_init || $(command -v tv) -nt $_tv_init ]]; then
    command tv init zsh >! "$_tv_init"
  fi
  source "$_tv_init"
  unset _tv_init
fi

#eval "$(rmrfrs --completions zsh)" &> /dev/null

# Generated completions are refreshed explicitly with `refresh-zsh-completions`.

# `fpath` and `compinit` are handled centrally in `~/.zshrc`; the dump is
# rebuilt automatically when a completion dir changes, so no manual compdefs.

# --- command: e <file> -> open in default editor ---
e() {
  $EDITOR "$@"
}

zstyle ':completion:*:*:e:*' sort false   # avoid slow sorting on big sets
_e() {
  setopt localoptions no_errexit noshwordsplit

  local cur="${words[CURRENT]}"
  local -a m

  # keep <TAB> instant until user types something meaningful
  # if (( ${#cur} < 2 )); then
  #   _files
  #   return 0
  # fi

  m=("${(@f)$(fd --color=never --follow \
        --max-depth 6 --max-results 200 \
        --exclude .git --exclude Library --exclude .config --exclude .cache --exclude .local \
        --glob "${cur}*" . 2>/dev/null)}")

  (( ${#m} )) && compadd -Q -- "${m[@]}" || _files
  return 0
}

compdef _e e

_edit-script() {
  local -a names descs dirs
  local file header base

  dirs=(
    "$DOTFILES_DIR/config/zsh/bin"
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
compdef _edit-script edit-script

#compdef dl-porn=yt-dlp
