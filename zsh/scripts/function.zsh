
function ffmpeg-convert-to-switch-webp() {
  local input_file="$1"
  local output_file="$2"
  local duration="$3"

  if [[ -z "$input_file" || -z "$output_file" || -z "$duration" ]]; then
    echo "Usage: webm_to_webp <input_file> <output_file> <duration_in_seconds>"
    return 1
  fi

	ffmpeg -i "$input_file" -t "$duration" -vcodec libwebp -lossless 0 -compression_level 6 -q:v 50 -loop 0 -preset picture -an -vsync 0 "$output_file"
  # ffmpeg -i "$input_file" -t "$duration" -vf "scale=1280:720:force_original_aspect_ratio=decrease,pad=1280:720:(ow-iw)/2:(oh-ih)/2" -vcodec libwebp -compression_level 6 -q:v 80 -loop 0 "$output_file"
}

function vlc-find() {
  local search_term="$1"
	# fd -i "$search_term" -E '.*\.(mp4|webp|webm|mkv|mov)$' --print0 | xargs -0 vlc --loop --random --no-repeat
	# fd -e mp4 -i "$search_term" | fzf --exact --multi --print0 --bind "enter:select-all+accept,ctrl-c:abort" | xargs -0 vlc
	fd -e mp4 -i "$search_term" | fzf --exact --multi --print0 --bind "enter:select-all+accept,ctrl-c:abort" | xargs -0 sh -c 'vlc --loop --random --no-repeat "$@"'
}
alias vlc-top-find="vlc-find \"\_\[\""

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


# yt-dlp
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
  local output_template="%(uploader)s - %(title)s [%(id)s].%(ext)s"
		#--postprocessor-args "ffmpeg:-metadata comment='%(webpage_url)s' -metadata synopsis='%(id)s' -codec copy" \
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

function yt-porn-rename {
    local url="$1"
    local file="$2"
    # Ensure URL and file are not empty
    if [[ -z "$url" || -z "$file" ]]; then
        echo "Usage: yt-porn-rename <url> <file>"
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

function yt-write-tags {
    local url="$1"
    local output_file="$2"

    # Ensure URL and output file are provided
    if [[ -z "$url" || -z "$output_file" ]]; then
        echo "Error: Both URL and output file path must be provided."
        return 1
    fi

    # Fetch metadata using yt-dlp and convert it to JSON
    local metadata=$(yt-dlp --skip-download --print-json "$url")

    # Check if metadata retrieval was successful
    if [[ -z "$metadata" ]]; then
        echo "Error: Failed to retrieve metadata."
        return 1
    fi

    # Write metadata and the source URL to the output file
    echo "Source URL: $url" > "$output_file"
    echo "$metadata" >> "$output_file"

    echo "Metadata written to: $output_file"
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