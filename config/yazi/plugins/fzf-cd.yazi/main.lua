local cwd = ya.sync(function()
	return tostring(cx.active.current.cwd)
end)

local function fail(s, ...)
	ya.notify({
		title = "fzf-cd",
		content = string.format(s, ...),
		timeout = 5,
		level = "error",
	})
end

-- Same picker fzf-cd-shell uses: pinned dirs + global sources via fzf-open.
local command = [=[
if ! command -v fzf-open >/dev/null 2>&1; then
	print -u2 -- "fzf-cd: fzf-open not found on PATH"
	exit 2
fi
fzf-open
]=]

return {
	entry = function()
		-- ya.hide() was renamed to ui.hide() in Yazi 26.x
		local permit = (ui.hide or ya.hide)()
		local output, err = Command("zsh")
			:arg({ "-c", command })
			:cwd(cwd())
			:stdin(Command.INHERIT)
			:stdout(Command.PIPED)
			:stderr(Command.INHERIT)
			:output()
		permit:drop()

		if not output then
			return fail("Run `fzf-cd` failed: %s", err)
		elseif not output.status.success and output.status.code ~= 1 and output.status.code ~= 130 then
			return fail("`fzf-cd` exited with code %s", output.status.code)
		end

		local target = output.stdout:gsub("\n$", "")
		if target == "" then
			return
		end

		ya.emit("cd", { target })
	end,
}
