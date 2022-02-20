require'lspconfig'.svelte.setup {
    on_attach = function()
      require'lsp.keymaps'()
    end,
    -- This makes sure tsserver is not used for formatting (I prefer prettier)
    -- on_attach = require'lsp'.common_on_attach,
    settings = {documentFormatting = false}
}

