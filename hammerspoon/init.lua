-- hs.loadSpoon("MPVController")
-- -- Bind hotkeys
-- spoon.MPVController:bindHotkeys({
-- 	next = { { "cmd", "alt" }, "right" },    -- Command+Alt+Right Arrow for next track
-- 	previous = { { "cmd", "alt" }, "left" }, -- Command+Alt+Left Arrow for previous track
-- 	toggle = { { "cmd", "alt" }, "space" },  -- Command+Alt+Space to toggle pause
-- 	forward = { { "alt" }, "right" },        -- Alt+Right Arrow to seek forward
-- 	backward = { { "alt" }, "left" },        -- Alt+Left Arrow to seek backward
-- 	volumeUp = { { "cmd", "alt" }, "up" },   -- Command+Alt+Up Arrow to increase volume
-- 	volumeDown = { { "cmd", "alt" }, "down" } -- Command+Alt+Down Arrow to decrease volume
-- })

-- Load the Spoon
hs.loadSpoon("TagSelectedFile")

-- Configure and start the Spoon
spoon.TagSelectedFile
		:bindHotkeys({
			tagFile = { { "cmd", "shift" }, "t" } -- You can customize the hotkey here
		})
-- Optional configuration
-- Set the path to the tag command if it's not at /usr/local/bin/tag
-- spoon.TagSelectedFile.tagCommand = "/opt/homebrew/bin/tag"
-- Enable or disable notifications
-- spoon.TagSelectedFile.showNotifications = true
		:start()

-- -- alt+tab replacement
-- hs.loadSpoon("FuzzySwitcher")
-- spoon.FuzzySwitcher:bindHotkeys({ show_switcher = { { "cmd" }, "space" } })
-- spoon.FuzzySwitcher:start()

-- Maximize focused window
hs.hotkey.bind({ "cmd", "shift" }, "up", function()
	print("call: window maximize");
	local win = hs.window.focusedWindow()
	if win then
		win:maximize()
	end
end)

-- -- Open yazi from finder
-- hs.hotkey.bind({ "rcmd" }, "return", function()
-- 	-- Check if Finder is frontmost right when the hotkey is pressed
-- 	local finder = hs.application.frontmostApplication()
-- 	if not finder or finder:name() ~= "Finder" then
-- 		-- Pass the event through to other applications
-- 		return false
-- 	end
--
-- 	-- Get the frontmost Finder window's path
-- 	local script = [[
--         tell application "Finder"
--             try
--                 set windowPath to POSIX path of (target of front window as alias)
--                 return windowPath
--             on error
--                 return ""
--             end try
--         end tell
--     ]]
-- 	local _, result, _ = hs.osascript.applescript(script)
-- 	if not result or result == "" then return false end
--
-- 	-- Open Kitty in the Finder's directory and run Yazi
-- 	hs.task.new("/opt/homebrew/bin/kitty", nil, { "@", "--hold", "--directory", result, "yazi" }):start()
-- 	return true
-- end)

-- Restore window to its previous state
hs.hotkey.bind({ "cmd", "shift" }, "down", function()
	local app = hs.application.frontmostApplication()
	if app then
		app:selectMenuItem({ "Window", "Zoom" })
	end
end)

-- Focus Kitty
-- Modified Kitty hotkey to toggle visibility
hs.hotkey.bind({ "rightcmd" }, "k", function()
	-- print("toggle: kitty");
	local kitty = hs.application.find("kitty")

	if kitty then
		-- If Kitty is the frontmost app, hide it
		if kitty:isFrontmost() then
			kitty:hide()
		else
			-- If Kitty exists but isn't frontmost, show and focus it
			kitty:unhide()
			kitty:activate()
		end
	else
		-- If Kitty isn't running, launch it
		hs.application.launchOrFocus("kitty")
	end
end)

