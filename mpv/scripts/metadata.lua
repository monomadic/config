local mp = require "mp"

-- Returns a human-friendly description of the file.
local function file_info()
	-- Human-readable file size.
	local fsize = mp.get_property_number("file-size", 0)
	local human_size = "Unknown size"
	if fsize > 0 then
		local units = { "B", "KB", "MB", "GB", "TB" }
		local unit_index, size = 1, fsize
		while size >= 1024 and unit_index < #units do
			size = size / 1024
			unit_index = unit_index + 1
		end
		human_size = string.format("%.1f%s", size, units[unit_index])
	end

	-- local bitrate = mp.get_property_native("video-bitrate")
	-- if bitrate then
	-- 	local bitrate = math.floor(mp.get_property_native("video-bitrate") / 1000000)
	-- 	human_size = string.format("%s %d Mbps", human_size, bitrate)
	-- end

	-- Video codec.
	local codec = mp.get_property("video-codec") or "unknown codec"
	codec = codec:match("^(.-)%s*/") or codec

	-- Determine resolution.
	local height = mp.get_property_number("height", 0)
	local resolution = "unknown resolution"
	if height >= 2160 then
		resolution = "4k"
	elseif height >= 1080 then
		resolution = "1080p"
	elseif height >= 720 then
		resolution = "720p"
	elseif height > 0 then
		resolution = tostring(height) .. "p"
	end

	-- Get frame rate.
	local fps_str = mp.get_property("container-fps")
	local fps = ""
	if fps_str and fps_str ~= "" then
		local fps_num = tonumber(fps_str)
		if fps_num then
			fps = string.format("%dfps", math.floor(fps_num + 0.5))
		end
	end

	return string.format(" %s    %s    %s%s", human_size, codec, resolution,
		fps ~= "" and (" @ " .. fps) or "")
end

mp.register_event("file-loaded", function()
	local meta = mp.get_property_native("metadata")
	if not meta then return end

	local titlefont = string.format("{\\fnHelvetica Neue} ")
	local nerdfont = string.format("{\\fnHack Nerd Font Mono} ")

	-- Get title or a fallback string if not defined.
	local line_1 = meta.title or "" -- mp.get_property("filename")
	local line_2 = meta.artist or ""

	-- Build ASS markup:
	-- {\\an7} aligns top left.
	-- {\\an1} aligns bottom left.
	-- {\\fs20}{\\b1} sets a larger bold font for the title.
	-- {\\fs16} sets a smaller font for subsequent metadata.
	local ass_text = string.format("%s{\\an1}{\\fs12}{\\b1}%s{\\b0}\\N{\\fs9}%s\\N{\\fs6}%s%s", titlefont, line_1, line_2,
		nerdfont, file_info())

	mp.set_osd_ass(0, 0, ass_text)
end)
