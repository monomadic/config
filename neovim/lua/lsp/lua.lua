-- lua formatting
return {
	{ "ckipp01/stylua-nvim", ft = { 'lua' } },
	{ "folke/neodev.nvim",
		-- after = "nvim-lspconfig",
		ft = "lua",
		config = function()
			require("neodev").setup {
				lspconfig = false
			}
			vim.api.nvim_create_autocmd("FileType", { pattern = "lua", callback = function()
				vim.lsp.start {
					name = "neodev",
					cmd = { "lua-language-server" },
					before_init = require("neodev.lsp").before_init,
					root_dir = vim.fn.getcwd(),
					settings = { Lua = {} },
				}
			end })
		end
	}
}