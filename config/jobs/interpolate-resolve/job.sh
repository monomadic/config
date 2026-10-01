#!/bin/zsh
# interpolate-resolve — job-folder workflow: frame-interpolate each input
# through DaVinci Resolve's Optical Flow, via the deployed interpolate-resolve.
#
# Tracked in the dotfiles repo (config/jobs/interpolate-resolve/job.sh) and
# linked into ~/jobs/interpolate-resolve/ by the Deployfile. Drop or `mv` a
# video into ~/jobs/interpolate-resolve/input/ and the result appears in
# output/ as <name>.hevc60.mp4.
#
# For one-off settings, interpolate-resolve-job sets up a separate generated
# workflow per fps/quality instead (interpolate-resolve-120fps, …); this one is
# the everyday default. Change it here, not in ~/jobs — that file is a link.
#
# Resolve renders one timeline at a time: set job-folder's Workers to 1 if
# another Resolve workflow may be busy at the same time.

set -euo pipefail

fps=60
quality=speed-warp-metal  # Speed Warp on Metal; interpolate-resolve --help lists the others
codec=hevc                # hevc (MP4) or prores (ProRes 422 Proxy MOV)

: "${INPUT:?run me through job-folder, or set INPUT and OUTPUT_DIR}"
: "${OUTPUT_DIR:?run me through job-folder, or set INPUT and OUTPUT_DIR}"

typeset -a render_args=( --fps "$fps" --quality "$quality" )
ext=mp4
if [[ "$codec" == prores ]]; then
  render_args+=( --prores )
  ext=mov
fi

# interpolate-resolve's own naming: clip.hevc60.mp4, clip.prores60.mov.
name="${INPUT_NAME:-${${INPUT:t}:r}}.$codec${fps//./p}.$ext"

# A rerun of an input that already finished is a no-op, not a second render.
if [[ -e "$OUTPUT_DIR/$name" ]]; then
  print -- "already done: $OUTPUT_DIR/$name"
  exit 0
fi

# Render into a hidden .partial/ and rename only on success, so a stopped or
# crashed render never leaves something in output/ that looks finished.
mkdir -p -- "$OUTPUT_DIR/.partial"
rm -f -- "$OUTPUT_DIR/.partial/$name"
"$HOME"/.local/bin/interpolate-resolve "${render_args[@]}" -- "${INPUT:A}" "$OUTPUT_DIR/.partial/$name"
mv -- "$OUTPUT_DIR/.partial/$name" "$OUTPUT_DIR/$name"
print -- "done: $OUTPUT_DIR/$name"
