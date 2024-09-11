ffmpeg-remux() {
  ffmpeg -i $1 -c copy $2
}

ffmpeg-recover-keyframes() {
  ffmpeg -i $1 -force_key_frames "expr:gte(t,n_forced*1)" -c:v libx264 -c:a copy $2
}
