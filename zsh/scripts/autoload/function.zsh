function ffmpeg-convert-to-switch-webp() {
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

function rename-file() {
  if [[ $# -eq 0 ]]; then
    echo "Usage: rename-file <filename>"
    return 1
  fi

  local original_file=$1
  if [[ ! -f $original_file ]]; then
    echo "Error: File '$original_file' not found."
    return 1
  fi

  local temp_file=$(mktemp /tmp/rename_file.XXXXXX)
  echo "$original_file" > "$temp_file"
  nvim "$temp_file"

  local new_filename=$(<"$temp_file")
  rm "$temp_file"

  if [[ -z "$new_filename" || "$new_filename" == "$original_file" ]]; then
    echo "No changes made."
    return 0
  fi

  if [[ -e "$new_filename" ]]; then
    echo "Error: File '$new_filename' already exists."
    return 1
  fi

  mv "$original_file" "$new_filename"
  echo "File renamed to '$new_filename'."
}

# Make the function available to zsh
autoload -Uz rename_file

function rename-append-resolution {
  if [[ -z "$1" ]]; then
    echo "Usage: ${0:t} <file.mp4>"
    return 1
  fi

  file="$1"
  resolution=$(ffprobe -v error -select_streams v:0 -show_entries stream=width,height -of csv=s=x:p=0 "$file")

  if [[ -z "$resolution" ]]; then
    echo "Error: Unable to determine resolution."
    return 1
  fi

	echo "Resolution found: $resolution"

  width=$(echo "$resolution" | cut -d'x' -f1)
  height=$(echo "$resolution" | cut -d'x' -f2)

  if (( height <= 720 )); then
    tag="[720]"
  elif (( height <= 1080 )); then
    tag="[1080]"
  elif (( height <= 1440 )); then
    tag="[1440]"
  else
    tag="[4K]"
  fi

  extension="${file##*.}"
  filename="${file%.*}"
  new_filename="${filename}${tag}.${extension}"

  if [[ -e "$new_filename" ]]; then
    echo "Warning: File '$new_filename' already exists. Do you want to overwrite it? (y/N)"
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
      echo "Operation aborted."
      return 1
    fi
  fi

  mv "$file" "$new_filename"
  echo "Renamed to: $new_filename"
}

# function porntag-rename() {
#   local input="$1"
#   echo "$input" | awk '{
#     while (match($0, /\[[^]]*\]/)) {
#         printf "%s%s", tolower(substr($0, 1, RSTART-1)), substr($0, RSTART, RLENGTH)
#         $0 = substr($0, RSTART + RLENGTH)
#     }
#     print tolower($0)
#   }'
# }

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

# function rename-all-dry-run {
#     for file in *; do
#         # Skip directories
#         [[ -d "$file" ]] && continue
#
# 				if [[ -f "$file" ]]; then
# 					local filename=$(basename "$file")
# 					local new_filename=$(rename-format-porn "$filename")
# 					echo "$file -> $new_filename\n"
# 				fi
# 		done
# }

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
alias fd-fzf="fd . |fzf-multi"
alias ff="fd-fzf"
alias fd-portrait="fd --fixed-strings '[portrait]' ."

# works
function vlc-filter() {
  local search_term="$1"
	# fd -i "$search_term" -E '.*\.(mp4|webp|webm|mkv|mov)$' --print0 | xargs -0 vlc --loop --random --no-repeat
	# fd -e mp4 -i "$search_term" | fzf --exact --multi --print0 --bind "enter:select-all+accept,ctrl-c:abort" | xargs -0 vlc
	fd -e mp4 -i "$search_term" | fzf --exact --multi --print0 --bind "enter:select-all+accept,ctrl-c:abort" | xargs -0 sh -c 'vlc --loop --random --no-repeat "$@"'
}

		#echo "$files" | xargs -0 -I{} open -a IINA --args --mpv-shuffle --mpv-loop-playlist "{}"

function iina-filter() {
  local search_term="$1"
  local files
  files=$(fd -e mp4 -i "$search_term" | fzf --exact --multi --print0 --bind "enter:select-all+accept,ctrl-c:abort")
  if [[ -n "$files" ]]; then
		for file in $files; do
				open -a IINA "$file"
		done
  fi
}

function open_with_iina() {
    local selected_files=$(fd . | fzf -m)
    if [[ -n "$selected_files" ]]; then
        for file in $selected_files; do
            open -a IINA "$file"
        done
    else
        echo "No files selected."
    fi
}

function vlc-play {
  local search_term="$1"
	fd --fixed-strings "$search_term" -0 |xargs -0 vlc
}
function vlc-play-top {
	vlc-play '🎖️'
}

function vlc-play-hot {
	vlc-play '🔥'
}

function vlc-play-cumshots {
	vlc-play '[cumshot]'
}

alias cd-babyblue-inbox="cd /Volumes/BabyBlue2TB/not-porn/___full-videos/_inbox"
alias cd-inbox="cd $HOME/_inbox"
alias vlc-top-find="vlc-filter \"\_\[\""
alias vlc-babyblue="cd-babyblue && vlc-filter"
alias vlc-babyblue-one="cd /Volumes/BabyBlue2TB/Videos/not-porn && vlc-find"
alias vlc-inbox="cd $HOME/_inbox && vlc-filter"
alias bbtop="cd-babyblue && vlc-play-top"
alias bbhot="cd-babyblue && vlc-play-hot"
alias is="index-search"
alias i="index-search"

local DIR_BABYBLUE="/Volumes/BabyBlue2TB"
alias bb-eject="diskutil eject $DIR_BABYBLUE"
alias bb-cd="cd $DIR_BABYBLUE"
alias cd-babyblue="cd $DIR_BABYBLUE/not-porn"
alias bb-play-vlc="cd-babyblue && vlc-filter"
alias bb-play-iina="cd-babyblue && iina-filter"

function vlc-find() {
		local search_term="$1"
		fd -e mp4 -i "$search_term" | fzf --exact --multi --print0 | xargs -0 sh -c 'vlc --loop --random --no-repeat "$@"'
}

function fd-video {
    local search_term="$1"
    fd -i "$search_term" -E '.*\.(mp4|webp|webm|mkv|mov)$'
}

function fzf-filter {
    fzf --exact --multi --print0
}

function vlc-find {
    local search_term="$1"
    fd-video "$search_term" | fzf-filter | xargs -0 vlc --loop --random --no-repeat
}

function vlc-ff {
  local search_term="$1"
	fd-video "$search_term" | fzf-filter |  xargs -0 sh -c 'echo '
}

# function fzf-vlc {
#     local file
#     file=$(fd . | fzf)
#     if [[ -n $file ]]; then
#         vlc "$file"
#     fi
# }
#
# function fzf-iina {
#     local file
#     file=$(fd . | fzf)
#     if [[ -n $file ]]; then
#         iina "$file"
#     fi
# }

function iina-find() {
  local search_term="$1"
  fd -0 -i "$search_term" -E '.*\.(mp4|webp|webm|mkv|mov)$' | xargs -0 -I{} open -a IINA '{}' --args --mpv-repeat=inf
}

# microcommit
function gc () {
	git add . &&
	git commit -a -m "$@" &&
	git pull &&
	git push
}

function quickls () {
	echo
	echo
	ll
	echo
	zle && zle reset-prompt
}; zle -N quickls;

function clear-reset {
	clear
	hello
	zle && zle reset-prompt
}; zle -N clear-reset;

function cd-up() {
	cd ..
	zle && zle reset-prompt
}; zle -N cd-up;


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

function yt-avc-format-filename-ex-old() {
  yt-dlp -f 'bestvideo[vcodec^=avc1]+bestaudio[acodec^=aac]/bestvideo[vcodec^=avc1]+bestaudio/best' \
    --cookies-from-browser brave \
    --merge-output-format mp4 \
    --output '[%(uploader)s] %(title)s.%(ext)s' \
    --embed-metadata \
		--embed-thumbnail \
    --postprocessor-args "ffmpeg:-metadata comment='https://youtube.com/watch?v=%(id)s' -codec copy" \
    "$@"
}

function yt-avc-format-filename-ex() {
  local url="$1"
  local output_template="[%(uploader)s] %(title)s.%(ext)s"
		#--postprocessor-args "ffmpeg:-metadata comment='%(webpage_url)s' -metadata synopsis='%(id)s' -codec copy" \
  yt-dlp -v \
		--format 'bestvideo[vcodec^=avc1]+bestaudio[acodec^=aac]/bestvideo[vcodec^=avc1]+bestaudio/best' \
    --cookies-from-browser brave \
    --merge-output-format mp4 \
    --embed-metadata \
    --embed-thumbnail \
    --embed-subs \
    --sub-format 'srt' \
    --convert-subs 'srt' \
    --write-auto-subs \
		--exec "echo {}; ffmpeg -i {} -metadata comment='%(webpage_url)s' -metadata synopsis='%(id)s' -codec copy '{}'" \
    --restrict-filenames \
    --ignore-errors \
    "${url}"
}

function yt-avc-format-filename-two-step() {
  local url="$1"
  local output_template="%(uploader)s - %(title)s [%(id)s].%(ext)s"

  yt-dlp -v \
    --format 'bestvideo[vcodec^=avc1]+bestaudio[acodec^=aac]/bestvideo[vcodec^=avc1]+bestaudio/best' \
    --cookies-from-browser brave \
    --merge-output-format mp4 \
    --output "${output_template}" \
    --embed-metadata \
    --embed-thumbnail \
    --embed-subs \
    --sub-format 'srt' \
    --convert-subs 'srt' \
    --write-auto-subs \
    --restrict-filenames \
    --ignore-errors \
    --exec "ffmpeg -i {} -metadata comment='$(yt-dlp --get-url "${url}" --skip-download)' -metadata synopsis='$(yt-dlp --get-id "${url}" --skip-download)' -codec copy {}_metadata.mp4" \
    "${url}"
}
alias yt-porn-two-step=yt-avc-format-filename-two-step

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

function yt-fetch-tags {
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

function yt-tag-and-rename() {
  # Check if the correct number of arguments is provided
  if [[ $# -ne 2 ]]; then
    echo "Usage: tag_file_with_metadata <file_path> <url>"
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

function rsync-one-way-backup-dry-run() {
  local source="$1"
  local destination="$2"

  if [[ -d "$source" && -d "$destination" ]]; then
    echo "Performing dry run to show changes..."
    rsync --archive --verbose --checksum --delete --dry-run --itemize-changes "$source/" "$destination/"

    echo "Review the above changes. Files marked with 'deleting' are to be deleted."
    read "response?Do you want to proceed with these changes? (y/n): "
    if [[ "$response" =~ ^[Yy]$ ]]; then
			rsync --archive --verbose --checksum --delete --ignore-existing "$source/" "$destination/"
      echo "Changes applied."
    else
      echo "Operation cancelled."
    fi
  else
    echo "Usage: rsync-one-way-backup-dry-run <source> <destination>"
  fi
}


function rename-ext() {
    local old_ext=$1
    local new_ext=$2

    # Check for the correct number of arguments
    if (( $# != 2 )); then
        echo "Usage: rename-ext <old_ext> <new_ext>"
        return 1
    fi

    # Loop over all files with the old extension in the current directory
    for file in *.$old_ext; do
        if [[ -f "$file" ]]; then
            local base_name="${file%.*}"
            mv "$file" "$base_name.$new_ext"
        fi
    done
}

# Example usage: rename-ext mov mp4

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