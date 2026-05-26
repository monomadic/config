#!/usr/bin/env zsh

set -euo pipefail

app_name="YaziQLPanel2"
bundle_id="com.rob.yaziquicklook.panel"
binary_name="YaziQuickLookPanel"
icon_name="preview-2.icns"

src_dir="${0:A:h}"
repo_dir="${src_dir:h:h}"
build_dir="${TMPDIR:-/private/tmp}/yazi-quicklook-build"
go_cache="${TMPDIR:-/private/tmp}/yazi-quicklook-gocache"
app="$src_dir/${app_name}.app"
icon_source="$repo_dir/assets/icons/$icon_name"

if [[ ! -f "$icon_source" ]]; then
	echo "Missing icon: $icon_source" >&2
	exit 1
fi

mkdir -p "$build_dir" "$go_cache"

arch="$(uname -m)"

case "$arch" in
	arm64)
		go_arch="arm64"
		;;
	x86_64)
		go_arch="amd64"
		;;
	*)
		echo "Unsupported architecture: $arch" >&2
		exit 1
		;;
esac

echo "Building Go binary for darwin/$go_arch..."

CGO_ENABLED=1 GOOS=darwin GOARCH="$go_arch" GOCACHE="$go_cache" go build \
	-trimpath \
	-ldflags="-s -w" \
	-o "$build_dir/$binary_name" \
	"$src_dir"

echo "Creating app bundle..."

mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"

rm -f "$app/Contents/MacOS/$binary_name"
cp "$build_dir/$binary_name" "$app/Contents/MacOS/$binary_name"
chmod +x "$app/Contents/MacOS/$binary_name"
rm -f "$app/Contents/Resources/$icon_name"
cp "$icon_source" "$app/Contents/Resources/$icon_name"
xattr -cr "$app"

cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "https://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleName</key>
	<string>${app_name}</string>

	<key>CFBundleDisplayName</key>
	<string>${app_name}</string>

	<key>CFBundleIdentifier</key>
	<string>${bundle_id}</string>

	<key>CFBundleExecutable</key>
	<string>${binary_name}</string>

	<key>CFBundleIconFile</key>
	<string>${icon_name}</string>

	<key>CFBundlePackageType</key>
	<string>APPL</string>

	<key>CFBundleSignature</key>
	<string>????</string>

	<key>CFBundleVersion</key>
	<string>1</string>

	<key>CFBundleShortVersionString</key>
	<string>1.0</string>

	<key>LSMinimumSystemVersion</key>
	<string>12.0</string>

	<key>LSUIElement</key>
	<true/>

	<key>NSAppleEventsUsageDescription</key>
	<string>Reveal the hovered Yazi file in Finder and open its native Quick Look preview.</string>
</dict>
</plist>
PLIST

echo "Signing app..."

codesign --force --deep --sign - "$app"

touch "$app"

echo
echo "Created:"
echo "  $app"
echo
echo "First run:"
echo "  open -n ${(q)app} --args /path/to/file"
echo
echo "If permissions are wedged:"
echo "  tccutil reset AppleEvents $bundle_id"
echo "  tccutil reset Accessibility $bundle_id"
