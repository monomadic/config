local utils = require 'utils'

local M = {}

M.new_prompt = function(default_path)
	vim.ui.input({ prompt = 'New File: ', default = default_path, completion = 'dir' },
		function(destination_file)
			if not destination_file then
				return
			end
			if utils.file_exists(destination_file) then
				print("File already exists at location: " .. destination_file)
				return
			end
			vim.cmd.vnew()                -- new buffer (vertical)
			vim.cmd.saveas(destination_file) -- save file
			-- vim.cmd.stopinsert()             -- enter normal mode
		end)
end

-- M.rename = function()
-- 	vim.ui.input({ prompt = "Rename: " }, function()
-- 		cmd.edit(newFilePath)
-- 		bwipeout("#")
-- 		vim.notify(("Renamed %q as %q."):format(oldName, newName))
-- 	end)
-- end

return M
