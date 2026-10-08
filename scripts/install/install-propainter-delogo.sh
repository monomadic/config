#!/bin/bash
# Install propainter-delogo: static logo/text removal with ProPainter, windowed
# around the box, shot-chunked, audio-preserving. It shells out to a ProPainter
# checkout, so this reuses the one install-propainter.sh put in ~/src/scrubber
# (pinned source, verified weights, a torch lockfile known to run on MPS here)
# instead of fetching a second copy.
set -euo pipefail
DEST="${SRC_PATH:-$HOME/src}/propainter-delogo"
SCRUBBER="${SCRUBBER:-$HOME/src/scrubber}"
BIN_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
REV=0b3f71b2d7d01c4cd6be6fc919af1ec0eb94db84 # 2026-08-25, "docs: performance section"
SMOKE=1
fail() { printf 'Error: %s\n' "$*" >&2; exit 1; }
while [ "$#" -gt 0 ]; do
  case "$1" in
    --dir) [ "$#" -ge 2 ] || fail '--dir needs a path'; DEST="$2"; shift 2 ;;
    --scrubber) [ "$#" -ge 2 ] || fail '--scrubber needs a path'; SCRUBBER="$2"; shift 2 ;;
    --skip-smoke-test) SMOKE=0; shift ;;
    -h|--help) printf 'Usage: %s [--dir PATH] [--scrubber PATH] [--skip-smoke-test]\nDefault: ~/src/propainter-delogo, using the ProPainter checkout in ~/src/scrubber (install-propainter.sh).\nRequires uv and ffmpeg/ffprobe on PATH.\n' "$0"; exit 0 ;;
    *) fail "Unknown argument: $1" ;;
  esac
done
[ "$(uname -s)" = Darwin ] && [ "$(uname -m)" = arm64 ] || fail 'Requires native Apple Silicon macOS.'
command -v uv >/dev/null || fail 'Install uv first.'
command -v ffmpeg >/dev/null && command -v ffprobe >/dev/null || fail 'ffmpeg and ffprobe must be on PATH (brew install ffmpeg).'
PROPAINTER="$SCRUBBER/ProPainter"
LOCK="$SCRUBBER/requirements-macos.lock"
[ -f "$PROPAINTER/inference_propainter.py" ] || fail "No ProPainter checkout at $PROPAINTER; run install-propainter.sh first."
[ -f "$LOCK" ] || fail "Missing $LOCK; run install-propainter.sh first."
# The same weights install-propainter.sh verified; inference loads them from
# the checkout's own weights/ dir and would otherwise download unverified copies.
while read -r name expected; do
  target="$PROPAINTER/weights/$name"
  [ -f "$target" ] || fail "Missing weight $target; rerun install-propainter.sh."
  [ "$(shasum -a 256 "$target" | awk '{print $1}')" = "$expected" ] || fail "Checksum mismatch: $target"
