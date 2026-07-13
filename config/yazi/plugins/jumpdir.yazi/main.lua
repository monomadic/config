local cwd = ya.sync(function()
	return tostring(cx.active.current.cwd)
end)

local function fail(s, ...)
	ya.notify({
		title = "jumpdir",
		content = string.format(s, ...),
		timeout = 5,
		level = "error",
	})
end

local function shell_name()
	local shell = os.getenv("SHELL")
	return shell and shell:match("[^/]+$") or "sh"
end

local function command(max_depth)
	return string.format(
		[[
if command -v fd >/dev/null 2>&1; then
	fd --type directory --absolute-path --max-depth %d --exclude .git --exclude node_modules . "$PWD" \
		| sed 's:/*$::'
else
	find "$PWD" -maxdepth %d -type d \
		-not -path "$PWD" \
		-not -path '*/.git/*' \
		-not -path '*/node_modules/*'
fi | if command -v fzf-cd >/dev/null 2>&1; then
	fzf-cd --prompt 'local > ' --header "Children of $PWD (depth %d)" --height 80%%
else
	fzf \
		--prompt='jumpdir> ' \
		--height=80%% \
		--reverse \
		--bind='ctrl-d:preview-page-down,ctrl-u:preview-page-up' \
		--bind='alt-r:execute-silent(open {})' \
		--preview='
			if command -v eza >/dev/null 2>&1; then
				eza --tree --level=2 --color=always -- {}
			else
				ls -la -- {}
			fi
		'
fi
]],
		max_depth,
		max_depth,
		max_depth
	)
end

return {
	entry = function(self, job)
		job = job or self
		local args = job and job.args or {}
		local max_depth = tonumber(args[1]) or 4
		-- ya.hide() was renamed to ui.hide() in Yazi 26.x
		local permit = (ui.hide or ya.hide)()
		local output, err = Command(shell_name())
			:arg({ "-c", command(max_depth) })
			:cwd(cwd())
			:stdin(Command.INHERIT)
			:stdout(Command.PIPED)
			:stderr(Command.INHERIT)
			:output()
		permit:drop()

		if not output then
			return fail("Run `fzf` failed: %s", err)
		elseif not output.status.success and output.status.code ~= 1 and output.status.code ~= 130 then
			return fail("`fzf` exited with code %s", output.status.code)
		end

		local target = output.stdout:gsub("\n$", "")
		if target == "" then
			return
		end

		ya.emit("cd", { target })
	end,
}
