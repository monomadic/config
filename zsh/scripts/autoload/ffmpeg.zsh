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

# Extract video stream (remove audio)
ffmpeg-extract-video() {
  if [[ $# -ne 2 ]]; then
    print -P "%F{red}Usage:%f ffmpeg-extract_video <input_file> <output_video_file>"
    return 1
  fi
  ffmpeg -i "$1" -an -c:v copy "$2"
}

# Extract audio from a video file
ffmpeg-extract-audio() {
  if [[ $# -ne 2 ]]; then
    print -P "%F{red}Usage:%f ffmpeg-extract_audio <input_file> <output_audio_file>"
    return 1
  fi
  ffmpeg -i "$1" -vn -c:a copy "$2"
}

# Combine multiple video files into one
ffmpeg-concat-videos() {
  if [[ $# -ne 2 ]]; then
    print -P "%F{red}Usage:%f ffmpeg-concat_videos <file_list.txt> <output_file>"
    print -P "Example file_list.txt contents: %F{green}file 'video1.mp4'\nfile 'video2.mp4'%f"
    return 1
  fi
  ffmpeg -f concat -safe 0 -i "$1" -c copy "$2"
}
