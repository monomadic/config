function _G.RustRunnable()
	local lsp = vim.lsp

	---comment
	---@return string build command
	---@return string|table args
	---@return any cwd
	local function get_command(c, results)
		local ret = " "
		local args = results[c].args

		local dir = args.workspaceRoot

		ret = vim.list_extend({}, args.cargoArgs or {})
		ret = vim.list_extend(ret, args.cargoExtraArgs or {})
		table.insert(ret, "--")
		ret = vim.list_extend(ret, args.executableArgs or {})

		return "cargo", ret, dir
	end

	-- Function to fetch Rust runnables
	local function get_rust_runnables(callback)
		lsp.buf_request(0, "experimental/runnables", {
			textDocument = vim.lsp.util.make_text_document_params(0),
			position = nil,
		}, function(err, result, _)
			if err then
				print("Error fetching runnables: " .. err.message)
				return
			end
			callback(result)
		end)
	end

	-- Function to create and display the output in a floating window
	local function open_floating_window(content)
		local buf = vim.api.nvim_create_buf(false, true)
		vim.api.nvim_buf_set_lines(buf, 0, -1, false, vim.split(content, "\n"))

		local width = vim.api.nvim_get_option("columns")
		local height = vim.api.nvim_get_option("lines")

		local win = vim.api.nvim_open_win(buf, true, {
			relative = "editor",
			width = math.ceil(width * 0.8),
			height = math.ceil(height * 0.8),
			col = width * 0.1,
			row = height * 0.1,
			style = "minimal",
			border = "single"
		})

		vim.api.nvim_win_set_option(win, 'wrap', false)
	end

	local function run_command(choice, result)
		-- do nothing if choice is too high or too low
		if not choice or choice < 1 or choice > #result then
			return
		end

		local command, args, cwd = get_command(choice, result)

		-- Concatenate command and args
		local command_str = command
		for _, arg in ipairs(args) do
			command_str = command_str .. " " .. arg
		end

		-- Execute the command and capture the output
		local handle = io.popen(command_str) -- Execute command with arguments
		local command_output = handle:read("*a")
		handle:close()

		-- Display the result in a floating window
		open_floating_window(command_output)
	end

	-- Function to select a runnable from a list and run it
	local function select_and_run_rust_runnable()
		get_rust_runnables(function(runnables)
			if runnables then
				local options = {}
				for i, runnable in ipairs(runnables) do
					table.insert(options, runnable.label)
				end

				-- Using vim.ui.select to show options
				vim.ui.select(options, { prompt = "Select a runnable" }, function(_, choice)
					run_command(choice, runnables)
				end)
			else
				print("No runnables found")
			end
		end)
	end

	-- Call select_and_run_rust_runnable
	select_and_run_rust_runnable()
end