done <<'WEIGHTS'
ProPainter.pth 12c070c4b48f374c91d8a2a17851140b85c159621080989f9e191bbc18bd6591
raft-things.pth fcfa4125d6418f4de95d84aec20a3c5f4e205101715a79f193243c186ac9a7e1
recurrent_flow_completion.pth 22939a1a7900da878dbe1ccd011d646b1bfb30b8290039d8ff0e0c2fefbfd283
WEIGHTS
if [ -e "$DEST" ]; then
  [ -d "$DEST/.git" ] || fail "Destination exists and is not a Git checkout: $DEST"
  [ "$(git -C "$DEST" remote get-url origin)" = https://github.com/QuantumWars/propainter-delogo.git ] || fail 'Unexpected origin; refusing to change this checkout.'
  [ "$(git -C "$DEST" rev-parse HEAD)" = "$REV" ] || fail 'Checkout revision differs from installer pin; preserve it and use --dir.'
  [ -z "$(git -C "$DEST" status --porcelain --untracked-files=no)" ] || fail 'Checkout has tracked edits; refusing to package them.'
else
  mkdir -p "$(dirname "$DEST")"
  git clone https://github.com/QuantumWars/propainter-delogo.git "$DEST"
  git -C "$DEST" checkout --detach "$REV"
fi
DEST=$(cd "$DEST" && pwd -P)
SCRUBBER=$(cd "$SCRUBBER" && pwd -P)
PROPAINTER="$SCRUBBER/ProPainter"
WORK=$(mktemp -d "${TMPDIR:-/tmp}/propainter-delogo-install.XXXXXX")
trap 'rm -rf "$WORK"' EXIT
# Reuse scrubber's managed Python 3.12 rather than downloading another.
export UV_PYTHON_INSTALL_DIR="$SCRUBBER/.tools/python"
if [ ! -d "$DEST/.venv" ]; then uv venv --python 3.12 --managed-python "$DEST/.venv"; fi
ENV_PY="$DEST/.venv/bin/python"
"$ENV_PY" -c 'import sys, platform; assert sys.version_info[:2] == (3, 12) and platform.machine() == "arm64", "Python 3.12 arm64 required; recreate .venv"'
# ProPainter runs under this interpreter (delogo invokes inference_propainter.py
# with sys.executable), so it needs ProPainter's deps: the lockfile that already
# works on this machine, which also covers delogo's numpy and opencv.
uv pip sync --python "$ENV_PY" "$LOCK"
uv pip install --python "$ENV_PY" --no-deps "$DEST"
uv pip check --python "$ENV_PY"
mkdir -p "$DEST/bin" "$DEST/.cache"
# Launcher: MPS fallback for the ops Metal lacks, and the ProPainter checkout
# filled in for `run` so the CLI needs only --input/--output/--box.
cat > "$DEST/bin/propainter-delogo" <<LAUNCHER
#!/bin/zsh
set -eu
ROOT="\${0:A:h:h}"
export PYTORCH_ENABLE_MPS_FALLBACK=1
export TORCH_HOME="\$ROOT/.cache/torch"
export MPLCONFIGDIR="\$ROOT/.cache/matplotlib"
args=("\$@")
if [[ " \$* " == *" run "* && " \$* " != *" --propainter "* ]]; then
  args+=(--propainter "$PROPAINTER")
fi
exec "\$ROOT/.venv/bin/propainter-delogo" "\${args[@]}"
LAUNCHER
chmod +x "$DEST/bin/propainter-delogo"
mkdir -p "$BIN_DIR"
ln -sfn "$DEST/bin/propainter-delogo" "$BIN_DIR/propainter-delogo"
"$ENV_PY" - "$PROPAINTER" <<'PY'
import sys, torch
sys.path.insert(0, sys.argv[1])
from model.misc import get_device
assert torch.backends.mps.is_available(), 'Apple GPU unavailable'
assert get_device().type == 'mps', f'ProPainter would run on {get_device()}, not MPS'
print('PyTorch:', torch.__version__, '| ProPainter device:', get_device())
PY
"$DEST/bin/propainter-delogo" --help > "$WORK/help.txt"
if [ "$SMOKE" = 1 ]; then
  printf '\nRunning sample removal (2s synthetic clip, first launch may build caches)...\n'
  # A moving pattern with a static white block over it, plus a tone so the
  # audio copy is exercised.
  ffmpeg -v error -y -f lavfi -i 'testsrc2=size=320x240:rate=24:duration=2' \
    -f lavfi -i 'sine=frequency=440:duration=2' \
    -vf 'drawbox=x=20:y=20:w=60:h=24:color=white:t=fill' \
    -c:v libx264 -pix_fmt yuv420p -c:a aac -shortest "$WORK/in.mp4"
  ( cd "$WORK" && "$DEST/bin/propainter-delogo" run --input in.mp4 --output out.mp4 \
      --box 20 20 60 24 --pad 40 --chunk-size 48 --raft-iter 4 --preset ultrafast )
  "$ENV_PY" - "$WORK/in.mp4" "$WORK/out.mp4" <<'PY'
import json, subprocess, sys
def probe(p):
    j = json.loads(subprocess.check_output(['ffprobe', '-v', 'error', '-show_streams', '-count_frames', '-of', 'json', p]))
    v = next(s for s in j['streams'] if s['codec_type'] == 'video')
    a = [s for s in j['streams'] if s['codec_type'] == 'audio']
    return (int(v['width']), int(v['height'])), int(v['nb_read_frames']), bool(a)
src, out = probe(sys.argv[1]), probe(sys.argv[2])
assert out[0] == src[0], f'size changed: {src[0]} -> {out[0]}'
assert out[1] == src[1], f'frame count changed: {src[1]} -> {out[1]}'
assert out[2], 'audio stream was not carried over'
print(f'Smoke test passed: {out[1]} frames at {out[0][0]}x{out[0][1]}, audio kept. This verifies execution, not repair quality.')
PY
fi
uv pip freeze --python "$ENV_PY" > "$DEST/.venv/installed-requirements.txt"
printf '\nInstalled propainter-delogo %s into %s\nLauncher: %s/propainter-delogo (ProPainter: %s)\n' "$(git -C "$DEST" rev-parse --short HEAD)" "$DEST" "$BIN_DIR" "$PROPAINTER"
printf 'Usage: propainter-delogo preview --input clip.mp4 --box X Y W H --at 5 --out box.png\n       propainter-delogo run --input clip.mp4 --output clean.mp4 --box X Y W H [--proc-scale 0.5 --raft-iter 12]\n'
printf 'MIT for the tool; ProPainter and its weights keep the S-Lab non-commercial license.\n'
