#!/bin/zsh

# Function to prompt for input with aligned labels
prompt_input() {
  printf "%-20s" "$1"
  read -r "$2"
}

# Prompt for user input
prompt_input "URL:" url
prompt_input "Creator:" creator
prompt_input "Title:" title
prompt_input "Site (FapHouse):" site
site=${site:-FapHouse}
prompt_input "Tags (optional):" tags
prompt_input "Is this a preview? (y/N):" is_preview

# Generate the formatted filename
if [[ -z "$tags" ]]; then
  formatted_filename="[$creator] $title [$site]"
else
  formatted_filename="[$creator] $title [$site] $tags"
fi

# Add #preview if it's a preview
if [[ "${is_preview:l}" == "y" || "${is_preview:l}" == "yes" ]]; then
  formatted_filename+=" #preview"
fi

# Add .mp4 extension
formatted_filename+=".mp4"

# Run the dl-url command
yt-url "$url" "$formatted_filename"

# Print a confirmation message
echo "Executed command: yt-url \"$url\" \"$formatted_filename\""
