--- @sync entry
--
-- spill-paste — paste the yanked files into the current directory through
-- spill, so every copy is verified (xxhash64) instead of trusted.
--
--   plugin spill-paste                 follow the yank mode (copy, or move if cut)
--   plugin spill-paste -- copy         always copy, even for a cut
--   plugin spill-paste -- move         always move (copy, verify, then delete)
--
-- A second argument picks the verification level — none, size or hash
-- (default) — e.g. `plugin spill-paste -- copy none` for a plain fast copy.
-- A move never runs below size, whatever is asked for.
--
-- The real work is in ~/.zsh/bin/spill-paste; this only collects the yank list
-- and the destination, and clears the yank once a move has run.

local function quote(value)
	if ya and ya.quote then
		return ya.quote(value)
	end
	return "'" .. tostring(value):gsub("'", "'\\''") .. "'"
end

return {
	entry = function(_, job)
		local paths = {}
		for _, u in pairs(cx.yanked) do
			paths[#paths + 1] = tostring(u)
		end

		if #paths == 0 then
			return ya.notify {
				title = "spill-paste",
				content = "Nothing yanked",
				timeout = 3,
				level = "warn",
			}
		end

		local mode = job.args[1]
		if mode ~= "copy" and mode ~= "move" then
			mode = cx.yanked.is_cut and "move" or "copy"
		end

		local verify = job.args[2]
		if verify ~= "none" and verify ~= "size" and verify ~= "hash" then
			verify = "hash"
		end

		local cwd = tostring(cx.active.current.cwd)

		-- The transfer runs in its own Kitty window rather than taking over the
		-- yazi pane, so a multi-gigabyte spill doesn't hold the file manager
		-- hostage. The cost is that nothing here sees the result, which is what
		-- --hold is for: the script keeps its own window up on failure, because
		-- this side can't report one.
		local cmd = "~/.zsh/bin/kitty-launch --window"
			.. " --cwd " .. quote(cwd)
			.. " --title " .. quote(mode == "move" and " spill move " or " spill copy ")
			.. " -- ~/.zsh/bin/spill-paste --hold --verify " .. verify

		if mode == "move" then
			cmd = cmd .. " --move"
		end
		cmd = cmd .. " -- " .. quote(cwd)
		for _, p in ipairs(paths) do
			cmd = cmd .. " " .. quote(p)
		end

		ya.emit("shell", { cmd, orphan = true })

		-- A move consumes the yank the way yazi's own paste-after-cut does; a
		-- copy leaves it, so the same set can be spilled somewhere else too.
		-- Detached, this fires before the transfer finishes — a move that fails
		-- keeps its files but loses the yank list.
		if mode == "move" then
			ya.emit("unyank", {})
		end
	end,
}
