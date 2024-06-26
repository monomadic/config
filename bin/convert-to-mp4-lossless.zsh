#!/usr/bin/env zsh

	(($# == 0)) && {
  echo "Usage: $0 file1.mov [file2.mov ...]"
  exit 1
}

for file in "$@" {
  [[ "$file" != *.mov ]] && {
    echo "Skipping $file: not a .mov file"
    continue
  }

  output="${file:r}.mp4"
  ffmpeg -i "$file" -c:v copy -c:a copy "$output" && {
    echo "Converted $file to $output"
  } || {
    echo "Failed to convert $file" >&2
  }
}
