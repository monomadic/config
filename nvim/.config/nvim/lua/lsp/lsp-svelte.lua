require("lspconfig").svelte.setup({
  on_attach = function(client, bufnr)
    require("aerial").setup({
      backends = { "lsp", "treesitter", "markdown" },
    })
    require("aerial").on_attach(client, bufnr)
    require('telescope').load_extension('aerial')
  end,
  -- This makes sure tsserver is not used for formatting (I prefer prettier)
  -- on_attach = require'lsp'.common_on_attach,
  -- settings = { documentFormatting = false },
})
