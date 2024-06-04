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

local MUSIC_VIDEO_FORMAT="%(artist)s - %(title)s.%(ext)s"

function yt-download-mp4() {
    local url="$1"

		if [[ -z "$url" ]]; then
				echo "Usage: ${0:t} <url>"
				return 1
		fi

    yt-dlp \
			--format "bestvideo[vcodec^=avc1]+bestaudio[acodec^=aac]/bestvideo[vcodec^=avc1]+bestaudio/best" \
			--output "${output_template}" \
			--cookies-from-browser brave \
			--embed-metadata \
			--merge-output-format mp4 \
			$@
}

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
		--format bestvideo+bestaudio/best \
    --output "${output_template}" \
    --cookies-from-browser brave \
    --embed-metadata \
    $0
}

function yt-music-video-mp4 {
  local url="$1"
  local output_template="%(artist)s - %(title)s.%(ext)s"

	if [[ -z "$url" ]]; then
			echo "Usage: ${0:t} <url>"
			return 1
	fi

  yt-dlp -v \
		--format 'bestvideo[vcodec^=avc1]+bestaudio[acodec^=aac]/bestvideo[vcodec^=avc1]+bestaudio/best' \
    --output "${output_template}" \
		--merge-output-format mp4 \
    --cookies-from-browser brave \
    --embed-metadata \
    $@
}

function yt-porn {
  local url="$1"
  local output_template="[%(uploader)s] %(title)s.%(ext)s"

	if [[ -z "$url" ]]; then
			echo "Usage: ${0:t} <url>"
			return 1
	fi

  yt-dlp -v \
    --output "${output_template}" \
		--format 'bestvideo[vcodec^=avc1]+bestaudio[acodec^=aac]/bestvideo[vcodec^=avc1]+bestaudio/best' \
    --cookies-from-browser brave \
    --merge-output-format mp4 \
    --embed-metadata \
    $@
}

function yt-nightly-porn {
  local url="$1"
  local output_template="[%(uploader)s] %(title)s.%(ext)s"

	if [[ -z "$url" ]]; then
			echo "Usage: ${0:t} <url>"
			return 1
	fi

  yt-dlp-nightly -v \
    --output "${output_template}" \
		--format 'bestvideo[vcodec^=avc1]+bestaudio[acodec^=aac]/bestvideo[vcodec^=avc1]+bestaudio/best' \
    --cookies-from-browser brave \
    --merge-output-format mp4 \
    --embed-metadata \
    $@
}

function yt-porn-firefox {
  local url="$1"
  local output_template="[%(uploader)s] %(title)s.%(ext)s"

	if [[ -z "$url" ]]; then
			echo "Usage: ${0:t} <url>"
			return 1
	fi

  yt-dlp -v \
    --output "${output_template}" \
		--format 'bestvideo[vcodec^=avc1]+bestaudio[acodec^=aac]/bestvideo[vcodec^=avc1]+bestaudio/best' \
		--user-agent "Mozilla/5.0 (compatible; Googlebot/2.1; +http://www.google.com/bot.html)" \
    --cookies-from-browser firefox \
    --merge-output-format mp4 \
    --embed-metadata \
    $@
}

function vid-info {
    if [ -z "$1" ]; then
        echo "Usage: video-info <video_file>"
        return 1
    fi

		echo "\n$1:"
    ffprobe -v error \
        -show_entries format=duration,size,bit_rate \
        -show_entries stream=codec_name,width,height,r_frame_rate \
        -of default=noprint_wrappers=1 "$@"
}

