local keymap = vim.keymap.set

local M = {
	buf = nil,
}

local create_float = function()
	local buf = vim.api.nvim_create_buf(false, true) -- new buffer for the term
	-- local selected_file = vim.fn.expand('%:p') -- the currently open filename
	-- vim.opt_local.filetype = "float"
	local row = 4
	local col = 4
	local width = vim.o.columns
	local height = vim.o.lines - 4
	local border = 'none'

	vim.api.nvim_buf_set_option(buf, "filetype", "float")
	vim.api.nvim_buf_set_option(buf, "buflisted", false) -- don't show in bufferlist
	--vim.opt.buflisted = false -- don't show in bufferlist
	vim.api.nvim_open_win(buf, true, { -- true here focuses the buffer
		relative = 'editor',
		row = row,
		col = col,
		width = width,
		height = height,
		border = border,
	})

	vim.wo.relativenumber = false -- turn off line numbers
	vim.wo.number = false

	return buf
end

M.show = function()
	if M.buf and vim.api.nvim_buf_is_valid(M.buf) then
		vim.api.nvim_open_win(M.buf, true, { -- true here focuses the buffer
			relative = 'editor',
			row = 2,
			col = 2,
			width = vim.o.columns,
			height = vim.o.lines - 2,
			border = 'none',
		})
	else
		M.buf = create_float()
		vim.fn.termopen(vim.o.shell) -- start terminal
	end

	-- local win = vim.api.nvim_get_current_win()
	-- vim.api.nvim_win_set_option(win, "winblend", 20)
	-- vim.api.nvim_win_set_buf(win, buf)

	vim.cmd.startinsert() -- start in insert mode
end

M.hide = function()
end

return M
