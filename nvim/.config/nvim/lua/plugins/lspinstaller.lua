local lsp_installer = require("nvim-lsp-installer")

lsp_installer.on_server_ready(function(server)
  if server.name == "rust_analyzer" then
    local opts = {
      server = vim.tbl_deep_extend("force", server:get_default_options(), {
        on_attach = on_attach,
        flags = {
          debounce_text_changes = 150,
        },
      }),
    }
    require("rust-tools").setup(coq.lsp_ensure_capabilities(opts))
    server:attach_buffers()
  else
    local opts = {
      on_attach = on_attach,
      flags = {
        debounce_text_changes = 150,
      },
    }
    server:setup(coq.lsp_ensure_capabilities(opts))
  end
  -- vim.cmd [[ do User LspAttachBuffers ]]
end)
