# https://youtube-dl.readthedocs.io/en/latest/
#
# %(title)s: The title of the video.
# %(id)s: The video ID.
# %(uploader)s: The name of the uploader.
# %(uploader_id)s: The uploader's ID.
# %(upload_date)s: The upload date (usually in YYYYMMDD format).
# %(duration)s: The duration of the video in seconds.
# %(format)s: The format code of the downloaded file.
# %(ext)s: The extension of the downloaded file (e.g., mp4, mp3).
# %(resolution)s: The resolution of the video.
# %(width)s and %(height)s: The width and height of the video.
# %(epoch)s: The Unix epoch timestamp at the start of download.
# %(autonumber)s: An automatically incremented number starting from 00001 or a specified value.

alias yt="yt-dlp"
alias yt-audio="yt-dlp -f 'bestaudio' --extract-audio --embed-metadata "
alias yt-avc="yt-dlp -f 'bestvideo[vcodec^=avc1]+bestaudio[acodec^=aac]/bestvideo[vcodec^=avc1]+bestaudio/best' --cookies-from-browser=brave --merge-output-format mp4 --embed-metadata "
alias yt-firefox="yt-dlp --cookies-from-browser=firefox "
alias yt-video="yt-dlp -f 'bestvideo[vcodec^=avc1]' --merge-output-format mp4 --cookies-from-browser=brave --embed-metadata "
alias yt-json-dump="yt-dlp --write-info-json --skip-download "
alias yt-json-description="jq '.description' "

function yt-music-video {
  local url="$1"
  local output_template="%(artist)s - %(title)s.%(ext)s"

	if [[ -z "$url" ]]; then
			echo "Usage: ${0:t} <url>"
			return 1
	fi

	# --format 'bestvideo[vcodec^=avc1]+bestaudio[acodec^=aac]/bestvideo[vcodec^=avc1]+bestaudio/best' \
  # --merge-output-format mp4 \
	#	--exec 'ffmpeg -i "{}" -metadata comment="%(webpage_url)s" -metadata synopsis="%(id)s" -codec copy "{}"' \
	#	--format 'bestvideo+bestaudio/best' \

  yt-dlp -v \
		--format bestvideo+bestaudio \
		--merge-output-format mp4 \
    --output "${output_template}" \
    --cookies-from-browser brave \
    --embed-metadata \
    "${url}"
}

function yt-porn {
  local url="$1"
  local output_template="[%(uploader)s] %(title)s.%(ext)s"

	if [[ -z "$url" ]]; then
			echo "Usage: ${0:t} <url>"
			return 1
	fi

	# ffmpeg -i "{}" -metadata comment="%(webpage_url)s" -metadata title="%(title)s" -codec copy "{}"
	#
	# download and rename
  yt-dlp -v \
    --output "${output_template}" \
		--format 'bestvideo[vcodec^=avc1]+bestaudio[acodec^=aac]/bestvideo[vcodec^=avc1]+bestaudio/best' \
    --cookies-from-browser brave \
    --merge-output-format mp4 \
    --embed-metadata \
    --embed-thumbnail \
    "${url}"
}

function mp4-tag-write-title {
  local file="$1"
  local title="$2"

	if [[ -z "$file" ]]; then
			echo "Usage: ${0:t} <file> <title>"
			return 1
	fi

	ffmpeg -i "${file}" -metadata title="%(title)s" -codec copy "${file}.tmp"
	mv "${file}.tmp" "${file}"

	echo "Successfully written title tag to ${file}"
}

function yt-porn-no-thumbnail {
  local url="$1"
  local output_template="[%(uploader)s] %(title)s.%(ext)s"

	if [[ -z "$url" ]]; then
			echo "Usage: ${0:t} <url>"
			return 1
	fi

	# download and rename
  yt-dlp -v \
    --output "${output_template}" \
		--format 'bestvideo[vcodec^=avc1]+bestaudio[acodec^=aac]/bestvideo[vcodec^=avc1]+bestaudio/best' \
    --cookies-from-browser brave \
    --merge-output-format mp4 \
    --embed-metadata \
    "${url}"

	# ffmpeg -i "{}" -metadata comment="%(webpage_url)s" -metadata title="%(title)s" -codec copy "{}"
}

