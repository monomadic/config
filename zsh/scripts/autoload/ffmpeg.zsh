ffmpeg-remux() {
  ffmpeg -i $1 -c copy $2
}

ffmpeg-recover-keyframes() {
  ffmpeg -i $1 -force_key_frames "expr:gte(t,n_forced*1)" -c:v libx264 -c:a copy $2
}

ffmpeg-info() {
  ffmpeg -i $1 -f null -
}

ffmpeg-repair-remux() {
  ffmpeg -i $1 -c:v copy -c:a copy $1-remux.mp4
}

ffprobe-info() {
  ffprobe -v error -show_streams -show_format $1
}
