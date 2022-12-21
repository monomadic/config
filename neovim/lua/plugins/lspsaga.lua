-- better lsp ui
return { "glepnir/lspsaga.nvim",
	-- requires = { 'neovim/nvim-lspconfig' },
	ft = { 'rust', 'typescript', 'javascript', 'lua' },
	config = function()
		local lsp_saga = require('lspsaga')

		vim.keymap.set("n", "<leader>sf", "<cmd>Lspsaga lsp_finder<CR>", { desc = "symbol finder (saga)" })
		vim.keymap.set("n", "<leader>sa", "<cmd>Lspsaga code_action<CR>", { desc = "code-actions (saga)" })
		vim.keymap.set("n", "<leader>sr", "<cmd>Lspsaga rename<CR>", { desc = "rename (saga)" })
		vim.keymap.set("n", "<leader>pd", "<cmd>Lspsaga peek_definition<CR>", { desc = "peek definition (saga)" })
		vim.keymap.set("n", 'K', '<cmd>Lspsaga hover_doc<CR>')
		vim.keymap.set("n", "<leader>Do", '<cmd>Lspsaga outline<CR>', { silent = true, desc = "outline (saga)" })
		vim.keymap.set("n", ']d', '<cmd>Lspsaga diagnostic_jump_next<cr>')
		vim.keymap.set("n", '[d', '<cmd>Lspsaga diagnostic_jump_prev<cr>')

		lsp_saga.init_lsp_saga {
			-- border_style = "none",
			show_outline = {
				saga_winblend = 30,
				jump_key = '<CR>',
			},
			code_action_icon = '',
			code_action_lightbulb = {
				enable = false,
			},
			-- symbol_in_winbar = {
			-- 	enable = true,
			-- }
		}
	end }
