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

function index-update {
		# Ensure the target directory exists
		mkdir -p "$HOME/doc/indexes"

		if ! index-run "$HOME/_inbox/" > "$HOME/doc/indexes/${HOSTNAME}_inbox.txt"; then
				echo "Error: Failed to create index for '$HOME/_inbox/'." >&2
				return 1
		fi
		echo "Indexed: $HOME/_inbox"

		if [[ -d "${babyblue}/not-porn" ]]; then
				index-run "${babyblue}/not-porn" > "$HOME/doc/indexes/BabyBlue2TB.txt" || {
						echo "Error: Failed to create index for '${babyblue}/not-porn'." >&2
				}
				echo "Indexed: ${babyblue}"
		else
				echo "Warning: ${babyblue} not found, skipping this index." >&2
		fi
}

function index-search {
    local search_term="$1"
    local index_dir="$HOME/doc/indexes"

    if [[ -z "$search_term" ]]; then
        echo "Usage: index-search <search_term>"
        return 1
    fi

    if [[ -d "$index_dir" ]]; then
        rg -i --fixed-strings --no-line-number --glob "*.txt" "$search_term" "$index_dir" || {
            echo "No matches found for '$search_term' in $index_dir." >&2
            return 1
        }
    else
        echo "Warning: Directory '$index_dir' does not exist." >&2
        return 1
    fi
}

function index-search-or {
		local index_dir="$HOME/doc/indexes"

		if [[ $# -eq 0 ]]; then
				echo "Usage: index-search-or <search_term1> <search_term2> ..."
				return 1
		fi

		if [[ -d "$index_dir" ]]; then
				local rg_command="rg -i --fixed-strings --no-line-number --glob '*.txt'"
				for term in "$@"; do
						rg_command+=" -e \"$term\""
        done
        eval "$rg_command \"$index_dir\"" || {
            echo "No matches found for the specified search terms in $index_dir." >&2
            return 1
        }
    else
        echo "Warning: Directory '$index_dir' does not exist." >&2
        return 1
    fi
}

function index-search-and {
    local index_dir="$HOME/doc/indexes"

    if [[ $# -eq 0 ]]; then
        echo "Usage: index-search-and <search_term1> <search_term2> ..."
        return 1
    fi

    if [[ -d "$index_dir" ]]; then
        local rg_command="rg -i --fixed-strings --no-line-number --glob '*.txt'"

        for term in "$@"; do
            rg_command+=" | rg -i --fixed-strings --no-line-number \"$term\""
        done

        rg_command+=" \"$index_dir\""

        eval "$rg_command" || {
            echo "No matches found for the specified search terms in $index_dir." >&2
            return 1
        }
    else
        echo "Warning: Directory '$index_dir' does not exist." >&2
        return 1
    fi
}
