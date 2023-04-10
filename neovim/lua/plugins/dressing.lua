return {
	"stevearc/dressing.nvim",

	init = function()
		require("dressing").setup({select = {backend = {"nui", "builtin"} }})
	end
}
