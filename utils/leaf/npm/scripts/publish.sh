#!/usr/bin/env bash
set -euo pipefail

REPO="RivoLink/leaf"
VERSION="${1:?Usage: publish.sh <version>}"
VERSION="${VERSION#v}"

declare -A BINARIES=(
  ["linux-x64"]="leaf-linux-x86_64"
  ["linux-arm64"]="leaf-linux-arm64"
  ["darwin-x64"]="leaf-macos-x86_64"
  ["darwin-arm64"]="leaf-macos-arm64"
  ["win32-x64"]="leaf-windows-x86_64.exe"
  ["android-arm64"]="leaf-android-arm64"
)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NPM_DIR="$SCRIPT_DIR/.."

npm version "$VERSION" --no-git-tag-version --prefix "$NPM_DIR"

for platform in "${!BINARIES[@]}"; do
  binary="${BINARIES[$platform]}"
  pkg_dir="$NPM_DIR/platforms/$platform"

  echo "Publishing @rivolink/leaf-$platform@$VERSION..."

  npm version "$VERSION" --no-git-tag-version --prefix "$pkg_dir"

  url="https://github.com/$REPO/releases/download/$VERSION/$binary"
  echo "Downloading $url"
  curl -fsSL "$url" -o "$pkg_dir/$binary"

  if [[ "$platform" == win32* ]]; then
    mv "$pkg_dir/$binary" "$pkg_dir/leaf.exe"
  else
    mv "$pkg_dir/$binary" "$pkg_dir/leaf"
    chmod +x "$pkg_dir/leaf"
  fi

  cp "$NPM_DIR/../README.md" "$pkg_dir/README.md"
  npm publish "$pkg_dir" --access public
  echo "Published @rivolink/leaf-$platform@$VERSION"

  rm -f "$pkg_dir/leaf" "$pkg_dir/leaf.exe" "$pkg_dir/README.md"
done

node -e "
  const fs = require('fs');
  const pkg = JSON.parse(fs.readFileSync('$NPM_DIR/package.json', 'utf8'));
  for (const dep of Object.keys(pkg.optionalDependencies)) {
    pkg.optionalDependencies[dep] = '$VERSION';
  }
  fs.writeFileSync('$NPM_DIR/package.json', JSON.stringify(pkg, null, 2) + '\n');
"

cp "$NPM_DIR/../README.md" "$NPM_DIR/README.md"
npm publish "$NPM_DIR" --access public
rm -f "$NPM_DIR/README.md"
echo "Published @rivolink/leaf@$VERSION"
