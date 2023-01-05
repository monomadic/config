-- TERMINAL
--
--	modal floats
--	local vim = vim

-- TODO: extract float creation into function
-- TODO: place term devicon in statusbar if term float exists
-- TODO: make term close C-/ in t
-- - remove <C-j> binding

local row = 2
local col = 2
local width = vim.o.columns
local height = vim.o.lines - 2
-- local term_height = math.ceil(0.7 * vim.o.lines)
local border = 'none'

-- local close_term = function()
-- 	background_float_buf = nil
-- 	local win = vim.api.nvim_get_current_win()
-- 	vim.api.nvim_win_close(win, true)
-- end

local background_float_buf = nil


-- hide float
vim.keymap.set('t', '<C-Space>', function()
	--background_float_buf = vim.api.nvim_get_current_buf()
	vim.api.nvim_win_hide(0)
end)

-- hide float
vim.keymap.set('t', '<C-t>', function()
	--background_float_buf = vim.api.nvim_get_current_buf()
	vim.api.nvim_win_hide(0)
end)

vim.keymap.set('t', '<C-/>', function()
	--background_float_buf = vim.api.nvim_get_current_buf()
	vim.api.nvim_win_hide(0)
end)

function ShowTerminal()
	if background_float_buf and vim.api.nvim_buf_is_valid(background_float_buf) then
		local buf = background_float_buf
		vim.api.nvim_open_win(buf, true, { -- true here focuses the buffer
			relative = 'editor',
			row = row,
			col = col,
			width = width,
			height = height,
			border = border,
		})
	else
		local buf = vim.api.nvim_create_buf(false, false) -- new buffer for the term (listed, scratch)
		background_float_buf = buf
		vim.api.nvim_buf_set_option(buf, "filetype", "terminal")
		vim.api.nvim_buf_set_option(buf, "buflisted", false) -- don't show in bufferlist
		vim.api.nvim_open_win(buf, true, { -- true here focuses the buffer
			relative = 'editor',
			row = row,
			col = col,
			width = width,
			height = height,
			border = border,
		})

		local win = vim.api.nvim_get_current_win()
		-- vim.api.nvim_win_set_option(win, "winblend", 20)
		vim.api.nvim_win_set_buf(win, buf)
		vim.wo.relativenumber = false -- turn off line numbers
		vim.wo.number = false
		vim.fn.termopen(vim.o.shell)
	end

	vim.cmd "startinsert" -- start in insert mode
end

vim.keymap.set('n', '<C-/>', ShowTerminal)

-- lf terminal float
local open_lf = function()
	local buf = vim.api.nvim_create_buf(false, true) -- new buffer for the term
	local selected_file = vim.fn.expand('%:p') -- the currently open filename

	vim.api.nvim_buf_set_option(buf, "filetype", "terminal")
	vim.api.nvim_buf_set_option(buf, "buflisted", false) -- don't show in bufferlist
	vim.api.nvim_open_win(buf, true, { -- true here focuses the buffer
		relative = 'editor',
		row = row,
		col = col,
		width = width,
		height = height,
		border = border,
	})

	local win = vim.api.nvim_get_current_win()
	-- vim.api.nvim_win_set_option(win, "winblend", 20)
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
end
vim.keymap.set('n', '<C-Space>', function() open_lf() end)

vim.api.nvim_create_autocmd("VimEnter", { pattern = "*", callback = function()
	-- default term colors
	vim.g.terminal_color_1 = '#F00'
	vim.g.terminal_color_2 = '#0F0'
	vim.g.terminal_color_3 = '#FF00FF'
	vim.g.terminal_color_4 = '#5CF'
	vim.g.terminal_color_5 = '#FFFF00'
	vim.g.terminal_color_6 = '#BAD'
	vim.g.terminal_color_7 = '#DAB'
	vim.g.terminal_color_8 = '#FAD'
	vim.g.terminal_color_9 = '#7AA'
	vim.g.terminal_color_10 = '#14FFFF'
	vim.g.terminal_color_11 = '#FF0000'
end })

-- local M = {}
--
-- M.member_function = function()
-- end
--
-- export M
