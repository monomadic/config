-- code outline window
return {
	'stevearc/aerial.nvim',

	config = function()
		require('aerial').setup {
			-- backends = { "treesitter", "lsp", "markdown", "man" },
			layout = {
				width = 26,
				height = 40,
			},
			autojump = true,
		}
	end
}
