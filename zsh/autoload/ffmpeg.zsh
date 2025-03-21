# Find repeating frames using SSIM (outputs logs)
ffmpeg-find-repeating-frames() {
  if [[ -z "$1" ]]; then
    echo "Usage: ffmpeg-find-repeating-frames <input_file>"
    echo "  input_file  - Source video file"
    return 1
  fi

  local input="$1"

  ffmpeg -i "$input" -vf "ssim" -f null -
}

# Extract a loop section from a video
ffmpeg-loop-section() {
  if [[ -z "$1" || -z "$2" || -z "$3" ]]; then
    echo "Usage: ffmpeg-loop-section <input_file> <start_time> <end_time> [output_file]"
    echo "  input_file   - Source video file"
    echo "  start_time   - Start time of loop (format: HH:MM:SS or seconds)"
    echo "  end_time     - End time of loop"
    echo "  output_file  - Output file (default: input_loop.mp4)"
    return 1
  fi

  local input="$1"
  local start_time="$2"
  local end_time="$3"
  local output="${4:-${input%.*}_loop.mp4}"

  ffmpeg -i "$input" -ss "$start_time" -to "$end_time" -c copy "$output"
}

# Create a seamless loop with crossfade
ffmpeg-create-seamless-loop() {
  if [[ -z "$1" || -z "$2" ]]; then
    echo "Usage: ffmpeg-create-seamless-loop <input_file> <loop_duration> [output_file]"
    echo "  input_file   - Input loop file"
    echo "  loop_duration - Duration of loop in seconds"
    echo "  output_file  - Output file (default: input_seamless.mp4)"
    return 1
  fi

  local input="$1"
  local duration="$2"
  local output="${3:-${input%.*}_seamless.mp4}"

  ffmpeg -i "$input" -filter_complex "xfade=transition=fade:duration=1:offset=$((duration-1))" "$output"
}

