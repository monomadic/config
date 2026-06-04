----------------------------------------------------------------------------------
--
-- yazi popup floating window opener
--
----------------------------------------------------------------------------------

local utils = require('utils')

local M = {
	buf = nil
}

local function close_float(win)
	if win and vim.api.nvim_win_is_valid(win) then
		local config = vim.api.nvim_win_get_config(win)
		if config.relative ~= "" or config.zindex then
			vim.api.nvim_win_close(win, true)
		end
	end
end

M.show = function()
	local selected_file = vim.fn.expand('%:p')
	local start_entry = selected_file ~= "" and selected_file or vim.fn.getcwd()
	M.buf = utils.create_floating_window()

	if not M.buf or not vim.api.nvim_buf_is_valid(M.buf) then
		print("Failed to create floating window")
		return
	end

	vim.cmd.startinsert()
	local chooser_file = vim.fn.tempname()
	local cwd_file = vim.fn.tempname()

	local process_cmd = table.concat({
		"yazi",
		vim.fn.shellescape(start_entry),
		vim.fn.shellescape("--chooser-file=" .. chooser_file),
		vim.fn.shellescape("--cwd-file=" .. cwd_file),
	}, " ")

	local win = vim.api.nvim_get_current_win()

	vim.fn.termopen(process_cmd, {
		on_exit = function()
			close_float(win)

			if vim.fn.filereadable(cwd_file) == 1 then
				local cwd = vim.fn.readfile(cwd_file)[1]
				if cwd and cwd ~= "" then
					vim.cmd("lcd " .. vim.fn.fnameescape(cwd))
				end
			end

			if vim.fn.filereadable(chooser_file) == 1 then
				for _, fname in ipairs(vim.fn.readfile(chooser_file)) do
					if fname ~= "" then
						vim.cmd("edit " .. vim.fn.fnameescape(fname))
					end
				end
			end

			vim.fn.delete(chooser_file)
			vim.fn.delete(cwd_file)
		end,
	})
end

return M
