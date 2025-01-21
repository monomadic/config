--- === MPVController ===
---
--- Control MPV player via IPC socket
---
local obj = {}
obj.__index = obj

-- Metadata
obj.name = "MPVController"
obj.version = "1.0"
obj.author = "Your Name"
obj.license = "MIT"

-- Socket path where MPV will listen
obj.socketPath = "/tmp/.mpv-socket"

function obj:sendCommand(command)
	-- Create a new Unix domain socket
	local socket = hs.socket.new()

	-- Connect to MPV's socket
	socket:connect(self.socketPath)

	-- Format command as JSON (MPV's IPC protocol uses JSON)
	local jsonCommand = hs.json.encode({
		command = command
	})

	-- Send the command
	socket:write(jsonCommand .. "\n")

	-- Close the socket
	socket:disconnect()
end

-- Common MPV control functions
function obj:playNext()
	self:sendCommand({ "playlist-next" })
end

function obj:playPrevious()
	self:sendCommand({ "playlist-prev" })
end

function obj:togglePause()
	self:sendCommand({ "cycle", "pause" })
end

function obj:seekForward()
	self:sendCommand({ "seek", "10" })
end

function obj:seekBackward()
	self:sendCommand({ "seek", "-10" })
end

function obj:volumeUp()
	self:sendCommand({ "add", "volume", "5" })
end

function obj:volumeDown()
	self:sendCommand({ "add", "volume", "-5" })
end

--- MPVController:bindHotkeys(mapping)
--- Method
--- Binds hotkeys for MPVController
---
--- Parameters:
---  * mapping - A table containing hotkey details for the following items:
---   * next - Next track
---   * previous - Previous track
---   * toggle - Toggle pause
---   * forward - Seek forward
---   * backward - Seek backward
---   * volumeUp - Increase volume
---   * volumeDown - Decrease volume
function obj:bindHotkeys(mapping)
	if mapping then
		local spec = {
			next = self.playNext,
			previous = self.playPrevious,
			toggle = self.togglePause,
			forward = self.seekForward,
			backward = self.seekBackward,
			volumeUp = self.volumeUp,
			volumeDown = self.volumeDown
		}

		for command, fn in pairs(spec) do
			if mapping[command] then
				hs.hotkey.bind(
					mapping[command][1],
					mapping[command][2],
					hs.fnutils.partial(fn, self)
				)
			end
		end
	end
	return self
end

return obj
