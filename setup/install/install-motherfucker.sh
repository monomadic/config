#!/bin/sh
# Build and install motherfucker (cache-free Spotlight replacement):
# binary into ~/.bin, LaunchAgent so it starts at login and stays resident.

set -e
cd "$(dirname "$0")/../../utils/motherfucker"
cargo build --release
mkdir -p "$HOME/.bin"
install -m 755 target/release/motherfucker "$HOME/.bin/motherfucker"

LABEL="com.nom.motherfucker"
PLIST="$HOME/Library/LaunchAgents/$LABEL.plist"
mkdir -p "$HOME/Library/LaunchAgents"
cat > "$PLIST" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>$LABEL</string>
	<key>ProgramArguments</key>
	<array>
		<string>$HOME/.bin/motherfucker</string>
	</array>
	<!-- launchd's default PATH is bare /usr/bin:/bin:/usr/sbin:/sbin;
	     [shortcuts] children (switchblade, ffmpeg, ...) need the real one. -->
	<key>EnvironmentVariables</key>
	<dict>
		<key>PATH</key>
		<string>$HOME/.bin:$HOME/.zsh/bin:$HOME/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin</string>
	</dict>
	<key>RunAtLoad</key>
	<true/>
	<key>KeepAlive</key>
	<true/>
	<key>ProcessType</key>
	<string>Interactive</string>
</dict>
</plist>
EOF

# Reload: boot out any previous agent, kill stray manual instances (the
# hotkey registration is exclusive), then bootstrap fresh.
launchctl bootout "gui/$(id -u)" "$PLIST" 2>/dev/null || true
pkill -x motherfucker 2>/dev/null || true
sleep 0.3
launchctl bootstrap "gui/$(id -u)" "$PLIST"

echo "installed: $HOME/.bin/motherfucker"
echo "agent:     $LABEL loaded (starts at login, relaunches on crash)"
