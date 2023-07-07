-- require("plenary.job")
--   :new({
--     command = command,
--     args = args,
--     cwd = cwd,
--     on_stdout = function(_, data)
--       vim.schedule(function()
--         append_qf(data)
--       end)
--     end,
--     on_stderr = function(_, data)
--       vim.schedule(function()
--         append_qf(data)
--       end)
--     end,
--   })
--   :start()

function _G.RustRunnable()
	local lsp = vim.lsp

	---comment
	---@param command string
	---@param args table
	function make_command_from_args(command, args)
		local ret = command .. " "

		for _, value in ipairs(args) do
			ret = ret .. value .. " "
		end

		return ret
	end

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

	-- -- Function to create and display the output in a floating window
	-- local function open_floating_window(content)
	-- 	local width = vim.api.nvim_get_option("columns")
	-- 	local height = vim.api.nvim_get_option("lines")
	--
	-- 	local buf = vim.api.nvim_create_buf(false, true)
	-- 	vim.api.nvim_buf_set_option(buf, "bufhidden", "wipe")
	-- 	--vim.api.nvim_buf_set_lines(buf, 0, -1, false, vim.split(content, "\n"))
	--
	-- 	local win = vim.api.nvim_open_win(buf, true, {
	-- 		relative = "editor",
	-- 		width = math.ceil(width * 0.8),
	-- 		height = math.ceil(height * 0.8),
	-- 		col = width * 0.1,
	-- 		row = height * 0.1,
	-- 		style = "minimal",
	-- 		border = "single"
	-- 	})
	--
	-- 	vim.api.nvim_win_set_option(win, "winhl", "Normal:NormalFloat")
	-- 	vim.api.nvim_win_set_option(win, 'wrap', false)
	-- end

	local function run_command(choice, result)
		-- do nothing if choice is too high or too low
		if not choice or choice < 1 or choice > #result then
			return
		end

		local command, args, cwd = get_command(choice, result)
		local cmd = make_command_from_args(command, args)
		open_floating_terminal_with_command(cmd)

		-- -- Define a callback to handle the job's output
		-- local output = {}
		-- local function on_stdout(_, data)
		-- 	for _, line in ipairs(data) do
		-- 		table.insert(output, line)
		-- 	end
		-- end
		--
		-- local function on_stderr(_, data)
		-- 	for _, line in ipairs(data) do
		-- 		table.insert(output, "" .. line)
		-- 	end
		-- end
		--
		-- -- Define a callback to handle the job's completion
		-- local function on_exit(_, _)
		-- 	-- Combine the output lines into a single string
		-- 	local command_output = table.concat(output, '\n')
		--
		-- 	-- Display the result in a floating window
		-- 	open_floating_window(command_output)
		-- end
		--
		-- -- Execute the command asynchronously
		-- local job_id = vim.fn.jobstart({ command, unpack(args) }, {
		-- 	cwd = cwd,
		-- 	on_stdout = on_stdout,
		-- 	on_stderr = on_stderr,
		-- 	on_exit = on_exit,
		-- })
		--
		-- -- Check if the job started successfully
		-- if job_id < 1 then
		-- 	print("Failed to start job")
		-- end
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
