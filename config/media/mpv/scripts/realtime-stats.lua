local mp = require "mp"
local msg = require "mp.msg"

local stats_timer = nil
local update_interval = 0.5 -- seconds

local function format_number(value)
	if not value then
		return "N/A"
	end
	if math.abs(value - math.floor(value + 0.5)) < 0.01 then
		return tostring(math.floor(value + 0.5))
	end
	return string.format("%.2f", value)
end

local function format_fps(value)
	if not value then
		return "N/A"
	end
	return format_number(value) .. "fps"
end

-- Display status: declared fps, estimated (rendered) fps, and bitrate
local function display_stats()
	local status = string.format("{\\\\fnFiraCode Nerd Font} ")

	-- FPS
	local declared_fps = tonumber(mp.get_property("container-fps"))
	if not declared_fps then
		declared_fps = tonumber(mp.get_property("fps"))
	end
	local actual_fps = tonumber(mp.get_property("estimated-vf-fps"))
	status = status .. string.format("󰣿 Framerate: %s (actual: %s)", format_fps(declared_fps), format_fps(actual_fps))

	-- BITRATE
	local bitrate = tonumber(mp.get_property_native("video-bitrate"))
	if bitrate then
		local formatted_bitrate = bitrate / 1000000
		status = status .. string.format("\n󰴙 Bitrate: %s Mbps", format_number(formatted_bitrate))
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
	mp.commandv("script-message", "realtime-stats-state", stats_timer and "yes" or "no")
end

mp.register_script_message("realtime-stats-query", function()
	mp.commandv("script-message", "realtime-stats-state", stats_timer and "yes" or "no")
end)

mp.add_key_binding(nil, "toggle_stats", toggle_stats)
mp.add_timeout(0, function()
	mp.commandv("script-message", "realtime-stats-state", "no")
end)
