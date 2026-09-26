#!/bin/bash
# Install the Apple Silicon VapourSynth filter in a separate upstream checkout.
set -euo pipefail
DEST="${SRC_PATH:-$HOME/src}/vs-propainter"
REV=35f9bb4df3e07abeadb1b8ce9d6a64a2174480b1 # v1.2.7
SMOKE=1
fail() { printf 'Error: %s\n' "$*" >&2; exit 1; }
while [ "$#" -gt 0 ]; do
  case "$1" in
    --dir) [ "$#" -ge 2 ] || fail '--dir needs a path'; DEST="$2"; shift 2 ;;
    --skip-smoke-test) SMOKE=0; shift ;;
    -h|--help) printf 'Usage: %s [--dir PATH] [--skip-smoke-test]\nDefault: ~/src/vs-propainter. Requires Homebrew and uv; installs VapourSynth if missing.\n' "$0"; exit 0 ;;
    *) fail "Unknown argument: $1" ;;
  esac
done
[ "$(uname -s)" = Darwin ] && [ "$(uname -m)" = arm64 ] || fail 'Requires native Apple Silicon macOS.'
command -v brew >/dev/null || fail 'Install Homebrew first.'
command -v uv >/dev/null || fail 'Install uv first.'
if ! brew list --versions vapoursynth | grep -q .; then brew install vapoursynth; fi
# Use the Python ABI supplied by Homebrew's VapourSynth, without exposing all
# system site-packages to this isolated environment.
VS_PREFIX=$(brew --prefix vapoursynth)
VS_SITE=$(find "$VS_PREFIX/libexec/lib" -type d -name site-packages | head -1)
[ -n "$VS_SITE" ] || fail 'Cannot locate Homebrew VapourSynth Python bindings.'
PY_ABI=$(basename "$(dirname "$VS_SITE")")
PYTHON="$(brew --prefix)/bin/$PY_ABI"
[ -x "$PYTHON" ] || fail "Missing matching Python: $PYTHON"
if [ -e "$DEST" ]; then
  [ -d "$DEST/.git" ] || fail "Destination exists and is not a Git checkout: $DEST"
  [ "$(git -C "$DEST" remote get-url origin)" = https://github.com/dan64/vs-propainter.git ] || fail 'Unexpected origin; refusing to change this checkout.'
  [ "$(git -C "$DEST" rev-parse HEAD)" = "$REV" ] || fail 'Checkout revision differs from installer pin; preserve it and use --dir.'
  [ -z "$(git -C "$DEST" status --porcelain --untracked-files=no)" ] || fail 'Checkout has tracked edits; refusing to package them.'
else
  mkdir -p "$(dirname "$DEST")"
  git clone https://github.com/dan64/vs-propainter.git "$DEST"
  git -C "$DEST" checkout --detach "$REV"
fi
DEST=$(cd "$DEST" && pwd -P)
WORK=$(mktemp -d "${TMPDIR:-/tmp}/vs-propainter-install.XXXXXX")
trap 'rm -rf "$WORK"' EXIT
if [ ! -d "$DEST/.venv" ]; then uv venv --python "$PYTHON" "$DEST/.venv"; fi
ENV_PY="$DEST/.venv/bin/python"
"$ENV_PY" -c 'import sys; assert "python%d.%d" % sys.version_info[:2] == sys.argv[1], "VapourSynth Python ABI changed; recreate .venv"' "$PY_ABI"
SITE=$("$ENV_PY" -c 'import sysconfig; print(sysconfig.get_path("purelib"))')
printf '%s\n' "$VS_SITE" > "$SITE/homebrew-vapoursynth.pth"
mkdir "$WORK/package"
git -C "$DEST" archive "$REV" | tar -x -C "$WORK/package"
# Upstream v1.2.7 lists CUDA unconditionally and omits runtime imports.
# Correct metadata in the temporary build only; leave the upstream repo intact.
"$ENV_PY" - "$WORK/package/pyproject.toml" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1])
s = p.read_text()
s = s.replace('"nvidia-cuda-runtime-cu12>=12.5.39",', '"opencv-python>=4.10",\n  "scipy>=1.14",\n  "einops>=0.8",')
# uv does not discover distributions behind Homebrew's .pth bridge.
# VapourSynth is an external prerequisite, checked at runtime below.
s = s.replace('"VapourSynth>=65",', '')
p.write_text(s)
PY
# Homebrew supplies VapourSynth; install the remaining runtime deps separately.
uv pip install --python "$ENV_PY" 'torch>=2.2' 'torchvision>=0.17' 'numpy>=1.26.4' 'Pillow>=10.1' 'opencv-python>=4.10' 'scipy>=1.14' 'einops>=0.8'
uv pip install --python "$ENV_PY" --no-deps "$WORK/package"
uv pip check --python "$ENV_PY"
WEIGHTS="$SITE/vspropainter/weights"
mkdir -p "$WEIGHTS"
while read -r name expected; do
  target="$WEIGHTS/$name"
  if [ ! -f "$target" ]; then
    curl --fail --location --retry 3 "https://github.com/sczhou/ProPainter/releases/download/v0.1.0/$name" -o "$WORK/$name"
    actual=$(shasum -a 256 "$WORK/$name" | awk '{print $1}')
    [ "$actual" = "$expected" ] || fail "Checksum mismatch: $name"
    mv "$WORK/$name" "$target"
  fi
  actual=$(shasum -a 256 "$target" | awk '{print $1}')
  [ "$actual" = "$expected" ] || fail "Checksum mismatch: $target"
  # Imports from the checkout must find the same models as installed imports.
  source_weight="$DEST/vspropainter/weights/$name"
  if [ ! -e "$source_weight" ] && [ ! -L "$source_weight" ]; then
    ln -s "$target" "$source_weight"
  elif [ "$(shasum -a 256 "$source_weight" | awk '{print $1}')" != "$expected" ]; then
    fail "Existing checkout weight differs: $source_weight"
  fi
  exclude="/vspropainter/weights/$name"
  grep -qxF "$exclude" "$DEST/.git/info/exclude" || printf '%s\n' "$exclude" >> "$DEST/.git/info/exclude"
