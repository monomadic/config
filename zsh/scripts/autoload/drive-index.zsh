local INDEX_DIR="$HOME/doc/indexes"

function index-run {
  if [[ -d "$1" ]]; then
    fd --type f --hidden --exclude '.*' --search-path "$1" || {
      echo "Error: Failed to index directory '$1'." >&2
      return 1
    }
  else
    echo "Warning: Directory '$1' does not exist." >&2
    return 1
  fi
}

function index-cat {
  cat $INDEX_DIR/*.txt
}

function index-cat-checked {
  cat $INDEX_DIR/*.txt | while read -r filepath; do
    if [ -f "$filepath" ]; then
      echo "$filepath"
    fi
  done
}

function index-select {
  index-cat-checked | fzf-multi
}

function index-select-multi {
  index-cat-checked | fzf-multi
}

function index-send-to-vlc {
  index-select-multi | sed 's/.*/"&"/' | xargs --verbose vlc
}

function index-send-to-iina {
  index-select-multi | sed 's/.*/"&"/' | xargs --verbose iina
}

function index-send-to-elmedia {
  index-select-multi | sed 's/.*/"&"/' | xargs --verbose open -a /Applications/Elmedia\ Video\ Player.app/Contents/MacOS/Elmedia\ Video\ Player
}

function index-update {
  # Ensure the target directory exists
  mkdir -p "${INDEX_DIR}"

  if ! index-run "$HOME/_inbox/" >"$HOME/doc/indexes/${HOSTNAME}_inbox.txt"; then
    echo "Error: Failed to create index for '$HOME/_inbox/'." >&2
    return 1
  fi
  echo "Indexed: $HOME/_inbox"

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
