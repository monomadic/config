return {
	"folke/which-key.nvim",

	init = function()
		require("keymaps").whichkey()
	end,

	config = function()
		require("which-key").setup {
			window = {
				border = "single",
				position = "top",
			},
			key_labels = {
				["<CR>"] = "",
				["<Tab>"] = "",
			},
			popup_mappings = {
				scroll_down = '<c-n>', -- binding to scroll down inside the popup
				scroll_up = '<c-p>', -- binding to scroll up inside the popup
			},
		}

		require("which-key").register(require('keymaps').whichkey(), {
			prefix = "<leader>",
		})

		vim.api.nvim_set_hl(0, "WhichKey", { fg = "#FFFF00" })
		vim.api.nvim_set_hl(0, "WhichKeyDesc", { bg = "none" })
		vim.api.nvim_set_hl(0, "WhichKeyFloat", { bg = "black" })
		vim.api.nvim_set_hl(0, "WhichKeyBorder", { bg = "black", fg = "#222222" })
		vim.api.nvim_set_hl(0, "WhichKeyGroup", { fg = "#00FFFF" })
		vim.api.nvim_set_hl(0, "WhichKeySeparator", { fg = "#888888" })
	end,
}
