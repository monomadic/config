----------------------------------------------------------------------------------
--
-- lf popup floating window opener
--
----------------------------------------------------------------------------------

local utils = require('utils')

local M = {
	buf = nil
}

M.show = function()
	local selected_file = vim.fn.expand('%:p') -- the currently open filename
	M.buf = utils.create_floating_window()

	if not M.buf or not vim.api.nvim_buf_is_valid(M.buf) then
		print("Failed to create floating window")
		return
	end

	vim.cmd.startinsert() -- start in insert mode
	local fileManagerTmpFile = vim.fn.tempname()
	local fileManagerTmpDir = vim.fn.tempname()

	local process_cmd = 'lf -last-dir-path="' ..
			fileManagerTmpDir .. '" -selection-path="' .. fileManagerTmpFile .. '" '

	if selected_file ~= "" then
		process_cmd = process_cmd .. '"' .. vim.fn.shellescape(selected_file) .. '"'
	end

	local win = vim.api.nvim_get_current_win()

	-- launch lf process
	vim.fn.termopen(process_cmd, {
		on_exit = function() -- job_id, exit_code, event_type
			-- if window is a float, close the window
			if vim.api.nvim_win_get_config(win).relative ~= "" then
				vim.api.nvim_win_close(win, true)
			end

			-- if lf correctly left us a tempfile
			if vim.fn.filereadable(fileManagerTmpFile) then
				local contents = {}
				-- grab the entries that were selected (one per line)
				for line in io.lines(fileManagerTmpFile) do
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

return M
