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
  ls-media | grep "clips" | grep "#top" | copy-flat ./clips
  ls-media | grep "scenes" | grep "#top" | copy-flat ./scenes
  ls-media --match "#portrait" | copy-flat ./portrait
  ls-media | grep "originals" | grep "#top" | copy-flat ./originals
}

alias @cache-copy="cd $LOCAL_CACHE_PATH && cache-copy-all"

media-cache-clear() {
  cd $LOCAL_CACHE_PATH && rm -rf **/*
}
