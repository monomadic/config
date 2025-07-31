# Indexing and offline searching for non-persistent volumes

mpv-stdin() {
  mpv --macos-fs-animation-duration=0 --no-native-fs --fs --input-ipc-server=/tmp/mpvsocket --mute=yes $@ --playlist=- >/dev/null 2>&1 &
}

media-search-pwd-sorted() {
  echo $PWD | sort-across-paths --sort modified --reverse | fzf-play
}

mpv-play-cache() {
  expand-paths $LOCAL_CACHE_PATHS | mpv-stdin --shuffle
}
alias @play-cache=mpv-play-cache

mpv-play-cache-clips() {
  expand-paths $LOCAL_CACHE_PATHS | sort-across-paths --sort modified --reverse | grep "\/clips\/" | mpv-stdin --shuffle
}
alias @play-cached-clips=mpv-play-cache-clips

mpv-play-cache-cs() {
  expand-paths $LOCAL_CACHE_PATHS | sort-across-paths --sort modified --reverse | grep "#cumshot" | mpv-stdin --shuffle
}
alias @play-cached-cs=mpv-play-cache-cs

mpv-play-cache-latest() {
  expand-paths $LOCAL_CACHE_PATHS | sort-across-paths --sort modified --reverse | mpv-stdin
}

mpv-play-local() {
  expand-paths $LOCAL_MEDIA_PATHS | mpv-stdin
}
alias .local=mpv-play-local

mpv-play-local() {
  expand-paths $LOCAL_MEDIA_PATHS | mpv-stdin
}
alias .local-play=mpv-play-local

fzf-local-sorted() {
  expand-paths $LOCAL_MEDIA_PATHS | sort-across-paths --sort modified --reverse | fzf-play --kitty
}
alias .search-local=fzf-local-sorted

mpv-play-local-sorted() {
  expand-paths $LOCAL_MEDIA_PATHS | sort-across-paths --sort modified --reverse | mpv-stdin
}

alias .local-sorted=mpv-play-local-sorted
alias .play-local-sorted=mpv-play-local-sorted

alias mpv-play-pwd="fd-video | mpv-stdin"
alias @play-pwd=mpv-play-pwd
alias .play-pwd=mpv-play-pwd
alias .play-pwd-shuffle=mpv-play-pwd --shuffle

alias @play-private="cd $PRIVATE_PHOTOS_LIBRARY/originals && @play-pwd"

alias @search-incomplete=media-search-incomplete-downloads

index-update() {
  emulate -L zsh

  [[ -d "$1" ]] && {
    fd --type f --hidden --exclude '.*' --search-path "$1" || {
      echo "Error: Failed to index directory '$1'." >&2
      return 1
    }
  } || {
    echo "Warning: Directory '$1' does not exist." >&2
    return 1
  }
}

index-cat() {
  cat $INDEX_DIR/*.txt
}

index-cat-checked() {
  cat $INDEX_DIR/*.txt | while read -r filepath; do
    [[ -f "$filepath" ]] && echo "$filepath"
  done
}

index-play() {
  index-cat | fzf-play
}
alias @play-index=index-play

grep-top() {
  grep -E '(?i)\[TOP\]|🎖️|\[\*\]|\#top'
}
alias fd-top="fd-video |grep-top"

grep-safe() {
  grep -v -E '#g(\.| |/)|#bi(\.| |/)|#unsafe(\.| |/)|#ts(\.| |/)'
}

grep-unsafe() {
  grep -E '#g(\.| |/)|#bi(\.| |/)|#unsafe(\.| |/)|#ts(\.| |/)'
}

index-play-top() {
  index-cat | grep-top | fzf-play
}

# list available files from the index and play them
index-play-checked() {
  index-cat-checked | grep-safe | fzf-play
}
alias @play-index-checked=index-play-checked

index-play-checked-top() {
  index-cat-checked | index-grep-top | fzf-play
}
