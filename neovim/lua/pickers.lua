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

M.open_same_filetype = function()
	builtin.find_files {
		path_display = { "truncate" },
		search_file = string.format("*.%s", vim.bo.filetype),
	}
end

M.wiki_open_page = function()
	builtin.find_files({ cwd = "~/wiki/" })
end

M.wiki_search = function()
	builtin.live_grep({ cwd = "~/wiki/" })
end

M.open_template = function()
	builtin.find_files({ cwd = "~/.config/nvim/templates/", follow = true })
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
	utils.select_file_at("~/config/neovim/")
end

M.git_commits = function()
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
	local previewers = require("telescope.previewers")
	local pickers = require("telescope.pickers")
	local sorters = require("telescope.sorters")
	local finders = require("telescope.finders")
	local filetype = vim.bo.filetype
	local template_dir = vim.fn.expand("~/.config/nvim/templates/" .. filetype .. "/")

	pickers.new {
		results_title = "Insert Template",
		finder = finders.new_oneshot_job({ "exa", template_dir }),
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
				-- new document
				vim.cmd.new()
				vim.cmd.saveas(selected_file[1])
				-- insert file at current position
				vim.cmd.read(file)
			end)
			return true
		end
	}:find()
end

 -- M.insert_template()

return M
