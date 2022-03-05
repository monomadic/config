require("lspconfig").rust_analyzer.setup({
  on_attach = function()
    print("attached rust_analyzer lsp")
    require("lsp.lsp-keymaps")()
  end,
  capabilites = require("cmp_nvim_lsp").update_capabilities(vim.lsp.protocol.make_client_capabilities()),
  settings = {
    ["rust-analyzer"] = {
      assist = {
        importMergeBehavior = "last",
        importPrefix = "by_self",
      },
      diagnostics = {
        disabled = { "unresolved-import" },
      },
      cargo = {
        loadOutDirsFromCheck = true,
      },
      procMacro = {
        enable = true,
      },
      checkOnSave = {
        command = "clippy",
      },
    },
  },
})
