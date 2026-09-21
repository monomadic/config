#!/usr/bin/env bash
# Install CleanPlate local AI rotoscoping and cleanup, including MatAnyone 2.
# Reuses existing media/models and preserves local source changes.
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
DEST="${SRC_PATH:-$HOME/src}/cleanplate"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
PYTHON="${PYTHON:-python3.12}"
FOOTAGE=1
CHECK_ONLY=0
MA2_COMMIT=0079197acd6d16a741f71558809c06c586c579e0

usage() {
  cat <<'HELP'
Install CleanPlate for macOS. Requires Git, Python 3.12, and FFmpeg.
Usage: install-cleanplate.sh [--dir PATH] [--skip-footage] [--check]

  --dir PATH       Checkout location (default: $SRC_PATH/cleanplate or ~/src/cleanplate)
  --skip-footage   Omit the Tears of Steel demo download
  --check          Verify an existing installation without installing or updating
  -h, --help       Show this help

Environment: PYTHON (default python3.12), INSTALL_DIR (default ~/.local/bin).
Installs runtime dependencies, SAM 2, MatAnyone 1/2, and ProPainter.
Models and footage need several GB. MatAnyone and ProPainter are non-commercial.
Reruns fetch upstream and fast-forward only when local history permits.
Local commits and edits are preserved. No app is started automatically.
HELP
}
fail() { printf 'Error: %s\n' "$*" >&2; exit 1; }
while [ "$#" -gt 0 ]; do
  case "$1" in
    --dir) [ "$#" -ge 2 ] && [ -n "$2" ] || fail '--dir requires a path'; DEST="$2"; shift 2 ;;
    --skip-footage) FOOTAGE=0; shift ;;
    --check) CHECK_ONLY=1; shift ;;
    -h|--help) usage; exit 0 ;;
    *) fail "Unknown argument: $1 (use --help)" ;;
  esac
done
[ "$(uname -s)" = Darwin ] || fail 'This installer supports macOS only.'
for tool in git ffmpeg ffprobe; do
  command -v "$tool" >/dev/null || fail "$tool is missing (install Git/FFmpeg first)."
done

verify() {
  (cd "$DEST" && .venv/bin/python - <<'PY'
import sys
assert sys.version_info[:2] == (3, 12), 'Python 3.12 required'
import torch
import sam2
from sam2.build_sam import build_sam2_video_predictor
from cleanplate.paths import SAM2_CKPT
from cleanplate import refine, remove
import app
assert SAM2_CKPT.is_file(), f'Missing checkpoint: {SAM2_CKPT}'
for model in ('matanyone', 'matanyone2'):
    ok, reason = refine.available(model)
    assert ok, reason
    __import__(f'{model}.utils.get_default_model')
ok, reason = remove.available()
assert ok, reason
print('CleanPlate runtime, tracking checkpoint, refinement imports, and removal files: OK')
print('PyTorch:', torch.__version__, '| Apple GPU available:', torch.backends.mps.is_available())
print('This checks installation readiness, not model inference or removal output.')
PY
  )
}
if [ "$CHECK_ONLY" = 1 ]; then
  [ -x "$DEST/.venv/bin/python" ] || fail "No environment at $DEST/.venv"
  verify
  exit 0
fi

command -v "$PYTHON" >/dev/null || fail 'Python 3.12 is missing. Install with: brew install python@3.12'
"$PYTHON" -c 'import sys; assert sys.version_info[:2] == (3,12), "Python 3.12 required"'
# Use the same non-destructive update policy as the other external-source installers.
. "$SCRIPT_DIR/lib/git-source-install.sh"
if [ ! -e "$DEST" ]; then
  mkdir -p "$(dirname "$DEST")"
  git clone https://github.com/shauryadata/cleanplate.git "$DEST"
else
  [ -e "$DEST/.git" ] || fail "$DEST exists but is not a Git checkout"
  origin=$(git -C "$DEST" remote get-url origin)
  case "$origin" in
    https://github.com/shauryadata/cleanplate|https://github.com/shauryadata/cleanplate.git|git@github.com:shauryadata/cleanplate.git) ;;
    *) fail "Unexpected checkout origin: $origin" ;;
  esac
  _gsi_sync "$DEST" >/dev/null
fi
DEST="$(cd "$DEST" && pwd -P)"
cd "$DEST"
mkdir .cleanplate-install.lock 2>/dev/null || fail 'Installation lock exists; check for another installer before removing it.'
trap 'rmdir "$DEST/.cleanplate-install.lock"' EXIT
if [ ! -e .venv ]; then
  "$PYTHON" -m venv .venv
fi
[ -x .venv/bin/python ] || fail 'Existing .venv is incomplete; repair it before rerunning.'
.venv/bin/python -c 'import sys; assert sys.version_info[:2] == (3,12), "Existing environment must use Python 3.12"'
export PY="$DEST/.venv/bin/python"
"$PY" -m pip install -r requirements.txt
for target in sam2 checkpoints matanyone propainter; do
  bash scripts/download.sh "$target"
done
if [ "$FOOTAGE" = 1 ]; then
  bash scripts/download.sh footage
fi

# Upstream's default refinement model is absent from its download script.
if [ ! -e vendor/matanyone2 ]; then
  git clone https://github.com/pq-yang/MatAnyone2.git vendor/matanyone2
  git -C vendor/matanyone2 checkout --detach "$MA2_COMMIT"
else
  [ -e vendor/matanyone2/.git ] || fail 'Existing vendor/matanyone2 is not a Git checkout'
  printf 'Reusing existing MatAnyone 2 checkout without modifying it.\n'
fi
"$PY" -m pip install --no-deps -e vendor/matanyone2
if [ ! -s checkpoints/matanyone2.pth ]; then
  curl --fail --location --retry 3 \
    https://github.com/pq-yang/MatAnyone2/releases/download/v1.0.0/matanyone2.pth \
    -o checkpoints/matanyone2.pth.part
  mv checkpoints/matanyone2.pth.part checkpoints/matanyone2.pth
fi
verify

# Generate a launcher with shell-quoted absolute paths, including paths with spaces.
mkdir -p "$INSTALL_DIR"
"$PY" - "$DEST" "$INSTALL_DIR/cleanplate" <<'PY'
from pathlib import Path
import shlex
import sys
root, target = map(Path, sys.argv[1:])
if target.is_symlink():
    raise SystemExit(f'Refusing to overwrite symlink: {target}')
marker = '# Generated by install-cleanplate.sh'
if target.exists() and marker not in target.read_text():
    raise SystemExit(f'Refusing to overwrite unrelated launcher: {target}')
target.write_text('#!/bin/sh\n' + marker + '\nset -eu\ncd ' + shlex.quote(str(root)) +
                  '\nexec ' + shlex.quote(str(root / '.venv/bin/python')) +
                  ' app.py --open "$@"\n')
target.chmod(0o755)
PY
printf '\nInstalled. Start with: %s/cleanplate\n' "$INSTALL_DIR"
printf 'Opens http://127.0.0.1:7860; Ctrl-C stops the server.\n'
printf 'First refinement may download ResNet weights. See THIRD_PARTY.md for model licenses.\n'
