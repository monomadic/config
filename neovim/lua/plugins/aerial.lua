return {
	'stevearc/aerial.nvim',
	config = function()
		require('aerial').setup {
			backends = { "lsp", "treesitter", "lsp", "markdown", "man" },
			layout = {
				width = 26,
			}
		}

		local keymap = vim.keymap.set
		-- keymap('n', '<C-]>', ':AerialNext<CR>', { desc = "Next aerial" })
		-- keymap('n', '<C-[>', ':AerialPrev<CR>', { desc = "Prev aerial" })
		keymap('n', '\\', ':AerialToggle<CR>', { desc = "Prev aerial" })
	end
}
