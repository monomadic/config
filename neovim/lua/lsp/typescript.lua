return { 'jose-elias-alvarez/typescript.nvim',
	ft = 'typescript',
	reqires = { 'lvimuser/lsp-inlayhints.nvim' },
	config = function()
		require("lsp-inlayhints").setup()
		require("typescript").setup({
			-- disable_commands = false, -- prevent the plugin from creating Vim commands
			debug = false, -- enable debug logging for commands
			go_to_source_definition = {
				fallback = true, -- fall back to standard LSP definition on failure
			},
			server = {
				on_attach = function(client, bufnr)
					require("lsp-format").on_attach(client, bufnr)
					require("lsp-inlayhints").on_attach(client, bufnr)
					require("lsp")
				end
			},
		})
	end }
