--- === TagSelectedFile ===
---
--- Spoon to run tag command on selected files in Finder with CMD+SHIFT+T
---
--- Download: https://github.com/username/TagSelectedFile.spoon

local obj = {}
obj.__index = obj

-- Metadata
obj.name = "TagSelectedFile"
obj.version = "1.0"
obj.author = "Your Name"
obj.homepage = "https://github.com/username/TagSelectedFile.spoon"
obj.license = "MIT - https://opensource.org/licenses/MIT"

-- Default settings
obj.hotkey = { { "cmd", "shift" }, "t" }
obj.tagCommand = "/usr/local/bin/tag"
obj.showNotifications = true

-- Internal variables
obj.finderHotkey = nil
obj.finderWatcher = nil

-- Function to get the selected file in Finder
function obj:getFinderSelection()
	local script = [[
        tell application "Finder"
            set theSelection to selection
            if theSelection is not {} then
                set filePath to POSIX path of (item 1 of theSelection as alias)
                return filePath
            end if
        end tell
    ]]
	return hs.osascript.applescript(script)
end

-- Function to run tag command and show output
function obj:runTagCommand(file)
	-- Create a task to run the tag command
	local task = hs.task.new(self.tagCommand, function(exitCode, stdOut, stdErr)
		if self.showNotifications then
			if exitCode == 0 then
				-- Show success notification with output
				hs.notify.new({
					title = "Tag Command Result",
					informativeText = stdOut or "Success!",
					setIdImage = hs.image.imageFromPath(
						"/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/TagsIcon.icns")
				}):send()
			else
				-- Show error notification
				hs.notify.new({
					title = "Tag Command Error",
					informativeText = stdErr or "Unknown error occurred",
					setIdImage = hs.image.imageFromPath(
						"/System/Library/CoreServices/CoreTypes.bundle/Contents/Resources/AlertStopIcon.icns")
				}):send()
			end
		end
	end, { file })

	task:start()
end

-- Initialize the spoon
function obj:init()
	return self
end

-- Start the spoon
function obj:start()
	-- Create the hotkey
	self.finderHotkey = hs.hotkey.new(self.hotkey[1], self.hotkey[2], function()
		-- Check if Finder is the active application
		local currentApp = hs.application.frontmostApplication()
		if currentApp:name() == "Finder" then
			-- Get selected file
			local ok, filePath = self:getFinderSelection()
			if ok and filePath then
				self:runTagCommand(filePath)
			elseif self.showNotifications then
				hs.notify.new({
					title = "Error",
					informativeText = "No file selected in Finder"
				}):send()
			end
		end
	end)

	-- Bind the hotkey only when Finder is active
	self.finderWatcher = hs.application.watcher.new(function(appName, eventType)
		if appName == "Finder" then
			if eventType == hs.application.watcher.activated then
				self.finderHotkey:enable()
			elseif eventType == hs.application.watcher.deactivated then
				self.finderHotkey:disable()
			end
		end
	end)

	-- Start the application watcher
	self.finderWatcher:start()

	return self
end

-- Stop the spoon
function obj:stop()
	if self.finderHotkey then
		self.finderHotkey:delete()
		self.finderHotkey = nil
	end

	if self.finderWatcher then
		self.finderWatcher:stop()
		self.finderWatcher = nil
	end

	return self
end

-- Bind hotkey with custom settings
function obj:bindHotkeys(mapping)
	if mapping.tagFile then
		self.hotkey = mapping.tagFile
		-- If already running, restart with new hotkey
		if self.finderHotkey then
			self:stop()
			self:start()
		end
	end
	return self
end

return obj
