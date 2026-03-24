#!/usr/bin/env zsh

(($# == 0)) && {
  print -u2 -- "Usage: $0 file1.mov [file2.mov ...]"
  exit 1
}

for file in "$@"; do
  [[ ${file:l} != *.mov ]] && {
    print -- "Skipping $file: not a .mov file"
    continue
  }

  output="${file:r}.mp4"

  if ffmpeg -i "$file" -c:v copy -c:a copy "$output"; then
    print -- "Converted $file to $output"
  else
    print -u2 -- "Failed to convert $file"
  fi
done
