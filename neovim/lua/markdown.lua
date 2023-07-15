local M = {}

function M.table_of_contents_jump()
	local headings = {}

	for line in vim.api.nvim_buf_get_lines(0, 0, -1, false) do
		if string.match(line, "^#+ .+") then
			table.insert(headings, line)
		end
	end

	local choice = vim.ui.select(headings, { prompt = "Select heading:" })

	if choice then
		local line_num = vim.fn.match(vim.api.nvim_buf_get_lines(0, 0, -1, false), "^" .. choice)
		vim.api.nvim_win_set_cursor(0, { line_num, 0 })
	end
end

vim.api.nvim_set_keymap("n", "<leader>Wh", M.table_of_contents_jump, { noremap = true })

return M
