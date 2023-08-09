return {
	{
		"github/copilot.vim",
		cmd = "Copilot",
		build = ":Copilot auth",
		config = function()
			vim.g.copilot_no_tab_map = true
			vim.g.copilot_assume_mapped = true
			vim.g.copilot_tab_fallback = ""

			vim.keymap.set('n', '<leader>Ad', ':Copilot disable<CR>', { desc = "copilot unload", silent = true })
			vim.keymap.set('n', '<leader>Ar', ':Copilot reload<CR>', { desc = "copilot reload", silent = true })
			vim.keymap.set('n', '<leader>As', ':Copilot status<CR>', { desc = "copilot status", silent = true })

			vim.keymap.set("i", "<C-J>", 'copilot#Accept("<CR>")', { silent = true, expr = true })
			vim.keymap.set("n", "<C-Enter>", ':Copilot panel<CR>', { silent = true })
		end,
	},

	-- { -- alternative client in lua
	-- 	"zbirenbaum/copilot.lua",
	-- 	cmd = "Copilot",
	-- 	event = "InsertEnter",
	-- 	config = function()
	-- 		require("copilot").setup({})
	-- 	end,
	-- },

	-- {
	-- 	"zbirenbaum/copilot-cmp",
	-- 	dependencies = {
	-- 		"github/copilot.vim"
	-- 	},
	-- 	config = function()
	-- 		require("copilot_cmp").setup()
	-- 	end
	-- }
}


-- cell 1: github.com/copilot.vim
-- cell 2: github.com/zbirenbaum/copilot.lua
-- cell 3: github.com/zbirenbaum/copilot-cmp


-- cell 1: github.com/copilot.vim
-- cell 2: github.com/zbirenbaum/copilot.lua
-- cell 3: github.com/zbirenbaum/copilot-cmp


-- cell 1: github.com/copilot.vim
-- cell 2: github.com/zbirenbaum/copilot.lua
-- cell 3: github.com/zbirenbaum/copilot-cmp
