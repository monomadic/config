-- convenience file operations (new, rename, etc)
return {
	"chrisgrieser/nvim-genghis",

	requires = {
		"stevearc/dressing.nvim",
		"rcarriga/nvim-notify"
	},

	config = function()
		local keymap = vim.keymap.set
		local genghis = require("genghis")
		keymap("n", "<leader>fc", genghis.copyFilepath, { desc = " copy path" })
		keymap("n", "<leader>fC", genghis.copyFilename, { desc = " copy filename" })
		keymap("n", "<leader>fr", genghis.renameFile, { desc = " rename" })
		keymap("n", "<leader>fn", genghis.createNewFile, { desc = " new" })
		keymap("n", "<leader>fd", genghis.duplicateFile, { desc = " duplicate" })
		keymap("n", "<leader>fx", genghis.chmodx, { desc = " chmod" })
		keymap("n", "<leader>ft", function() genghis.trashFile { trashLocation = "your/path" } end, { desc = "﬒ trash" }) -- default: '$HOME/.Trash'.
		keymap("x", "<leader>x", genghis.moveSelectionToNewFile)
	end
}
