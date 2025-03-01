# Indexing and offline searching for non-persistent volumes

# todo: rename this cmd
alias media-ls=ls-media

# Search media files and play with fzf
fzf-safe-media() {
  ls-media | grep-safe | fzf-play
}
alias .play=fzf-safe-media

fzf-media-top() {
  ls-media | grep-top | grep-safe | fzf-play
}

fzf-search-local() {
  ls-media --match-string "$HOME" | grep-safe | fzf-play
}
alias @search-local=fzf-search-local

ls-media-untagged() {
  ls-media | grep -v '#' | fzf-play --kitty
}

mpv-stdin() {
  mpv --macos-fs-animation-duration=0 --no-native-fs --fs --input-ipc-server=/tmp/mpvsocket --mute=yes $@ --playlist=- >/dev/null 2>&1 &
}

alias media-play-safe="ls-media | grep-safe | mpv-play --shuffle"
alias media-play-unsafe="ls-media | rep-unsafe | mpv-play --shuffle"

mpv-play-loops() {
  ls-media | grep "\/loops\/" | grep-safe | mpv-stdin --shuffle --loop-file=1 --length=10
}
alias @play-loops=mpv-play-loops
alias .loops=mpv-play-loops

media-play-pwd-latest() {
  echo $PWD | sort-across-paths --sort modified | mpv-stdin
}
alias .play-pwd-latest=media-play-pwd-latest
alias ppwd=media-play-pwd-latest

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

alias media-search-pwd="fd-video | fzf-play --kitty"

alias @search-pwd-sorted=fzf-play-pwd-sorted

alias @play-private="cd $PRIVATE_PHOTOS_LIBRARY/originals && @play-pwd"

mpv-play-external-drives() {
  mpv --macos-fs-animation-duration=0 --no-native-fs --fs --mute=yes --shuffle /Volumes/**/Movies/* >/dev/null 2>&1 &
}
alias @play-external=mpv-play-external-drives

media-search-incomplete-downloads-pwd() {
  fd mp4.part$ | fzf-play
}

media-search-incomplete-downloads() {
  cd $HOME/Movies/Porn/originals/_inbox &&
    fd mp4.part$ | fzf-play
}
alias @search-incomplete=media-search-incomplete-downloads

mpv-play-incomplete-downloads() {
  cd $HOME/Movies/Porn/originals/_inbox &&
    ls *.mp4.part | mpv-play
}
alias @play-incomplete

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

media-play-all-local() {
  ls-media-paths | grep $HOME | mpv --macos-fs-animation-duration=0 --no-native-fs --fs --loop-file=1 --shuffle --playlist=-
}
