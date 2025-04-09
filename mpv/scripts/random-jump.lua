local mp   = require "mp"
local math = require "math"

math.randomseed(os.time())

local pending_jump = false

local function random_jump()
	local pl_count = mp.get_property_number("playlist-count", 0)
	if pl_count < 1 then
		mp.msg.warn("No playlist items available!")
		return
	end

	-- Pause playback before switching files.
	mp.set_property("pause", "yes")

	local rnd_index = math.random(0, pl_count - 1)
	mp.msg.info(string.format("Jumping to playlist index %d", rnd_index))
	mp.set_property_number("playlist-pos", rnd_index)

	pending_jump = true
end

mp.register_script_message("random_jump", random_jump)

mp.register_event("file-loaded", function()
	if not pending_jump then
		return
	end
	pending_jump = false

	local duration = mp.get_property_number("duration", 0)
	if duration and duration > 0 then
		local rnd_time = math.random() * duration
		mp.msg.info(string.format("Seeking to random position %.2f seconds", rnd_time))
		mp.commandv("seek", rnd_time, "absolute")
	else
		mp.msg.warn("Could not obtain file duration.")
	end
	-- Resume playback after the seek.
	mp.set_property("pause", "no")
end)
