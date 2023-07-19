local builtin = require 'telescope.builtin'
local utils = require 'utils'

local M = {}

M.open_with_extension = function(ext)
	builtin.find_files {
		path_display = { "truncate" },
		search_file = string.format("*.%s", ext),
	}
end

M.open_files = function()
	builtin.find_files {
		path_display = { "truncate" },
		hidden = true,
	}
end

function M.open_same_filetype()
	-- Get the current filetype
	local filetype = vim.api.nvim_buf_get_option(0, 'filetype')

	if filetype then
		builtin.find_files {
			path_display = { "truncate" },
			cwd = vim.fn.expand("%:p:h"),
			filetype = filetype,
		}
	else
		builtin.find_files {
			path_display = { "truncate" },
			cwd = vim.fn.expand("%:p:h"),
		}
	end
end

M.wiki_open_page = function()
	builtin.find_files({ cwd = vim.g.wiki_directory })
end

M.wiki_search = function()
	builtin.live_grep({ cwd = vim.g.wiki_directory })
end

M.open_template = function()
	builtin.find_files({ cwd = vim.g.template_directory, follow = true })
end

M.open_test = function()
	builtin.find_files {
		path_display = { "truncate" },
		search_file = string.format("*.%s", utils.current_file_extension()),
		cwd = "tests/",
		follow = true,
	}
end

M.open_config_file = function()
	utils.select_file_at(vim.g.neovim_config_directory)
end

function M.git_commits()
	builtin.git_commits()
end

M.git_status = function()
	builtin.git_status()
end

M.git_branches = function()
	builtin.git_branches()
end

M.lsp_workspace_symbols = function()
	builtin.lsp_workspace_symbols { path_display = "hidden", prompt_title = "", preview_title = "" }
end

M.lsp_document_functions = function()
	builtin.lsp_document_symbols {
		symbols = "function",
		prompt_title = "",
		preview_title = "",
		borderchars = { " ", " ", " ", " ", " ", " ", " ", " " }
	}
end

M.lsp_document_enums = function()
	builtin.lsp_document_symbols { symbols = "enum" }
end

M.list_keymaps = function()
	builtin.keymaps()
end

M.insert_template = function()
	local pickers = require("telescope.pickers")
	local sorters = require("telescope.sorters")
	local finders = require("telescope.finders")

	local filetype = vim.bo.filetype
	-- if current buffer has no filetype
	if filetype and filetype ~= '' then
		filetype = filetype .. "/"
	end

	local template_dir = vim.fn.expand(vim.g.template_directory .. filetype)
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
