local utils = require 'mp.utils'

mp.add_key_binding("Ctrl+k", "open_in_kitty", function()
	local path = mp.get_property("path")
	if path then
		local directory = utils.split_path(path)
		utils.subprocess_detached({ args = { "kitty", "-e", "lf", directory } })
	else
		mp.msg.warn("No file currently playing")
	end
end)
