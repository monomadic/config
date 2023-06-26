return {
	-- also see: https://github.com/VonHeikemen/lsp-zero.nvim
	-- lspconfig (with mason)
	{
		"williamboman/mason.nvim",
		cmd = {
			"Mason",
			"MasonInstall",
			"MasonUninstall",
			"MasonUninstallAll",
			"MasonLog",
			"MasonUpdate",
			"MasonUpdateAll",
		},
		opts = {
			ui = {
				icons = {
					package_installed = "✓",
					package_uninstalled = "✗",
					package_pending = "⟳",
				},
			},
		},
		build = ":MasonUpdate",
		config = function(_, opts)
			require("mason").setup(opts)
		end
	},

	{
		"neovim/nvim-lspconfig",
		dependencies = {
			{
				"williamboman/mason-lspconfig.nvim",
				cmd = { "LspInstall", "LspUninstall" },
				opts = function(_, opts)
					if not opts.handlers then opts.handlers = {} end
					opts.handlers[1] = function(server)
						require("base.utils.lsp").setup(server)
					end
				end,
				config = function(_, opts)
					require("mason-lspconfig").setup(opts)
					require("base.utils").event "MasonLspSetup"
				end,
			},
		},
		event = "User BaseFile",
	},

	{
		"williamboman/mason-lspconfig.nvim",
		dependencies = { "neovim/nvim-lspconfig", "williamboman/mason.nvim" },
		ft = { "solidity", "cpp", "markdown" },

		config = function()
			require("mason-lspconfig").setup {
				-- ensure_installed = { 'sumneko_lua' },
				--automatic_installation = true,
			}
			-- require('lspconfig').sumneko_lua.setup {}
			-- https://github.com/artempyanykh/marksman/blob/main/Tests/default.marksman.toml
			require('lspconfig').marksman.setup {}

			require('lspconfig').solidity.setup {
				-- on_attach = on_attach, -- probably you will need this.
				-- capabilities = capabilities,
				settings = {
					-- example of global remapping
					solidity = {
						includePath = '',
						remapping = { ["@OpenZeppelin/"] = 'OpenZeppelin/openzeppelin-contracts@4.6.0/' }
					},
				},
			}
			require 'lspconfig'.solidity_ls.setup {}
		end
	}
}
