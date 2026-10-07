#!/bin/zsh
# av1-quality — job-folder workflow: encode each input to AV1 (MKV) with
# encode-av1's "quality" profile (SVT-AV1 preset 4 / CRF 21, keeps fine detail; ~2x slower than balanced).
#
# Tracked in the dotfiles repo (config/jobs/av1-quality/job.sh) and linked
# into ~/jobs/av1-quality/ by the Deployfile. Drop or `mv` a video into its
# input/ and the result appears in output/ as <name>.av1.mkv.
#
# The codec settings live in bin/encode-av1 (10-bit 4:2:0 Main profile for the
# Apple hardware decoder; audio, subtitles and attachments copied). Change the
# profile or container here, not in ~/jobs — that file is a link. av1-quality
# is the same workflow with profile=quality.
#
# SVT-AV1 uses every core: set job-folder's Workers to 1 if another encode
# workflow may be busy at the same time.

set -euo pipefail

profile=quality   # fast | balanced | quality | archive — encode-av1 --help lists them
container=mkv       # mkv (copies every track) or mp4 (QuickTime-openable, AAC audio, no subs)

: "${INPUT:?run me through job-folder, or set INPUT and OUTPUT_DIR}"
: "${OUTPUT_DIR:?run me through job-folder, or set INPUT and OUTPUT_DIR}"

# encode-av1's own naming: clip.av1.mkv.
name="${INPUT_NAME:-${${INPUT:t}:r}}.av1.$container"

# A rerun of an input that already finished is a no-op, not a second encode.
if [[ -e "$OUTPUT_DIR/$name" ]]; then
  print -- "already done: $OUTPUT_DIR/$name"
  exit 0
fi

# Encode into a hidden .partial/ and rename only on success, so a stopped or
# crashed encode never leaves something in output/ that looks finished.
# encode-av1 removes its own output on failure, so nothing is left to resume.
mkdir -p -- "$OUTPUT_DIR/.partial"
rm -f -- "$OUTPUT_DIR/.partial/$name"
"$HOME"/.local/bin/encode-av1 --"$profile" --"$container" \
  --output-dir "$OUTPUT_DIR/.partial" -- "${INPUT:A}"
mv -- "$OUTPUT_DIR/.partial/$name" "$OUTPUT_DIR/$name"
print -- "done: $OUTPUT_DIR/$name"
