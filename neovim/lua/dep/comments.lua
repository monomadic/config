-- `gcc` line comment
-- `gcA` line comment at eol
-- `gc0` line comment at bol
-- `gco` line comment at line-open
-- `gbc` block comment

return {
	{ "numToStr/Comment.nvim",
		function()
			require('Comment').setup()
		end
	},
}
