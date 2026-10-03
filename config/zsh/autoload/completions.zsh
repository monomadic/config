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

zstyle ':completion:*:*:e:*' sort false   # keep our depth-first order
_e() {
  setopt localoptions no_errexit noshwordsplit

  local cur="${words[CURRENT]}"
  local -a m

  # Candidates are handed to fzf (TAB is bound to fzf_completion), so this
  # only has to produce a good *superset*: fzf does the narrowing.
  #   - substring match on the full path, not a basename-prefix glob, so
  #     "keyb" finds bin/ls-keybindings and "zsh/key" works
  #   - files and directories; --hidden so dotfiles show up
  #   - fd walks in parallel, so --max-results alone returns an arbitrary
  #     subset: fetch generously and rank shallow paths first instead
  m=("${(@f)$(fd --color=never --follow --hidden \
        --max-depth 6 --max-results 3000 --strip-cwd-prefix \
        --fixed-strings --full-path \
        --exclude .git --exclude Library --exclude .cache --exclude .local \
        --exclude node_modules --exclude target \
        -- "$cur" 2>/dev/null \
      | awk -F/ '{ print NF "\t" $0 }' | sort -s -n -k1,1 | cut -f2-)}")

  (( ${#m} )) && compadd -Q -f -- "${m[@]}" || _files
  return 0
}

compdef _e e

_edit-script() {
  local -a names descs dirs
  local file header base

  dirs=(
    "$DOTFILES_DIR/bin"
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
