return {
	"ray-x/go.nvim",
	dependencies = { -- optional packages
		"ray-x/guihua.lua",
		"neovim/nvim-lspconfig",
		"nvim-treesitter/nvim-treesitter",
	},
	config = function()
		local capabilities = require('cmp_nvim_lsp').default_capabilities(vim.lsp.protocol.make_client_capabilities())
		require("go").setup {
			capabilities = capabilities,
			lsp_cfg = {
				-- server = {
				-- 	on_attach = function(client, bufnr)
				-- 		require("lsp-format").on_attach(client, bufnr)
				-- 		require("lsp-inlayhints").on_attach(client, bufnr)
				-- 		require("lsp")
				-- 	end
				-- }
			}
		}
	end,
	event = { "CmdlineEnter" },
	ft = { "go", 'gomod' },
	build = ':lua require("go.install").update_all_sync()' -- if you need to install/update all binaries
}
