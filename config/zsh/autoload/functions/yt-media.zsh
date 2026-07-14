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
					"$@"
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
