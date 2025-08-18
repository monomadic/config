apple-music-dl() {
  gamdl \
		--cookies-path="$HOME/.config/music.apple.com_cookies.txt" \
		--output-path='.' \
    --template-folder-album='' \
    --template-folder-compilation='' \
    --template-folder-no-album='' \
    --template-file-single-disc='{artist} - {title}' \
    --template-file-multi-disc='{artist} - {title}' \
    --template-file-no-album='{artist} - {title}' \
    $@
}

apple-music-dl-hq() {
  gamdl \
		--cookies-path="$HOME/.config/music.apple.com_cookies.txt" \
		--output-path='.' \
    --template-folder-album='' \
    --template-folder-compilation='' \
    --template-folder-no-album='' \
    --template-file-single-disc='{artist} - {title}' \
    --template-file-multi-disc='{artist} - {title}' \
    --template-file-no-album='{artist} - {title}' \
		--codec-music-video='h265' \
		--codec-song='ask' \
    $@
}

mpv-vertical-stack-2() {
  if [[ $# -ne 2 ]]; then
    echo "Usage: ${0:t} <video1> <video2>"
    return 1
  fi

  local video1="$1"
  local video2="$2"

  mpv "$video1" --external-file="$video2" --lavfi-complex='[vid1] [vid2] hstack [vo]'
}

mpv-vertical-stack-2-scaled() {
  if [[ $# -ne 2 ]]; then
    echo "Usage: ${0:t} <video1> <video2>"
    return 1
  fi

  local video1="$1"
  local video2="$2"

  mpv "$video1" --external-file="$video2" \
    --lavfi-complex='[vid1]scale=-1:1024[scaled1];[vid2]scale=-1:1024[scaled2];[scaled1][scaled2]hstack[vo]'
}

mpv-vertical-stack-3() {
  if [[ $# -ne 3 ]]; then
    echo "Usage: ${0:t} <video1> <video2> <video3>"
    return 1
  fi

  local video1="$1"
  local video2="$2"
  local video3="$3"

  mpv "$video1" --external-file="$video2" --external-file="$video3" \
    --lavfi-complex='[vid1]scale=-1:1024[scaled1];[vid2]scale=-1:1024[scaled2];[vid3]scale=-1:1024[scaled3];[scaled1][scaled2][scaled3]hstack=inputs=3[vo]'
}

mpv-vertical-stack-3-stdin() {
  local -a files
  local current_dir="$PWD"

  echo "Debug: Current directory is $current_dir"

  # Read lines into an array
  local i=0
  while IFS= read -r line && ((i < 3)); do
    echo "Debug: Reading line $((i+1)): '$line'"
    if [[ -n "$line" ]]; then
      if [[ "$line" = /* ]]; then
        files[i]="$line"
      else
        files[i]="$current_dir/$line"
      fi
      ((i++))
    fi
  done

  echo "Debug: Array contents:"
  printf "files[%d]='%s'\n" "${!files[@]}" "${files[@]}"

  if [[ ${#files[@]} -ne 3 ]]; then
    echo "Error: Need exactly 3 valid file paths, got ${#files[@]}"
    return 1
  fi

  for ((i=0; i<${#files[@]}; i++)); do
    echo "Debug: File $((i+1)) is: '${files[i]}'"
    if [[ ! -f "${files[i]}" ]]; then
      echo "Error: File does not exist: ${files[i]}"
      return 1
    fi
  done

  # Now play the videos
  mpv \
    "${files[0]}" \
    "--external-file=${files[1]}" \
    "--external-file=${files[2]}" \
    "--lavfi-complex=[vid1]scale=-1:1024[scaled1];[vid2]scale=-1:1024[scaled2];[vid3]scale=-1:1024[scaled3];[scaled1][scaled2][scaled3]vstack=inputs=3[vo]"
}

# Find all subdirectories and check if they are Git repository roots
fd-git-repositories() {
	fd -t d . -d 10 --absolute-path | while read -r dir; do
			if [[ -d "$dir/.git" ]]; then
					echo "$dir"
			fi
	done
}

lsd-get-icon() {
    if [[ "$1" == "--help" || "$#" -ne 1 ]]; then
        echo "Usage: ${0:t} <file_or_directory>"
        echo "Extracts and displays the icon for the given file or directory using lsd."
        return 1
    fi

    target="$1"
    if [[ ! -e "$target" ]]; then
        echo "Error: '$target' does not exist."
        return 1
    fi

    dir="${target:h}"    # Get directory part of the path
    name="${target:t}"   # Get base name of the file/directory

    output=$(lsd --icon=always "$dir" | grep "$name")
    if [[ -n "$output" ]]; then
        echo "${output[1]}" # Extract the first character (icon)
    else
        echo "Error: Could not find icon for '$target'."
        return 1
    fi
}

exa-get-icon() {
    if [[ "$1" == "--help" || "$#" -ne 1 ]]; then
        echo "Usage: ${0:t} <file_or_directory>"
        echo "Extracts and displays the NerdFont icon for the given file or directory using exa."
        return 1
    fi

    target="$1"
    if [[ ! -e "$target" ]]; then
        echo "Error: '$target' does not exist."
        return 1
    fi

    dir="${target:h}"    # Get directory part of the path
    name="${target:t}"   # Get base name of the file/directory

    output=$(exa --icons -1 "$dir" | grep "$name")
    if [[ -n "$output" ]]; then
        echo "${output[1]}" # Extract the first character (icon)
    else
        echo "Error: Could not find icon for '$target'."
        return 1
    fi
}

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

function rename-format-porn() {
  local input="$1"
  echo "$input" | awk '{
    while (match($0, /\[[^]]*\]/)) {
        # Convert text before the match to lowercase and replace _ with space
        converted = tolower(substr($0, 1, RSTART-1))
        gsub("_", " ", converted)
				gsub(/ i /, " I ", converted)
        printf "%s%s", converted, substr($0, RSTART, RLENGTH)
        $0 = substr($0, RSTART + RLENGTH)
    }
    # Convert remaining text to lowercase and replace _ with space
    remaining = tolower($0)
		gsub(/[_-]/, " ", remaining)
		gsub(/ i /, " I ", converted)
    print remaining
  }'
}

function rename-porn {
  if [[ -z "$1" ]]; then
    echo "Usage: ${0:t} <file1.mp4> [file2.mp4] [...]"
    return 1
  fi

  for file in "$@"; do
    # Skip directories
    [[ -d "$file" ]] && continue

    local filename=$(basename "$file")
    local new_filename=$(rename-format-porn "$filename")

    echo "Rename:\n\t$filename\n\t$new_filename\n\nOk? (y/N)"
    read -r response

    if [[ "$response" =~ ^[Yy]$ ]]; then
      if [[ -e "$new_filename" ]]; then
				echo -e "Warning: '$new_filename' already exists.\nOverwrite? (y/N)"
        read -r response
        if [[ ! "$response" =~ ^[Yy]$ ]]; then
          echo "Operation aborted."
          continue
        fi
      fi
      echo "Renaming '$filename' to '$new_filename'"
      mv "$filename" "$new_filename"
    else
      echo "Skipping '$filename'"
    fi

		echo "Batch rename completed."
  done
}

function rename_files_dry_run() {
    for file in *; do
        # Skip directories
        [[ -d "$file" ]] && continue

        # Extract parts within brackets
        parts_with_brackets=()
        temp_file="$file"
        while [[ "$temp_file" =~ \[([^]]+)\] ]]; do
            parts_with_brackets+=("${match[1]}")
            temp_file="${temp_file//\[[^]]*\]/}"
        done

        # Lowercase and replace characters
        new_name="${temp_file:l}"
        new_name="${new_name//-/ }"
        new_name="${new_name//_/ }"

        # Re-insert parts within brackets
        for part in "${parts_with_brackets[@]}"; do
            new_name="${new_name/[]/[$part]}"
        done

        # Print old and new name
        echo "Would rename: \"$file\" -> \"$new_name\""
    done
}

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

# microcommit
function gc () {
	git add . &&
	git commit -a -m "$@" &&
	git pull &&
	git push
}

function clear-reset {
	clear
	zle && zle reset-prompt
}

function cd-up() {
	cd ..
	zle && zle reset-prompt
}


function yt-avc-format-filename() {
  yt-dlp -f 'bestvideo[vcodec^=avc1]+bestaudio[acodec^=aac]/bestvideo[vcodec^=avc1]+bestaudio/best' \
    --cookies-from-browser brave \
    --merge-output-format mp4 \
    --output '[%(uploader)s] %(title)s.%(ext)s' \
    --embed-metadata \
		--embed-thumbnail \
    --exec "ffmpeg -i {} -metadata comment='https://youtube.com/watch?v=%(id)s' -codec copy {}_tagged.%(ext)s" \
    "$@"
}

function yt-filename() {
    local video_url="$1"

    # Check if the URL argument is provided
    if [[ -z "$video_url" ]]; then
        echo "Usage: print_ytdlp_filename <video_url>"
        return 1
    fi

    # yt-dlp command to get the filename
    yt-dlp --get-filename \
			    --output '[%(uploader)s] %(title)s.%(ext)s' \
					"$video_url"
}

function yt-download-tagged-file() {
  local url="$1"
  # Template to control the output filename
  local output_template="%(uploader)s - %(title)s [%(id)s].%(ext)s"

  # Download the video using yt-dlp with embedded metadata
  yt-dlp -v \
    --format 'bestvideo+bestaudio' \
    --merge-output-format mp4 \
    --output "${output_template}" \
    --embed-metadata \
    --restrict-filenames \
    "${url}"

  # Generate new filename based on metadata (this assumes yt-dlp successfully downloads and names the file)
  local downloaded_filename=$(ls -t | grep -m 1 '\.mp4$')

  # Rename the file based on specific metadata (using a sample schema here, adjust as needed)
  local new_filename="$(yt-dlp --get-title "${url}" --skip-download) - $(yt-dlp --get-id "${url}" --skip-download).mp4"

  # Renaming the file
  mv "${downloaded_filename}" "${new_filename}"

  # Add URL and other metadata using ffmpeg
  ffmpeg -i "${new_filename}" -metadata comment="${url}" -codec copy "${new_filename%.mp4}_tagged.mp4"

  # Cleanup if needed
  rm "${new_filename}"
}

function yt-tag-rename {
    local url="$1"
    local file="$2"
    # Ensure URL and file are not empty
    if [[ -z "$url" || -z "$file" ]]; then
        echo "Usage: ${0:t} <url> <file>"
        return 1
    fi

		# Fetch metadata using yt-dlp and convert it to JSON
		local metadata=$(yt-dlp --verbose --skip-download --print-json "${url}")

		# Parse JSON to get uploader, title, and id
		local uploader=$(echo "${metadata}" | jq -r '.uploader // "Unknown Uploader"')
		local title=$(echo "${metadata}" | jq -r '.title // "Unknown Title"')
		local id=$(echo "${metadata}" | jq -r '.id // "Unknown ID"')
		local ext="${file##*.}"

    # Ensure uploader, title, and extension are not empty
    if [[ -z "$uploader" || -z "$title" ]]; then
        echo "Error: Unable to extract complete file information from URL."
        return 1
    fi

		# Capitalize each word of the title
		title="${(C)title:gs/ / }"

		# Construct new filename based on specified output template
		local new_filename="[${uploader}] ${title}.${ext}"

    # Rename the file
    mv "$file" "${file:h}/$new_filename"
    echo "\nSuccess! File renamed to: $new_filename"
}

yt-fetch-tags() {
    local url="$1"

    # Ensure URL and output file are provided
    if [[ -z "$url" ]]; then
				echo "Usage: ${0:t} <url>"
        return 1
    fi

    # Fetch metadata using yt-dlp and convert it to JSON
    local metadata=$(yt-dlp --skip-download --print-json "$url")

    # Check if metadata retrieval was successful
    if [[ -z "$metadata" ]]; then
        echo "Error: Failed to retrieve metadata."
        return 1
    fi

		echo "Metadata: ${metadata}"

    # # Write metadata and the source URL to the output file
    # echo "Source URL: $url" > "$output_file"
    # echo "$metadata" >> "$output_file"
    #
    # echo "Metadata written to: $output_file"
}

function yt-write-tags {
    local url="$1"
    local file_path="$2"

    # Ensure URL and output file are provided
    if [[ -z "$url" || -z "$file_path" ]]; then
				echo "Usage: ${0:t} <url> <file_path>"
        return 1
    fi

    # Fetch metadata using yt-dlp and convert it to JSON
    local metadata=$(yt-dlp --skip-download --print-json "$url")

    # Check if metadata retrieval was successful
    if [[ -z "$metadata" ]]; then
        echo "Error: Failed to retrieve metadata."
        return 1
    fi

		echo "Metadata: ${metadata}"

    # # Write metadata and the source URL to the output file
    # echo "Source URL: $url" > "$output_file"
    # echo "$metadata" >> "$output_file"
    #
    # echo "Metadata written to: $output_file"
}

# Example usage:
# url="https://www.youtube.com/watch?v=VIDEO_ID"
# file="path/to/downloaded_file.mp4"
# rename_file_with_yt_dlp "$url" "$file"

# Example usage:
# url="https://www.youtube.com/watch?v=VIDEO_ID"
# file="path/to/downloaded_file.mp4"
# rename_file_with_yt_dlp "$url" "$file"

function yt-rename() {
  # Check if the correct number of arguments is provided
  if [[ $# -ne 2 ]]; then
		echo "Usage: ${0:t} <file_path> <url>"
    return 1
  fi

  local file_path="$1"
  local url="$2"

  # Check if the file exists
  if [[ ! -f "${file_path}" ]]; then
    echo "Error: File does not exist at ${file_path}"
    return 1
  fi

  # Fetch metadata using yt-dlp and convert it to JSON
  local metadata=$(yt-dlp --skip-download --print-json "${url}")

  # Parse JSON to get uploader, title, and id
  local uploader=$(echo "${metadata}" | jq -r '.uploader // "Unknown Uploader"')
  local title=$(echo "${metadata}" | jq -r '.title // "Unknown Title"')
  local id=$(echo "${metadata}" | jq -r '.id // "Unknown ID"')
  local ext="${file_path##*.}"

  # Capitalize each word of the title
  title="${(C)title:gs/ / }"

  # Construct new filename based on specified output template
  local new_filename="[${uploader}] ${title}.${ext}"
  echo "New Filename: ${new_filename}"

	mv "${file_path}" "${new_filename}"

  # # Tag the original file with additional metadata including the URL
  # ffmpeg -i "${file_path}" -metadata title="${title}" -metadata synopsis="${id}" -metadata comment="${url}" -codec copy "${file_path%.*}_tagged.${ext}"

  # # Check if the tagged file was created successfully
  # if [[ -f "${file_path%.*}_tagged.${ext}" ]]; then
  #   # Remove the original file
  #   rm "${file_path}"
  #   # Rename the tagged file to the new filename
  #   mv "${file_path%.*}_tagged.${ext}" "${new_filename}"
  #   echo "File has been successfully tagged and renamed to '${new_filename}'."
  # else
  #   echo "Error: Failed to create tagged file."
  #   return 1
  # fi
}

function media-tag-download-and-rename() {
  # Check if the correct number of arguments is provided
  if [[ $# -ne 2 ]]; then
    echo "Usage: ${0:t} <file_path> <url>"
    echo "Purpose: download tags with yt-dlp and embed them into a given file."
    return 1
  fi

  local file_path="$1"
  local url="$2"

  # Check if the file exists
  if [[ ! -f "${file_path}" ]]; then
    echo "Error: File does not exist at ${file_path}"
    return 1
  fi

  # Fetch metadata using yt-dlp and convert it to JSON
  local metadata=$(yt-dlp --skip-download --print-json "${url}")

  # Parse JSON to get uploader, title, and id
  local uploader=$(echo "${metadata}" | jq -r '.uploader // "Unknown Uploader"')
  local title=$(echo "${metadata}" | jq -r '.title // "Unknown Title"')
  local id=$(echo "${metadata}" | jq -r '.id // "Unknown ID"')
  local ext="${file_path##*.}"

  # Capitalize each word of the title
  title="${(C)title:gs/ / }"

  # Construct new filename based on specified output template
  local new_filename="[${uploader}] ${title}.${ext}"

  echo "New Filename: ${new_filename}"

  # Tag the original file with additional metadata including the URL
  ffmpeg -i "${file_path}" -metadata title="${title}" -metadata synopsis="${id}" -metadata comment="${url}" -codec copy "${new_filename}"

  # Check if the tagged file was created successfully
  if [[ -f "${file_path%.*}_tagged.${ext}" ]]; then
    # Remove the original file
    rm "${file_path}"
    # Rename the tagged file to the new filename
    mv "${file_path%.*}_tagged.${ext}" "${new_filename}"
    echo "File has been successfully tagged and renamed to '${new_filename}'."
  else
    echo "Error: Failed to create tagged file."
    return 1
  fi
}


function rsync-one-way-backup() {
  local source="$1"
  local destination="$2"

  if [[ -d "$source" && -d "$destination" ]]; then
    rsync --archive --verbose --delete --ignore-existing "$source/" "$destination/"
  else
    echo "Usage: rsync-one-way-backup <source> <destination>"
  fi
}

function convert_mp4_to_switch_webp() {
    local input_dir=$1
    local output_dir=$2

    # Check if input and output directories are provided
    if [[ -z "$input_dir" || -z "$output_dir" ]]; then
        echo "Usage: convert_mp4_to_animated_webp <input_directory> <output_directory>"
        return 1
    fi

    # Create the output directory if it doesn't exist
    mkdir -p "$output_dir"

    # Convert all MP4 files in the input directory to animated WEBP
    for f in "$input_dir"/*.mp4; do
        local base_name=$(basename "$f" .mp4)
        ffmpeg -i "$f" \
               -vf "scale=1280:800,setsar=1" \
               -loop 0 \
               -c:v libwebp \
               -preset picture \
               -an \
               -compression_level 6 \
               -q:v 75 \
               "$output_dir/$base_name.webp"
        echo "Converted $f to $output_dir/$base_name.webp"
    done
    echo "All files have been converted."
}

# images
function magick_to_webp_lossy() {
  magick "$1" -quality 75 -define webp:lossless=false "${1%.*}.webp"
}
