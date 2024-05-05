
function yt-dlp-get-music-video {
  local url="$1"
  local output_template="%(uploader)s - %(title)s.%(ext)s"

	if [[ -z "$url" ]]; then
			echo "Usage: ${0:t} <url>"
			return 1
	fi

  yt-dlp -v \
		--format 'bestvideo[vcodec^=avc1]+bestaudio[acodec^=aac]/bestvideo[vcodec^=avc1]+bestaudio/best' \
    --cookies-from-browser brave \
    --merge-output-format mp4 \
    --output "${output_template}" \
    --embed-metadata \
    --embed-thumbnail \
		--exec "ffmpeg -i {} -metadata comment='%(webpage_url)s' -metadata synopsis='%(id)s' -codec copy '{}'" \
    --ignore-errors \
    "${url}"
}
