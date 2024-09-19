# detect available media paths
ls-media-paths-checked() {
  for media_path in $(ls-media-paths); do
    echo $media_path
  done
}

# select from all media paths to cd into
cd-media() {
  local selected_path
  local media_paths = $(ls-media-paths $(expand_paths $LOCAL_CACHE_PATHS))

  selected_path=$(media-paths | fzf --height 40% --reverse)

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
ls-tags() {
  fd -t f '#' -x basename {} \; | grep -o '#[a-zA-Z0-9_-]\+' | sort -u
}

cache-copy-all() {
  echo "Caching clips..."
  ls-media --match-string "clips" --match-string "#top" | copy-flat "$LOCAL_CACHE_PATH/clips"

  echo "\nCaching scenes..."
  ls-media --match-string "scenes" --match-string "#top" | copy-flat "$LOCAL_CACHE_PATH/scenes"

  echo "\nCaching portraits..."
  ls-media --match-string "#portrait" --match-string "#top" | copy-flat "$LOCAL_CACHE_PATH/portrait"

  # echo "\nCaching originals..."
  # ls-media --match-string "originals" --match-string "#top" | copy-flat "$LOCAL_CACHE_PATH/originals"
}
alias @cache-update="cache-copy-all && cd $LOCAL_CACHE_PATH"

media-cache-clear() {
  cd $LOCAL_CACHE_PATH && rm -rf **/*
}
