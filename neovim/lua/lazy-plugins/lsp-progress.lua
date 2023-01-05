-- show lsp progress
return {
	'j-hui/fidget.nvim',
	config = function()
		require("fidget").setup {
			text = { spinner = "dots" },
			align = { bottom = false },
			window = { border = "none" }
		}
	end
}
