# h.265 encoding with CRF (best quality with variable bitrate)
encode-h265-crf() {
  if [[ -z "$1" ]]; then
    echo "Usage: encode-h265-crf <input_file> [output_file] [crf] [preset]"
    echo "  input_file   - Source video file"
    echo "  output_file  - Output file (default: input_h265.mp4)"
    echo "  crf          - Quality factor (lower is better, default: 22)"
    echo "  preset       - x265 preset (default: slow, options: ultrafast, fast, medium, slow, veryslow)"
    return 1
  fi

  local input="$1"
  local output="${2:-${input%.*}_h265.mp4}"
  local crf="${3:-22}"
  local preset="${4:-slow}"

  ffmpeg -i "$input" -c:v libx265 -preset "$preset" -crf "$crf" -tune grain \
         -x265-params "rd=6:psy-rd=2.0:aq-mode=3:aq-strength=1.2:qcomp=0.7" \
         -pix_fmt yuv420p10le -c:a aac -b:a 128k "$output"
}

# h.265 2-pass encoding (for strict bitrate control)
encode-h265-2pass() {
  if [[ -z "$1" ]]; then
    echo "Usage: h265_2pass <input_file> [output_file] [bitrate] [maxrate] [bufsize] [preset]"
    echo "  input_file   - Source video file"
    echo "  output_file  - Output file (default: input_h265.mp4)"
    echo "  bitrate      - Target bitrate (default: 15M)"
    echo "  maxrate      - Max bitrate (default: 30M)"
    echo "  bufsize      - Buffer size (default: 30M)"
    echo "  preset       - x265 preset (default: slow)"
    return 1
  fi

  local input="$1"
  local output="${2:-${input%.*}_h265.mp4}"
  local bitrate="${3:-15M}"
  local maxrate="${4:-30M}"
  local bufsize="${5:-30M}"
  local preset="${6:-slow}"

  ffmpeg -i "$input" -c:v libx265 -preset "$preset" -b:v "$bitrate" -maxrate "$maxrate" -bufsize "$bufsize" -pass 1 \
         -f null /dev/null && \
  ffmpeg -i "$input" -c:v libx265 -preset "$preset" -b:v "$bitrate" -maxrate "$maxrate" -bufsize "$bufsize" -pass 2 \
         -c:a aac -b:a 128k "$output"
}

