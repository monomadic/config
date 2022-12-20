-- TODO: https://github.com/nvim-telescope/telescope.nvim/wiki/Extensions
-- TODO: colors for input box and select
return {
	{ 'nvim-telescope/telescope-fzf-native.nvim',
		run = 'cmake -S. -Bbuild -DCMAKE_BUILD_TYPE=Release && cmake --build build --config Release && cmake --install build --prefix build' },
	{ 'nvim-telescope/telescope.nvim',
		as = "telescope",
		requires = { 'nvim-lua/plenary.nvim' },
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
					prompt_prefix = "  ",
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
							fuzzy = true, -- false will only do exact matching
							override_generic_sorter = true, -- override the generic sorter
							override_file_sorter = true, -- override the file sorter
							case_mode = "smart_case", -- or "ignore_case" or "respect_case"
							-- the default case_mode is "smart_case"
						}
					}
				},
			}

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

			-- telescope keymaps
			vim.keymap.set("n", "go", function()
				require('telescope.builtin').find_files()
			end, { desc = "open" })

			vim.keymap.set("n", '<leader>b', '<cmd>Telescope buffers<cr>', { desc = "buffers…" })

			-- vim.keymap.set("n", '<leader>f', function()
			-- 	require('telescope.builtin').find_files { path_display = { "truncate" }, prompt_title = "", preview_title = "" }
			-- end)

			vim.keymap.set("n", '<leader>o', OpenFiles, { desc = "open…" })

			vim.keymap.set("n", 'to', '<cmd>Telescope oldfiles<cr>')
			-- vim.keymap.set("n", '<leader>g', '<cmd>Telescope live_grep<cr>')
			vim.keymap.set('n', 'tgb', '<Cmd>Telescope git_branches<cr>')
			vim.keymap.set('n', 'tgc', '<Cmd>Telescope git_bcommits<cr>')
			vim.keymap.set('n', 'tgd', '<Cmd>Telescope git_status<cr>')
			vim.keymap.set('n', 'tk', '<Cmd>Telescope keymaps<cr>')
			vim.keymap.set('n', 'tld', '<Cmd>Telescope lsp_definitions<cr>')
			vim.keymap.set('n', 'tli', '<Cmd>Telescope lsp_implementations<cr>')
			-- vim.keymap.set('n', '<leader>S', '<Cmd>Telescope lsp_document_symbols<cr>', { desc = "DoCuMeNt" })
			vim.keymap.set('n', 'tlw', function()
				require('telescope.builtin').lsp_workspace_symbols { path_display = "hidden", prompt_title = "", preview_title = "" }
			end)
			vim.keymap.set('n', 'tlf', function()
				require('telescope.builtin').lsp_document_symbols { symbols = "function", prompt_title = "", preview_title = "",
					borderchars = { " ", " ", " ", " ", " ", " ", " ", " " } }
			end)

			vim.keymap.set('n', 'tr', '<Cmd>Telescope resume<cr>')
			vim.keymap.set('n', 'tt', '<Cmd>TodoTelescope<cr>')

			-- vim.keymap.set("n", "ts", function()
			-- 	require("luasnip.loaders.from_snipmate").lazy_load()
			-- 	require('telescope').load_extension('luasnip')
			-- 	vim.api.nvim_command('Telescope luasnip')
			-- end)



		end },

		setup = function()
			require("keymaps").telescope()
			require("colors").telescope()
		end,
}