function yt-dlp-download-and-embed-tags {
  local url="$1"

	if [[ -z "$url" ]]; then
			echo "Usage: ${0:t} <url>"
			return 1
	fi

  yt-dlp \
		--verbose \
		--format 'bestvideo[vcodec^=avc1]+bestaudio[acodec^=aac]/bestvideo[vcodec^=avc1]+bestaudio/best' \
    --cookies-from-browser brave \
    --merge-output-format mp4 \
    --embed-metadata \
    --embed-thumbnail \
		--exec 'ffmpeg -i "{}" -metadata comment="%(webpage_url)s" -metadata title="%(title)s" -codec copy "{}"' \
    "${url}"
}

# not working
function mp4-tag-fetch {
  local url="$1"
	if [[ -z "$url" ]]; then
			echo "Usage: ${0:t} <url>"
			return 1
	fi

	# yt-dlp --verbose --skip-download --print-json "${url}"
	# fetch metadata
	local json_metadata=$(yt-dlp --verbose --skip-download --print-json "${url}")

	echo "description: "
	echo "$json_metadata" |jq '.description'
}

function mp4-tag-rename {
  # Ensure an argument is provided
  if [[ -z "$1" ]]; then
    echo "Usage: ${0:t} <file>"
    return 1
  fi

  # Check if the file exists
  if [[ ! -f "$1" ]]; then
    echo "File does not exist: $1"
    return 1
  fi

  # Extract metadata using ffprobe
  artist=$(ffprobe -v error -show_entries format_tags=artist -of default=noprint_wrappers=1:nokey=1 "$1")
  title=$(ffprobe -v error -show_entries format_tags=title -of default=noprint_wrappers=1:nokey=1 "$1")

  # Check if both artist and title tags are available
  if [[ -z "$artist" || -z "$title" ]]; then
    echo "Required tags are missing in the file: $1"
    return 1
  fi

  # Construct new filename
  new_filename="[${artist}] ${title}.mp4"

  # Rename the file
  mv "$1" "$new_filename"

  echo "File renamed to: $new_filename"
}

# Usage:
# rename_mp4 "example.mp4"

# function mp4-add-tag {
#     # Check if ffmpeg is installed
#     if ! command -v ffmpeg &> /dev/null; then
#         echo "ffmpeg is not installed."
#         return 1
#     fi
#
#     # Check for correct number of arguments
#     if [ "$#" -ne 3 ]; then
#         echo "Usage: tag_mp4 <filename> <tag_name> <tag_content>"
#         return 1
#     fi
#
#     local file_name="$1"
#     local tag_name="$2"
#     local tag_content="$3"
#
#     # Check if the file exists
#     if [[ ! -f "$file_name" ]]; then
#         echo "File does not exist: $file_name"
#         return 1
#     fi
#
#     # Command to add or modify the metadata tag
#     ffmpeg -i "$file_name" -metadata "$tag_name"="$tag_content" -codec copy "temp_$file_name" && mv "temp_$file_name" "$file_name"
#
#     echo "Tagged $file_name with $tag_name: $tag_content"
# }

# alias mp4-tag-comment="

function mp4-tag-list {
    # Check if ffprobe is installed
    if ! command -v ffprobe &> /dev/null; then
        echo "ffprobe is not installed."
        return 1
    fi

    # Check for correct number of arguments
    if [ "$#" -ne 1 ]; then
        echo "Usage: list_mp4_tags <filename>"
        return 1
    fi

    local file_name="$1"

    # Check if the file exists
    if [[ ! -f "$file_name" ]]; then
        echo "File does not exist: $file_name"
        return 1
    fi

    # Command to list metadata
    echo "Metadata for ${file_name}:"
    ffprobe -loglevel error -show_entries format_tags -of ini "$file_name"
}
