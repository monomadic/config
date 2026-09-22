#!/bin/bash
# Build a relocatable, ad-hoc signed app. Nothing is installed or launched.
set -euo pipefail
cd -- "$(dirname -- "$0")"
export MACOSX_DEPLOYMENT_TARGET=13.0
cargo build --release --locked
app="$PWD/target/release/WiFi Widget.app"
mkdir -p "$app/Contents/MacOS"
cp target/release/wifi-widget "$app/Contents/MacOS/wifi-widget"
cat > "$app/Contents/Info.plist" <<'PLIST'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
<key>CFBundleIdentifier</key><string>com.jayu.wifi-widget</string>
<key>CFBundleName</key><string>WiFi Widget</string>
<key>CFBundleDisplayName</key><string>WiFi Widget</string>
<key>CFBundleExecutable</key><string>wifi-widget</string>
<key>CFBundlePackageType</key><string>APPL</string>
<key>CFBundleShortVersionString</key><string>0.1.0</string>
<key>CFBundleVersion</key><string>1</string>
<key>LSMinimumSystemVersion</key><string>13.0</string>
<key>LSUIElement</key><true/>
<key>NSHighResolutionCapable</key><true/>
<key>NSLocationWhenInUseUsageDescription</key><string>WiFi Widget uses Location permission to display the connected Wi-Fi network name and access point. It does not collect your location.</string>
</dict></plist>
PLIST
/usr/bin/plutil -lint "$app/Contents/Info.plist"
/usr/bin/codesign --force --sign - "$app"
/usr/bin/codesign --verify --strict "$app"
printf 'Built %s\nOpen it with: open "%s"\n' "$app" "$app"
