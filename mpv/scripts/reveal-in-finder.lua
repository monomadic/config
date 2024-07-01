local utils = require 'mp.utils'

mp.add_key_binding("Ctrl+f", "reveal_in_finder", function()
    local path = mp.get_property("path")
    if path then
        utils.subprocess({args = {"open", "-R", path}})
    else
        mp.msg.warn("No file currently playing")
    end
end)
