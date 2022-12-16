return { 'nvim-telescope/telescope.nvim',
	as = "telescope",
	config = function()
		require('telescope').setup {
			defaults = {
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
				file_sorter = require("telescope.sorters").get_fuzzy_file,
				set_env = { ["COLORTERM"] = "truecolor" },
				file_ignore_patterns = { ".git/", ".cache", "%.o", "%.a", "%.out", "%.class", "%.pdf", "%.mkv", "%.mp4", "%.zip",
					"*.lock", "node_modules", "target" },
				generic_sorter = require("telescope.sorters").get_generic_fuzzy_sorter,
				path_display = { "truncate" },
				winblend = 10,
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
			},
		}

		-- telescope keymaps
		vim.keymap.set("n", "go", function()
			require('telescope.builtin').find_files()
		end, { desc = "open" })

		-- vim.keymap.set("n", '<C-b>', '<cmd>Telescope buffers<cr>', {desc = "buffers…" })
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

		local prompt_bg = "#000000"
		local results_bg = "#000000"
		local preview_bg = "#000000"

		vim.api.nvim_set_hl(0, "TelescopeBorder", { fg = prompt_bg, bg = prompt_bg })

		vim.api.nvim_set_hl(0, "TelescopePromptBorder", { fg = prompt_bg, bg = prompt_bg })
		vim.api.nvim_set_hl(0, "TelescopePromptNormal", { fg = "White", bg = prompt_bg })
		vim.api.nvim_set_hl(0, "TelescopePromptPrefix", { fg = "White" }) -- the icon
		vim.api.nvim_set_hl(0, "TelescopePromptTitle", { fg = prompt_bg, bg = prompt_bg })

		vim.api.nvim_set_hl(0, "TelescopePreviewTitle", { fg = preview_bg, bg = preview_bg })
		vim.api.nvim_set_hl(0, "TelescopePreviewBorder", { fg = preview_bg, bg = preview_bg })
		vim.api.nvim_set_hl(0, "TelescopePreviewNormal", { bg = preview_bg })

		vim.api.nvim_set_hl(0, "TelescopeResultsTitle", { fg = results_bg, bg = results_bg })
		vim.api.nvim_set_hl(0, "TelescopeResultsBorder", { fg = results_bg, bg = results_bg })
		vim.api.nvim_set_hl(0, "TelescopeResultsNormal", { bg = results_bg })
	end }
