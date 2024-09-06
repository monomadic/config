cd-media() {
  local selected_path
  selected_path=$(ls-media-paths | fzf --height 40% --reverse)

  if [ -n "$selected_path" ]; then
    cd "$selected_path" || return
  else
    echo "No directory selected."
  fi
}
alias cdm=cd-media
