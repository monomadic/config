#!/bin/zsh

# Check if at least two arguments are provided
if [[ $# -lt 2 ]]; then
  echo "Usage: $0 <search_string> <replace_string> [--write]"
  exit 1
fi

# Assign arguments to variables
local search_string="$1"
local replace_string="$2"
local write_flag="$3"

# Read files from stdin
while IFS= read -r file; do
  if [[ ! -f $file ]]; then
    echo "File not found: $file"
    continue
  fi

  # Use `grep` to check if the file contains the search string
  if grep -q "$search_string" "$file"; then
    echo "File: $file"
    echo "Before:"
    grep --color=always -n "$search_string" "$file"
    echo "After:"

    # Perform the replacement and show the result
    sed -e "s/$search_string/$replace_string/g" "$file"

    if [[ "$write_flag" == "--write" ]]; then
      # Write changes to file
      sed -i.bak -e "s/$search_string/$replace_string/g" "$file"
      echo "Changes written to $file"
    else
      echo "Dry run: changes not written to $file"
    fi
  else
    echo "No occurrences of '$search_string' found in $file"
  fi
  echo
done
