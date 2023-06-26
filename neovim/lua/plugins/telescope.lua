return {
	'nvim-telescope/telescope.nvim',
	as = 'telescope',
	dependencies = { 'nvim-lua/plenary.nvim' },

	config = function()
		require('telescope').setup {
			defaults = {
				vimgrep_arguments = {
					"rg",
					"-L",
					"--color=never",
					"--no-heading",
					"--with-filename",
					"--line-number",
					"--column",
					"--smart-case",
				},
				prompt_prefix = "   ",
				selection_caret = "  ",
				entry_prefix = "  ",
				initial_mode = "insert",
				selection_strategy = "reset",
				sorting_strategy = "ascending",
				layout_strategy = "horizontal",
				prompt_title = "",
				results_title = "",
				layout_config = {
					horizontal = {
						prompt_position = "top",
						preview_width = 0.55,
						results_width = 0.8,
					},
					vertical = {
						mirror = false,
					},
					width = 0.87,
					height = 0.80,
					preview_cutoff = 120,
				},
				file_sorter = require("telescope.sorters").get_fzy_sorter,
				set_env = { ["COLORTERM"] = "truecolor" },
				file_ignore_patterns = { ".git/", ".cache", "%.o", "%.a", "%.out", "%.class", "%.pdf", "%.mkv", "%.mp4", "%.zip",
					"*.lock", "node_modules", "target" },
				generic_sorter = require("telescope.sorters").get_generic_fuzzy_sorter,
				path_display = { "truncate" },
				-- winblend = 10,
				-- border = false,
				mappings = {
					i = {
						["<Esc>"] = "close",
						["<Tab>"] = "close",
						["<C-l>"] = require("telescope.actions.layout").toggle_preview,
						["<C-u>"] = false,
					},
				},
				extensions_list = { "themes", "terms" },
				extensions = {
					fzf = {
						fuzzy = true,             -- false will only do exact matching
						override_generic_sorter = true, -- override the generic sorter
						override_file_sorter = true, -- override the file sorter
						case_mode = "smart_case", -- or "ignore_case" or "respect_case"
						-- the default case_mode is "smart_case"
					}
				}
			},
		}

		-- set default lsp providers to use telescope
		local telescope = require('telescope.builtin')
		vim.lsp.handlers["textDocument/implementation"] = telescope.lsp_implementations
		vim.lsp.handlers["workspace/symbol"] = telescope.lsp_workspace_symbols

		vim.api.nvim_create_autocmd('LspAttach', {
			callback = function(args)
				local client = vim.lsp.get_client_by_id(args.data.client_id)
				if client.server_capabilities.referencesProvider then
					vim.lsp.handlers["textDocument/references"] = telescope.lsp_references
				end
			end,
		})

		vim.keymap.set("n", "tP", function()
			local previewers = require("telescope.previewers")
			local pickers = require("telescope.pickers")
			local sorters = require("telescope.sorters")
			local finders = require("telescope.finders")

			pickers.new {
				results_title = "Resources",
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
		end)
	end,

	init = function()
		require("colors").telescope()
		require("keymaps").telescope()
	end,
}
