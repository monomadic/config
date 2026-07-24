require("ls-deluxe-colors"):setup()

-- Right-click opens the "open with" picker instead of opening the file directly
function Entity:click(event, up)
	if up or event.is_middle then
		return
	end

	ya.emit("reveal", { self._file.url })
	if event.is_right then
		ya.emit("open", { interactive = true })
	end
end

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

-- Publish the live cwd for other Kitty panes.
--
-- Answer "what's your cwd?" over Yazi's DDS bus.
--
-- Yazi tracks its cwd internally; on navigation it also chdir()s its own
-- process, so `kitty @ ls` sees the right directory — but ONLY while yazi
-- itself is the foreground process. The moment yazi spawns a blocking child
-- (fzf, a shell command), that child becomes the pane's foreground process and
-- kitty reports the child instead. copy-kitty-next-pane hit exactly this: a
-- target pane sitting in an fzf session reported the wrong cwd.
--
-- Rather than route cwd through kitty at all, we let yazi answer directly.
-- copy-kitty-next-pane broadcasts a "cwd-req" on the DDS bus; every yazi
-- replies with its live cwd tagged with its own pid. The requester matches the
-- reply whose pid is in the target kitty window's process tree — which finds
-- the yazi even when fzf is in front of it. No sidecar files, no OSC 7.
local yazi_pid = (function()
	-- The shell we popen is yazi's child, so its $PPID *is* this yazi's pid —
	-- the same pid kitty lists in the window's process tree. Resolved once.
	local h = io.popen("echo $PPID")
	if not h then
		return nil
	end

	local pid = h:read("*l")
	h:close()
	return pid
end)()

-- `cwd-req` body is ignored; the reply carries { pid, cwd }. ps.pub_to(0, …)
-- broadcasts to every instance (0 == all), which the requester's `ya sub`
-- picks up. sub_remote only fires for messages from *other* instances, which
-- is exactly right — a yazi never needs to answer its own question.
if yazi_pid then
	ps.sub_remote("cwd-req", function()
		ps.pub_to(0, "cwd-rep", { pid = yazi_pid, cwd = tostring(cx.active.current.cwd) })
	end)
end

ps.sub("cd", function()
	ya.emit("shell", { kitty_title_command(cx.active.current.cwd), orphan = true })
end)

Status:children_add(function()
	local handle = io.popen("df -h . | awk 'NR==2 {print $4}'")
	if not handle then
		return ""
	end

	local result = handle:read("*a"):gsub("%s+", "")
	handle:close()
	return "􀤂  " .. result .. " "
end, 500, Status.RIGHT)
