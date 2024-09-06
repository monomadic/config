# Set variables
local INDEX_DIR="$HOME/doc/indexes"
export MASTER_COPY_PATH="/Volumes/BabyBlue2TB"

# function most-recent() {
#   xargs -d '\n' ls -lt | tac
# }

function ls-tags() {
  fd -t f '#' -x basename {} \; | grep -o '#[a-zA-Z0-9_-]\+' | sort -u
}

# Detect and print media paths
function media-detect() {
  for media_path in $(ls-media-paths); do
    echo "Path found: $media_path"
  done
}

function cache-all() {
  ls-media | grep "clips" | grep "#top" | copy-flat ./clips
  ls-media | grep "scenes" | grep "#top" | copy-flat ./scenes
  ls-media --match "#portrait" | copy-flat ./portrait
  ls-media | grep "originals" | grep "#top" | copy-flat ./originals
}

# Search media files and play with fzf
function fzf-safe-media() {
  ls-media | grep-safe | fzf-play
}
alias .play=fzf-safe-media

function fzf-media-top() {
  ls-media | grep-top | grep-safe | fzf-play
}
alias top=fzf-media-top
alias .search-top=fzf-media-top

function fzf-safe-media-latest() {
  ls-media --sort modified | grep-safe | fzf-play
}

# Include unsafe files
function fzf-media-all() {
  ls-media | fzf-play --kitty
}

function fzf-media-cache {
  cd $HOME/Movies/Cache && fd-video | fzf-play
}
alias @cache
alias .cache

# Define aliases
alias @play="fzf-safe-media"
alias @play-all="fzf-media-all"
alias @play-latest="fzf-safe-media-latest"

# Update the alias to use the new function
alias @media-stats="ls-media-stats"

# Update the alias to use the new function
alias @media-stats="ls-media-stats"

# Update the alias to use the new function
alias @media-stats="ls-media-stats"

# Update the alias to use the new function
alias @media-stats="ls-media-stats"

mpv-stdin() {
  mpv --macos-fs-animation-duration=0 --no-native-fs --fs --loop-playlist --playlist=-
}

mpv-play-sorted() {
  ls-media --sort modified --reverse | mpv-stdin
}

mpv-play-pwd-latest() {
  echo $PWD | sort-across-paths --sort modified --reverse | mpv-stdin
}
alias .play-pwd-latest=mpv-play-pwd-latest

fzf-play-pwd-sorted() {
  echo $PWD | sort-across-paths --sort modified --reverse | fzf-play
}

mpv-play-cache() {
  expand-paths $LOCAL_CACHE_PATHS | mpv --macos-fs-animation-duration=0 --no-native-fs --fs --loop-playlist --playlist=-
}

mpv-play-cache-latest() {
  expand-paths $LOCAL_CACHE_PATHS | sort-across-paths --sort modified --reverse | mpv-stdin
}

mpv-play-local() {
  expand-paths $LOCAL_MEDIA_PATHS | mpv-stdin
}
alias .local

mpv-play-local() {
  expand-paths $LOCAL_MEDIA_PATHS | mpv-stdin
}
alias .local-search=mpv-play-local

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

alias .latest=mpv-play-sorted
alias .play-latest=mpv-play-sorted
alias .play-cache=mpv-play-cache

alias @play-private="cd $PRIVATE_PHOTOS_LIBRARY/originals && @play-pwd"

mpv-play-all() {
  ls-media-paths | mpv-stdin
}

function play-with-mpv-debug() {
  while IFS= read -r file; do
    echo "Playing: $file" # For debugging
    echo "$file"
  done | mpv --macos-fs-animation-duration=0 --no-native-fs --fs --playlist=-
}

