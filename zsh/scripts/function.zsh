
function ffmpeg-convert-to-switch-webp() {
  local input_file="$1"
  local output_file="$2"
  local duration="$3"

  if [[ -z "$input_file" || -z "$output_file" || -z "$duration" ]]; then
    echo "Usage: webm_to_webp <input_file> <output_file> <duration_in_seconds>"
    return 1
  fi

	ffmpeg -i "$input_file" -t "$duration" -vcodec libwebp -lossless 0 -compression_level 6 -q:v 50 -loop 0 -preset picture -an -vsync 0 "$output_file"
  # ffmpeg -i "$input_file" -t "$duration" -vf "scale=1280:720:force_original_aspect_ratio=decrease,pad=1280:720:(ow-iw)/2:(oh-ih)/2" -vcodec libwebp -compression_level 6 -q:v 80 -loop 0 "$output_file"
}

function vlc-find() {
  local search_term="$1"
	# fd -i "$search_term" -E '.*\.(mp4|webp|webm|mkv|mov)$' --print0 | xargs -0 vlc --loop --random --no-repeat
	# fd -e mp4 -i "$search_term" | fzf --exact --multi --print0 --bind "enter:select-all+accept,ctrl-c:abort" | xargs -0 vlc
	fd -e mp4 -i "$search_term" | fzf --exact --multi --print0 --bind "enter:select-all+accept,ctrl-c:abort" | xargs -0 sh -c 'vlc --loop --random --no-repeat "$@"'
}
alias vlc-top-find="vlc-find \"\_\[\""

function iina-find() {
  local search_term="$1"
  fd -0 -i "$search_term" -E '.*\.(mp4|webp|webm|mkv|mov)$' | xargs -0 -I{} open -a IINA '{}' --args --mpv-repeat=inf
}

# microcommit
function gc () {
	git add . &&
	git commit -a -m "$@" &&
	git pull &&
	git push
}

function quickls () {
	echo
	echo
	ll
	echo
	zle && zle reset-prompt
}; zle -N quickls;

function clear-reset {
	clear
	hello
	zle && zle reset-prompt
}; zle -N clear-reset;

function cd-up() {
	cd ..
	zle && zle reset-prompt
}; zle -N cd-up;


# yt-dlp
function yt-avc-format-filename() {
  yt-dlp -f 'bestvideo[vcodec^=avc1]+bestaudio[acodec^=aac]/bestvideo[vcodec^=avc1]+bestaudio/best' \
    --cookies-from-browser brave \
    --merge-output-format mp4 \
    -o '[%(uploader)s] %(title)s.%(ext)s' \
    --embed-metadata \
    --exec "ffmpeg -i {} -metadata comment='https://youtube.com/watch?v=%(id)s' -codec copy {}_tagged.%(ext)s" \
    "$@"
}

function yt-avc-format-filename-ex() {
  yt-dlp -f 'bestvideo[vcodec^=avc1]+bestaudio[acodec^=aac]/bestvideo[vcodec^=avc1]+bestaudio/best' \
    --cookies-from-browser brave \
    --merge-output-format mp4 \
    -o '[%(uploader)s] %(title)s.%(ext)s' \
    --embed-metadata \
    --postprocessor-args "ffmpeg:-metadata comment='https://youtube.com/watch?v=%(id)s' -codec copy" \
    "$@"
}

function yt-print-filename() {
    local video_url="$1"

    # Check if the URL argument is provided
    if [[ -z "$video_url" ]]; then
        echo "Usage: print_ytdlp_filename <video_url>"
        return 1
    fi

    # yt-dlp command to get the filename
    yt-dlp --get-filename \
			    --output '[%(uploader)s] %(title)s.%(ext)s' \
					"$video_url"
}

function rsync-one-way-backup() {
  local source="$1"
  local destination="$2"

  if [[ -d "$source" && -d "$destination" ]]; then
    rsync --archive --verbose --delete --ignore-existing "$source/" "$destination/"
  else
    echo "Usage: rsync-one-way-backup <source> <destination>"
  fi
}

function rsync-one-way-backup-dry-run() {
  local source="$1"
  local destination="$2"

  if [[ -d "$source" && -d "$destination" ]]; then
    echo "Performing dry run to show changes..."
    rsync --archive --verbose --checksum --delete --dry-run --itemize-changes "$source/" "$destination/"

    echo "Review the above changes. Files marked with 'deleting' are to be deleted."
    read "response?Do you want to proceed with these changes? (y/n): "
    if [[ "$response" =~ ^[Yy]$ ]]; then
			rsync --archive --verbose --checksum --delete --ignore-existing "$source/" "$destination/"
      echo "Changes applied."
    else
      echo "Operation cancelled."
    fi
  else
    echo "Usage: rsync-one-way-backup-dry-run <source> <destination>"
  fi
}


function rename-ext() {
    local old_ext=$1
    local new_ext=$2

    # Check for the correct number of arguments
    if (( $# != 2 )); then
        echo "Usage: rename-ext <old_ext> <new_ext>"
        return 1
    fi

    # Loop over all files with the old extension in the current directory
    for file in *.$old_ext; do
        if [[ -f "$file" ]]; then
            local base_name="${file%.*}"
            mv "$file" "$base_name.$new_ext"
        fi
    done
}

# Example usage: rename-ext mov mp4

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
