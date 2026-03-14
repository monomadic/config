local M = {}

M.table_of_contents_jump = function()
	local headings = {}
	local lines = vim.api.nvim_buf_get_lines(0, 0, -1, false)

	for i, line in ipairs(lines) do
		if string.match(line, "^#+ ") then
			headings[i] = line
		end
	end

	vim.ui.select(headings, { prompt = "Table of Contents" }, function(choice)
		if choice then
			for i, line in ipairs(lines) do
				if line == choice then
					vim.api.nvim_win_set_cursor(0, { i, 0 })
					break
				end
			end
		end
	end)
end

return M
