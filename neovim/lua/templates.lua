local M = {}

M.new_file_from_template = function()
	local pickers = require("telescope.pickers")
	local sorters = require("telescope.sorters")
	local finders = require("telescope.finders")

	local filetype = vim.bo.filetype
	if filetype and filetype ~= '' then
		filetype = filetype .. "/"
	end

	local template_dir = vim.fn.expand("~/.config/nvim/templates/" .. filetype)
	local current_directory = vim.fn.expand('%:p:h')

	pickers.new {
		results_title = "Insert Template",
		finder = finders.new_oneshot_job({ "fd", "--type", "symlink", ".", template_dir }),
		sorter = sorters.get_generic_fuzzy_sorter(),
		previewer = require 'telescope.previewers'.vim_buffer_cat.new({}),
		attach_mappings = function(bufnr, map)
			local actions = require 'telescope.actions'
			map("i", "<CR>", function()
				local selected_file = require('telescope.actions.state').get_selected_entry()[1]
				if not selected_file then
					return
				end

				vim.ui.input({ prompt = 'Save template as: ', default = current_directory .. '/', completion = 'dir' },
					function(destination_file)
						if not destination_file then
							return
						end
						actions.close(bufnr)  -- close telescope window
						vim.cmd.vnew()        -- new buffer (vertical)
						vim.cmd.read(selected_file) -- read template into buffer
						-- If the first line is empty, delete it
						vim.api.nvim_command("if getline(1) == '' | execute '1delete' | endif")
						vim.cmd.saveas(destination_file) -- save file
						vim.cmd.stopinsert()       -- enter normal mode
					end)
			end)
			return true
		end
	}:find()
end

return M
