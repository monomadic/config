-- null-lsp: a generic lsp server providing lsp functions to neovim on behalf of various tools
return {
	"jose-elias-alvarez/null-ls.nvim",
	dependencies = { "nvim-lua/plenary.nvim", "lukas-reineke/lsp-format.nvim" },

	config = function()
		local null_ls = require("null-ls")
		-- https://github.com/jose-elias-alvarez/null-ls.nvim/tree/main/lua/null-ls/builtins/formatting

		-- null_ls.register {
		-- 	name = "markdown_source",
		-- 	filetypes = { "markdown", "vimwiki" },
		-- 	sources = {
		-- 		null_ls.builtins.formatting.prettier,
		-- 		-- null_ls.builtins.diagnostics.proselint, -- prosemd is better
		-- 		-- null_ls.builtins.code_actions.proselint,
		-- 	},
		-- }

		-- null_ls.register {
		-- 	name = "rustfmt",
		-- 	filetypes = { "rust" },
		-- 	sources = { formatting.rustfmt },
		-- }

		null_ls.setup {
			sources = {
				null_ls.builtins.formatting.taplo,   -- cargo install taplo-cli --locked
				null_ls.builtins.formatting.black.with({ -- python
					filetypes = { "python" }
				}),
				null_ls.builtins.formatting.isort.with({ -- python
					filetypes = { "python" }
				}),
				null_ls.builtins.formatting.prettier.with({
					filetypes = { "html", "json", "yaml", "markdown", "vimwiki", "graphql", "snippets" },
				}),
				null_ls.builtins.diagnostics.jsonlint, -- brew install jsonlint
				null_ls.builtins.hover.dictionary.with {
					filetypes = { "markdown", "vimwiki" }
				}, -- markdown spellcheck
			},
			on_attach = function(client, buf)
				require("lsp-format").on_attach(client, buf)

				-- disable this dumb mapping
				-- local augroup = vim.api.nvim_create_augroup("LspFormatting", {})
				-- if client.supports_method("textDocument/formatting") then
				-- 	vim.api.nvim_clear_autocmds({ group = augroup, buffer = bufnr })
				-- 	vim.api.nvim_create_autocmd("BufWritePre", {
				-- 		group = augroup,
				-- 		buffer = bufnr,
				-- 		callback = function()
				-- 			-- on 0.8, you should use vim.lsp.buf.format({ bufnr = bufnr }) instead
				-- 			vim.lsp.buf.formatting_sync()
				-- 		end,
				-- 	})
				-- end
			end,
		}
	end,
}
