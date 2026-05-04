---@sync entry
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

return {
	entry = function(_, job)
		local cwd = job and job.args and job.args[1]
		if not cwd or cwd == "" then
			return
		end

		ya.emit("shell", { kitty_title_command(cwd), orphan = true })
	end,
}
