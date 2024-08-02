local INDEX_DIR="$HOME/doc/indexes"
alias fd-video="fd -i -e mp4 -e avi -e mkv -e mov -e wmv -e flv -e webm --color=always"

local MEDIA_PATHS=(
	/Volumes/**/not-porn/(N)
  "$HOME/Media/Porn/"
)

function ls-media() {
  for media_path in "${MEDIA_PATHS[@]}"; do
    fd-video . $media_path --type f
  done
}

function search-media() {
  ls-media | fzf-play
}
alias @media-search=search-media

# not working
function play-with-mpv() {
  mpv --macos-fs-animation-duration=0 --no-native-fs --fs --playlist=-
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

index-grep-top() {
  grep -E '\[TOP\]|🎖️|\[\*\]'
}

function index-grep-safe {
  grep -v -E '\[g\]|\[bi\]|\[unsafe\]'
}

function index-play-top {
  index-cat | index-grep-top | fzf-play
}

# list available files from the index and play them
function index-play-checked {
  index-cat-checked | index-grep-safe | fzf-play
}
alias @play-index-checked=index-play-checked

function index-play-checked-top {
  index-cat-checked | index-grep-top | fzf-play
}

# just search the index without filtering or playing
function index-list {
  index-cat | fzf --ansi --exact
}

# function index-send-to-iina {
#   index-select-multi | sed 's/.*/"&"/' | xargs --verbose iina
# }

# function index-send-to-elmedia {
#   index-select-multi | sed 's/.*/"&"/' | xargs --verbose open -a /Applications/Elmedia\ Video\ Player.app/Contents/MacOS/Elmedia\ Video\ Player
# }

function index-update {
  # Ensure the target directory exists
  mkdir -p "${INDEX_DIR}"

  if ! index-run "$HOME/_inbox/" >"$HOME/doc/indexes/${HOSTNAME}_inbox.txt"; then
    echo "Error: Failed to create index for '$HOME/_inbox/'." >&2
    return 1
  fi
  echo "Indexed: $HOME/_inbox as ${HOSTNAME}_inbox"

  if [[ -d "${babyblue}/not-porn" ]]; then
    index-run "${babyblue}/not-porn" >"$HOME/doc/indexes/BabyBlue2TB.txt" || {
      echo "Error: Failed to create index for '${babyblue}/not-porn'." >&2
    }
    echo "Indexed: ${babyblue}"
  else
    echo "Warning: ${babyblue} not found, skipping this index." >&2
  fi
}

function index-search {
  local search_term="$1"

  if [[ -z "$search_term" ]]; then
    echo "Usage: index-search <search_term>"
    return 1
  fi

  if [[ -d "$INDEX_DIR" ]]; then
    rg -i --fixed-strings --no-line-number --glob "*.txt" "$search_term" "$INDEX_DIR" || {
      echo "No matches found for '$search_term' in $INDEX_DIR." >&2
      return 1
    }
  else
    echo "Warning: Directory '$INDEX_DIR' does not exist." >&2
    return 1
  fi
}

function index-search-or {
  if [[ $# -eq 0 ]]; then
    echo "Usage: index-search-or <search_term1> <search_term2> ..."
    return 1
  fi

  if [[ -d "$INDEX_DIR" ]]; then
    local rg_command="rg -i --fixed-strings --no-line-number --glob '*.txt'"
    for term in "$@"; do
      rg_command+=" -e \"$term\""
    done
    eval "$rg_command \"$INDEX_DIR\"" || {
      echo "No matches found for the specified search terms in $INDEX_DIR." >&2
      return 1
    }
  else
    echo "Warning: Directory '$INDEX_DIR' does not exist." >&2
    return 1
  fi
}

function index-search-and {
  if [[ $# -eq 0 ]]; then
    echo "Usage: index-search-and <search_term1> <search_term2> ..."
    return 1
  fi

  if [[ -d "$INDEX_DIR" ]]; then
    local rg_command="rg -i --fixed-strings --no-line-number --glob '*.txt'"
    for term in "$@"; do
      rg_command+=" | rg -i --fixed-strings --no-line-number \"$term\""
    done
    rg_command+=" \"$INDEX_DIR\""
    eval "$rg_command" || {
      echo "No matches found for the specified search terms in $INDEX_DIR." >&2
      return 1
    }
  else
    echo "Warning: Directory '$INDEX_DIR' does not exist." >&2
    return 1
  fi
}
