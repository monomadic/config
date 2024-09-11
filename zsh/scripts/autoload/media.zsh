# detect available media paths
media-detect() {
  for media_path in $(ls-media-paths); do
    echo $media_path
  done
}

# select from all media paths to cd into
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

cd-inbox() {
  cd $MEDIA_INBOX_PATH
}
alias .inbox=cd-inbox

# list all unique tags found in files under the present directory
function ls-tags() {
  fd -t f '#' -x basename {} \; | grep -o '#[a-zA-Z0-9_-]\+' | sort -u
}
