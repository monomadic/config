local mp = require "mp"
local msg = require "mp.msg"

local stats_timer = nil
local update_interval = 0.5 -- seconds

-- Display status: declared fps, estimated (rendered) fps, and bitrate
local function display_stats()
	local status = string.format("{\\\\fnFiraCode Nerd Font} ")

	-- FPS
	local declared_fps = mp.get_property("container-fps")
	if not declared_fps then
		declared_fps = mp.get_property("fps") or "N/A"
	end
	local actual_fps = mp.get_property("estimated-vf-fps") or "N/A"
	status = status .. string.format("󰣿 Framerate: %dfps (actual: %dfps)", declared_fps, actual_fps)

	-- BITRATE
	local bitrate = mp.get_property_native("video-bitrate")
	if bitrate then
		local formatted_bitrate = math.floor(mp.get_property_native("video-bitrate") / 1000000)
		status = status .. string.format("\n󰴙 Bitrate: %d Mbps", formatted_bitrate)
	end

	-- OSD message duration slightly longer than the update interval
	mp.osd_message(status, update_interval + 0.1)
end

-- Toggle the stats display on/off
local function toggle_stats()
	if stats_timer then
		stats_timer:kill()
		stats_timer = nil
		mp.osd_message("", 0) -- clear OSD immediately
		msg.info("FPS/Bitrate stats toggled OFF.")
	else
		stats_timer = mp.add_periodic_timer(update_interval, display_stats)
		msg.info("FPS/Bitrate stats toggled ON.")
	end
end

-- Bind toggle function to F7
mp.add_key_binding("F7", "toggle_stats", toggle_stats)
