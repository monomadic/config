local M = {}

M.insert_template = function()
	local previewers = require("telescope.previewers")
	local pickers = require("telescope.pickers")
	local sorters = require("telescope.sorters")
	local finders = require("telescope.finders")

	pickers.new {
		results_title = "Insert Template",
		-- Run an external command and show the results in the finder window
		finder = finders.new_oneshot_job({ "fd" }),
		sorter = sorters.get_fuzzy_file(),
		previewer = previewers.new_buffer_previewer {
			define_preview = function(self, entry, status)
				-- Execute another command using the highlighted entry
				return require('telescope.previewers.utils').job_maker(
					{ "bat" },
					self.state.bufnr,
					{
						callback = function(bufnr, content)
							if content ~= nil then
								require('telescope.previewers.utils').regex_highlighter(bufnr, 'terraform')
							end
						end,
					})
			end
		},
	}:find()
end

return M
