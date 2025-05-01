local mp   = require "mp"
local math = require "math"

math.randomseed(os.time())

local pending_jump = false

-- Jump to a random playlist file and then random time inside it
function random_playlist_jump()
	local pl_count = mp.get_property_number("playlist-count", 0)
	if pl_count < 1 then
		mp.msg.warn("No playlist items available!")
		return
	end

	mp.set_property("pause", "yes")

	local rnd_index = math.random(0, pl_count - 1)
	mp.msg.info(string.format("Jumping to playlist index %d", rnd_index))
	mp.set_property_number("playlist-pos", rnd_index)

	pending_jump = true
end

-- After file loads, jump to a random position inside it (full duration)
mp.register_event("file-loaded", function()
	if not pending_jump then return end
	pending_jump = false

	local duration = mp.get_property_number("duration", 0)
	if duration and duration > 0 then
		local rnd_time = math.random() * duration
		mp.msg.info(string.format("Seeking to random position %.2f seconds", rnd_time))
		mp.commandv("seek", rnd_time, "absolute")
	else
		mp.msg.warn("Could not obtain file duration.")
	end
	mp.set_property("pause", "no")
end)

-- Jump within current file but exclude first/last 10%
local function random_seek_within_file()
	local duration = mp.get_property_number("duration", 0)
	if not duration or duration <= 0 then
		mp.msg.warn("Invalid duration; cannot seek.")
		return
	end

	local min_time = duration * 0.10
	local max_time = duration * 0.90
	local rnd_time = min_time + math.random() * (max_time - min_time)
	mp.msg.info(string.format("Random seek within file to %.2f seconds", rnd_time))
	mp.commandv("seek", rnd_time, "absolute")
end

-- Autojump mode toggle
local active = false
local timer = nil

local function toggle_auto_jump()
	active = not active
	if active then
		timer = mp.add_periodic_timer(5, function()
			random_playlist_jump()
		end)
		mp.osd_message("Autojump: ON")
	else
		if timer then
			timer:kill()
			timer = nil
		end
		mp.osd_message("Autojump: OFF")
	end
end

-- Key bindings
mp.add_key_binding("ENTER", "random_playlist_jump", random_playlist_jump)
mp.add_key_binding("a", "toggle_auto_jump", toggle_auto_jump)
mp.add_key_binding("j", "random_seek_within_file", random_seek_within_file)
