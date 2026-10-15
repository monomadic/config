#!/bin/sh
# Pin upstream algorithm, model and precompiled Metal shaders together. No Xcode Metal compiler needed.
set -eu
cd "$(dirname "$0")"
[ "$(uname -m)" = arm64 ] || { echo 'rife-video requires Apple silicon' >&2; exit 1; }
commit=1fef4869a44fca32848102b9f6a774a20eb50db6
archive_sha=2340d5353afec68fd3c5ab4138a36d139aadd99b862842abee3b3db2ca864690
mkdir -p .build-deps
if [ ! -d .build-deps/rife-metal/.git ]; then
  git clone --no-checkout https://github.com/cinemore/rife-metal.git .build-deps/rife-metal
fi
actual=$(git -C .build-deps/rife-metal rev-parse HEAD 2>/dev/null || true)
if [ "$actual" != "$commit" ] || [ ! -f .build-deps/rife-metal/Sources/RifeMetal/RifeInterpolator.swift ]; then
  git -C .build-deps/rife-metal fetch origin "$commit"
  git -C .build-deps/rife-metal checkout --detach "$commit"
fi
# Cached public release; validate before using its shader library on every build.
archive=.build-deps/rife-metal-0.1.7-arm64.tar.gz
if [ ! -f "$archive" ]; then
  curl --fail --location --retry 3 https://github.com/cinemore/rife-metal/releases/download/v0.1.7/rife-metal-macos-arm64.tar.gz -o "$archive.part"
  mv "$archive.part" "$archive"
fi
printf '%s  %s\n' "$archive_sha" "$archive" | shasum -a 256 -c -
mkdir -p .build-deps/release
tar -xzf "$archive" -C .build-deps/release
shader=$(find .build-deps/release -name default.metallib -type f | head -1)
[ -n "$shader" ] || { echo 'release contains no Metal shader library' >&2; exit 1; }
cp "$shader" .build-deps/rife-metal/Sources/RifeMetalCore/default.metallib
# Integration-only manifest: upstream Swift/shader/model code is not patched.
cat > .build-deps/rife-metal/Package.swift <<'SWIFT'
// swift-tools-version: 5.10
import PackageDescription
let package = Package(
    name: "RifeMetal", platforms: [.macOS(.v13)],
    products: [.library(name: "RifeMetal", targets: ["RifeMetal", "RifeMetalCore"])],
    targets: [
        .target(name: "RifeMetal", dependencies: ["RifeMetalCore"], resources: [.copy("Resources/rife-v4.26.rmw")]),
        .target(name: "RifeMetalCore", exclude: ["Shaders"], resources: [.copy("default.metallib")]),
    ]
)
SWIFT
