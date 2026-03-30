local mp = require 'mp'
local options = require 'mp.options'

local opts = {
	num_entries = 5,
	min_width = 40,
	max_width = 100,
}
options.read_options(opts)

local PLAYLIST_OSD_SECS = 5
local playlist_visible_until = 0

local function get_osd_width()
	local w = mp.get_property_number("osd-width", 1280)
	-- Approximate character width (rough estimate: 10-12 pixels per char)
	local char_width = math.floor(w / 10)
	return math.max(opts.min_width, math.min(opts.max_width, char_width))
end

local function truncate_string(str, max_len)
	if #str <= max_len then
		return str
	end
	return str:sub(1, max_len - 1) .. "…"
end

local function format_time(seconds)
	if not seconds or seconds < 0 then return "" end
	local h = math.floor(seconds / 3600)
	local m = math.floor((seconds % 3600) / 60)
	local s = math.floor(seconds % 60)
	if h > 0 then
		return string.format("%d:%02d:%02d", h, m, s)
	else
		return string.format("%d:%02d", m, s)
	end
end

local function render_playlist(duration)
	local playlist = mp.get_property_native("playlist")
	if not playlist or #playlist == 0 then
		return mp.osd_message("Playlist is empty")
	end

	local pos = mp.get_property_number("playlist-pos")
	local total = #playlist
	
	if not pos then
		return mp.osd_message("Playlist is empty")
	end
	
	local width = get_osd_width()
	local lines = {}
	
	-- Header
	local header = string.format("Playlist [%d/%d]", pos + 1, total)
	local padding = math.floor((width - #header) / 2)
	table.insert(lines, string.rep("─", width))
	table.insert(lines, string.rep(" ", padding) .. header)
	table.insert(lines, string.rep("─", width))
	
	local start_idx = math.max(0, pos - opts.num_entries)
	local end_idx = math.min(total - 1, pos + opts.num_entries)
	
	if start_idx > 0 then
		table.insert(lines, "   ⋮")
	end
	
	for i = start_idx, end_idx do
		local item = playlist[i + 1]
		local filename = item.filename or ""
		local basename = filename:match("([^/\\]+)$") or filename
		local title = item.title or basename:match("(.+)%..+$") or basename
		
		-- Get duration if available
		local duration_str = ""
		if item.duration and item.duration > 0 then
			duration_str = format_time(item.duration)
		end
		
		local prefix = (i == pos) and "▶ " or "  "
		local available_width = width - #prefix - #duration_str - (duration_str ~= "" and 1 or 0)
		
		local display_title = truncate_string(title, available_width)
		
		if duration_str ~= "" then
			-- Right-align duration
			local padding_len = width - #prefix - #display_title - #duration_str
			local line = prefix .. display_title .. string.rep(" ", padding_len) .. duration_str
			table.insert(lines, line)
		else
			table.insert(lines, prefix .. display_title)
		end
	end
	
	if end_idx < total - 1 then
		table.insert(lines, "   ⋮")
	end
	
	table.insert(lines, string.rep("─", width))
	
	mp.osd_message(table.concat(lines, "\n"), duration or PLAYLIST_OSD_SECS)
end

local function show_playlist()
	playlist_visible_until = mp.get_time() + PLAYLIST_OSD_SECS
	render_playlist(PLAYLIST_OSD_SECS)
end

mp.add_key_binding(nil, "show-minimal-playlist", show_playlist)

mp.observe_property("playlist", "native", function()
	local remaining = playlist_visible_until - mp.get_time()
	if remaining > 0 then
		render_playlist(remaining)
	end
end)
