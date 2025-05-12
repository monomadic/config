local obj = {}
obj.__index = obj

obj.name = "RemoteCmdInjector"
obj.version = "1.1"
obj.author = "ChatGPT"
obj.license = "MIT"

-- Configurable defaults
obj.targetAppBundleID = "com.apple.RemoteDesktop" -- Adjust if needed
obj.triggerMod = "rightalt"                       -- Hammerspoon calls it rightalt
obj.keys = {}                                     -- Keys to intercept

local tap

function obj:start()
	tap = hs.eventtap.new({ hs.eventtap.event.types.keyDown }, function(event)
		local flags = event:getFlags()
		local key = event:getCharacters(true)

		if flags[obj.triggerMod] and not (flags.cmd or flags.ctrl or flags.alt or flags.shift) then
			for _, remapKey in ipairs(obj.keys) do
				if key == remapKey then
					-- Focus the app using bundle ID (more reliable than :activate())
					hs.application.launchOrFocusByBundleID(obj.targetAppBundleID)

					-- Inject CMD+<key> after a short delay to ensure focus
					hs.timer.doAfter(0.3, function()
						hs.eventtap.keyStroke({ "cmd" }, remapKey, 0)
					end)

					return true -- block original event
				end
			end
		end

		return false
	end)

	tap:start()
end

function obj:stop()
	if tap then
		tap:stop()
		tap = nil
	end
end

return obj
