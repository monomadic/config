ip-address() {
	ifconfig | grep inet | awk '$1=="inet" && $2!="127.0.0.1" {print $2}'
}

wifi-ssid() {
	networksetup -getairportnetwork en0 | awk -F': ' '{print $2}'
}

rm-ds-store() {
  local input_dir="$1"
	fd '.DS_Store' $input_dir --exec rm -f
}

ffmpeg-convert-to-switch-webp() {
  local input_file="$1"
  local output_file="$2"
  local duration="$3"

  if [[ -z "$input_file" || -z "$output_file" || -z "$duration" ]]; then
    echo "Usage: ${0:t} <input_file> <output_file> <duration_in_seconds>"
    return 1
  fi

	ffmpeg -i "$input_file" -t "$duration" -vcodec libwebp -lossless 0 -compression_level 6 -q:v 50 -loop 0 -preset picture -an -vsync 0 "$output_file"
  # ffmpeg -i "$input_file" -t "$duration" -vf "scale=1280:720:force_original_aspect_ratio=decrease,pad=1280:720:(ow-iw)/2:(oh-ih)/2" -vcodec libwebp -compression_level 6 -q:v 80 -loop 0 "$output_file"
}

# function rename-format-porn() {
#   local input="$1"
#   echo "$input" | awk '{
#     while (match($0, /\[[^]]*\]/)) {
#         # Convert text before the match to lowercase and replace _ with space
#         converted = tolower(substr($0, 1, RSTART-1))
#         gsub("_", " ", converted)
# 				gsub(/ i /, " I ", converted)
#         printf "%s%s", converted, substr($0, RSTART, RLENGTH)
#         $0 = substr($0, RSTART + RLENGTH)
#     }
#     # Convert remaining text to lowercase and replace _ with space
#     remaining = tolower($0)
# 		gsub(/[_-]/, " ", remaining)
# 		gsub(/ i /, " I ", converted)
#     print remaining
#   }'
# }

# function rename-porn {
#   if [[ -z "$1" ]]; then
#     echo "Usage: ${0:t} <file1.mp4> [file2.mp4] [...]"
#     return 1
#   fi

#   for file in "$@"; do
#     # Skip directories
#     [[ -d "$file" ]] && continue

#     local filename=$(basename "$file")
#     local new_filename=$(rename-format-porn "$filename")

#     echo "Rename:\n\t$filename\n\t$new_filename\n\nOk? (y/N)"
#     read -r response

#     if [[ "$response" =~ ^[Yy]$ ]]; then
#       if [[ -e "$new_filename" ]]; then
# 				echo -e "Warning: '$new_filename' already exists.\nOverwrite? (y/N)"
#         read -r response
#         if [[ ! "$response" =~ ^[Yy]$ ]]; then
#           echo "Operation aborted."
#           continue
#         fi
#       fi
#       echo "Renaming '$filename' to '$new_filename'"
#       mv "$filename" "$new_filename"
#     else
#       echo "Skipping '$filename'"
#     fi

# 		echo "Batch rename completed."
#   done
# }

# function rename_files_dry_run() {
#     for file in *; do
#         # Skip directories
#         [[ -d "$file" ]] && continue

#         # Extract parts within brackets
#         parts_with_brackets=()
#         temp_file="$file"
#         while [[ "$temp_file" =~ \[([^]]+)\] ]]; do
#             parts_with_brackets+=("${match[1]}")
#             temp_file="${temp_file//\[[^]]*\]/}"
#         done

#         # Lowercase and replace characters
#         new_name="${temp_file:l}"
#         new_name="${new_name//-/ }"
#         new_name="${new_name//_/ }"

#         # Re-insert parts within brackets
#         for part in "${parts_with_brackets[@]}"; do
#             new_name="${new_name/[]/[$part]}"
#         done

#         # Print old and new name
#         echo "Would rename: \"$file\" -> \"$new_name\""
#     done
# }

function rename-as-tag-format() {
  for file in *; do
    if [[ -f $file ]]; then
      # Extract parts within and outside brackets
      parts=($(echo $file | grep -oE '\[[^]]*\]|[^[]+'))

      new_name=""
      for part in "${parts[@]}"; do
        if [[ $part == \[*\] ]]; then
          new_name+="$part"
        else
          new_name+=$(echo "$part" | tr '[:upper:]' '[:lower:]')
					# new_name+=$(echo "$part" | tr '[:upper:]' '[:lower:]' | tr '-_' ' ')
        fi
      done

      echo "$file" "$new_name"
      #mv -- "$file" "$new_name"
    fi
  done
}

function fzf-multi {
	fzf --exact --multi --bind "enter:select-all+accept,ctrl-c:abort" --header "search type: multi-select, fuzzy search, smart case" --color=header:#888888
}

# # microcommit
# function gc () {
# 	git add . &&
# 	git commit -a -m "$@" &&
# 	git pull &&
# 	git push
# }

function clear-reset {
	clear
	zle && zle reset-prompt
}

function cd-up() {
	cd ..
	zle && zle reset-prompt
}
