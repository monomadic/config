return {
	'folke/trouble.nvim',
	dependencies = "nvim-tree/nvim-web-devicons",
	config = function()
		local icons = require('icons').icons

		vim.keymap.set('n', '<leader>d', "<Cmd>Trouble workspace_diagnostics<CR>", { desc = "diagnostics" })
		vim.keymap.set('n', '<leader>d', "<Cmd>Trouble workspace_diagnostics<CR>", { desc = "diagnostics" })

		require("trouble").setup {
			signs = {
				-- icons / text used for a diagnostic
				error = icons.error,
				warning = icons.warn,
				hint = icons.hint,
				information = icons.info,
				other = icons.other,
			},
		}
	end
}
