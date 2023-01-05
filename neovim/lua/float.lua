local M = {
	buf = nil,
	cmd = nil,
}

M.init = function()
end

local create_window = function()
	local row = 2
	local col = 2
	local width = vim.o.columns
	local height = vim.o.lines - 2
	-- local term_height = math.ceil(0.7 * vim.o.lines)
	local border = 'none'

	local buf = vim.api.nvim_create_buf(false, true) -- new buffer for the term
	-- local selected_file = vim.fn.expand('%:p') -- the currently open filename

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
end

M.show = function()
	if M.buf and vim.api.nvim_buf_is_valid(M.buf) then
		local row = 2
		local col = 2
		local width = vim.o.columns
		local height = vim.o.lines - 2
		-- local term_height = math.ceil(0.7 * vim.o.lines)
		local border = 'none'

		vim.api.nvim_open_win(M.buf, true, { -- true here focuses the buffer
			relative = 'editor',
			row = row,
			col = col,
			width = width,
			height = height,
			border = border,
		})
	else
		create_window()
	end

	local win = vim.api.nvim_get_current_win()
	-- vim.api.nvim_win_set_option(win, "winblend", 20)
	--vim.api.nvim_win_set_buf(win, buf)
	vim.wo.relativenumber = false -- turn off line numbers
	vim.wo.number = false
	vim.fn.termopen(vim.o.shell)

	vim.cmd.startinsert() -- start in insert mode
end

M.hide = function()
end

M.show()

return M
