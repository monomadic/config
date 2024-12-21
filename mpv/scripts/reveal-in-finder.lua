local utils = require 'mp.utils'
mp.add_key_binding("Ctrl+f", "reveal_in_finder", function()
	local path = mp.get_property("path")
	if not path then
		mp.msg.warn("No file currently playing")
		return
	end

	local result = utils.subprocess({
		args = { "/usr/bin/open", "-R", path },
		cancellable = false,
	})

	if result.error ~= nil then
		mp.msg.error("Failed to reveal file: " .. (result.error or "unknown error"))
	end
end)
