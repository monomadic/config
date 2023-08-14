return {
	{
		"github/copilot.vim",
		cmd = "Copilot",
		build = ":Copilot auth",
		config = function()
			-- disable cmp
			require('cmp').setup.buffer({ enabled = false })
			require('copilot').setup({
				panel = {
					enabled = true,
					auto_refresh = false,
					keymap = {
						jump_prev = "[[",
						jump_next = "]]",
						accept = "<CR>",
						refresh = "gr",
						open = "<M-CR>"
					},
					layout = {
						position = "bottom", -- | top | left | right
						ratio = 0.4
					},
				},
				suggestion = {
					enabled = true,
					auto_trigger = false,
					debounce = 75,
					keymap = {
						accept = "<M-l>",
						accept_word = false,
						accept_line = false,
						next = "<C-]>",
						prev = "<C-[>",
						dismiss = "<M-]>",
					},
				},
				filetypes = {
					yaml = false,
					markdown = false,
					help = false,
					gitcommit = false,
					gitrebase = false,
					hgcommit = false,
					svn = false,
					cvs = false,
					["."] = false,
				},
				copilot_node_command = 'node', -- Node.js version must be > 16.x
				server_opts_overrides = {},
			})
			vim.g.copilot_no_tab_map = true
			vim.g.copilot_assume_mapped = true
			vim.g.copilot_tab_fallback = ""

			vim.keymap.set('n', '<leader>Ad', ':Copilot disable<CR>', { desc = "copilot unload", silent = true })
			vim.keymap.set('n', '<leader>Ar', ':Copilot reload<CR>', { desc = "copilot reload", silent = true })
			vim.keymap.set('n', '<leader>As', ':Copilot status<CR>', { desc = "copilot status", silent = true })

			-- vim.keymap.set("i", "<C-e>", 'copilot#Accept("<CR>")', { silent = true, expr = true })
			-- vim.keymap.set("n", "<C-Enter>", ':Copilot panel<CR>', { silent = true })
		end,
	},
}
