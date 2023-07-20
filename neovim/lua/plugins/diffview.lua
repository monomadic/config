-- git merge conflict resolution
return {
	'sindrets/diffview.nvim',
	cmd = { "DiffviewOpen", "DiffviewFocusFiles" },
	config = function()
		local key = require('utils').key

		key('n', '<leader>gr', ':DiffviewOpen<CR>', 'resolve conflicts')

		require("diffview").setup({})
	end
}
