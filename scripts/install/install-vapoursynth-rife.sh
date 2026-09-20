#!/usr/bin/env bash
# Build and install the RIFE ncnn/Vulkan plugin for VapourSynth on Apple Silicon.
#
# Frames flow through VapourSynth in memory (no PNG round trip, no scratch
# disk) and out to ffmpeg with vspipe. The plugin bundles ncnn and every RIFE
# model up to v4.26 in its own tree; they are built from source here because
# there are no macOS binaries.
#
# Installs:
#   $PLUGIN_DIR/librife.dylib          (default ~/.local/lib/vapoursynth)
#   $PLUGIN_DIR/models/rife-v4.6_ensembleFalse ...   (only the models in $MODELS)
#
# Homebrew's VapourSynth is the pip-style build: it reads no vapoursynth.conf
# and autoloads only from its own site-packages plugins/ dir, so the library is
# also symlinked there. rife-vapoursynth loads it by absolute path anyway.
#
# Requires Homebrew: vapoursynth vapoursynth-bestsource molten-vk vulkan-loader
# vulkan-headers meson ninja cmake libomp

set -euo pipefail

REPO="${REPO:-https://github.com/styler00dollar/VapourSynth-RIFE-ncnn-Vulkan}"
SRC_DIR="${SRC_DIR:-$HOME/.cache/build/VapourSynth-RIFE-ncnn-Vulkan}"
PLUGIN_DIR="${PLUGIN_DIR:-$HOME/.local/lib/vapoursynth}"
MODELS="${MODELS:-rife-v4.6_ensembleFalse,rife-v4.25-lite_ensembleFalse,rife-v4.26_ensembleFalse}"
BREW_PREFIX="$(brew --prefix)"

log() { printf '==> %s\n' "$*"; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }

[[ "$(uname -s)" == "Darwin" && "$(uname -m)" == "arm64" ]] || die "Apple Silicon macOS only"
for f in vapoursynth vapoursynth-bestsource molten-vk vulkan-loader vulkan-headers meson ninja cmake libomp; do
  brew list --formula "$f" >/dev/null 2>&1 || die "missing brew formula: $f (brew install $f)"
done
for cmd in git meson ninja cmake pkg-config vspipe; do
  command -v "$cmd" >/dev/null 2>&1 || die "missing command: $cmd"
done
pkg-config --exists vapoursynth || die "pkg-config cannot find vapoursynth"

# ---- source ----------------------------------------------------------------

if [[ -d "$SRC_DIR/.git" ]]; then
  log "Updating $SRC_DIR"
  git -C "$SRC_DIR" pull --ff-only
else
  log "Cloning $REPO"
  mkdir -p "$(dirname "$SRC_DIR")"
  git clone --depth 1 "$REPO" "$SRC_DIR"
fi
log "Fetching ncnn + glslang submodules"
git -C "$SRC_DIR" submodule update --init --recursive --depth 1

for m in ${MODELS//,/ }; do
  [[ -d "$SRC_DIR/models/$m" ]] || die "model not in plugin tree: $m"
done

# ---- build -----------------------------------------------------------------

# libomp is keg-only; ncnn's cmake and the plugin's meson both need to see it.
# MoltenVK provides the Vulkan ICD; the loader and headers come from brew.
export CPPFLAGS="-I$BREW_PREFIX/opt/libomp/include ${CPPFLAGS:-}"
export LDFLAGS="-L$BREW_PREFIX/opt/libomp/lib ${LDFLAGS:-}"
export CMAKE_PREFIX_PATH="$BREW_PREFIX/opt/libomp:$BREW_PREFIX${CMAKE_PREFIX_PATH:+:$CMAKE_PREFIX_PATH}"
export VULKAN_SDK="$BREW_PREFIX"

# Homebrew's vapoursynth.pc defines no libdir, which meson.build reads for the
# install dir; give it a shim .pc that adds one (we copy the plugin ourselves).
pc_dir="$(mktemp -d "${TMPDIR:-/tmp}/rife-pc.XXXXXX")"
trap 'rm -rf "$pc_dir"' EXIT
{ printf 'libdir=%s\n' "$PLUGIN_DIR"; pkg-config --print-provides vapoursynth >/dev/null; cat "$(pkg-config --variable=pcfiledir vapoursynth)/vapoursynth.pc"; } > "$pc_dir/vapoursynth.pc"
export PKG_CONFIG_PATH="$pc_dir${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"

cd "$SRC_DIR"
if [[ ! -f build/build.ninja ]]; then
  log "Configuring (meson)"
  rm -rf build
  meson setup build --buildtype=release
fi
log "Building (this compiles ncnn and its shaders; several minutes)"
ninja -C build

built="$(find build -name 'librife*.dylib' -o -name 'librife*.so' | head -1)"
[[ -n "$built" ]] || die "build produced no librife library"

# ---- install ---------------------------------------------------------------

log "Installing to $PLUGIN_DIR"
mkdir -p "$PLUGIN_DIR/models"
cp "$built" "$PLUGIN_DIR/librife.dylib"
for m in ${MODELS//,/ }; do
  rm -rf "$PLUGIN_DIR/models/$m"
  cp -R "$SRC_DIR/models/$m" "$PLUGIN_DIR/models/$m"
done

# ---- verify ----------------------------------------------------------------

log "Verifying"
autoload_dir="$(python3 -c 'import vapoursynth._utils as u; print(u.get_plugin_dir())')"
if [[ -d "$autoload_dir" ]]; then
  ln -sfn "$PLUGIN_DIR/librife.dylib" "$autoload_dir/librife.dylib"
  log "Autoload symlink: $autoload_dir/librife.dylib"
fi
first_model="${MODELS%%,*}"
python3 - "$PLUGIN_DIR" "$first_model" <<'EOF'
import sys, vapoursynth as vs
core = vs.core
names = [p.namespace for p in core.plugins()]
if "rife" not in names:
    core.std.LoadPlugin(sys.argv[1] + "/librife.dylib")
    print("warning: autoload did not pick up librife.dylib; loaded explicitly")
clip = core.std.BlankClip(width=64, height=64, format=vs.RGBS, length=4, fpsnum=30)
out = core.rife.RIFE(clip, model_path=sys.argv[1] + "/models/" + sys.argv[2], factor_num=2, gpu_thread=1)
out.get_frame(1)
print("rife ok:", out.num_frames, "frames from 4 at", out.fps)
EOF

log "Done: $PLUGIN_DIR/librife.dylib (models: $MODELS)"
