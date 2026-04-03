local M = {
	buf = nil,
	win = nil,
}

local function create_float()
	local buf = vim.api.nvim_create_buf(false, true)
	local width = vim.api.nvim_get_option("columns")
	local height = vim.api.nvim_get_option("lines")

	vim.api.nvim_buf_set_option(buf, "filetype", "float")
	vim.api.nvim_buf_set_option(buf, "buflisted", false)
	vim.api.nvim_buf_set_option(buf, "bufhidden", "wipe")

	local win = vim.api.nvim_open_win(buf, true, {
		width = math.ceil(width * 0.8),
		height = math.ceil(height * 0.8),
		col = math.ceil(width * 0.1),
		row = math.ceil(height * 0.1),
		style = "minimal",
		border = "single",
		relative = "editor",
	})

	vim.api.nvim_win_set_option(win, "winblend", 20)
	vim.api.nvim_win_set_option(win, "number", false)
	vim.api.nvim_win_set_option(win, "relativenumber", false)

	return buf, win
end

M.show = function()
	local width = vim.api.nvim_get_option("columns")
	local height = vim.api.nvim_get_option("lines")

	if M.buf and vim.api.nvim_buf_is_valid(M.buf) and M.win and vim.api.nvim_win_is_valid(M.win) then
		vim.api.nvim_open_win(M.buf, true, {
			width = math.ceil(width * 0.8),
			height = math.ceil(height * 0.8),
			col = math.ceil(width * 0.1),
			row = math.ceil(height * 0.1),
			style = "minimal",
			border = "single",
			relative = "editor",
		})
		vim.api.nvim_win_set_option(M.win, "winblend", 20)
	else
		M.buf, M.win = create_float()
		vim.fn.termopen(vim.o.shell)
	end

	vim.cmd("startinsert")
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
