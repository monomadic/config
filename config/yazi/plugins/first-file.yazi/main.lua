--- @sync entry
--- Jump the cursor to the first non-directory entry in the current directory.
---
--- Replaces the upstream `first-non-directory.yazi` plugin, which shipped an
--- `init.lua` entrypoint (pre-25.x layout) and computed a cursor delta by hand.
--- `reveal` takes a url, so this stays correct regardless of how the cursor
--- index is counted.
return {
	entry = function()
		local files = cx.active.current.files
		for i = 1, #files do
			if not files[i].cha.is_dir then
				ya.emit("reveal", { files[i].url })
				return
			end
		end

		ya.notify {
			title = "Goto first file",
			content = "No files here — every entry is a directory",
			level = "warn",
			timeout = 3,
		}
	end,
}
