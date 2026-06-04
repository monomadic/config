----------------------------------------------------------------------------------
--
-- joshuto popup floating window opener
--
----------------------------------------------------------------------------------

local utils = require('utils')

local M = {
	buf = nil
}

M.show = function()
	local selected_file = vim.fn.expand('%:p') -- the currently open filename
	M.buf = utils.create_floating_window()

	vim.cmd.startinsert() -- start in insert mode
	local chooser_tmpfile = vim.fn.tempname()

	local process_cmd = 'joshuto --file-chooser --output-file ' .. chooser_tmpfile

	if selected_file ~= "" then
		process_cmd = process_cmd .. '"' .. selected_file .. '"'
	end

	local win = vim.api.nvim_get_current_win()

		-- launch file chooser process
	vim.fn.termopen(process_cmd, {
		on_exit = function() -- job_id, exit_code, event_type
			-- if window is a float, close the window
			if vim.api.nvim_win_get_config(win).zindex then
				vim.api.nvim_win_close(win, true)
			end

			-- if the chooser correctly left us a tempfile
			if vim.loop.fs_stat(chooser_tmpfile) then
				local contents = {}
				-- grab the entries that were selected (one per line)
				for line in io.lines(chooser_tmpfile) do
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
