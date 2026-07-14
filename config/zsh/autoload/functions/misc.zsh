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

settings-forklift-make-default() {
  defaults write -g NSFileViewer -string com.binarynights.ForkLift;
  defaults write com.apple.LaunchServices/com.apple.launchservices.secure LSHandlers -array-add '{LSHandlerContentType="public.folder";LSHandlerRoleAll="com.binarynights.ForkLift";}'
}

settings-finder-make-default() {
  defaults delete -g NSFileViewer
  defaults write com.apple.LaunchServices/com.apple.launchservices.secure LSHandlers -array-add '{LSHandlerContentType="public.folder";LSHandlerRoleAll="com.apple.finder";}'
}


# ytdl-stream
# Download with yt-dlp while simultaneously playing the growing file in mpv.
# Best with progressive/single-file formats. For sites that only offer split
# DASH/HLS streams, playback may not begin until muxing finishes.

yt-dlp-stream() {
  emulate -L zsh
  set -euo pipefail

  local url="${1:-}"
  shift || true

  [[ -n "$url" ]] || {
    print -u2 "usage: ytdl-stream <url> [yt-dlp args...]"
    return 1
  }

  command -v yt-dlp >/dev/null 2>&1 || {
    print -u2 "yt-dlp not found"
    return 1
  }

  command -v mpv >/dev/null 2>&1 || {
    print -u2 "mpv not found"
    return 1
  }

  local out_dir="${YTDL_STREAM_DIR:-$HOME/Movies/Streaming}"
  mkdir -p -- "$out_dir"

  local stamp title safe_title ext final_file
  stamp="$(date +%Y%m%d-%H%M%S)"

  # Get a stable title/ext up front so we can predict the filename.
  # Force a single-file/progressive preference so mpv can follow a growing file.
  title="$(yt-dlp \
    --print "%(title)s" \
    -f "b/bv*+ba/b" \
    --no-playlist \
    -- "$url" 2>/dev/null | head -n1)"

  ext="$(yt-dlp \
    --print "%(ext)s" \
    -f "b/bv*+ba/b" \
    --no-playlist \
    -- "$url" 2>/dev/null | head -n1)"

  [[ -n "$title" ]] || title="video"
  [[ -n "$ext" ]] || ext="mp4"

  # Keep this simple and filesystem-safe.
  safe_title="$(print -r -- "$title" \
    | tr '/:' '  ' \
    | tr -cd '[:alnum:][:space:]._+-[](){}#' \
    | sed -E 's/[[:space:]]+/ /g; s/^ +| +$//g')"

  final_file="$out_dir/${stamp} ${safe_title}.${ext}"

  print -r -- "→ $final_file"

  # Start download in background.
  #
  # --no-part is important here: mpv needs to open the real output path while it
  # grows. This works best when yt-dlp is writing one media file directly.
  yt-dlp \
    --no-part \
    --continue \
    --no-playlist \
    -f "b/bv*+ba/b" \
    -o "$final_file" \
    -- "$url" "$@" &
  local ytdlp_pid=$!

  # Wait for file creation and a little data.
  local waited=0
  while (( waited < 300 )); do
    if [[ -f "$final_file" ]]; then
      local sz
      sz=$(stat -f '%z' -- "$final_file" 2>/dev/null || echo 0)
      if (( sz > 1048576 )); then
        break
      fi
    fi

    if ! kill -0 "$ytdlp_pid" 2>/dev/null; then
      wait "$ytdlp_pid"
      local rc=$?
      (( rc == 0 )) || {
        print -u2 "yt-dlp exited before playable data was written"
        return "$rc"
      }
      break
    fi

    sleep 1
    (( waited++ ))
  done

  [[ -f "$final_file" ]] || {
    print -u2 "output file was not created"
    wait "$ytdlp_pid" || true
    return 1
  }

  # Play the growing file. The extra demux/cache headroom helps with files
  # that are still being appended. mpv supports local file playback and caching
  # behavior via its documented options.  [oai_citation:0‡mpv.io](https://mpv.io/manual/stable/?utm_source=chatgpt.com)
  mpv \
    --force-seekable=yes \
    --cache=yes \
    --demuxer-max-bytes=512MiB \
    --demuxer-readahead-secs=120 \
    -- "$final_file"

  wait "$ytdlp_pid"
}
