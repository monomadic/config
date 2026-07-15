# ============================================================================
# File Listing & Navigation
# ============================================================================

#alias l="eza --icons --group-directories-first"
alias l="lsd --group-directories-first"
# alias la="eza --icons --group-directories-first --all"
alias lh="eza --icons --group-directories-first --all"
alias ll="echo && lsd --icon always --long --depth 1 --blocks name --group-directories-first --color always && echo"
alias lla="echo && eza --icons --group-directories-first --all --no-time --no-permissions --no-user -l --ignore-glob '.DS_Store' && echo"
alias lll="lsd --icon always --long --depth 1 --group-directories-first --color always"
alias lln="eza --icons --all -l --sort=date"
alias ll-fzf="eza --icons --color=always --group-directories-first --no-permissions --no-user -l --ignore-glob '.DS_Store' | fzf --ansi"

alias new="print -rl -- **/*(.om[1,200])"

# 10 most-recent files (newest first by default)
# --reverse => newest at bottom
# recent
# recent ~/Downloads 20
# recent --reverse
# recent ~/Downloads 15 --reverse
#
recent() {
  local dir="."
  local n=10
  local reverse=false

  for arg in "$@"; do
    case "$arg" in
      --reverse) reverse=true ;;
      *)
        [[ -z "${dir_set:-}" ]] && { dir="$arg"; dir_set=1; } \
        || n="$arg"
        ;;
    esac
  done

  if command -v eza >/dev/null 2>&1; then
    local sort_flags=(--sort=modified)
    $reverse && sort_flags+=(--reverse)

    eza --icons --color=always \
      --only-files \
      "${sort_flags[@]}" \
      --long --time-style=relative \
      --no-permissions --no-user \
      -- "$dir" | head -n "$n"
  else
    # macOS BSD ls fallback
    if $reverse; then
      command ls -lt -- "$dir" | head -n $((n + 1)) | tail -n "$n"
    else
      command ls -lt -- "$dir" | head -n $((n + 1))
    fi
  fi
}
# quick alias for the common case
alias lr='recent . 10 --reverse'

alias up="cd .."
alias gr="cd /"
cdd() { local d; d="$(fzf-open --local)" && cd -- "$d"; }
alias fd-dirs="fd -t d -d 15 -E '.*' -E 'Library'"
alias fd-empty="fd --type empty"

# ============================================================================
# Editor & Tools
# ============================================================================

alias vi=hx
alias vim=hx
alias edit=$EDITOR
alias n="fzf-edit"
alias eb="edit-bin"
alias es="edit-script"

alias f="noglob fetch"
alias sb=switchblade
alias prev="fzf --layout=reverse --preview 'bat --style=numbers --color=always --line-range :500 {}'"
alias sb-fast-new="print -rl -- **/*(.om[1,200]) | sb"

alias .tab=fzf-tablature
alias t=fzf-tablature
alias .chordpro="cd '${TABLATURE_DIR}/ChordPro' && chordpro-tui ."
