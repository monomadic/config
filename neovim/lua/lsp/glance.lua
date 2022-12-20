-- lsp naviation
-- https://github.com/DNLHC/glance.nvim
return {
	'dnlhc/glance.nvim',
	config = function()
		require('glance').setup({
			winbar = { enable = true }
		})
		require('keymaps').glance()
	end,
}
