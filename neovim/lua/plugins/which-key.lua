return {
	"folke/which-key.nvim",
	event = "VeryLazy",

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

		require("which-key").register({
			mode = { "n", "v" },
			["["] = { name = "+prev" },
			["]"] = { name = "+next" },
			["g"] = { name = "+goto" },
			["z"] = { name = "fold" },
			["<leader>"] = { name = "Leader Menu" },
			["<leader>,"] = { name = "Settings" },
			["<leader>B"] = { name = "Buffer" },
			["<leader>C"] = { name = "Config" },
			["<leader>D"] = { name = "Document" },
			["<leader>F"] = { name = "file" },
			["<leader>G"] = { name = "Git" },
			["<leader>I"] = { name = "Insert" },
			["<leader>L"] = { name = "LSP" },
			["<leader>N"] = { name = "New" },
			["<leader>O"] = { name = "Open" },
			["<leader>P"] = { name = "Peek" },
			["<leader>R"] = { name = "Run" },
			["<leader>S"] = { name = "Symbol" },
			["<leader>T"] = { name = "Templates" },
			["<leader>W"] = { name = "Wiki" },
			["<leader>g"] = { name = "goto" },
			["<leader>j"] = { name = "jump" },
			["<leader>l"] = { name = "list" },
			["<leader>w"] = { name = "workspace" },
		})

		vim.api.nvim_set_hl(0, "WhichKey", { fg = "#FFFF00" })
		vim.api.nvim_set_hl(0, "WhichKeyDesc", { bg = "none" })
		vim.api.nvim_set_hl(0, "WhichKeyFloat", { bg = "black" })
		vim.api.nvim_set_hl(0, "WhichKeyBorder", { bg = "black", fg = "#222222" })
		vim.api.nvim_set_hl(0, "WhichKeyGroup", { fg = "#00FFFF" })
		vim.api.nvim_set_hl(0, "WhichKeySeparator", { fg = "#888888" })
	end,
}
