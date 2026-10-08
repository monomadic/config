#!/bin/zsh
# focus-fix-2x-60fps-av1 — job-folder workflow: three workflows in a row, each
# step fed the previous step's output:
#
#   1. topaz-proteus-focus-fix-2x   Proteus Focus Fix Dynamic at 2x (topaz-encode)
#   2. interpolate-resolve          Resolve Optical Flow to 60fps (interpolate-resolve)
#   3. av1-balanced                 AV1, encode-av1's balanced profile (encode-av1)
#
# Tracked in the dotfiles repo (config/jobs/focus-fix-2x-60fps-av1/job.sh) and
# linked into ~/jobs/focus-fix-2x-60fps-av1/ by the Deployfile. Drop or `mv` a
# video into its input/ and the result appears in output/ as
# <name>.focusfix2x.60fps.av1.mkv.
#
# The settings below are copies of the three single-step workflows; keep them
# in step with those job.sh files. Change them here, not in ~/jobs — that file
# is a link.
#
# Intermediates live in output/.partial/<name>/ and each step is skipped when
# its finished file is already there, so an input dragged back from failed/
# resumes at the step that failed (the Topaz step also resumes mid-encode). The
# folder is removed once the AV1 file is in output/. Budget the disk for it: a
# 2x HEVC at 40 Mbps plus Resolve's 60fps render of it.
#
# Topaz, Resolve and SVT-AV1 each want the whole machine: set job-folder's
# Workers to 1 if another workflow may be busy at the same time.

set -euo pipefail

# Step 1 — topaz-proteus-focus-fix-2x. The intermediate is MP4 rather than that
# workflow's MKV: it is Resolve's input next, and MP4 is native to it.
preset_name="Proteus Focus Fix Dynamic 2x"
filter='tvai_up=model=prob-4:scale=2:preblur=0.80:noise=0:details=1:halo=0:blur=1:compression=0.66:prenoise=0:estimate=8:grain=0.83:gsize=1.2:parameters=grain_type=gaussian\\:grain_sigma=0.59:device=0:vram=0.95:instances=1'
video_args='-c:v hevc_videotoolbox -profile:v main -tag:v hvc1 -pix_fmt yuv420p -allow_sw 1 -g 30 -b:v 40M -constant_bit_rate 1'

# Step 2 — interpolate-resolve.
fps=60
quality=speed-warp-metal  # interpolate-resolve --help lists the others

# Step 3 — av1-balanced.
profile=balanced   # fast | balanced | quality | archive — encode-av1 --help lists them
container=mkv      # mkv (copies every track) or mp4 (QuickTime-openable, AAC audio, no subs)

: "${INPUT:?run me through job-folder, or set INPUT and OUTPUT_DIR}"
: "${OUTPUT_DIR:?run me through job-folder, or set INPUT and OUTPUT_DIR}"

stem="${INPUT_NAME:-${${INPUT:t}:r}}"
name="$stem.focusfix2x.${fps//./p}fps.av1.$container"

# A rerun of an input that already finished is a no-op, not a second chain.
if [[ -e "$OUTPUT_DIR/$name" ]]; then
  print -- "already done: $OUTPUT_DIR/$name"
  exit 0
fi

work="$OUTPUT_DIR/.partial/$stem"
upscaled="$work/1-focus-fix-2x.mp4"
interpolated="$work/2-interpolated-${fps//./p}fps.mkv"
mkdir -p -- "$work"

# 1. topaz-encode keeps its own books: a finished output is skipped, and
# --resume continues an interrupted NAME.frag.mp4 from its last keyframe.
print -- "[1/3] Topaz $preset_name"
"$HOME"/.local/bin/topaz-encode --resume \
  --preset_name "$preset_name" \
  --filter_complex "$filter" \
  --output_ext mp4 \
  --video_args "$video_args" \
  --metadata "videoai=[Focus Fix] Proteus Focus Fix Dynamic, 2x upscale" \
  --output "$upscaled" -- "${INPUT:A}"
[[ -s "$upscaled" ]] || { print -u2 -- "topaz-encode left no output: $upscaled"; exit 1 }

# 2. Resolve writes straight to its target, so render to a .partial name and
# rename on success — a stopped render must not pass for a finished step.
if [[ -e "$interpolated" ]]; then
  print -- "[2/3] already interpolated: ${interpolated:t}"
else
  print -- "[2/3] Resolve Optical Flow to ${fps}fps ($quality)"
  rm -f -- "${interpolated:r}.partial.mkv"
  "$HOME"/.local/bin/interpolate-resolve --fps "$fps" --quality "$quality" --mkv \
    -- "$upscaled" "${interpolated:r}.partial.mkv"
  mv -- "${interpolated:r}.partial.mkv" "$interpolated"
fi

# 3. encode-av1 skips an existing output, and a killed encode leaves one
# behind, so clear it first. Its name is the input's stem plus .av1.<container>.
print -- "[3/3] AV1 $profile"
mkdir -p -- "$work/av1"
encoded="$work/av1/${interpolated:t:r}.av1.$container"
rm -f -- "$encoded"
"$HOME"/.local/bin/encode-av1 --"$profile" --"$container" \
  --output-dir "$work/av1" -- "$interpolated"
mv -- "$encoded" "$OUTPUT_DIR/$name"

rm -rf -- "$work"
print -- "done: $OUTPUT_DIR/$name"
