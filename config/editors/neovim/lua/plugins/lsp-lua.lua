-- lua formatting
return {
	{
		"ckipp01/stylua-nvim",
		dependencies = "nvim-lspconfig",
		ft = { 'lua' }
	},

	{
		"folke/neodev.nvim",
		dependencies = "nvim-lspconfig",
		ft = "lua",

		config = function()
			require("neodev").setup {
				lspconfig = false,
				library = { plugins = { "neotest" }, types = true },
			}
			vim.api.nvim_create_autocmd("FileType", {
				pattern = "lua",
				callback = function()
					vim.lsp.start {
						name = "neodev",
						cmd = { "lua-language-server" },
						before_init = require("neodev.lsp").before_init,
						root_dir = vim.fn.getcwd(),
						settings = { Lua = {} },
					}
				end
			})
		end
	}
}