-- Function to get the Kitty window or create one if it doesn't exist
function getOrCreateKittyWindow()
	local kitty = hs.application.find('kitty')
	if not kitty then
		return nil
	end

	-- Get all Kitty windows
	local windows = kitty:allWindows()

	-- Try to find an existing floating window
	for _, window in ipairs(windows) do
		if window:isFloating() then
			return window
		end
	end

	-- If no floating window exists, create one
	kitty:selectMenuItem({ "Shell", "New OS Window" })
	hs.timer.usleep(100000) -- Wait a bit for the window to be created

	-- Get the new window (should be the last one created)
	local newWindows = kitty:allWindows()
	return newWindows[#newWindows]
end

-- Function to center and show the Kitty window
function toggleFloatingKitty()
	hs.alert.show("hit")

	local window = getOrCreateKittyWindow()
	if not window then
		hs.alert.show("Kitty is not running")
		return
	end

	if window:isVisible() then
		window:hide()
	else
		-- Get the screen frame
		local screen = hs.screen.mainScreen()
		local screenFrame = screen:frame()

		-- Set window size (adjust these values as needed)
		local windowWidth = 800
		local windowHeight = 600

		-- Calculate position to center the window
		local x = screenFrame.x + (screenFrame.w - windowWidth) / 2
		local y = screenFrame.y + (screenFrame.h - windowHeight) / 2

		-- Set window frame and show it
		window:setFrame(hs.geometry.rect(x, y, windowWidth, windowHeight))
		window:setFloating(true)
		window:focus()
	end
end

-- Bind right-command + p to toggle the floating Kitty window
-- hs.hotkey.bind({ "rcmd" }, "p", toggleFloatingKitty)
-- local hyperFn = { "fn" } -- `fn` as the modifier
--
-- -- Function to select "Window -> Zoom" from the menu bar
-- function zoomCurrentWindow()
-- 	local app = hs.application.frontmostApplication()
-- 	if app then
-- 		local menuPath = { "Window", "Zoom" } -- Menu hierarchy
-- 		app:selectMenuItem(menuPath)
-- 	else
-- 		hs.alert.show("No frontmost application")
-- 	end
-- end
--
-- -- Bind Globe + K (fn + K) to the zoom function
-- hs.hotkey.bind(hyperFn, "K", function()
-- 	zoomCurrentWindow()
-- end)
--
-- -- Notify Hammerspoon is ready
-- hs.alert.show("Hammerspoon loaded")
--
-- hs.hotkey.bind({ "cmd" }, "return", function()
-- 	-- Attempt to discover KITTY_LISTEN_ON dynamically
-- 	local socketPath = hs.execute(
-- 		"ps aux | grep '[k]itty' | grep -- '--listen-on' | awk -F'--listen-on=' '{print $2}' | awk '{print $1}'")
-- 	socketPath = socketPath:gsub("\n", "") -- Remove trailing newline
--
-- 	-- Fallback to a default socket path if discovery fails
-- 	if not socketPath or socketPath == "" then
-- 		socketPath = "unix:/tmp/kitty-socket" -- Replace with your fixed socket path if needed
-- 	end
--
-- 	-- Build and execute the Kitty command
-- 	local kittyPath = "/opt/homebrew/bin/kitty"
-- 	local kittyCommand = string.format('%s @ --to %s launch --type tab --cwd "%s"', kittyPath, socketPath,
-- 		os.getenv("HOME"))
-- 	local output, status, type, rc = hs.execute(kittyCommand, true)
--
-- 	-- Show debugging output
-- 	if not status then
-- 		hs.alert.show(string.format("Failed: %s", output or "No output"))
-- 	else
-- 		hs.alert.show("Kitty tab launched successfully!")
-- 	end
-- end)

-- hs.hotkey.bind({"cmd", "alt"}, "return", function()
--     -- Define the Kitty command
--     local kitty_cmd = "/Applications/kitty.app/Contents/MacOS/kitty @ new-window yazi"
--
--     -- Execute the command in the shell
--     hs.execute(kitty_cmd)
-- end)



-- -- Define a hotkey to create a dual-pane layout with Finder windows
-- hs.hotkey.bind({ "cmd", "alt" }, "D", function()
-- 	-- Get the main screen's frame
-- 	local screen = hs.screen.mainScreen():frame()
--
-- 	-- Launch or focus Finder
-- 	hs.application.launchOrFocus("Finder")
-- 	local finder = hs.application.find("Finder")
--
-- 	-- Make sure we have at least two Finder windows
-- 	local windows = finder:allWindows()
-- 	if #windows < 2 then
-- 		-- Create a new Finder window if needed
-- 		hs.applescript.applescript([[
--             tell application "Finder"
--                 make new Finder window
--             end tell
--         ]])
-- 		-- Small delay to let the window appear
-- 		hs.timer.doAfter(0.2, function()
-- 			windows = finder:allWindows()
-- 			arrangeDualPane(windows, screen)
-- 		end)
-- 	else
-- 		arrangeDualPane(windows, screen)
-- 	end
-- end)
--
-- -- Function to arrange windows in dual pane
-- function arrangeDualPane(windows, screen)
-- 	-- Left pane
-- 	windows[1]:setFrame({
-- 		x = screen.x,
-- 		y = screen.y,
-- 		w = screen.w / 2,
-- 		h = screen.h
-- 	})
--
-- 	-- Right pane
-- 	windows[2]:setFrame({
-- 		x = screen.x + screen.w / 2,
-- 		y = screen.y,
-- 		w = screen.w / 2,
-- 		h = screen.h
-- 	})
-- end

-- reload hammerspoon
hs.hotkey.bind({ "cmd", "ctrl" }, "R", function()
	hs.reload()
	hs.notify.new({ title = "Hammerspoon", informativeText = "Config reloaded" }):send()
end)

-- -- switch to remote
-- hs.hotkey.bind({ "cmd" }, "2", function()
-- 	local ssApp = hs.application.get("Screen Sharing")
-- 	if ssApp then
-- 		ssApp:activate()            -- bring Screen Sharing to front
-- 		screenSharingHotkey:disable() -- prevent recursion
-- 		hs.timer.doAfter(0.3, function()
-- 			hs.eventtap.keyStroke({ "cmd" }, "k")
-- 			screenSharingHotkey:enable()
-- 		end)
-- 	else
-- 		hs.alert.show("Screen Sharing app is not running")
-- 	end
-- end)
--
-- -- switch local
-- hs.hotkey.bind({ "cmd" }, "1", function()
-- 	local desktopSwitchHotkey = hs.hotkey.bind({ "rcmd" }, "1", function()
-- 		local mainScreen = hs.screen.mainScreen()
-- 		local uuid = mainScreen:getUUID()
-- 		local desktops = hs.spaces.allSpaces()[uuid]
-- 		if desktops and #desktops > 0 then
-- 			local desktop1 = desktops[1]
-- 			local allWindows = hs.window.allWindows()
-- 			local targetWindow = nil
-- 			for _, win in ipairs(allWindows) do
-- 				local winSpaces = hs.spaces.windowSpaces(win)
-- 				if winSpaces then
-- 					for _, sp in ipairs(winSpaces) do
-- 						if sp == desktop1 then
-- 							targetWindow = win
-- 							break
-- 						end
-- 					end
-- 				end
-- 				if targetWindow then break end
-- 			end
--
-- 			if targetWindow then
-- 				local targetApp = targetWindow:application()
-- 				if targetApp then
-- 					targetApp:activate()     -- focus the app that's on Desktop 1
-- 					desktopSwitchHotkey:disable() -- prevent recursion
-- 					hs.timer.doAfter(0.3, function()
-- 						hs.eventtap.keyStroke({ "cmd" }, "k")
-- 						desktopSwitchHotkey:enable()
-- 					end)
-- 				else
-- 					hs.alert.show("No application found for the window on Desktop 1")
-- 				end
-- 			else
-- 				hs.alert.show("No window found on Desktop 1")
-- 			end
-- 		else
-- 			hs.alert.show("No desktops found for the main screen")
-- 		end
-- 	end)
-- end)

--- SPACES STUFF
---
---
local spaces = require("hs.spaces")

hs.hotkey.bind({ "cmd", "ctrl" }, "e", function()
	hs.alert.show("Switching to left screen")
	local screen = hs.screen.allScreens()[1]          -- Left screen
	local space = spaces.layout()[screen:getUUID()][1] -- First space on left screen

	-- Switch to the space
	spaces.gotoSpace(space)

	-- Move the mouse to the center of the screen to ensure focus
	local point = hs.geometry.rectMidPoint(screen:frame())
	hs.mouse.absolutePosition(point)

	-- Optional: Ensure a window on that screen gets focus
	-- Uncomment this if just moving the mouse isn't enough
	-- local win = hs.window.focusedWindow()
	-- if win and win:screen() ~= screen then
	--     local windowsOnScreen = hs.fnutils.filter(hs.window.allWindows(), function(w)
	--         return w:screen() == screen
	--     end)
	--     if #windowsOnScreen > 0 then
	--         windowsOnScreen[1]:focus()
	--     end
	-- end
end)

hs.hotkey.bind({ "cmd", "ctrl" }, "w", function()
	hs.alert.show("Switching to right screen")
	local screen = hs.screen.allScreens()[2]          -- Right screen
	local space = spaces.layout()[screen:getUUID()][1] -- First space on right screen

	-- Switch to the space
	spaces.gotoSpace(space)

	-- Move the mouse to the center of the screen to ensure focus
	local point = hs.geometry.rectMidPoint(screen:frame())
	hs.mouse.absolutePosition(point)

	-- Optional: Ensure a window on that screen gets focus
	-- Uncomment this if just moving the mouse isn't enough
	local win = hs.window.focusedWindow()
	if win and win:screen() ~= screen then
		local windowsOnScreen = hs.fnutils.filter(hs.window.allWindows(), function(w)
			return w:screen() == screen
		end)
		if #windowsOnScreen > 0 then
			windowsOnScreen[1]:focus()
		end
	end
end)

hs.alert.show("  hammerspoon config reloaded  ")
