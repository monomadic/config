#!/bin/bash

# Get the current directory path
WIDGET_PATH="$(pwd)/free-disk-space-widget"

# Create the free-disk-space-widget.autostart file with the specified content
cat > "$HOME/Library/LaunchAgents/free-disk-space-widget.autostart.plist" << EOL
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
  <dict>
    <key>Label</key>
    <string>free-disk-space-widget.autostart</string>
    <key>ProgramArguments</key>
    <array>
      <string>$WIDGET_PATH</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
  </dict>
</plist>
EOL

# Set the correct permissions for the launch agent
chmod 644 "$HOME/Library/LaunchAgents/free-disk-space-widget.autostart.plist"

echo "Created autostart file at: $HOME/Library/LaunchAgents/free-disk-space-widget.autostart.plist"
