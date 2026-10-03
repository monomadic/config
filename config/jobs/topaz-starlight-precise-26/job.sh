#!/bin/zsh
# topaz-starlight-precise-26 — job-folder workflow: encode each input with
# Starlight Precise 2.6 at source size, via the deployed neuroserver-encode.
#
# Tracked in the dotfiles repo (config/jobs/topaz-starlight-precise-26/job.sh)
# and linked into ~/jobs/topaz-starlight-precise-26/ by the Deployfile. Drop or
# `mv` a video into its input/ and the result appears in output/ as
# "<name> [Topaz - Starlight Precise 2.6].mkv".
#
# The model settings mirror bin/lib/topaz-presets/enhancement/
# starlight-precise-26.toml; keep the two in step. Change them here, not in
# ~/jobs — that file is a link.
#
# Starlight is minutes per frame and wants the whole machine: set job-folder's
# Workers to 1 if another Topaz workflow may be busy at the same time. Sources
# narrower than 640 pixels crash the model, and its weights download only once
# the Topaz app has run it.

set -euo pipefail

preset_name="Starlight Precise 2.6"
model=slp-26
store=slp26
params='{"softness":1}'
profile=hevc-cbr-40mbps   # an output/ preset; prores-422-proxy is the other useful one
ext=mkv                   # that profile's container

: "${INPUT:?run me through job-folder, or set INPUT and OUTPUT_DIR}"
: "${OUTPUT_DIR:?run me through job-folder, or set INPUT and OUTPUT_DIR}"

name="${INPUT_NAME:-${${INPUT:t}:r}} [Topaz - $preset_name].$ext"

# A rerun of an input that already finished is a no-op, not a second render.
if [[ -e "$OUTPUT_DIR/$name" ]]; then
  print -- "already done: $OUTPUT_DIR/$name"
  exit 0
fi

# Render into a hidden .partial/ and rename only on success, so a stopped or
# crashed render never leaves something in output/ that looks finished. The
# partial is kept on failure: --resume picks it up from the last whole chunk
# when the input is dragged back from failed/.
mkdir -p -- "$OUTPUT_DIR/.partial"
"$HOME"/.local/bin/neuroserver-encode --resume \
  --input "${INPUT:A}" \
  --model "$model" --store "$store" --params "$params" \
  --output-profile "$profile" \
  --preset-name "$preset_name" \
  --metadata "videoai=[Polish] $preset_name" \
  --output "$OUTPUT_DIR/.partial/$name"
mv -- "$OUTPUT_DIR/.partial/$name" "$OUTPUT_DIR/$name"
print -- "done: $OUTPUT_DIR/$name"
