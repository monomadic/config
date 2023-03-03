-- convenience file operations (new, rename, etc)
return {
	"chrisgrieser/nvim-genghis",
	dependencies = {
		"stevearc/dressing.nvim",
		"rcarriga/nvim-notify"
	},
	config = function()
		local keymap = vim.keymap.set
		local genghis = require("genghis")
		keymap("n", "<leader>Fc", genghis.copyFilepath, { desc = " copy path" })
		keymap("n", "<leader>FC", genghis.copyFilename, { desc = " copy filename" })
		keymap("n", "<leader>Fr", genghis.renameFile, { desc = " rename" })
		keymap("n", "<leader>Fn", genghis.createNewFile, { desc = " new" })
		keymap("n", "<leader>Nf", genghis.createNewFile, { desc = " file" })
		keymap("n", "<leader>Fd", genghis.duplicateFile, { desc = " duplicate" })
		keymap("n", "<leader>Fx", genghis.chmodx, { desc = " chmod" })
		keymap("n", "<leader>Ft", function() genghis.trashFile { trashLocation = "your/path" } end, { desc = "﬒ trash" }) -- default: '$HOME/.Trash'.
		keymap("x", "<leader>x", genghis.moveSelectionToNewFile)
	end
}
