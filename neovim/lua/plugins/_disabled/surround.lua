-- surround completion
-- use { "numToStr/Surround.nvim" }
-- maps <C-s>
return {
	"ur4ltz/surround.nvim",

	config = function()
		require "surround".setup { mappings_style = "sandwich" }
	end
}
