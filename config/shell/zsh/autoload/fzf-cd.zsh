# ── cd-fzf widget ─────────────────────────────────────────────────────────────
#
# Config: single-quoted strings so variables/globs expand at invocation time.
# N  = null-glob (no error on no match)
# /  = directories only

typeset -ga _cd_fzf_globs=(
  '$HOME/*(N/)'
  '$HOME/Movies/*(N/)'
  '/Volumes/*/Movies/Porn/*(N/)'
)

typeset -ga _cd_fzf_commands=(
  # 'fd --type d --max-depth 4 . $HOME/Documents'
  # 'fd --type d . /Volumes/Data/Projects'
)

# ── path generator ────────────────────────────────────────────────────────────

function _cd_fzf_paths() {
  local glob
  for glob in "${_cd_fzf_globs[@]}"; do
    # eval expands vars + glob qualifiers at call time; array avoids word-split
    local -a expanded
    eval "expanded=( ${glob} )" 2>/dev/null
    (( ${#expanded} )) && print -l -- "${expanded[@]}"
  done

  local -a pids
  for cmd in "${_cd_fzf_commands[@]}"; do
    eval "$cmd" 2>/dev/null &
    pids+=($!)
  done
  wait "${pids[@]}"
}

# ── widget ────────────────────────────────────────────────────────────────────

function _cd_fzf_pick() {
  fzf --height=50% --reverse \
      --prompt='cd ❯ ' \
      --preview='eza --tree --level=2 --color=always {} 2>/dev/null || ls -la {}'
}

function _cd_fzf_widget() {
  local selected
  selected=$(
    _cd_fzf_paths \
      | sort -u \
      | _cd_fzf_pick
  )

  if [[ -n "$selected" ]]; then
    BUFFER="cd ${(q-)selected}"
    zle accept-line
  else
    zle reset-prompt
  fi
}

zle -N _cd_fzf_widget
bindkey '^G' _cd_fzf_widget
