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

local M = {}

function M:entry(job)
	if not (os.getenv("KITTY_WINDOW_ID") or os.getenv("KITTY_LISTEN_ON")) then
		return
	end

	local cwd = job.args[1]
	if not cwd or cwd == "" then
		return
	end

	local title = "󰘳 " .. basename(cwd)
	Command("kitty")
		:args({ "@", "set-tab-title", title })
		:stdout(Command.PIPED)
		:stderr(Command.PIPED)
		:status()
	Command("kitty")
		:args({ "@", "set-window-title", "--temporary", title })
		:stdout(Command.PIPED)
		:stderr(Command.PIPED)
		:status()
end

return M
