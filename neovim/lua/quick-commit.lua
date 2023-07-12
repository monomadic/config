function QuickCommit()
	local handle = io.popen('git rev-parse --is-inside-work-tree')
	if handle == nil then
		return
	end

	local result = handle:read("*a")
	handle:close()

	if result:find('true') == nil then
		print("This directory is not a git repository.")
		return
	end

	-- add all changed files to staging
	vim.cmd('!git add -A')

	-- check if there are changes to commit
	handle = io.popen('git diff --cached --exit-code')
	if handle == nil then
		return
	end
	result = handle:read("*a")
	handle:close()

	if result:find("changed") == nil then
		print("There are no changes to commit.")
		return
	end

	-- prompts for a commit message
	vim.ui.input({ prompt = "Commit message: ", default = "add: " }, function(message)
		if not message then
			return
		end
		-- creates the commit
		vim.cmd('!git commit -m "' .. message .. '"')
		print("Commit successful")
	end)
end
