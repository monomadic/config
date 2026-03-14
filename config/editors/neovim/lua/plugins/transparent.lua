-- make background highlight groups transparent
return {
	'xiyaowong/nvim-transparent',

	config = function()
		vim.g.transparent_enabled = true

		require("transparent").setup {
			extra_groups = { "Normal", "NvimTreeNormal", "ModeMsg", "MsgArea" }, -- MsgArea is command line
		}
	end
}
