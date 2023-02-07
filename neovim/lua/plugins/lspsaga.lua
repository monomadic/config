-- better lsp ui
return {
	"glepnir/lspsaga.nvim",
	event = "BufRead",
	ft = { 'rust', 'typescript', 'javascript', 'lua' },
	config = function()
		vim.keymap.set("n", "<leader>Sf", "<cmd>Lspsaga lsp_finder<CR>", { desc = "symbol finder (saga)" })
		vim.keymap.set("n", "<leader>a", "<cmd>Lspsaga code_action<CR>", { desc = "code-actions (saga)" })
		vim.keymap.set("n", "<leader>Sa", "<cmd>Lspsaga code_action<CR>", { desc = "code-actions (saga)" })
		vim.keymap.set("n", "<leader>Sr", "<cmd>Lspsaga rename<CR>", { desc = "rename (saga)" })
		vim.keymap.set("n", "<leader>Pd", "<cmd>Lspsaga peek_definition<CR>", { desc = "peek definition (saga)" })
		vim.keymap.set("n", 'K', '<cmd>Lspsaga hover_doc<CR>')
		vim.keymap.set("n", "<leader>Do", '<cmd>Lspsaga outline<CR>', { silent = true, desc = "outline (saga)" })
		-- map('n', ']d', ":Lspsaga diagnostic_jump_next<CR>", { desc = "next diagnostic" })
		-- map('n', '[d', ":Lspsaga diagnostic_jump_prev<CR>", { desc = "prev diagnostic" })

		vim.keymap.set("n", "<leader>Sf", "<cmd>Lspsaga lsp_finder<CR>", { desc = "symbol finder (saga)" })

		vim.keymap.set('n', ']d', "<Cmd>Lspsaga diagnostic_jump_next<CR>", { desc = "Next diagnostic" })
		vim.keymap.set('n', '[d', "<Cmd>Lspsaga diagnostic_jump_prev<CR>", { desc = "Previous diagnostic" })

		vim.api.nvim_set_hl(0, "SagaBorder", { fg = "#1e1d2d", bg = "#1e1d2d" })
		vim.api.nvim_set_hl(0, "DiagnosticTitleSymbol", { fg = "#1e1d2d", bg = "#1e1d2d" })

		require("lspsaga").setup({
			ui = {
				border = "single",
				code_action = '',
			},
			show_outline = {
				saga_winblend = 30,
				jump_key = '<CR>',
			},
			diagnostic = {
				show_code_action = true,
				show_source = true,
				jump_num_shortcut = true,
				--1 is max
				max_width = 0.7,
				custom_fix = "Actions",
				custom_msg = "Message",
				text_hl_follow = false,
				border_follow = true,
				keys = {
					exec_action = "o",
					quit = "q",
					go_action = "g"
				},
			},
			lightbulb = {
				enable = false,
			},
			symbol_in_winbar = {
				enable = false,
				separator = " / ",
				hide_keyword = true,
				show_file = true,
				folder_level = 2,
				respect_root = true,
				color_mode = false, -- only icons have color
			},
			code_action_lightbulb = {
				enable = false,
			},
			-- symbol_in_winbar = {
			-- 	enable = true,
			-- }
		})
	end,
	dependencies = { { "nvim-tree/nvim-web-devicons" } }
}
