-- async formatting
-- https://github.com/lukas-reineke/lsp-format.nvim
return {
	'lukas-reineke/lsp-format.nvim',
	enabled = "false",
	config = function()
		require('lsp-format').setup()
	end
}
