local function documentHighlight(client, bufnr)
  -- Set autocommands conditional on server_capabilities
  if client.resolved_capabilities.document_highlight then
    vim.api.nvim_exec(
      [[
      hi LspReferenceRead cterm=bold ctermbg=red guibg=#464646
      hi LspReferenceText cterm=bold ctermbg=red guibg=#464646
      hi LspReferenceWrite cterm=bold ctermbg=red guibg=#464646
      augroup lsp_document_highlight
        autocmd! * <buffer>
        autocmd CursorHold <buffer> lua vim.lsp.buf.document_highlight()
        autocmd CursorMoved <buffer> lua vim.lsp.buf.clear_references()
      augroup END
    ]],
      false
    )
  end
end

local lsp_config = {}

function lsp_config.common_on_attach(client, bufnr)
  require("lsp.lsp-keymaps")()
  documentHighlight(client, bufnr)
end

function lsp_config.tsserver_on_attach(client, bufnr)
  lsp_config.common_on_attach(client, bufnr)
  client.resolved_capabilities.document_formatting = false
end

function lsp_config.rust_analyzer_on_attach(client, bufnr)
  lsp_config.common_on_attach(client, bufnr)
  client.resolved_capabilities.document_formatting = false
end

-- Use a loop to conveniently both setup defined servers
-- and map buffer local keybindings when the language server attaches
--local nvim_lsp = require'lspconfig'
--local servers = {"rust_analyzer", "tsserver"}
-- for _, lsp in ipairs(servers) do nvim_lsp[lsp].setup {on_attach = function()
--   require'lsp.lsp-keymaps'()
-- end} end

return lsp_config
