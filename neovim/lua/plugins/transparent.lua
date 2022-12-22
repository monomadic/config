-- make background highlight groups transparent
return { 'xiyaowong/nvim-transparent',
	config = function()
		require("transparent").setup {
			enable = true,
			extra_groups = { "Normal", "NvimTreeNormal", "ModeMsg", "MsgArea" }, -- MsgArea is command line
		}
	end
}
