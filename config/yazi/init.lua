require("mactag"):setup({
	keys = {
		r = "Red",
		o = "Orange",
		y = "Yellow",
		g = "Green",
		b = "Blue",
		p = "Purple",
	},
	colors = {
		Red = "#ee7b70",
		Orange = "#f5bd5c",
		Yellow = "#fbe764",
		Green = "#91fc87",
		Blue = "#5fa3f8",
		Purple = "#cb88f8",
	},
})

Status:children_remove(2, Status.LEFT) -- length
Status:children_remove(4, Status.RIGHT) -- permissions
Status:children_remove(5, Status.RIGHT) -- percentage

local function readable_size(bytes)
	local units = { "B", "KiB", "MiB", "GiB", "TiB", "PiB" }
	local size = bytes
	local unit = 1
	while size >= 1024 and unit < #units do
		size = size / 1024
		unit = unit + 1
	end

	if unit == 1 then
		return string.format("%d %s", size, units[unit])
	end
	return string.format("%.1f %s", size, units[unit])
end

local function size_of(file)
	if not file then
		return nil
	end

	if file.size then
		local size = file:size()
		if size then
			return size
		end
	end

	if file.cha and not file.cha.is_dir then
		return file.cha.len
	end
end

local function selected_size(tab)
	local selected = tab.selected
	local selected_count = #selected
	if selected_count == 0 then
		return nil
	end

	local selected_urls = {}
	for _, url in pairs(selected) do
		selected_urls[tostring(url)] = true
	end

	local matched = 0
	local unknown = 0
	local total = 0
	for i = 1, #tab.current.files do
		local file = tab.current.files[i]
		if file and selected_urls[tostring(file.url)] then
			matched = matched + 1
			local size = size_of(file)
			if size then
				total = total + size
			else
				unknown = unknown + 1
			end
		end
	end
	unknown = unknown + selected_count - matched

	if unknown == selected_count then
		return "size pending"
	end

	local size = readable_size(total)
	if unknown > 0 then
		size = size .. "+"
	end

	return size
end

local function file_size_status()
	local tab = cx.active
	local size = selected_size(tab)
	if not size then
		local hovered = tab.current.hovered
		local bytes = size_of(hovered)
		size = bytes and readable_size(bytes)
	end

	if not size then
		return ui.Span("")
	end

	return ui.Span(" " .. size .. " "):fg("lightblue")
end

Status:children_add(file_size_status, 1000, Status.RIGHT)

require("ffmpeg-stats"):setup({
	duration = false,
	resolution = false,
	codec = false,
	fps = false,
	bitrate = false,
	audio_codec = false,
	audio_channels = false,
	format = false,
	aspect = false,
})

local uname = Command("uname"):arg("-s"):output()
if uname and uname.status and uname.status.success and uname.stdout:match("Linux") then
	require("fs-usage"):setup()
end

local function basename(path)
	path = tostring(path):gsub("/+$", "")
	if path == "" then
		return "/"
	end

	local home = os.getenv("HOME")
	if home and path == home then
		return "~"
	end

	return path:match("([^/]+)$") or path
end

local function shell_quote(value)
	if ya and ya.quote then
		return ya.quote(value)
	end

	return "'" .. tostring(value):gsub("'", "'\"'\"'") .. "'"
end

local function kitty_title_command(cwd)
	local title = "󰘳 " .. basename(cwd)
	local quoted_title = shell_quote(title)

	return table.concat({
		'target="${KITTY_LISTEN_ON:-}"',
		'if [ -z "$target" ]; then',
		'for socket in /tmp/kitty-$USER /tmp/kitty /tmp/kitty-*; do',
		'[ -S "$socket" ] || continue',
		'target="unix:$socket"',
		"break",
		"done",
		"fi",
		'[ -n "$target" ] || exit 0',
		'kitty @ --to "$target" set-tab-title --match state:focused ' .. quoted_title .. " >/dev/null 2>&1",
		'kitty @ --to "$target" set-window-title --match state:focused --temporary ' .. quoted_title .. " >/dev/null 2>&1",
	}, "; ")
end

ps.sub("cd", function()
	ya.emit("shell", { kitty_title_command(cx.active.current.cwd), orphan = true })
end)
