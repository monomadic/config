local utils = require 'mp.utils'
local pending_restore = nil

local function basename(path)
	return path and path:match("([^/\\]+)$") or path
end

local function replace_playlist_with_dir()
	local filepath = mp.get_property("path")
	if not filepath then
		mp.osd_message("No file currently playing")
		return
	end

	-- Get the directory of the current file
	local dir = utils.split_path(filepath)
	if not dir then
		mp.osd_message("Failed to determine directory")
		return
	end

	-- Extract the directory name from the full directory path
	local dirname = dir:match("([^/]+)/?$") or "Unknown"

	-- Use `fd` to list all files in the directory
	local args = { "fd", "--type", "f", "--color", "never", ".", dir }
	local result = utils.subprocess({ args = args, cancellable = false })
	if result.status ~= 0 then
		mp.osd_message("Failed to list directory files")
		return
	end

	-- Collect files into a table
	local files = {}
	for file in result.stdout:gmatch("[^\r\n]+") do
		table.insert(files, file)
	end

	-- Check if there are any files
	local num_files = #files
	if num_files == 0 then
		mp.osd_message("No files found in directory")
		return
	end

	local current_filename = mp.get_property("filename")
	local current_time = mp.get_property_number("time-pos")
	local was_paused = mp.get_property_bool("pause")
	local target_index = 1
	local found_current = false

	for i, file in ipairs(files) do
		if basename(file) == current_filename then
			target_index = i
			found_current = true
			break
		end
	end

	pending_restore = {
		filename = basename(files[target_index]),
		time_pos = found_current and current_time or nil,
		pause = was_paused,
	}

	mp.set_property_bool("pause", true)
	mp.commandv("loadfile", files[1], "replace")
	for i = 2, #files do
		mp.commandv("loadfile", files[i], "append")
	end

	if target_index > 1 then
		mp.add_timeout(0, function()
			mp.commandv("playlist-play-index", tostring(target_index - 1))
		end)
	end

	-- Display an OSD message with the directory name and file count
	mp.osd_message(string.format("Playing %s (%d items)", dirname, num_files))
end

mp.register_event("file-loaded", function()
	if not pending_restore then return end
	if mp.get_property("filename") ~= pending_restore.filename then
		return
	end

	local restore = pending_restore
	pending_restore = nil

	if restore.time_pos and restore.time_pos > 0 then
		mp.commandv("seek", restore.time_pos, "absolute", "exact")
	end

	if restore.pause ~= nil then
		mp.set_property_bool("pause", restore.pause)
	end
end)

mp.add_key_binding(nil, "replace-playlist", replace_playlist_with_dir)
