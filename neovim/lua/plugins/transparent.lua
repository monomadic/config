-- make background highlight groups transparent
return { 'xiyaowong/nvim-transparent',
	config = function()
		require("transparent").setup {
			enable = true,
			extra_groups = { "NvimTreeNormal", "ModeMsg" },
		}
	end
}
