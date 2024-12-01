yazi-jump() {
  local tmp="$(mktemp -t "yazi-cwd.XXXXX")"
  yazi "$@" --cwd-file="$tmp"
  if cwd="$(cat -- "$tmp")" && [ -n "$cwd" ] && [ "$cwd" != "$PWD" ]; then
    cd -- "$cwd"
    zle reset-prompt
  fi
  rm -f -- "$tmp"
}

# Enhanced directory jumping function with early termination
fzf-jump() {
  set +o monitor
  local fd_pid dir timeout=30

  # Create a named pipe for fd output
  local pipe=$(mktemp -u)
  mkfifo "$pipe"

  # Cleanup function to handle interrupts and cleanup
  cleanup() {
    kill $fd_pid 2>/dev/null
    rm -f "$pipe"
    trap - INT EXIT
  } >/dev/null 2>&1
  trap cleanup INT EXIT

  # Run fd with timeout in background, writing to named pipe
  {
    nohup timeout $timeout fd --type d --no-hidden --max-depth 10 . >"$pipe" 2>/dev/null &
  } >/dev/null
  disown

  fd_pid=$!

  # Run fzf with the preview window showing tree or ls output
  dir=$(fzf --preview 'tree -C {} 2>/dev/null || ls -la {}' \
    --bind 'ctrl-d:preview-page-down,ctrl-u:preview-page-up' \
    --height=50% \
    --border \
    --prompt="Directory > " <"$pipe" 2>/dev/null)

  # Change to selected directory if valid
  [[ -n "$dir" && -d "$dir" ]] && cd "$dir"

  cleanup
} 2>/dev/null
