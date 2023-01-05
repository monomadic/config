	-- side scrollbar with git and diagnostics support
	return {
		"petertriho/nvim-scrollbar",
		dependencies = "kevinhwang91/nvim-hlslens",

		config = function()
			require('scrollbar').setup{
				marks = {
					Cursor = {
						text = " "
					}
				}
			}

			require("scrollbar.handlers.search").setup {}
		end
	}
