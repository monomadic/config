-- https://github.com/stevearc/dressing.nvim
return {
	"stevearc/dressing.nvim",

	init = function()
		require("dressing").setup({
			select = { enabled = true, backend = { "nui", "builtin" } },
			input = { enabled = true, insert_only = true, start_in_insert = true, backend = { "nui", "builtin" } },
		})
	end
}
