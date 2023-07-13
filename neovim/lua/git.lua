function GitPush()
	local function open_floating_terminal_with_command(command)
		local buf = vim.api.nvim_create_buf(false, true)
		local width = vim.api.nvim_get_option("columns")
		local height = vim.api.nvim_get_option("lines")

		vim.api.nvim_buf_set_option(buf, "bufhidden", "wipe")

		local win = vim.api.nvim_open_win(buf, true, {
			relative = "editor",
			width = math.ceil(width * 0.8),
			height = math.ceil(height * 0.8),
			col = math.ceil(width * 0.1),
			row = math.ceil(height * 0.1),
			style = "minimal",
			border = "single"
		})

		vim.api.nvim_win_set_option(win, "winhl", "Normal:NormalFloat")
		vim.cmd("terminal " .. command)
	end

	open_floating_terminal_with_command("git push")
end

function AiCommit()
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
	vim.fn.system('!git add -A')

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
		vim.cmd('!aicommit')
		print("Commit successful")
	end)
end

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
	vim.fn.system('!git add -A')

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
