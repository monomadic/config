typeset -g _cd_fzf_sources_file="${ZSH_DOTFILES_DIR:-$HOME/config/config/zsh}/fzf-cd-sources.zsh"
[[ -r "$_cd_fzf_sources_file" ]] && source "$_cd_fzf_sources_file"

typeset -ga _cd_fzf_global_static
typeset -ga _cd_fzf_global_commands
typeset -gi _cd_fzf_local_max_depth

(( ${#_cd_fzf_global_static} )) || _cd_fzf_global_static=(
  '$HOME/Movies'
  '$HOME/Pictures'
)
(( ${#_cd_fzf_global_commands} )) || _cd_fzf_global_commands=()
(( _cd_fzf_local_max_depth > 0 )) || _cd_fzf_local_max_depth=5

_cd_fzf_emit_static() {
  emulate -L zsh

  local entry path
  for entry in "${_cd_fzf_global_static[@]}"; do
    local -a expanded
    eval "expanded=( ${entry} )" 2>/dev/null
    for path in "${expanded[@]}"; do
      [[ -d "$path" ]] && print -r -- "${path%/}"
    done
  done
}

_cd_fzf_local_paths() {
  fd --type d --absolute-path --max-depth "${_cd_fzf_local_max_depth}" . "$PWD" 2>/dev/null \
    | sed 's:/*$::'
}

_cd_fzf_header_path() {
  print -r -- "${PWD/#$HOME/~}"
}

_cd_fzf_stream_unique() {
  emulate -L zsh

  typeset -A seen
  local line

  while IFS= read -r line; do
    line="${line%/}"
    [[ -n "$line" ]] || continue
    [[ -n ${seen[$line]:-} ]] && continue
    seen[$line]=1
    print -r -- "$line"
  done
}

_cd_fzf_spawn_global_sources() {
  emulate -L zsh

  local sink="$1" cmd
  reply=()

  (_cd_fzf_emit_static > "$sink") &
  reply+=($!)

  for cmd in "${_cd_fzf_global_commands[@]}"; do
    (/bin/zsh -lc "$cmd" > "$sink" 2>/dev/null) &
    reply+=($!)
  done
}

_cd_fzf_spawn_local_sources() {
  emulate -L zsh

  local sink="$1"
  reply=()

  (_cd_fzf_local_paths > "$sink") &
  reply+=($!)
}

_cd_fzf_pick() {
  command fzf-cd "$@"
}

_cd_tv_pick() {
  command tv-cd "$@"
}

_cd_fzf_collect_descendants() {
  emulate -L zsh

  local -a frontier next_frontier descendants
  local parent child

  frontier=("$@")
  while (( ${#frontier} )); do
    next_frontier=()
    for parent in "${frontier[@]}"; do
      while IFS= read -r child; do
        [[ -n "$child" ]] || continue
        descendants+=("$child")
        next_frontier+=("$child")
      done < <(ps -axo pid=,ppid= | awk -v parent="$parent" '$2 == parent { print $1 }')
    done
    frontier=("${next_frontier[@]}")
  done

  print -r -l -- "${descendants[@]}"
}

_cd_fzf_kill_tree() {
  emulate -L zsh

  local -a roots descendants pids
  local pid

  roots=("$@")
  (( ${#roots} )) || return 0

  descendants=("${(@f)$(_cd_fzf_collect_descendants "${roots[@]}")}")
  pids=("${descendants[@]}" "${roots[@]}")

  for pid in "${pids[@]}"; do
    [[ "$pid" == <-> ]] || continue
    kill -TERM "$pid" >/dev/null 2>&1 || true
  done

  sleep 0.05

  for pid in "${pids[@]}"; do
    [[ "$pid" == <-> ]] || continue
    kill -KILL "$pid" >/dev/null 2>&1 || true
  done
}

_cd_fzf_cleanup_streamed_picker() {
  emulate -L zsh

  local raw_fifo="$1" filtered_fifo="$2" result_file="$3" keepalive_fd="$4"
  shift 4

  if [[ "$keepalive_fd" == <-> ]]; then
    eval "exec ${keepalive_fd}>&-" >/dev/null 2>&1 || true
  fi

  _cd_fzf_kill_tree "$@"
  wait "$@" >/dev/null 2>&1 || true
  rm -f "$raw_fifo" "$filtered_fifo" "$result_file"
}

_cd_fzf_run_streamed_picker() {
  emulate -L zsh
  setopt localoptions pipefail no_monitor

  local picker_fn="$1" mode="$2" prompt="$3" header="$4"
  local raw_fifo filtered_fifo result_file selected exit_code picker_pid dedupe_pid keepalive_fd pid
  local -a producer_pids

  raw_fifo="$(mktemp -u "${TMPDIR:-/tmp}/${picker_fn}-stream.XXXXXX")" || return 1
  filtered_fifo="$(mktemp -u "${TMPDIR:-/tmp}/${picker_fn}-filtered.XXXXXX")" || return 1
  result_file="$(mktemp "${TMPDIR:-/tmp}/${picker_fn}-result.XXXXXX")" || return 1
  mkfifo "$raw_fifo" "$filtered_fifo" || {
    rm -f "$raw_fifo" "$filtered_fifo"
    rm -f "$result_file"
    return 1
  }

  if [[ "$picker_fn" == _cd_tv_pick ]]; then
    "$picker_fn" --prompt "$prompt" --header "$header" --source-command "cat ${(q)filtered_fifo}" > "$result_file" &
  else
    "$picker_fn" --prompt "$prompt" --header "$header" < "$filtered_fifo" > "$result_file" &
  fi
  picker_pid=$!

  _cd_fzf_stream_unique < "$raw_fifo" > "$filtered_fifo" &
  dedupe_pid=$!

  exec {keepalive_fd}> "$raw_fifo"
  trap '_cd_fzf_cleanup_streamed_picker "$raw_fifo" "$filtered_fifo" "$result_file" "$keepalive_fd" "$picker_pid" "$dedupe_pid" "${producer_pids[@]}"; return 130' INT TERM HUP

  case "$mode" in
    global)
      _cd_fzf_spawn_global_sources "$raw_fifo"
      producer_pids=("${reply[@]}")
      ;;
    local)
      _cd_fzf_spawn_local_sources "$raw_fifo"
      producer_pids=("${reply[@]}")
      ;;
    *)
      trap - INT TERM HUP
      _cd_fzf_cleanup_streamed_picker "$raw_fifo" "$filtered_fifo" "$result_file" "$keepalive_fd" "$picker_pid" "$dedupe_pid" "${producer_pids[@]}"
      return 2
      ;;
  esac

  exec {keepalive_fd}>&-

  wait "$picker_pid"
  exit_code=$?

  if [[ -s "$result_file" ]]; then
    IFS= read -r selected < "$result_file"
  fi

  trap - INT TERM HUP
  _cd_fzf_cleanup_streamed_picker "$raw_fifo" "$filtered_fifo" "$result_file" "" "$dedupe_pid" "${producer_pids[@]}"

  if [[ $exit_code -eq 0 && -n "$selected" ]]; then
    print -r -- "$selected"
  fi
  return $exit_code
}

_cd_fzf_pick_global() {
  _cd_fzf_run_streamed_picker _cd_fzf_pick global '' 'Pinned directories'
}

_cd_fzf_pick_local() {
  _cd_fzf_run_streamed_picker \
    _cd_fzf_pick \
    local \
    'local ❯ ' \
    "Children of $(_cd_fzf_header_path) (depth ${_cd_fzf_local_max_depth})"
}

_cd_tv_pick_global() {
  _cd_fzf_run_streamed_picker _cd_tv_pick global '' 'Pinned directories'
}

_cd_tv_pick_local() {
  _cd_fzf_run_streamed_picker \
    _cd_tv_pick \
    local \
    'local ❯ ' \
    "Children of $(_cd_fzf_header_path) (depth ${_cd_fzf_local_max_depth})"
}
