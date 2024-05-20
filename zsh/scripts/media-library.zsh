function search-library {
  local query="$1"
	fd -i "$search_term"
}
