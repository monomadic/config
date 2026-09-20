-- Script to open Screen Sharing in fullscreen with dynamic resolution

-- Connect to the remote machine
tell application "Screen Sharing"
	open location "M4.local"

	-- Wait for connection to establish
	delay 3

	-- Get the front window
	tell application "System Events"
		tell process "Screen Sharing"
			-- Enter fullscreen mode
			click menu item "Enter Full Screen" of menu "View" of menu bar 1

			-- Optional: Set dynamic resolution (screen size follows window size)
			-- First open Connection menu
			click menu "Connection" of menu bar 1

			-- Enable "Scale to fit available space"
			click menu item "Scale to fit available space" of menu "Connection" of menu bar 1

			-- Additional option: Use adaptive quality if preferred
			-- click menu item "Adaptive Quality" of menu "Connection" of menu bar 1
		end tell
	end tell
end tell
