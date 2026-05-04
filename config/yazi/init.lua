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

require("simple-status"):setup()

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

require("dual-pane"):setup()

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