# Converts input video to the Steam Deck native resolution (1280x800) at 60fps using AV1.
function ffmpeg-encode-steamdeck-av1() {
    if [[ $# -ne 1 ]]; then
        echo "Usage: ffmpeg-encode-steamdeck-av1() <input_file>"
        return 1
    fi
    local infile="$1"
    local base="${infile%.*}"
    local ext="${infile##*.}"
    local outfile="${base}#av1_native.${ext}"
    ffmpeg -i "$infile" -vf "scale=1280:800" -r 60 -c:v libaom-av1 -crf 30 -b:v 0 -c:a aac -b:a 256k "$outfile"
}

# Converts input video to the Steam Deck native resolution (1280x800) at 60fps using HEVC.
function ffmpeg-encode-steamdeck-hevc() {
    if [[ $# -ne 1 ]]; then
        echo "Usage: ffmpeg-encode-steamdeck-hevc() <input_file>"
        return 1
    fi
    local infile="$1"
    local base="${infile%.*}"
    local ext="${infile##*.}"
    local outfile="${base}#hevc_native.${ext}"
    ffmpeg -i "$infile" -vf "scale=1280:800" -r 60 -c:v libx265 -preset fast -crf 22 -c:a aac -b:a 256k "$outfile"
}

# Converts input video to the Steam Deck native resolution (1280x800) at 60fps using AV1,
# while printing a progress percentage. The output filename is auto-generated with a "#av1_native_progress" tag.
function ffmpeg-encode-steamdeck-av1-progress() {
    if [[ $# -ne 1 ]]; then
        echo "Usage: ffmpeg-encode-steamdeck-av1-progress <input_file>"
        return 1
    fi

    local infile="$1"
    local base="${infile%.*}"
    local ext="${infile##*.}"
    local outfile="${base}#av1_native_progress.${ext}"

    # Get total duration (in seconds) using ffprobe
    local duration
    duration=$(ffprobe -v error -select_streams v:0 -show_entries format=duration \
               -of default=noprint_wrappers=1:nokey=1 "$infile")
    if [[ -z "$duration" ]]; then
        echo "Error: Could not determine duration for $infile"
        return 1
    fi

    # Convert duration to milliseconds (rounded)
    local duration_ms
    duration_ms=$(printf "%.0f" "$(echo "$duration * 1000" | bc -l)")

    echo "Total duration: ${duration} seconds."

    # Run ffmpeg with progress reporting.
    # The progress info is output as key=value pairs which we parse for out_time_ms.
    ffmpeg -i "$infile" -vf "scale=1280:800" -r 60 \
        -c:v libaom-av1 -cpu-used 4 -crf 30 -b:v 0 \
        -c:a aac -b:a 256k \
        -progress pipe:1 "$outfile" 2>&1 | while IFS= read -r line; do
            if [[ "$line" == out_time_ms=* ]]; then
                # Get the current output time in milliseconds
                local out_time_ms=${line#out_time_ms=}
                # Calculate percentage (if duration_ms > 0)
                if (( duration_ms > 0 )); then
                    local percent
                    percent=$(echo "scale=2; ($out_time_ms / $duration_ms) * 100" | bc -l)
                    # Print progress (overwrite the same line)
                    echo -ne "Progress: ${percent}%\r"
                fi
            elif [[ "$line" == progress=end ]]; then
                echo -e "\nEncoding complete: ${outfile}"
            fi
        done
}

# Converts input video to the Steam Deck native resolution (1280x800) at 60fps using HEVC,
# while printing a progress percentage. The output filename is auto-generated with a "#hevc_native_progress" tag.
function ffmpeg-encode-steamdeck-hevc-progress() {
    if [[ $# -ne 1 ]]; then
        echo "Usage: ffmpeg-encode-steamdeck-hevc-progress <input_file>"
        return 1
    fi

    local infile="$1"
    local base="${infile%.*}"
    local ext="${infile##*.}"
    local outfile="${base}#hevc_native_progress.${ext}"

    # Get total duration (in seconds) using ffprobe
    local duration
    duration=$(ffprobe -v error -select_streams v:0 -show_entries format=duration \
               -of default=noprint_wrappers=1:nokey=1 "$infile")
    if [[ -z "$duration" ]]; then
        echo "Error: Could not determine duration for $infile"
        return 1
    fi

    # Convert duration to milliseconds (rounded)
    local duration_ms
    duration_ms=$(printf "%.0f" "$(echo "$duration * 1000" | bc -l)")

    echo "Total duration: ${duration} seconds."

    # Run ffmpeg with progress reporting.
    # The progress info is output as key=value pairs which we parse for out_time_ms.
    ffmpeg -i "$infile" -vf "scale=1280:800" -r 60 \
        -c:v libx265 -preset fast -crf 22 \
        -c:a aac -b:a 256k \
        -progress pipe:1 "$outfile" 2>&1 | while IFS= read -r line; do
            if [[ "$line" == out_time_ms=* ]]; then
                # Get the current output time in milliseconds
                local out_time_ms=${line#out_time_ms=}
                # Calculate percentage (if duration_ms > 0)
                if (( duration_ms > 0 )); then
                    local percent
                    percent=$(echo "scale=2; ($out_time_ms / $duration_ms) * 100" | bc -l)
                    # Print progress (overwrite the same line)
                    echo -ne "Progress: ${percent}%\r"
                fi
            elif [[ "$line" == progress=end ]]; then
                echo -e "\nEncoding complete: ${outfile}"
            fi
        done
}

# h.265 2-pass encoding (for strict bitrate control)
ffmpeg-encode-h265-2pass() {
  if [[ -z "$1" ]]; then
    echo "Usage: ffmpeg-encode-h265-2pass <input_file> [output_file] [bitrate] [maxrate] [bufsize] [preset]"
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

# Compresses input video in HEVC (libx265) while preserving original resolution, framerate, and HDR metadata.
# Output file is auto‐generated with a "#hevc_hdr" tag.
function ffmpeg-encode-hdr-hevc-progress() {
    if [[ $# -ne 1 ]]; then
        echo "Usage: ffmpeg-encode-hdr-hevc-progress <input_file>"
        return 1
    fi

    local infile="$1"
    local base="${infile%.*}"
    local ext="${infile##*.}"
    local outfile="${base}#hevc_hdr.${ext}"

    # Get total duration (in seconds) using ffprobe.
    local duration
    duration=$(ffprobe -v error -select_streams v:0 -show_entries format=duration \
               -of default=noprint_wrappers=1:nokey=1 "$infile")
    if [[ -z "$duration" ]]; then
        echo "Error: Could not determine duration for $infile"
        return 1
    fi

    # Convert duration to milliseconds.
    local duration_ms
    duration_ms=$(printf "%.0f" "$(echo "$duration * 1000" | bc -l)")

    echo "Total duration: ${duration} seconds."

    ffmpeg -i "$infile" \
           -c:v libx265 -preset slow -crf 18 \
           -pix_fmt yuv420p10le \
           -x265-params "hdr10=1:colorprim=bt2020:transfer=smpte2084:colormatrix=bt2020nc" \
           -c:a aac -b:a 256k \
           -progress pipe:1 "$outfile" 2>&1 | while IFS= read -r line; do
               if [[ "$line" == out_time_ms=* ]]; then
                   local out_time_ms=${line#out_time_ms=}
                   if (( duration_ms > 0 )); then
                       local percent
                       percent=$(echo "scale=2; ($out_time_ms / $duration_ms) * 100" | bc -l)
                       echo -ne "Progress: ${percent}%\r"
                   fi
               elif [[ "$line" == progress=end ]]; then
                   echo -e "\nEncoding complete: ${outfile}"
               fi
           done
}

# Compresses input video in AV1 (libaom-av1) while preserving original resolution, framerate, and HDR metadata.
# Output file is auto‐generated with a "#av1_hdr" tag.
function ffmpeg-encode-hdr-av1-progress() {
    if [[ $# -ne 1 ]]; then
        echo "Usage: ffmpeg-encode-hdr-av1-progress <input_file>"
        return 1
    fi

    local infile="$1"
    local base="${infile%.*}"
    local ext="${infile##*.}"
    local outfile="${base}#av1_hdr.${ext}"

    # Get total duration (in seconds) using ffprobe.
    local duration
    duration=$(ffprobe -v error -select_streams v:0 -show_entries format=duration \
               -of default=noprint_wrappers=1:nokey=1 "$infile")
    if [[ -z "$duration" ]]; then
        echo "Error: Could not determine duration for $infile"
        return 1
    fi

    # Convert duration to milliseconds.
    local duration_ms
    duration_ms=$(printf "%.0f" "$(echo "$duration * 1000" | bc -l)")

    echo "Total duration: ${duration} seconds."

    ffmpeg -i "$infile" \
           -c:v libaom-av1 -cpu-used 4 -crf 30 -b:v 0 \
           -pix_fmt yuv420p10le \
           -color_primaries bt2020 -color_trc smpte2084 -colorspace bt2020nc \
           -c:a aac -b:a 256k \
           -progress pipe:1 "$outfile" 2>&1 | while IFS= read -r line; do
               if [[ "$line" == out_time_ms=* ]]; then
                   local out_time_ms=${line#out_time_ms=}
                   if (( duration_ms > 0 )); then
                       local percent
                       percent=$(echo "scale=2; ($out_time_ms / $duration_ms) * 100" | bc -l)
                       echo -ne "Progress: ${percent}%\r"
                   fi
               elif [[ "$line" == progress=end ]]; then
                   echo -e "\nEncoding complete: ${outfile}"
               fi
           done
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
