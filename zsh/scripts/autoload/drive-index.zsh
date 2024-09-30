# Indexing and searching for non-persistent volumes

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
alias top=fzf-media-top
alias .search-top=fzf-media-top

alias media-ls-clips="media-ls --match-string '/clips/'"
media-search-clips() {
  media-ls-clips | grep-safe | fzf-play
}
alias @search-clips=fzf-search-clips
alias .search-clips=fzf-search-clips
alias search-clips=fzf-search-clips

fzf-search-local() {
  ls-media --match-string "$HOME" | grep-safe | fzf-play
}
alias @search-local=fzf-search-local

fzf-media-untagged() {
  ls-media | grep -v '#' | fzf-play --kitty
}

alias .top=media-play-top
alias .suki=mpv-play-suki

media-play-cache() {
  cd $HOME/Movies/Cache && fd-video | fzf-play
}
alias @cache=media-play-cache
alias .cache=media-play-cache

mpv-stdin() {
  mpv --macos-fs-animation-duration=0 --no-native-fs --fs --loop-playlist --input-ipc-server=/tmp/mpvsocket --mute=yes $@ --playlist=- >/dev/null 2>&1 &
}

alias media-play-safe="media-ls | grep-safe | mpv-play --shuffle"
alias media-play-unsafe="media-ls | grep-unsafe | mpv-play --shuffle"

alias media-ls-suki="ls-media --match-regex '#suki'"
alias media-play-suki="media-ls-suki | mpv-play --shuffle"

alias media-ls-top="ls-media --match-regex '#top'"
alias media-play-top="media-ls-top | mpv-play"
alias @play-top=media-play-top

alias media-ls-clips="ls-media --match-string '/clips/'"
alias media-play-clips="media-ls-clips | mpv-play --shuffle"
alias media-search-clips="media-ls-clips | fzf-play"

# alias media-ls-clips-top="ls-media --match-regex 'clips.*#top'"
# alias media-ls-clips-top="ls-media --match-regex 'clips' --match-regex '#top'"
alias media-ls-clips-top="media-ls-clips --match-string '#top'"
alias media-play-clips-top="media-ls-clips-top | mpv-play --shuffle"
alias media-search-clips-top="media-ls-clips-top | fzf-play"

alias media-ls-clips-top-latest="media-ls-clips-top --sort-modified --reverse"
alias media-search-clips-top-latest="media-ls-clips-top-latest | fzf-play"
alias media-play-clips-top-latest="media-ls-clips-top-latest | mpv-play"

alias media-ls-clips-suki-top-cumshot="ls-media --match-regex 'clips.*#(suki|top|cumshot)'"
alias media-play-clips-suki-top-cumshot="media-ls-clips-suki-top-cumshot | mpv-play --shuffle"
alias media-search-clips-suki-top-cumshot="media-ls-clips-suki-top-cumshot | fzf-play"

alias @play-clips=media-play-clips
alias .clips=media-play-clips

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

media-search-pwd() {
  fd-video | fzf-play
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

alias fzf-search-pwd="fd-video | fzf-play --kitty"
alias @search-pwd=fzf-search-pwd
alias .search-pwd=fzf-search-pwd

alias @search-pwd-sorted=fzf-play-pwd-sorted
alias .search-pwd-sorted=fzf-play-pwd-sorted

alias .latest=media-play-latest
alias .play-latest=media-play-latest
alias .play-cache=mpv-play-cache

alias @play-private="cd $PRIVATE_PHOTOS_LIBRARY/originals && @play-pwd"

mpv-play-external-drives() {
  mpv --macos-fs-animation-duration=0 --no-native-fs --fs --loop-playlist --mute=yes --shuffle /Volumes/**/Movies/* >/dev/null 2>&1 &
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
  ls-media-paths | grep $HOME | mpv --macos-fs-animation-duration=0 --no-native-fs --fs --loop-playlist --loop-file=1 --shuffle --playlist=-
}