# Function to remux a file (rebuild the container without re-encoding)
ffmpeg-remux() {
  if [[ $# -ne 2 ]]; then
    print -P "%F{red}Usage:%f ffmpeg-remux <input_file> <output_file>"
    return 1
  fi
  ffmpeg -i "$1" -c copy "$2"
}

# Function to recover keyframes (useful for streamable formats)
ffmpeg-recover-keyframes() {
  if [[ $# -ne 2 ]]; then
    print -P "%F{red}Usage:%f ffmpeg-recover_keyframes <input_file> <output_file>"
    return 1
  fi
  ffmpeg -i "$1" -force_key_frames "expr:gte(t,n_forced*1)" -c:v libx264 -c:a copy "$2"
}

# Function to display info about an input file using ffmpeg
ffmpeg-info() {
  if [[ $# -ne 1 ]]; then
    print -P "%F{red}Usage:%f ffmpeg-info <input_file>"
    return 1
  fi
  ffmpeg -i "$1" -f null -
}

ffmpeg-trim-intro() {
	for file in *.mp4; do
			ffmpeg -i "$file" -c copy -ss 00:00:03 "${file%.*}_nointro.mp4"
	done
}

ffmpeg-dump-frames() {
    if [[ $# -ne 1 ]]; then
        echo "Usage: ffmpeg-dump-frames <input_file>"
        return 1
    fi

    local input_file="$1"
    local output_dir="${input_file%.*}_frames"

    mkdir -p "$output_dir" || {
        echo "Failed to create output directory: $output_dir"
        return 1
    }

    ffmpeg -i "$input_file" "$output_dir/frame_%04d.png"
}

# Function to display errors in an input file using ffmpeg
ffmpeg-check-errors() {
  trap "return 1" INT  # Handle Ctrl+C

  [[ $# -eq 0 ]] && { print -P "%F{red}Usage:%f ffmpeg-check-errors <input_file> [input_file2 ...]"; return 1 }

  for file in $@; do
    print -P "%F{blue}Checking:%f $file"
    ffmpeg -v error -i "$file" -f null /dev/null || break
  done
}

ffmpeg-check() {
  trap "return 1" INT  # Handle Ctrl+C

  [[ $# -eq 0 ]] && { print -P "%F{red}Usage:%f ffmpeg-check <input_file> [input_file2 ...]"; return 1 }

  for file in $@; do
    print -P "%F{blue}Checking:%f $file"
    ffmpeg -i "$file" -f null /dev/null || break
  done
}

ffprobe-check-errors() {
  trap "return 1" INT

  [[ $# -eq 0 ]] && { print -P "%F{red}Usage:%f ffprobe-check-errors <input_file> [input_file2 ...]"; return 1 }

  for file in $@; do
    print -P "%F{blue}Checking:%f $file"
    ffprobe -v error "$file" || break
  done
}

# Function to remux and "repair" a file
ffmpeg-repair-remux() {
  if [[ $# -ne 1 ]]; then
    print -P "%F{red}Usage:%f ffmpeg-repair_remux <input_file>"
    return 1
  fi
  local output_file="${1%.*}-remux.mp4"
  ffmpeg -i "$1" -c:v copy -c:a copy "$output_file"
}

# Function to get detailed stream and format info using ffprobe
ffprobe-info() {
  if [[ $# -ne 1 ]]; then
    print -P "%F{red}Usage:%f ffprobe_info <input_file>"
    return 1
  fi
  ffprobe -v error -show_streams -show_format "$1"
}

# # Extract video stream (remove audio)
# ffmpeg-extract-video() {
#   if [[ $# -ne 1 ]]; then
#     print -P "%F{red}Usage:%f ffmpeg-extract-video <input_file>"
#     return 1
#   fi
#
#   # Split the filename and extension
#   local base="${1%.*}"
#   local ext="${1##*.}"
#
#   # Create output filename with -demux before extension
#   local output="${base}.video.${ext}"
#
#   ffmpeg -i "$1" -an -c:v copy "$output"
# }
#
# # Extract audio from a video file
# ffmpeg-extract-audio() {
#   if [[ $# -ne 1 ]]; then
#     print -P "%F{red}Usage:%f ffmpeg-extract-audio <input_file>"
#     return 1
#   fi
#
#   # Split the filename and extension
#   local base="${1%.*}"
#   local ext="${1##*.}"
#
#   # Create output filename with -demux before extension
#   local output="${base}.audio.${ext}"
#
#   ffmpeg -i "$1" -vn -c:a copy "$output"
# }
#
# # Extract video and audio from a video file
# ffmpeg-demux-simple() {
#   if [[ $# -ne 1 ]]; then
#     print -P "%F{red}Usage:%f ffmpeg-demux <input_file>"
#     return 1
#   fi
#
#   # Split the filename and extension
#   local base="${1%.*}"
#   local ext="${1##*.}"
#
#   # Create output filenames with -demux before extension
#   local video_output="${base}.video.${ext}"
#   local audio_output="${base}.audio.${ext}"
#
#   # Extract video (no audio) and audio (no video) in parallel
#   ffmpeg -i "$1" \
#     -map 0:v:0 -c:v copy -an "$video_output" \
#     -map 0:a:0 -c:a copy -vn "$audio_output"
# }

# Combine multiple video files into one
ffmpeg-concat-videos() {
  if [[ $# -lt 2 ]]; then
    print -P "%F{red}Usage:%f ffmpeg-concat_videos <output_file> <input_file1> [input_file2 ...]"
    print -P "%F{red}Example:%f ffmpeg-concat_videos output.mp4 video1.mp4 video2.mp4"
    return 1
  fi

  local output_file=$1
  shift
  local temp_file=$(mktemp)

  # Create the temporary file list for ffmpeg
  for file in "$@"; do
    print "file '$file'" >> "$temp_file"
  done

  ffmpeg -f concat -safe 0 -i "$temp_file" -c copy "$output_file"
  rm -f "$temp_file"
}

# Simple container rewrapping (lossless)
ffmpeg-rewrap-container() {
    if [ $# -ne 1 ]; then
        echo "Usage: ffmpeg-rewrap-container <input_file>"
        echo "Performs lossless container rewrapping of media files using ffmpeg"
        echo "Example: ffmpeg-rewrap-container movie.mp4"
        return 1
    fi

    input_file="$1"
    filename="${input_file%.*}"
    extension="${input_file##*.}"
    output_file="${filename}-rewrap.${extension}"

    ffmpeg -i "$input_file" -c copy "$output_file"
}