function index-run() {
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

function index-cat {
  cat $INDEX_DIR/*.txt
}

index-cat-checked() {
  cat $INDEX_DIR/*.txt | while read -r filepath; do
    [[ -f "$filepath" ]] && echo "$filepath"
  done
}

function index-play {
  index-cat | fzf-play
}
alias @play-index=index-play

grep-top() {
  grep -E '(?i)\[TOP\]|🎖️|\[\*\]|\#top'
}
alias fd-top="fd-video |grep-top"

function grep-safe {
  grep -v -E '#g(\.| |/)|#bi(\.| |/)|#unsafe(\.| |/)|#ts(\.| |/)'
}

function index-play-top {
  index-cat | grep-top | fzf-play
}

# list available files from the index and play them
function index-play-checked {
  index-cat-checked | grep-safe | fzf-play
}
alias @play-index-checked=index-play-checked

function index-play-checked-top {
  index-cat-checked | index-grep-top | fzf-play
}

function media-play-all-local {
  ls-media-paths | grep $HOME | mpv --macos-fs-animation-duration=0 --no-native-fs --fs --loop-playlist --loop-file=1 --shuffle --playlist=-
}

# # just search the index without filtering or playing
# function index-list {
#   index-cat | fzf --ansi --exact
# }
#
# # function index-send-to-iina {
# #   index-select-multi | sed 's/.*/"&"/' | xargs --verbose iina
# # }
#
# # function index-send-to-elmedia {
# #   index-select-multi | sed 's/.*/"&"/' | xargs --verbose open -a /Applications/Elmedia\ Video\ Player.app/Contents/MacOS/Elmedia\ Video\ Player
# # }
#
# function index-update {
#   # Ensure the target directory exists
#   mkdir -p "${INDEX_DIR}"
#
#   if ! index-run "$HOME/_inbox/" >"$HOME/doc/indexes/${HOSTNAME}_inbox.txt"; then
#     echo "Error: Failed to create index for '$HOME/_inbox/'." >&2
#     return 1
#   fi
#   echo "Indexed: $HOME/_inbox as ${HOSTNAME}_inbox"
#
#   if [[ -d "${babyblue}/not-porn" ]]; then
#     index-run "${babyblue}/not-porn" >"$HOME/doc/indexes/BabyBlue2TB.txt" || {
#       echo "Error: Failed to create index for '${babyblue}/not-porn'." >&2
#     }
#     echo "Indexed: ${babyblue}"
#   else
#     echo "Warning: ${babyblue} not found, skipping this index." >&2
#   fi
# }
#
# function index-search {
#   local search_term="$1"
#
#   if [[ -z "$search_term" ]]; then
#     echo "Usage: index-search <search_term>"
#     return 1
#   fi
#
#   if [[ -d "$INDEX_DIR" ]]; then
#     rg -i --fixed-strings --no-line-number --glob "*.txt" "$search_term" "$INDEX_DIR" || {
#       echo "No matches found for '$search_term' in $INDEX_DIR." >&2
#       return 1
#     }
#   else
#     echo "Warning: Directory '$INDEX_DIR' does not exist." >&2
#     return 1
#   fi
# }
#
# function index-search-or {
#   if [[ $# -eq 0 ]]; then
#     echo "Usage: index-search-or <search_term1> <search_term2> ..."
#     return 1
#   fi
#
#   if [[ -d "$INDEX_DIR" ]]; then
#     local rg_command="rg -i --fixed-strings --no-line-number --glob '*.txt'"
#     for term in "$@"; do
#       rg_command+=" -e \"$term\""
#     done
#     eval "$rg_command \"$INDEX_DIR\"" || {
#       echo "No matches found for the specified search terms in $INDEX_DIR." >&2
#       return 1
#     }
#   else
#     echo "Warning: Directory '$INDEX_DIR' does not exist." >&2
#     return 1
#   fi
# }
#
# function index-search-and {
#   if [[ $# -eq 0 ]]; then
#     echo "Usage: index-search-and <search_term1> <search_term2> ..."
#     return 1
#   fi
#
#   if [[ -d "$INDEX_DIR" ]]; then
#     local rg_command="rg -i --fixed-strings --no-line-number --glob '*.txt'"
#     for term in "$@"; do
#       rg_command+=" | rg -i --fixed-strings --no-line-number \"$term\""
#     done
#     rg_command+=" \"$INDEX_DIR\""
#     eval "$rg_command" || {
#       echo "No matches found for the specified search terms in $INDEX_DIR." >&2
#       return 1
#     }
#   else
#     echo "Warning: Directory '$INDEX_DIR' does not exist." >&2
#     return 1
#   fi
# }
