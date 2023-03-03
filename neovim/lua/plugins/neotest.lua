return {
	"nvim-neotest/neotest",
	dependencies = {
		"nvim-lua/plenary.nvim",
		"nvim-treesitter/nvim-treesitter",
		"antoinemadec/FixCursorHold.nvim",
		"rouge8/neotest-rust",
	},
	config = function()
		local neotest = require("neotest")

		neotest.setup {
			adapters = {
				require("neotest-rust") {
					args = { "--no-capture" },
				}
			}
		}

		vim.keymap.set('n', '<leader>t', function()
			neotest.run.run(vim.fn.expand("%"))
		end, { desc = "test" })

		vim.keymap.set('n', '<leader>To', function()
			neotest.output.open()
		end, { desc = "open test" })
	end
}
