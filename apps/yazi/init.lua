function cd()
	local command = "fd --type d --hidden --exclude .git | fzf"
	local handle = io.popen(command)
	local output = handle:read("*a")
	handle:close()

	-- Remove trailing newline
	output = output:gsub("[\n\r]", "")

	if output ~= "" then
		ya.manager.cd(output)
	end
end