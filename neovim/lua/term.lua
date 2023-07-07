local M = {
	buf = nil,
}

local create_float = function()
	local buf = vim.api.nvim_create_buf(false, true) -- new buffer for the term
	local width = vim.api.nvim_get_option("columns")
	local height = vim.api.nvim_get_option("lines")

	vim.api.nvim_buf_set_option(buf, "filetype", "float")
	vim.opt.buflisted = false -- don't show in bufferlist
	vim.opt.bufhidden = "wipe"

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

M.show = function()
	local width = vim.api.nvim_get_option("columns")
	local height = vim.api.nvim_get_option("lines")

	if M.buf and vim.api.nvim_buf_is_valid(M.buf) then
		local win = vim.api.nvim_open_win(M.buf, true, { -- true here focuses the buffer
			width = math.ceil(width * 0.8),
			height = math.ceil(height * 0.8),
			col = math.ceil(width * 0.1),
			row = math.ceil(height * 0.1),
			style = "minimal",
			border = "single",
			relative = "editor",
		})
		vim.api.nvim_win_set_option(win, "winblend", 20)
	else
		M.buf = create_float()
		vim.fn.termopen(vim.o.shell) -- start terminal
	end

	-- vim.api.nvim_win_set_buf(win, buf)

	vim.cmd.startinsert() -- start in insert mode
end

M.hide = function()
	if M.win and vim.api.nvim_win_is_valid(M.win) then
		vim.api.nvim_win_hide(M.win)
	end
end

M.close = function()
	if M.win and vim.api.nvim_win_is_valid(M.win) then
		vim.api.nvim_win_close(M.win, true)
	end
	if M.buf and vim.api.nvim_buf_is_valid(M.buf) then
		vim.api.nvim_buf_delete(M.buf, { force = true })
	end
	M.buf = nil
	M.win = nil
end

return M
