local M = {}

M.open_with_extension = function(ext)
	require('telescope.builtin').find_files({ search_file = ext })
end

M.insert_template = function()
	local previewers = require("telescope.previewers")
	local pickers = require("telescope.pickers")
	local sorters = require("telescope.sorters")
	local finders = require("telescope.finders")
	local filetype = vim.bo.filetype
	local template_dir = vim.fn.expand("~/.config/nvim/templates/" .. filetype .. "/")

	pickers.new {
		results_title = "Insert Template",
		finder = finders.new_oneshot_job({ "ls", template_dir }),
		--finder = finders.new_oneshot_job({ "fd", ".", template_dir }),
		sorter = sorters.get_generic_fuzzy_sorter(),
		-- previewer = require'telescope.previewers'.vim_buffer_cat.new({}),
		previewer = previewers.new_buffer_previewer {
			define_preview = function(self, entry, status)
				return require('telescope.previewers.utils').job_maker(
					{ "bat" },
					self.state.bufnr,
					{
						cwd = template_dir,
						callback = function(bufnr, content)
							if content ~= nil then
								--require('telescope.previewers.utils').(bufnr, 'terraform')
								require('telescope.previewers.utils').regex_highlighter(bufnr, 'terraform')
							end
						end,
					})
			end
		},
		attach_mappings = function(bufnr, map)
			local actions = require 'telescope.actions'
			map("i", "<CR>", function()
				local selected_file = require('telescope.actions.state').get_selected_entry()
				local file = vim.fn.expand(template_dir .. selected_file[1])
				-- close telescope window
				actions.close(bufnr)
				-- insert file at current position
				vim.cmd.read(file)
			end)
			return true
		end
	}:find()
end

-- M.insert_template()

return M
