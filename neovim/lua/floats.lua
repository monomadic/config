-- modal floats

-- local targetwin = { win = vim.api.nvim_get_current_win(), buf = vim.api.nvim_get_current_buf() }
local background_float_buf = nil

-- local close_term = function()
-- 	background_float_buf = nil
-- 	local win = vim.api.nvim_get_current_win()
-- 	vim.api.nvim_win_close(win, true)
-- end

vim.keymap.set('t', '<C-Space>', function()
	-- hide float
	--background_float_buf = vim.api.nvim_get_current_buf()
	vim.api.nvim_win_hide(0)
end)

vim.keymap.set('n', '<C-Space>', function()
	if background_float_buf and vim.api.nvim_buf_is_valid(background_float_buf) then
		local buf = background_float_buf
		vim.api.nvim_open_win(buf, true, { -- true here focuses the buffer
			relative = 'editor',
			row = math.floor(0.05 * vim.o.lines),
			col = math.floor(0.1 * vim.o.columns),
			width = math.ceil(0.8 * vim.o.columns),
			height = math.ceil(0.7 * vim.o.lines),
			border = 'solid'
		})
	else
		local buf = vim.api.nvim_create_buf(false, false) -- new buffer for the term (listed, scratch)
		background_float_buf = buf
		vim.api.nvim_buf_set_option(buf, "filetype", "terminal")
		vim.api.nvim_buf_set_option(buf, "buflisted", false) -- don't show in bufferlist
		vim.api.nvim_open_win(buf, true, { -- true here focuses the buffer
			relative = 'editor',
			row = math.floor(0.05 * vim.o.lines),
			col = math.floor(0.1 * vim.o.columns),
			width = math.ceil(0.8 * vim.o.columns),
			height = math.ceil(0.7 * vim.o.lines),
			border = 'solid'
		})

		local win = vim.api.nvim_get_current_win()
		vim.api.nvim_win_set_option(win, "winblend", 20)
		vim.api.nvim_win_set_buf(win, buf)
		vim.wo.relativenumber = false -- turn off line numbers
		vim.wo.number = false
		vim.fn.termopen(vim.o.shell)
	end

	vim.cmd "startinsert" -- start in insert mode
end)

-- custom terminal float
vim.keymap.set('n', '<C-p>', function()
	local buf = vim.api.nvim_create_buf(false, true) -- new buffer for the term
	local selected_file = vim.fn.expand('%:p') -- the currently open filename

	vim.api.nvim_buf_set_option(buf, "filetype", "terminal")
	vim.api.nvim_buf_set_option(buf, "buflisted", false) -- don't show in bufferlist
	vim.api.nvim_open_win(buf, true, { -- true here focuses the buffer
		relative = 'editor',
		row = 1, -- math.floor(0.1 * vim.o.lines),
		col = 2, -- math.floor(0.1 * vim.o.columns),
		width = math.ceil(1.0 * vim.o.columns) - 2,
		height = vim.o.lines - 4, --math.ceil(0.8 * vim.o.lines),
		border = 'solid'
	})

	local win = vim.api.nvim_get_current_win()
	vim.api.nvim_win_set_option(win, "winblend", 20)
	vim.api.nvim_win_set_buf(win, buf)
	vim.wo.relativenumber = false -- turn off line numbers
	vim.wo.number = false

	vim.cmd "startinsert" -- start in insert mode

	local lf_tmpfile = vim.fn.tempname()
	local lf_tmpdir = vim.fn.tempname()

	local process_cmd = 'lf -last-dir-path="' ..
			lf_tmpdir .. '" -selection-path="' .. lf_tmpfile .. '" '

	if selected_file ~= "" then
		process_cmd = process_cmd .. '"' .. selected_file .. '"'
	end

	-- launch lf process
	vim.fn.termopen(process_cmd, {
		on_exit = function() -- job_id, exit_code, event_type
			-- if window is a float, close the window
			if vim.api.nvim_win_get_config(win).zindex then
				vim.api.nvim_win_close(win, true)
			end

			-- if lf correctly left us a tempfile
			if vim.loop.fs_stat(lf_tmpfile) then
				local contents = {}
				-- grab the entries that were selected (one per line)
				for line in io.lines(lf_tmpfile) do
					table.insert(contents, line)
				end
				if not vim.tbl_isempty(contents) then
					--vim.api.nvim_win_close(0, true) -- close current (0) with force

					for _, fname in pairs(contents) do
						-- and open them for editing
						vim.cmd(("%s %s"):format('edit', fname))
					end
				end
			end
		end,
	})
end)

vim.api.nvim_create_autocmd("VimEnter", { pattern = "*", callback = function()
	vim.api.nvim_set_hl(0, "NormalFloat", {})
	vim.api.nvim_set_hl(0, "Floaterm", { bg = "Black" })
	vim.api.nvim_set_hl(0, "FloatermBorder", { bg = "Black" })
	vim.api.nvim_set_hl(0, "FloatBorder", { bg = "Black" })
end })