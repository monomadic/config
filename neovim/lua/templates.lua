--
-- template picker using telescope
--
local M = {}

M.new_file_from_template = function()
	local utils = require('utils')
	local pickers = require("telescope.pickers")
	local sorters = require("telescope.sorters")
	local finders = require("telescope.finders")
	local Path = require("plenary.path")
	local entry_display = require("telescope.pickers.entry_display")
	local devicons = require("nvim-web-devicons")

	local filetype = vim.bo.filetype
	if filetype and filetype ~= '' then
		filetype = filetype .. "/"
	end

	local function get_files_recursive(directory)
		local files = {}
		local p = io.popen('fd --type symlink . "' .. directory .. '"')
		if p then
			for entry in p:lines() do
				local relative_path = entry
				table.insert(files, relative_path)
			end
		end
		return files
	end

	local template_dir = vim.fn.expand("~/.config/nvim/templates/")
	local current_directory = vim.fn.expand('%:p:h')

	pickers.new {
		prompt_title = "Insert Template",
		finder = finders.new_table {
			results = get_files_recursive(template_dir),
			entry_maker = function(entry)
				local file_path = Path:new(entry)
				local icon, icon_highlight = devicons.get_icon(file_path.filename, file_path.filetype)
				local displayer = entry_display.create({
					separator = " ",
					items = {
						{ width = 1 },
						{ remaining = true },
					}
				})
				return {
					display = function()
						local pretty_path = file_path.filename:gsub("^" .. template_dir, "");
						return displayer({ { icon, icon_highlight }, { pretty_path} })
					end,
					ordinal = file_path.filename,
					value = entry,
					filename = file_path.filename,
				}
			end
		},

		sorter = sorters.get_generic_fuzzy_sorter(),

		previewer = require 'telescope.previewers'.vim_buffer_cat.new({}),

		attach_mappings = function(bufnr, map)
			local actions = require 'telescope.actions'
			map("i", "<CR>", function()
				local selected_file = require('telescope.actions.state').get_selected_entry().value
				if not selected_file then
					return
				end
				local new_file_path = current_directory .. '/' .. selected_file:match("([^/]+)$");
				vim.ui.input({ prompt = 'Save template as: ', default = new_file_path, completion = 'dir' },
					function(destination_file)
						if not destination_file then
							return
						end
						if utils.file_exists(destination_file) then
							print("File already exists at location: " .. destination_file)
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
