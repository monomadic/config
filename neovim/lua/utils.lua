-- UTILS

-- TODO: vim.diagnostic.reset() -- hide all diagnostics

local icons = require('icons');
local M = {}

M.show_code_actions = function()
	local bufnr = vim.api.nvim_get_current_buf()

	local params = vim.lsp.util.make_range_params()
	params.context = { diagnostics = vim.lsp.diagnostic.get_line_diagnostics(bufnr) }

	local actions = {}

	vim.lsp.buf_request_all(bufnr, 'textDocument/codeAction', params, function(results)
		for _, res in pairs(results or {}) do
			if res.result then
				vim.list_extend(actions, res.result)
			end
		end

		if #actions == 0 then
			print("No actions available")
			return
		end

		local titles = {}
		for _, action in ipairs(actions) do
			table.insert(titles, action.title)
		end

		vim.ui.select(titles, { prompt = "Select code action:" }, function(choice)
			local action = actions[choice]
			vim.lsp.buf.execute_command(action.command)
		end)
	end)
end

M.goto_next_diag_wip = function(opts)
	local next_diag = vim.diagnostic.get_next(opts)
	if next_diag == nil then
		return
	end

	-- diag found, move there
	vim.diagnostic.goto_next({ float = false })

	-- show code actions
	local bufnr = vim.api.nvim_get_current_buf()
	local params = vim.lsp.util.make_range_params()
	local context = { diagnostics = vim.lsp.diagnostic.get_line_diagnostics(bufnr) }
	params.context = context

	vim.lsp.buf_request_all(bufnr, 'textDocument/codeAction', params, function(results)
		local has_actions = false
		local actions = {}
		for _, res in pairs(results or {}) do
			if res.result and type(res.result) == 'table' and next(res.result) ~= nil then
				has_actions = true
				for _, action in pairs(res.result) do
					table.insert(actions, action.title)
				end
				break
			end
		end

		if has_actions then
			local height = #actions
			local width = math.max(unpack(vim.tbl_map(string.len, actions)))
			print(vim.inspect(actions))
			vim.lsp.util.open_floating_preview(actions, 'plaintext', { height = height, width = width })

			vim.diagnostic.open_float({ prefix = actions })
		end
	end)
end

-- Prints an object
M.inspect = function(obj)
	print(vim.inspect(obj))
end

M.file_exists = function(fname)
	local stat = vim.loop.fs_stat(fname)
	return (stat and stat.type) or false
end

M.current_file_extension = function()
	local current_file = vim.fn.expand('%')
	local file_extension = string.match(current_file, '%.([^%.]+)$')
	return file_extension
end

M.key = function(mode, keymap, callback, desc)
	vim.keymap.set(mode, keymap, callback, { desc = desc, silent = true })
end

-- Returns (row, col) of the current cursor position
M.current_pos = function()
	return vim.api.nvim_win_get_cursor(0)
end

-- Returns current row of the cursor position
M.current_row = function()
	local row, _ = unpack(vim.api.nvim_win_get_cursor(0))
	return row
end

-- Returns current col of the cursor position
M.current_col = function()
	local _, col = unpack(vim.api.nvim_win_get_cursor(0))
	return col
end


M.open_config = function()
	vim.fn.chdir '~/config/neovim'
	vim.cmd 'edit init.lua'
end

M.highlight_range = function(range, buf, hl_namespace, hl_group)
	---@type integer, integer, integer, integer
	local start_row, start_col, end_row, end_col = unpack(range)
	---@diagnostic disable-next-line: missing-parameter
	vim.highlight.range(buf, hl_namespace, hl_group, { start_row, start_col }, { end_row, end_col })
end

M.select_file_at = function(dir)
	vim.fn.chdir(dir)
	require('telescope.builtin').find_files({ cwd = dir, follow = true })
end

-- load another project in neovim and open the root file
M.switch_workspace = function(dir)
	vim.fn.chdir(dir)
	GoRoot()
end

-- close a buffer
M.close_buffer = function(bufnr)
	if vim.bo.buftype == "terminal" then
		vim.cmd(vim.bo.buflisted and "set nobl | enew" or "hide")
	else
		bufnr = bufnr or vim.api.nvim_get_current_buf()
		-- require("nvchad_ui.tabufline").tabuflinePrev()
		vim.cmd("confirm bd" .. bufnr)
	end
end

-- close all but current buf
M.close_all_buffers = function()
	local bufs = vim.api.nvim_list_bufs()
	for _, buf in ipairs(bufs) do
		M.close_buffer(buf)
	end
end

