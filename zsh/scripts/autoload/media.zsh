# detect available media paths
ls-media-paths-checked() {
  for media_path in $(ls-media-paths); do
    echo $media_path
  done
}

# list all unique tags found in files under the present directory
fd-tags() {
  fd -t f '#' -x basename {} \; | grep -o '#[a-zA-Z0-9_-]\+' | sort -u
}

fd-creators() {
  fd -t f '#' -x basename {} \; | grep -o '#[a-zA-Z0-9_-]\+' | sort -u
}

# media-cache-to-discboy() {
#   media list top-clips | copy-flat "/Volumes/Discboy 512/clips"
# }
#
# media-cache-copy-top-clips() {
#   local dest="$1"
#
#   echo "Caching clips to $dest"
#   ls-media --match-string '/clips/' --match-regex '#(suki|top|cumshot)' | copy-flat "$dest/clips"
# }
#
# media-cache-copy-all() {
#   if [ ! -d "$MASTER_MEDIA_DIR" ]; then
#     echo "Error: MASTER_MEDIA_DIR '$MASTER_MEDIA_DIR' does not exist. Is the master volume connected?"
#     return 1
#   fi
#
#   echo "Caching clips..."
#   ls-media --match-string '/clips/' --match-regex '#(suki|top|cumshot)' | copy-flat "$LOCAL_CACHE_PATH/clips"
#
#   echo "Caching #top scenes..."
#   ls-media --match-string '/scenes/' --match-string "#top" | copy-flat "$LOCAL_CACHE_PATH/scenes"
#
#   echo "Caching #top portraits..."
#   ls-media --match-string "#portrait" --match-string "#top" --match-string "$MASTER_MEDIA_DIR" | copy-flat "$LOCAL_CACHE_PATH/portrait"
#
#   # echo "\nCaching originals..."
#   # ls-media --match-string "originals" --match-string "#top" | copy-flat "$LOCAL_CACHE_PATH/originals"
# }
# alias @cache-update="cache-copy-all && cd $LOCAL_CACHE_PATH"

media-cache-clear() {
  cd $LOCAL_CACHE_PATH && rm -rf **/*
}
