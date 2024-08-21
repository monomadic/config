# Set variables
local INDEX_DIR="$HOME/doc/indexes"
export MASTER_COPY_PATH="/Volumes/BabyBlue2TB"

function fd-video() {
  fd -t f -e mp4 -e avi -e mkv -e mov -e wmv -e flv -e webm --color=always "$@"
}

# List media files using fd-video
function ls-media-unsafe() {
  # Iterate over each media path
  for media_path in $(ls-media-paths); do
    # Use fd only for valid directories
    if [[ -d "$media_path" ]]; then
      fd-video . "$media_path" --type f
    fi
  done
}

# List media files across media paths.
# - handles special characters
# - preserves whitespace, prevents backslash interpretation
# - does not create a subshell for `ls-media-paths`
function ls-media() {
  while IFS= read -r media_path; do
    if [[ -d "$media_path" ]]; then
      fd-video . "$media_path"
    fi
  done < <(ls-media-paths)
}

function ls-tags() {
  fd -t f '#' -x basename {} \; | grep -o '#[a-zA-Z0-9_-]\+' | sort -u
}

# Detect and print media paths
function media-detect() {
  for media_path in $(ls-media-paths); do
    echo "Path found: $media_path"
  done
}

# Cache media files containing "[Top]" in their names
function media-cache-top() {
  local destination_dir="$1"

  # Ensure the destination directory is provided
  if [[ -z "$destination_dir" ]]; then
    echo "Usage: media-cache-top <destination_dir>"
    return 1
  fi

  # Check if the source directory exists
  if [[ ! -d "$MASTER_COPY_PATH" ]]; then
    echo "Source directory $MASTER_COPY_PATH does not exist."
    return 1
  fi

  # Create destination directory if it doesn't exist
  mkdir -p "$destination_dir"

  # Use fd to find files with [Top] and copy them
  fd -i -e mp4 -e avi -e mkv -e mov -e wmv -e flv -e webm -g "*[Top]*" "$MASTER_COPY_PATH" \
    -x cp -- '{}' "$destination_dir"

  echo "Files containing '[Top]' have been copied to $destination_dir"
}

# Search media files and play with fzf
function search-media() {
  ls-media | grep-safe | fzf-play
}

# Include unsafe files
function search-media-all() {
  ls-media | fzf-play
}

# Define aliases
alias @media-copy-top="media-cache-top"
alias @media-search-all="search-media-all"
alias @media-play-all="search-media-all"
alias @media-search="search-media"

# Update the alias to use the new function
alias @media-stats="ls-media-stats"

# Ensure fd-video searches recursively
function fd-video() {
  fd -t f -e mp4 -e avi -e mkv -e mov -e wmv -e flv -e webm --color=always -d 20 "$@"
}

# Update the alias to use the new function
alias @media-stats="ls-media-stats"

# Update the alias to use the new function
alias @media-stats="ls-media-stats"

# Update the alias to use the new function
alias @media-stats="ls-media-stats"

# New play-all-now function
function play-all-now() {
  local playlist_file=$(mktemp)
  ls-media | tr '\n' '\0' >"$playlist_file"

  if [ -s "$playlist_file" ]; then
    xargs -0 mpv --macos-fs-animation-duration=0 --no-native-fs --fs <"$playlist_file"
  else
    echo "No media files found."
  fi

  rm "$playlist_file"
}
alias @play-all-now="play-all-now"

alias @play-local="ls-media | grep $HOME | fzf-play"

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

function grep-top() {
  grep -E '\[TOP\]|🎖️|\[\*\]'
}
alias fd-top="fd-video |grep-top"

function grep-safe {
  grep -v -E '\[g\]|\[bi\]|\[unsafe\]'
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