function vid-info-color() {
    if [ -z "$1" ]; then
        echo "Usage: vid-info-color <video_file>"
        return 1
    fi

    # Colors
    RED=$(tput setaf 1)
    GREEN=$(tput setaf 2)
    YELLOW=$(tput setaf 3)
    BLUE=$(tput setaf 4)
    MAGENTA=$(tput setaf 5)
    CYAN=$(tput setaf 6)
    RESET=$(tput sgr0)

    # Output the video file name
    echo "\n${CYAN}File:${RESET} ${GREEN}$1${RESET}\n"

    # Get video info
    info=$(ffprobe -v error \
        -show_entries format=duration,size,bit_rate \
        -show_entries stream=codec_name,width,height,r_frame_rate \
        -of default=noprint_wrappers=1 "$@")

    # Display the information with color formatting
    echo "${YELLOW}Video Information:${RESET}"
    echo "$info" | while IFS= read -r line; do
        key=$(echo "$line" | cut -d'=' -f1)
        value=$(echo "$line" | cut -d'=' -f2-)

        case $key in
            duration) echo "${MAGENTA}Duration:${RESET} ${value}" ;;
            size) echo "${MAGENTA}Size:${RESET} ${value}" ;;
            bit_rate) echo "${MAGENTA}Bit Rate:${RESET} ${value}" ;;
            codec_name) echo "${MAGENTA}Codec Name:${RESET} ${value}" ;;
            width) echo "${MAGENTA}Width:${RESET} ${value}" ;;
            height) echo "${MAGENTA}Height:${RESET} ${value}" ;;
            r_frame_rate) echo "${MAGENTA}Frame Rate:${RESET} ${value}" ;;
            *) echo "${key}: ${value}" ;;
        esac
    done
}

