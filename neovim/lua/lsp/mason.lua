return {
	-- also see: https://github.com/VonHeikemen/lsp-zero.nvim
	-- lspconfig (with mason)
	{ "williamboman/mason.nvim", config = function()
		require("mason").setup {}
	end },
	{ "williamboman/mason-lspconfig.nvim",
		requires = { "neovim/nvim-lspconfig" },
		after = "mason.nvim",
		ft = { "lua", "solidity" },
		config = function()
			require("mason-lspconfig").setup {
				-- ensure_installed = { 'sumneko_lua' },
				--automatic_installation = true,
			}
			-- require('lspconfig').sumneko_lua.setup {}
			require('lspconfig').solidity.setup {
				-- on_attach = on_attach, -- probably you will need this.
				-- capabilities = capabilities,
				settings = {
					-- example of global remapping
					solidity = { includePath = '', remapping = { ["@OpenZeppelin/"] = 'OpenZeppelin/openzeppelin-contracts@4.6.0/' } }
				},
			}
			require 'lspconfig'.solidity_ls.setup {}
		end
	}
}
