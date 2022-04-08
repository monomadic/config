local lsp_installer = require("nvim-lsp-installer")

lsp_installer.on_server_ready(function(server)
  if server.name == "rust_analyzer" then
    require("lsp.lsp-rust-tools") -- provides type-hints, rust-runnables
    server:attach_buffers()
    server:setup({})
  else
    local opts = {
      on_attach = on_attach,
      flags = {
        debounce_text_changes = 150,
      },
    }
    server:setup({})
  end
  -- vim.cmd [[ do User LspAttachBuffers ]]
end)
