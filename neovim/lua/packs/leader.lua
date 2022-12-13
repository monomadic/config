return {
		'folke/which-key.nvim',
		config = function()
			require("which-key").setup {
				window = {
					border = "double",
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

			require("which-key").register({
				b = "buffers...",
				f = {
					name = "find",
				},
				d = {
					name = "document",
					f = { function() vim.lsp.buf.format { async = true } end,	"format" }
				},
				j = {
					name = "jump",
				},
				l = {
					name = "list",
					c = { ":FzfLua commands<CR>", "commands"},
					h = { ":FzfLua command_history<CR>", "command history"},
					k = { ":FzfLua keymaps<CR>", "keymaps"},
					r = { ":FzfLua oldfiles<CR>", "recent files"},
					t = { ":FzfLua colorschemes<CR>", "themes"},
				},
				o = "open...",
				s = {
					name = "symbol",
					a = { ":Lspsaga code_action<CR>", "code actions" }
				}
			}, { prefix = "<leader>" })

			print("WhichKey loaded.")
		end
}