function yt-batch-porn {
  local output_template="$HOME/_inbox/[%(uploader)s] %(title)s.%(ext)s"
	local batch_file="$HOME/.ytdl-batch-porn"

	if [[ ! -e "$batch_file" ]]; then
		echo "$batch_file not found, creating..."
		touch "$batch_file"
	fi

  yt-dlp -v \
    --output "${output_template}" \
		--format 'bestvideo[vcodec^=avc1]+bestaudio[acodec^=aac]/bestvideo[vcodec^=avc1]+bestaudio/best' \
    --cookies-from-browser firefox \
    --merge-output-format mp4 \
    --batch-file "${batch_file}" \
    --download-archive '$HOME/.ytdl-archive' \
    --embed-metadata \
    --add-metadata \
    --parse-metadata '%(title)s:%(meta_title)s' \
    --parse-metadata '%(uploader)s:%(meta_artist)s' \
    --write-info-json \
    --write-annotations \
		--get-comments \
		--check-formats \
		--concurrent-fragments 3 \
		$@

	# move .json files out
	mv $HOME/_inbox/*.json $HOME/_inbox/
}

function yt-fg-archive {
    # Define format string for better readability
    local format_str="(
        bestvideo[vcodec^=av01][height>=4320][fps>30]/bestvideo[vcodec^=vp09.02][height>=4320][fps>30]/bestvideo[vcodec^=vp09.00][height>=4320][fps>30]/bestvideo[vcodec^=avc1][height>=4320][fps>30]/bestvideo[height>=4320][fps>30]/
        bestvideo[vcodec^=av01][height>=4320]/bestvideo[vcodec^=vp09.02][height>=4320]/bestvideo[vcodec^=vp09.00][height>=4320]/bestvideo[vcodec^=avc1][height>=4320]/bestvideo[height>=4320]/
        bestvideo[vcodec^=av01][height>=2880][fps>30]/bestvideo[vcodec^=vp09.02][height>=2880][fps>30]/bestvideo[vcodec^=vp09.00][height>=2880][fps>30]/bestvideo[vcodec^=avc1][height>=2880][fps>30]/bestvideo[height>=2880][fps>30]/
        bestvideo[vcodec^=av01][height>=2880]/bestvideo[vcodec^=vp09.02][height>=2880]/bestvideo[vcodec^=vp09.00][height>=2880]/bestvideo[vcodec^=avc1][height>=2880]/bestvideo[height>=2880]/
        bestvideo[vcodec^=av01][height>=2160][fps>30]/bestvideo[vcodec^=vp09.02][height>=2160][fps>30]/bestvideo[vcodec^=vp09.00][height>=2160][fps>30]/bestvideo[vcodec^=avc1][height>=2160][fps>30]/bestvideo[height>=2160][fps>30]/
        bestvideo[vcodec^=av01][height>=2160]/bestvideo[vcodec^=vp09.02][height>=2160]/bestvideo[vcodec^=vp09.00][height>=2160]/bestvideo[vcodec^=avc1][height>=2160]/bestvideo[height>=2160]/
        bestvideo[vcodec^=av01][height>=1440][fps>30]/bestvideo[vcodec^=vp09.02][height>=1440][fps>30]/bestvideo[vcodec^=vp09.00][height>=1440][fps>30]/bestvideo[vcodec^=avc1][height>=1440][fps>30]/bestvideo[height>=1440][fps>30]/
        bestvideo[vcodec^=av01][height>=1440]/bestvideo[vcodec^=vp09.02][height>=1440]/bestvideo[vcodec^=vp09.00][height>=1440]/bestvideo[vcodec^=avc1][height>=1440]/bestvideo[height>=1440]/
        bestvideo[vcodec^=av01][height>=1080][fps>30]/bestvideo[vcodec^=vp09.02][height>=1080][fps>30]/bestvideo[vcodec^=vp09.00][height>=1080][fps>30]/bestvideo[vcodec^=avc1][height>=1080][fps>30]/bestvideo[height>=1080][fps>30]/
        bestvideo[vcodec^=av01][height>=1080]/bestvideo[vcodec^=vp09.02][height>=1080]/bestvideo[vcodec^=vp09.00][height>=1080]/bestvideo[vcodec^=avc1][height>=1080]/bestvideo[height>=1080]/
        bestvideo[vcodec^=av01][height>=720][fps>30]/bestvideo[vcodec^=vp09.02][height>=720][fps>30]/bestvideo[vcodec^=vp09.00][height>=720][fps>30]/bestvideo[vcodec^=avc1][height>=720][fps>30]/bestvideo[height>=720][fps>30]/
        bestvideo[vcodec^=av01][height>=720]/bestvideo[vcodec^=vp09.02][height>=720]/bestvideo[vcodec^=vp09.00][height>=720]/bestvideo[vcodec^=avc1][height>=720]/bestvideo[height>=720]/
        bestvideo[vcodec^=av01][height>=480][fps>30]/bestvideo[vcodec^=vp09.02][height>=480][fps>30]/bestvideo[vcodec^=vp09.00][height>=480][fps>30]/bestvideo[vcodec^=avc1][height>=480][fps>30]/bestvideo[height>=480][fps>30]/
        bestvideo[vcodec^=av01][height>=480]/bestvideo[vcodec^=vp09.02][height>=480]/bestvideo[vcodec^=vp09.00][height>=480]/bestvideo[vcodec^=avc1][height>=480]/bestvideo[height>=480]/
        bestvideo[vcodec^=av01][height>=360][fps>30]/bestvideo[vcodec^=vp09.02][height>=360][fps>30]/bestvideo[vcodec^=vp09.00][height>=360][fps>30]/bestvideo[vcodec^=avc1][height>=360][fps>30]/bestvideo[height>=360][fps>30]/
        bestvideo[vcodec^=av01][height>=360]/bestvideo[vcodec^=vp09.02][height>=360]/bestvideo[vcodec^=vp09.00][height>=360]/bestvideo[vcodec^=avc1][height>=360]/bestvideo[height>=360]/
        bestvideo[vcodec^=av01][height>=240][fps>30]/bestvideo[vcodec^=vp09.02][height>=240][fps>30]/bestvideo[vcodec^=vp09.00][height>=240][fps>30]/bestvideo[vcodec^=avc1][height>=240][fps>30]/bestvideo[height>=240][fps>30]/
        bestvideo[vcodec^=av01][height>=240]/bestvideo[vcodec^=vp09.02][height>=240]/bestvideo[vcodec^=vp09.00][height>=240]/bestvideo[vcodec^=avc1][height>=240]/bestvideo[height>=240]/
        bestvideo[vcodec^=av01][height>=144][fps>30]/bestvideo[vcodec^=vp09.02][height>=144][fps>30]/bestvideo[vcodec^=vp09.00][height>=144][fps>30]/bestvideo[vcodec^=avc1][height>=144][fps>30]/bestvideo[height>=144][fps>30]/
        bestvideo[vcodec^=av01][height>=144]/bestvideo[vcodec^=vp09.02][height>=144]/bestvideo[vcodec^=vp09.00][height>=144]/bestvideo[vcodec^=avc1][height>=144]/bestvideo[height>=144]
    )+(bestaudio[acodec^=opus]/bestaudio)/best"

    # Define common options
    local common_opts="
        --verbose
        --force-ipv4
        --sleep-requests 1
        --sleep-interval 5
        --max-sleep-interval 30
        --ignore-errors
        --no-continue
        --no-overwrites
        --download-archive archive.log
        --add-metadata
        --parse-metadata '%(title)s:%(meta_title)s'
        --parse-metadata '%(uploader)s:%(meta_artist)s'
        --write-description
        --write-info-json
        --write-annotations
        --write-thumbnail
        --embed-thumbnail
        --all-subs
        --embed-subs
        --get-comments
        --check-formats
        --concurrent-fragments 3
        --match-filter '!is_live & !live'
        --output '%(title)s - %(uploader)s - %(upload_date)s/%(title)s - %(uploader)s - %(upload_date)s [%(id)s].%(ext)s'
        --merge-output-format 'mkv'
        --datebefore '$(date --date="30 days ago" +%Y%m%d)'
        --throttled-rate 100K
        --batch-file '$HOME/.ytdl-batch'
    "

    # Run yt-dlp with all options
    yt-dlp --format "$format_str" $common_opts 2>&1 | tee output.log
}


function tag-comment() {
    if [[ $# -lt 2 ]]; then
        echo "Usage: yt_comment_tag <video_file> <comment>"
        return 1
    fi

    local video_file="$1"
    local comment="$2"

    if [[ ! -f "$video_file" ]]; then
        echo "Error: File '$video_file' not found."
        return 1
    fi

    # Generate the output file name with "_tagged" appended
    local tagged_video="${video_file%.*}_tagged.${video_file##*.}"

    # Add the comment as a metadata tag using ffmpeg
    ffmpeg -i "$video_file" -metadata comment="$comment" -codec copy "$tagged_video"
    if [[ $? -eq 0 ]]; then
        echo "Tagged video saved as: $tagged_video"
    else
        echo "Error tagging the video."
        return 1
    fi
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

function convert-to-mp4() {
    local input_file="$1"
    local output_file="${input_file%.*}.mp4"

    if [ -z "$1" ]; then
        echo "Usage: convert-to-mp4 <input_file>"
        return 1
    fi

    ffmpeg -i "$input_file" -c copy "$output_file"

    if [ $? -eq 0 ]; then
        echo "Conversion successful: $output_file"
    else
        echo "Conversion failed"
    fi
}

function tag-rename {
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

function tag-embed() {
    # Check for two arguments
    if [ $# -ne 2 ]; then
        echo "Usage: ${0:t} <file> <url>"
        return 1
    fi

    local file="$1"
    local url="$2"

    # Ensure yt-dlp, ffprobe, and ffmpeg are installed
    for cmd in yt-dlp ffprobe ffmpeg; do
        if ! command -v $cmd &> /dev/null; then
            echo "$cmd is not installed. Please install it first."
            return 1
        fi
    done

    # Extract metadata using yt-dlp
    metadata=$(yt-dlp --skip-download --print-json "$url")
    if [ $? -ne 0 ]; then
        echo "Failed to extract metadata from $url"
        return 1
    fi

    # Use jq to extract relevant metadata
    title=$(echo "$metadata" | jq -r '.title')
    uploader=$(echo "$metadata" | jq -r '.uploader')
    upload_date=$(echo "$metadata" | jq -r '.upload_date')
    description=$(echo "$metadata" | jq -r '.description')

    # Extract existing metadata from the file using ffprobe
    existing_metadata=$(ffprobe -v quiet -print_format json -show_format "$file")
    if [ $? -ne 0 ]; then
        echo "Failed to extract existing metadata from $file"
        return 1
    fi

    # Add new metadata using ffmpeg
    ffmpeg -i "$file" \
        -metadata title="$title" \
        -metadata artist="$uploader" \
        -metadata description="$description" \
        -metadata date="$upload_date" \
        -codec copy "temp_$file"

    if [ $? -eq 0 ]; then
        mv "temp_$file" "$file"
        echo "Metadata added successfully to $file"
    else
        echo "Failed to add metadata to $file"
        rm "temp_$file"
        return 1
    fi
}

# Example usage:
# add_metadata "localfile.mp4" "https://www.youtube.com/watch?v=dQw4w9WgXcQ"

function tag-print {
    # Check if ffprobe is installed
    if ! command -v ffprobe &> /dev/null; then
        echo "ffprobe is not installed."
        return 1
    fi

    # Check for correct number of arguments
    if [ "$#" -ne 1 ]; then
        echo "Usage: tag-print <filename>"
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
