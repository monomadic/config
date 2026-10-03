#!/bin/zsh
# topaz-proteus-focus-fix-2x — job-folder workflow: encode each input with
# Proteus Focus Fix Dynamic at 2x, via the deployed topaz-encode.
#
# Tracked in the dotfiles repo (config/jobs/topaz-proteus-focus-fix-2x/job.sh)
# and linked into ~/jobs/topaz-proteus-focus-fix-2x/ by the Deployfile. Drop or
# `mv` a video into its input/ and the result appears in output/ as
# "<name> [Topaz - Proteus Focus Fix Dynamic 2x].mkv".
#
# The filter is bin/lib/topaz-presets/enhancement/proteus-focus-fix-dynamic.toml
# with @SCALE@ filled as scale=2, and the codec arguments are
# output/hevc-cbr-40mbps.toml; keep them in step. Change them here, not in
# ~/jobs — that file is a link.

set -euo pipefail

preset_name="Proteus Focus Fix Dynamic 2x"
filter='tvai_up=model=prob-4:scale=2:preblur=0.80:noise=0:details=1:halo=0:blur=1:compression=0.66:prenoise=0:estimate=8:grain=0.83:gsize=1.2:parameters=grain_type=gaussian\\:grain_sigma=0.59:device=0:vram=0.95:instances=1'
ext=mkv
video_args='-c:v hevc_videotoolbox -profile:v main -tag:v hvc1 -pix_fmt yuv420p -allow_sw 1 -g 30 -b:v 40M -constant_bit_rate 1'

: "${INPUT:?run me through job-folder, or set INPUT and OUTPUT_DIR}"
: "${OUTPUT_DIR:?run me through job-folder, or set INPUT and OUTPUT_DIR}"

# topaz-encode does the rest itself: the in-flight file is NAME.frag.mkv, a
# finished output is skipped, and --resume continues an interrupted one.
exec "$HOME"/.local/bin/topaz-encode --resume \
  --preset_name "$preset_name" \
  --filter_complex "$filter" \
  --output_ext "$ext" \
  --video_args "$video_args" \
  --metadata "videoai=[Focus Fix] Proteus Focus Fix Dynamic, 2x upscale" \
  --output-dir "$OUTPUT_DIR" -- "$INPUT"
