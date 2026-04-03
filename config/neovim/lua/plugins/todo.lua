-- highlight TODO comments
return {
	"folke/todo-comments.nvim",
	dependencies = "nvim-lua/plenary.nvim",
	config = function()
		require('todo-comments').setup {}
		-- local Search = require("todo-comments.search")
		-- Search.search(function(results)
		-- 	print(vim.inspect(results))
		-- end)
	end
}
