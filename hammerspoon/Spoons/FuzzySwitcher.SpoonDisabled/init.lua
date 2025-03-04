-- Add this to your Hammerspoon init.lua file

-- Define a hotkey to create a dual-pane layout with Finder windows
hs.hotkey.bind({ "cmd", "alt" }, "D", function()
	-- Get the main screen's frame
	local screen = hs.screen.mainScreen():frame()

	-- Launch or focus Finder
	hs.application.launchOrFocus("Finder")
	local finder = hs.application.find("Finder")

	-- Make sure we have at least two Finder windows
	local windows = finder:allWindows()
	if #windows < 2 then
		-- Create a new Finder window if needed
		hs.applescript.applescript([[
            tell application "Finder"
                make new Finder window
            end tell
        ]])
		-- Small delay to let the window appear
		hs.timer.doAfter(0.2, function()
			windows = finder:allWindows()
			arrangeDualPane(windows, screen)
		end)
	else
		arrangeDualPane(windows, screen)
	end
end)

-- Function to arrange windows in dual pane
function arrangeDualPane(windows, screen)
	-- Left pane
	windows[1]:setFrame({
		x = screen.x,
		y = screen.y,
		w = screen.w / 2,
		h = screen.h
	})

	-- Right pane
	windows[2]:setFrame({
		x = screen.x + screen.w / 2,
		y = screen.y,
		w = screen.w / 2,
		h = screen.h
	})
end
