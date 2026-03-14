local utils = require 'mp.utils'

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

	-- Clear the playlist and load files
	mp.command("playlist-clear")
	for _, file in ipairs(files) do
		mp.commandv("loadfile", file, "append-play")
	end

	-- Display an OSD message with the directory name and file count
	mp.osd_message(string.format("Playing %s (%d items)", dirname, num_files))
end

mp.add_key_binding("d", "replace-playlist", replace_playlist_with_dir)
