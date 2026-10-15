#!/bin/sh
# Resident Vulkan backend. Model files remain external; no image-codec dependency.
set -eu
cd "$(dirname "$0")"
commit=13338e38debe2e400b3eeecf6792312d01a692f9
for tool in cmake brew git; do
  command -v "$tool" >/dev/null || { echo "missing build dependency: $tool" >&2; exit 1; }
done
molten=$(brew --prefix molten-vk)
headers=$(brew --prefix vulkan-headers)
[ -f "$molten/lib/libMoltenVK.a" ] || { echo 'Install molten-vk and vulkan-headers with Homebrew first.' >&2; exit 1; }
mkdir -p .build-deps
if [ ! -d .build-deps/rife-ncnn/.git ]; then
  git clone --filter=blob:none --no-checkout https://github.com/TNTwise/rife-ncnn-vulkan.git .build-deps/rife-ncnn
  git -C .build-deps/rife-ncnn sparse-checkout set --no-cone /src/ /LICENSE
fi
if ! git -C .build-deps/rife-ncnn cat-file -e "$commit^{commit}" 2>/dev/null; then
  git -C .build-deps/rife-ncnn fetch origin "$commit"
fi
git -C .build-deps/rife-ncnn checkout --detach "$commit"
git -C .build-deps/rife-ncnn submodule update --init --depth 1 src/ncnn
git -C .build-deps/rife-ncnn/src/ncnn submodule update --init --depth 1 glslang
cmake -S Native -B .build-deps/native -DCMAKE_BUILD_TYPE=Release \
  -DRIFE_SOURCE="$PWD/.build-deps/rife-ncnn/src" \
  -DVulkan_INCLUDE_DIR="$headers/include" -DVulkan_LIBRARY="$molten/lib/libMoltenVK.a" \
  -DCMAKE_OSX_DEPLOYMENT_TARGET=14.0
cmake --build .build-deps/native -j "${BUILD_JOBS:-8}"
