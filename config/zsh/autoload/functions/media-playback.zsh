apple-music-dl() {
  gamdl \
		--cookies-path="$HOME/.config/music.apple.com_cookies.txt" \
		--output-path='.' \
    --album-folder-template='' \
    --compilation-folder-template='' \
    --no-album-folder-template='' \
    --single-disc-file-template='{artist} - {title}' \
    --multi-disc-file-template='{artist} - {title}' \
    --no-album-file-template='{artist} - {title}' \
    $@
}

apple-music-dl-hq() {
  gamdl \
		--cookies-path="$HOME/.config/music.apple.com_cookies.txt" \
		--output-path='.' \
    --album-folder-template='' \
    --compilation-folder-template='' \
    --no-album-folder-template='' \
    --single-disc-file-template='{artist} - {title}' \
    --multi-disc-file-template='{artist} - {title}' \
    --no-album-file-template='{artist} - {title}' \
		--music-video-codec-priority='h265' \
		--song-codec-priority='ask' \
    $@
}

mpv-vertical-stack-2() {
  if [[ $# -ne 2 ]]; then
    echo "Usage: ${0:t} <video1> <video2>"
    return 1
  fi

  local video1="$1"
  local video2="$2"

  mpv "$video1" --external-file="$video2" --lavfi-complex='[vid1] [vid2] hstack [vo]'
}

mpv-vertical-stack-2-scaled() {
  if [[ $# -ne 2 ]]; then
    echo "Usage: ${0:t} <video1> <video2>"
    return 1
  fi

  local video1="$1"
  local video2="$2"

  mpv "$video1" --external-file="$video2" \
    --lavfi-complex='[vid1]scale=-1:1024[scaled1];[vid2]scale=-1:1024[scaled2];[scaled1][scaled2]hstack[vo]'
}

mpv-vertical-stack-3() {
  if [[ $# -ne 3 ]]; then
    echo "Usage: ${0:t} <video1> <video2> <video3>"
    return 1
  fi

  local video1="$1"
  local video2="$2"
  local video3="$3"

  mpv "$video1" --external-file="$video2" --external-file="$video3" \
    --lavfi-complex='[vid1]scale=-1:1024[scaled1];[vid2]scale=-1:1024[scaled2];[vid3]scale=-1:1024[scaled3];[scaled1][scaled2][scaled3]hstack=inputs=3[vo]'
}

mpv-vertical-stack-3-stdin() {
  local -a files
  local current_dir="$PWD"

  echo "Debug: Current directory is $current_dir"

  # Read lines into an array
  local i=0
  while IFS= read -r line && ((i < 3)); do
    echo "Debug: Reading line $((i+1)): '$line'"
    if [[ -n "$line" ]]; then
      if [[ "$line" = /* ]]; then
        files[i]="$line"
      else
        files[i]="$current_dir/$line"
      fi
      ((i++))
    fi
  done

  echo "Debug: Array contents:"
  printf "files[%d]='%s'\n" "${!files[@]}" "${files[@]}"

  if [[ ${#files[@]} -ne 3 ]]; then
    echo "Error: Need exactly 3 valid file paths, got ${#files[@]}"
    return 1
  fi

  for ((i=0; i<${#files[@]}; i++)); do
    echo "Debug: File $((i+1)) is: '${files[i]}'"
    if [[ ! -f "${files[i]}" ]]; then
      echo "Error: File does not exist: ${files[i]}"
      return 1
    fi
  done

  # Now play the videos
  mpv \
    "${files[0]}" \
    "--external-file=${files[1]}" \
    "--external-file=${files[2]}" \
    "--lavfi-complex=[vid1]scale=-1:1024[scaled1];[vid2]scale=-1:1024[scaled2];[vid3]scale=-1:1024[scaled3];[scaled1][scaled2][scaled3]vstack=inputs=3[vo]"
}

# Find all subdirectories and check if they are Git repository roots
fd-git-repositories() {
	fd -t d . -d 10 --absolute-path | while read -r dir; do
			if [[ -d "$dir/.git" ]]; then
					echo "$dir"
			fi
	done
}
