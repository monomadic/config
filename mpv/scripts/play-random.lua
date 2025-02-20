-- local function debug_message(message)
-- 	mp.msg.info(message)   -- This will show in terminal output
-- 	mp.osd_message(message) -- This will show on screen
-- end

local random_player = {
	jump_random = function(self)
		local count = mp.get_property_number("playlist-count", 0)
		if count <= 1 then return end

		local new_pos = math.random(0, count - 1)

		-- debug_message("Jumping to position: " .. new_pos)
		mp.set_property("playlist-pos", new_pos)
	end
}

-- mp.add_key_binding("SPACE", "jump-to-random-file", function()
-- 	random_player:jump_random()
-- end)

mp.register_script_message("jump-to-random-file", function()
	random_player:jump_random()
end)


-- --playlist-start=random