-- create a floating window from a buffer
M.create_floating_window = function()
	local width = vim.api.nvim_get_option("columns")
	local height = vim.api.nvim_get_option("lines")

	local row = 5
	local col = 3
	local border = 'single'
	-- local term_height = math.ceil(0.7 * vim.o.lines)

	local buf = vim.api.nvim_create_buf(false, true) -- new buffer for the term
	-- local selected_file = vim.fn.expand('%:p') -- the currently open filename
	-- vim.opt_local.filetype = "float"

	vim.api.nvim_buf_set_option(buf, "filetype", "float")
	vim.api.nvim_buf_set_option(buf, "buflisted", false) -- don't show in bufferlist
	--vim.opt.buflisted = false -- don't show in bufferlist

	local win = vim.api.nvim_open_win(buf, true, { -- true here focuses the buffer
		width = math.ceil(width * 0.8),
		height = math.ceil(height * 0.8),
		col = math.ceil(width * 0.1),
		row = math.ceil(height * 0.1),
		style = "minimal",
		border = "single",
		relative = "editor",
	})
	vim.api.nvim_win_set_option(win, "winblend", 20)

	vim.wo.relativenumber = false -- turn off line numbers
	vim.wo.number = false

	return buf
end

M.get_icon = function(name)
	-- local icon, icon_hl = devicons.get_icon(name, string.match(name, "%a+$"))

	local icon = icons.devicons[name]

	if not icon then
		icon = icons.devicons["default_icon"]
	end

	return icon.icon
end

function OpenFiles()
	require('telescope.builtin').find_files { path_display = { "truncate" }, prompt_title = "", preview_title = "" }
end

M.format = function()
	vim.lsp.buf.format { async = true }
end

function Format()
	vim.lsp.buf.format { async = true }
end

function ToggleLineNumbers()
	if vim.wo.number == true then
		vim.wo.relativenumber = false -- turn off line numbers
		vim.wo.number = false
	else
		vim.wo.relativenumber = true -- turn off line numbers
		vim.wo.number = true
	end
end

function Build()
	print("no build command found for this project")
end

M.go_root = function()
	GoRoot()
end

-- go to root project file
function GoRoot()
	-- local files = {"src/lib.rs"}
	--
	-- for file in files do
	-- 	if M.file_exists(file) then
	-- 		vim.cmd.edit(file)
	-- 	end
	-- end

	if M.file_exists("src/lib.rs") then
		vim.cmd.edit 'src/lib.rs'
	elseif M.file_exists("src/main.rs") then
		vim.cmd ':edit src/main.rs'
	elseif M.file_exists("index.md") then
		vim.cmd ':edit index.md'
	elseif M.file_exists("src/index.ts") then
		vim.cmd ':edit src/index.ts'
	elseif M.file_exists("init.lua") then
		vim.cmd ':edit init.lua'
	elseif M.file_exists("README.md") then
		vim.cmd ':edit README.md'
	else
		-- print("no root file found.")
	end
end

function GoPackagerFile()
	if M.file_exists("Cargo.toml") then
		vim.cmd ':edit Cargo.toml'
	elseif M.file_exists("package.json") then
		vim.cmd ':edit package.json'
	else
		print("no package manifest found.")
	end
end

function LSPClients()
	local status = ''
	local ids = vim.lsp.get_active_clients()

	for _, client in ipairs(ids) do
		if vim.lsp.buf_is_attached(0, client.id) then
			status = status .. "%#LspActive# " .. client.name .. " "
		else
			status = status .. "%#LspInactive# " .. client.name .. " "
		end
	end
	return status .. "%#Normal#"
end

function GitBranch()
	if vim.b.branch_name ~= "" then
		return string.format(" %s", vim.b.branch_name)
	else
		return ""
	end
end

function LSPWorkspaceDiagnostics(bufnr)
	local count = {}
	local levels = {
		errors = "Error",
		warnings = "Warn",
		info = "Info",
		hints = "Hint",
	}

	for k, level in pairs(levels) do
		count[k] = vim.tbl_count(vim.diagnostic.get(bufnr, { severity = level }))
	end

	local errors = ""
	local warnings = ""
	local hints = ""
	local info = ""

	if count["errors"] ~= 0 then
		errors = "%#BarDiagnosticError# " .. count["errors"] .. " "
	end
	if count["warnings"] ~= 0 then
		warnings = "%#BarDiagnosticWarn# " .. count["warnings"] .. " "
	end
	if count["hints"] ~= 0 then
		hints = "%#BarDiagnosticHint# " .. count["hints"] .. " "
	end
	if count["info"] ~= 0 then
		info = "%#BarDiagnosticInformation# " .. count["info"] .. " "
	end

	return errors .. warnings .. hints .. info .. "%#Normal#"
end

-- shortcut alias to inspect
function i(obj)
	Utils.inspect(obj)
end

return M
