-- show lsp progress
return {
	'j-hui/fidget.nvim',
	config = function()
		require("fidget").setup {
			text = {
				spinner = "dots",
				commenced = "",
				completed = "",
			},
			align = { bottom = false },
			window = {
				border = "none",
				relative = "win"
			},
		}
	end
}
