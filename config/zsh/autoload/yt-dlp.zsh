alias yt-json-dump="yt-dlp --write-info-json --skip-download "
alias yt-json-description="jq '.description' "
alias yt-thumbnail="yt-dlp --skip-download --write-thumbnail "
alias yt-thumbnail-jpg="yt-dlp --skip-download --write-thumbnail --convert-thumbnails jpg "
alias yt-porn="yt-dlp --porn"

local MUSIC_VIDEO_FORMAT="%(artist)s - %(title)s.%(ext)s"

ffprobe-show-info() {
  if [ -z "$1" ]; then
    echo "Usage: $0 <video_file>"
    return 1
  fi

  echo "\n$1:"
  ffprobe -v error \
    -show_entries format=duration,size,bit_rate \
    -show_entries stream=codec_name,width,height,r_frame_rate \
    -of default=noprint_wrappers=1 "$@"
}

ffprobe-info-color() {
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
    if ! command -v $cmd &>/dev/null; then
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

# https://youtube-dl.readthedocs.io/en/latest/
#
#	%(title)s: The title of the video.
#	%(id)s: The video identifier.
#	%(url)s: The URL of the video.
#	%(extractor)s: The name of the extractor (site/scraper).
#	%(upload_date)s: The upload date in YYYYMMDD format.
#	%(uploader)s: The uploader of the video.
#	%(uploader_id)s: The uploader identifier.
#	%(channel)s: The channel name.
#	%(channel_id)s: The channel identifier.
#	%(duration)s: The duration of the video in seconds.
#	%(view_count)s: The number of views.
#	%(like_count)s: The number of likes.
#	%(dislike_count)s: The number of dislikes.
#	%(comment_count)s: The number of comments.
#	%(ext)s: The file extension.
#	%(format)s: The format of the file.
#	%(format_id)s: The format identifier.
#	%(playlist)s: The name of the playlist.
#	%(playlist_index)s: The index of the video in the playlist.
#	%(playlist_id)s: The playlist identifier.
#	%(playlist_title)s: The playlist title.
#	%(playlist_uploader)s: The uploader of the playlist.
#	%(playlist_uploader_id)s: The uploader identifier of the playlist.
#	%(epoch)s: The UNIX timestamp of the download.
#	%(autonumber)s: A five-digit sequential number starting at 00001.
#	%(chapter)s: The name of the chapter the video is part of.
#	%(series)s: The series the video is part of.
#	%(season_number)s: The season number of the series.
#	%(episode_number)s: The episode number of the series.
#	%(track)s: The track name of the video.
#	%(artist)s: The artist of the video.
#	%(album)s: The album of the video.
#	%(genre)s: The genre of the video.
#	%(location)s: The location where the video was recorded.
#	%(resolution)s: The resolution of the video.
#	%(bitrate)s: The bitrate of the video.
#	%(filesize)s: The filesize of the video.
#	%(filesize_approx)s: The approximate filesize of the video.

# ============================================================================
# yt-dlp aliases & functions (re-homed from alias.zsh)
# ============================================================================
alias yt-dlp-ignore-archive="yt-dlp --no-download-archive"
alias yt-list-fields="yt-dlp --skip-download --print \"%()#j\" "
alias yt-batch-edit='$EDITOR $YT_DLP_BATCH_FILE'
yt-dlp-list-formats() {
  (( $# )) || { print -u2 "usage: yt-dlp-list-formats <yt-dlp args>"; return 2; }
  yt-dlp --no-download-archive --list-formats "$@"
}

yt-dlp-porn-batch-file() {
  (( $# )) || { print -u2 "usage: dl-porn-batch-file <batch_file> ...<yt-dlp args>"; return 2; }
  local batch_file=$1; shift
  yt-dlp --porn --batch-file="$batch_file" "$@"
}
# ============================================================================
# YT-DLP
# ============================================================================

yt-retag() {
  emulate -L zsh; set -euo pipefail
  local url="$1" src="$2" tpl="${3:-'%(title)s (%(extractor)s).%(ext)s'}"
  local dir="${src:A:h}" ext="${src:A:e}"

  local target
  target="$(yt-dlp --print-name --alias porn "$tpl" "$url")"
  target="${target%%$'\n'*}"
  [[ "$target" = /* ]] || target="$dir/$target"

  # keep existing ext by default
  target="${target%.*}.$ext"
  mv -- "${src:A}" "$target"
  print -r -- "$target"
}

yt-debug-extract() {
  yt-dlp -j -v "$@" \
  | jq '{title, fulltitle, description, channel, uploader, creator, creators, cast, tags, categories}'
}

yt-queue-add() {
  print -r -- "$@" >> ~/.yt-dlp-queue
}

yt-queue-run() {
  local archive_file="${YT_DLP_ARCHIVE_FILE:-$HOME/Library/Mobile Documents/com~apple~CloudDocs/Sync/archive.txt}"

  yt-dlp \
    -a ~/.yt-dlp-queue \
    --download-archive "$archive_file" \
    -P ~/Movies/Porn/Downloads \
    --ignore-errors \
    --newline \
    -R 20 --fragment-retries 20 --retry-sleep 2 \
    --concurrent-fragments 4
}

yt-queue-run-ext() {
  local archive_file="${YT_DLP_ARCHIVE_FILE:-$HOME/Library/Mobile Documents/com~apple~CloudDocs/Sync/archive.txt}"

  yt-dlp \
    -a ~/.yt-dlp-queue \
    --download-archive "$archive_file" \
    -P "/Volumes/Footage 1tb/" \
    --ignore-errors \
    --newline \
    -R 20 --fragment-retries 20 --retry-sleep 2 \
    --concurrent-fragments 4
}

alias .qa=yt-queue-add
alias .qr=yt-queue-run-ext
alias yt-dlp-youtube-embedded="yt-dlp --cookies-from-browser brave --continue --progress --verbose --retries infinite --fragment-retries infinite --socket-timeout 15 -f bestvideo+ba/best --embed-metadata --extractor-args 'youtube:player-client=tv_embedded'"
alias yt-dlp-tui="auto-ytdlp --concurrent 2 --download-dir $HOME/Movies/Porn/Downloads --archive-file $ICLOUD_HOME/Movies/Porn/archive.txt"