done <<'WEIGHTS'
ProPainter.pth 12c070c4b48f374c91d8a2a17851140b85c159621080989f9e191bbc18bd6591
raft-things.pth fcfa4125d6418f4de95d84aec20a3c5f4e205101715a79f193243c186ac9a7e1
recurrent_flow_completion.pth 22939a1a7900da878dbe1ccd011d646b1bfb30b8290039d8ff0e0c2fefbfd283
WEIGHTS
# Missing Metal operators may use CPU; the selected model device remains MPS.
export PYTORCH_ENABLE_MPS_FALLBACK=1
cd "$WORK"
"$ENV_PY" - <<'PY'
import torch, vapoursynth as vs, vspropainter
assert vs.__version__.release_major >= 68, 'VapourSynth R68 or later required'
assert torch.backends.mps.is_available(), 'Apple GPU unavailable'
print('PyTorch:', torch.__version__, '| VapourSynth:', vs.__version__, '| MPS available')
PY
if [ "$SMOKE" = 1 ]; then
  "$ENV_PY" - "$WORK" <<'PY'
import sys, time
import numpy as np
from PIL import Image
import vapoursynth as vs
from vspropainter import propainter
mask = np.zeros((96, 128), dtype=np.uint8)
mask[40:56, 56:72] = 255
path = sys.argv[1] + '/mask.png'
Image.fromarray(mask).save(path)
clip = vs.core.std.BlankClip(width=128, height=96, length=12, format=vs.RGB24, color=[80,120,160])
t0 = time.monotonic()
result = propainter(clip, img_mask_path=path, length=12, neighbor_length=4, enable_fp16=False, sc_threshold=0)
for n in range(result.num_frames):
    frame = result.get_frame(n)
    assert (frame.width, frame.height) == (128, 96)
    assert np.isfinite(np.asarray(frame[0])).all()
print(f'Smoke test passed: {result.num_frames} frames, {time.monotonic()-t0:.1f}s. This verifies execution, not repair quality.')
PY
fi
uv pip freeze --python "$ENV_PY" > "$DEST/.venv/installed-requirements.txt"
printf '\nInstalled vs-propainter v1.2.7 into %s\nPython: %s\nUse PYTORCH_ENABLE_MPS_FALLBACK=1 when running scripts.\n' "$DEST" "$ENV_PY"
printf 'This is a VapourSynth Python filter, not a standalone video CLI. Model weights retain the upstream ProPainter license.\n'
