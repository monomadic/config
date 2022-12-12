return {
		'folke/which-key.nvim',
		config = function()
			require("which-key").setup {
				window = {
					border = "double",
					position = "top",
				}
			}
			print("WhichKey loaded.")
		end
}
