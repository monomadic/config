-- surround completion
-- use { "numToStr/Surround.nvim" }
return {
	"ur4ltz/surround.nvim",

	config = function()
		require "surround".setup { mappings_style = "sandwich" }
	end
}
