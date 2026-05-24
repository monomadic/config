#!/usr/bin/env zsh
# ==============================================================================
#  build.zsh
#
#  Builds a no-Dock macOS .app:
#    Open In Forklift.app
#
#  Behavior:
#    - grabs Finder's first selected item
#    - if nothing is selected, grabs the front Finder window folder
#    - opens it in ForkLift
# ==============================================================================

set -euo pipefail

app_name="Open In Forklift"
bundle_id="com.rob.finder-first-selection-to-forklift"
binary_name="finder-first-to-forklift"
icon_name="forklift1.icns"

src_dir="${0:A:h}"
repo_dir="${src_dir:h:h}"
build_dir="$src_dir/build"
app="$src_dir/${app_name}.app"
icon_source="$repo_dir/assets/icons/$icon_name"

if [[ ! -f "$icon_source" ]]; then
	echo "Missing icon: $icon_source" >&2
	exit 1
fi

mkdir -p "$build_dir"

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

echo "󰜄 Building Go binary for darwin/$go_arch..."

GOOS=darwin GOARCH="$go_arch" go build \
	-trimpath \
	-ldflags="-s -w" \
	-o "$build_dir/$binary_name" \
	"$src_dir/main.go"

echo "󰣆 Creating app bundle..."

rm -rf "$app"

mkdir -p "$app/Contents/MacOS"
mkdir -p "$app/Contents/Resources"

cp "$build_dir/$binary_name" "$app/Contents/MacOS/$binary_name"
chmod +x "$app/Contents/MacOS/$binary_name"
cp "$icon_source" "$app/Contents/Resources/$icon_name"

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
	<string>Open the first selected Finder item in ForkLift.</string>
</dict>
</plist>
PLIST

echo "󰒃 Signing app..."

codesign --force --deep --sign - "$app"

touch "$app"

echo
echo "Created:"
echo "  $app"
echo
echo "Dotter deploy target:"
echo "  ~/Applications/${app_name}.app"
echo
echo "First run:"
echo "  dotter-deploy"
echo "  open ~/Applications/${(q)app_name}.app"
echo
echo "If permissions are wedged:"
echo "  tccutil reset AppleEvents $bundle_id"
echo "  open ~/Applications/${(q)app_name}.app"
