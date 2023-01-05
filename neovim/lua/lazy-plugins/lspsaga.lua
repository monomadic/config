-- better lsp ui
return {
	"glepnir/lspsaga.nvim",

	dependencies = { 'neovim/nvim-lspconfig' },

	ft = { 'rust', 'typescript', 'javascript', 'lua' },

	config = function()
		local lsp_saga = require('lspsaga')

		vim.keymap.set("n", "<leader>Sf", "<cmd>Lspsaga lsp_finder<CR>", { desc = "symbol finder (saga)" })
		vim.keymap.set("n", "<leader>Sa", "<cmd>Lspsaga code_action<CR>", { desc = "code-actions (saga)" })
		vim.keymap.set("n", "<leader>Sr", "<cmd>Lspsaga rename<CR>", { desc = "rename (saga)" })
		vim.keymap.set("n", "<leader>Pd", "<cmd>Lspsaga peek_definition<CR>", { desc = "peek definition (saga)" })
		vim.keymap.set("n", 'K', '<cmd>Lspsaga hover_doc<CR>')
		vim.keymap.set("n", "<leader>Do", '<cmd>Lspsaga outline<CR>', { silent = true, desc = "outline (saga)" })
		-- map('n', ']d', ":Lspsaga diagnostic_jump_next<CR>", { desc = "next diagnostic" })
		-- map('n', '[d', ":Lspsaga diagnostic_jump_prev<CR>", { desc = "prev diagnostic" })

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
