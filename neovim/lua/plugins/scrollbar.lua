	-- side scrollbar with git and diagnostics support
	return { "petertriho/nvim-scrollbar",
		config = function()
			require('scrollbar').setup{
				marks = {
					Cursor = {
						text = " "
					}
				}
			}
		end
	}
