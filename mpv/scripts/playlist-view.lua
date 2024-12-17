local mp = require 'mp'
local options = require 'mp.options'

local opts = {
	num_entries = 5, -- Number of entries to show above and below current item
}
options.read_options(opts)

function show_playlist()
	local playlist = mp.get_property_native("playlist")
	local pos = mp.get_property_number("playlist-pos")
	local total = #playlist

	if not pos or not total or total == 0 then
		return mp.osd_message("Playlist is empty")
	end

	local str = string.format("\n=== Playlist [%d/%d] ===\n", pos + 1, total)

	local start_idx = math.max(0, pos - opts.num_entries)
	local end_idx = math.min(total - 1, pos + opts.num_entries)

	-- Show if there are more entries above
	if start_idx > 0 then
		str = str .. "   ⋮\n"
	end

	for i = start_idx, end_idx do
		local item = playlist[i + 1]
		local title = item.title or mp.get_property("filename/no-ext")
		local prefix = (i == pos) and "▶ " or "  "
		str = str .. string.format("%s%s\n", prefix, title)
	end

	-- Show if there are more entries below
	if end_idx < total - 1 then
		str = str .. "   ⋮\n"
	end

	mp.osd_message(str, 5)
end

-- Key binding to show playlist
mp.add_key_binding("P", "show-minimal-playlist", show_playlist)

-- Update playlist view when playlist changes
mp.observe_property("playlist", "native", function(name, value)
	-- Only update if playlist is currently shown
	if mp.get_property_number("osd-msg-counter", 0) > 0 then
		show_playlist()
	end
end)
