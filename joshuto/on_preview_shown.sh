#!/usr/bin/env bash

FILE_PATH="$1"			# Full path of the previewed file
PREVIEW_X_COORD="$2"		# x coordinate of upper left cell of preview area
PREVIEW_Y_COORD="$3"		# y coordinate of upper left cell of preview area
PREVIEW_WIDTH="$4"		# Width of the preview pane (number of fitting characters)
PREVIEW_HEIGHT="$5"		# Height of the preview pane (number of fitting characters)

TMP_FILE="$HOME/.cache/joshuto/thumbcache.png"

mimetype=$(file --mime-type -Lb "$FILE_PATH")

function image {
	kitty +kitten icat \
		--transfer-mode=file \
		--clear 2>/dev/null
	kitty +kitten icat \
		--transfer-mode=file \
		--place "${PREVIEW_WIDTH}x${PREVIEW_HEIGHT}@${PREVIEW_X_COORD}x${PREVIEW_Y_COORD}" \
		"$1" 2>/dev/null
}

case "$mimetype" in
	video/*)
		;;
	image/*)
		image "${FILE_PATH}"
		;;
	*)
		kitty +kitten icat \
			--transfer-mode=file \
			--clear 2>/dev/null
		;;
esac
