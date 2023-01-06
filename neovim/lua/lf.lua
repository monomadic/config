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

	vim.cmd.startinsert() -- start in insert mode
	local lf_tmpfile = vim.fn.tempname()
	local lf_tmpdir = vim.fn.tempname()

	local process_cmd = 'lf -last-dir-path="' ..
			lf_tmpdir .. '" -selection-path="' .. lf_tmpfile .. '" '

	if selected_file ~= "" then
		process_cmd = process_cmd .. '"' .. selected_file .. '"'
	end

	local win = vim.api.nvim_get_current_win()

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

return M
