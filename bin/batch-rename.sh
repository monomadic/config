#!/bin/bash

# Check if a directory is provided
if [ -z "$1" ]; then
    echo "Usage: $(basename "$0") <directory>"
    exit 1
fi

DIRECTORY=$(realpath "$1")  # Get the absolute path of the directory

# Create a temporary file
TMPFILE=$(mktemp)

# List all files in the provided directory and create an editable column next to it
ls "$DIRECTORY" | awk '{print $0 "\t" $0}' > "$TMPFILE"

# Open the temporary file in Neovim for editing
nvim "$TMPFILE"

# Process renaming
while IFS=$'\t' read -r oldname newname; do
    # Rename the file if new name is provided and different from old
    if [[ -n "$newname" && "$oldname" != "$newname" && -e "$DIRECTORY/$oldname" ]]; then
        mv -v -- "$DIRECTORY/$oldname" "$DIRECTORY/$newname"
    fi
done < "$TMPFILE"

# Remove the temporary file
rm "$TMPFILE"
