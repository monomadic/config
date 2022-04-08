local on_attach = function(client, bufnr)
  --print("attached lsp")

  -- builtin completion
  --vim.api.nvim_buf_set_option(bufnr, 'omnifunc', 'v:lua.vim.lsp.omnifunc')

  local opts = { noremap = true, silent = true }

  -- See `:help vim.lsp.*` for documentation on any of the below functions
  vim.api.nvim_buf_set_keymap(bufnr, "n", "gD", "<cmd>lua vim.lsp.buf.declaration()<CR>", opts)
  vim.api.nvim_buf_set_keymap(bufnr, "n", "gd", "<cmd>lua vim.lsp.buf.definition()<CR>", opts)
  vim.api.nvim_buf_set_keymap(bufnr, "n", "K", "<cmd>lua vim.lsp.buf.hover()<CR>", opts)
  vim.api.nvim_buf_set_keymap(bufnr, "n", "gi", "<cmd>lua vim.lsp.buf.implementation()<CR>", opts)
  vim.api.nvim_buf_set_keymap(bufnr, "n", "gr", "<cmd>lua vim.lsp.buf.references()<CR>", opts)
  vim.api.nvim_buf_set_keymap(bufnr, "n", "<C-k>", "<cmd>lua vim.lsp.buf.signature_help()<CR>", opts)
  vim.api.nvim_buf_set_keymap(bufnr, "n", "<space>wa", "<cmd>lua vim.lsp.buf.add_workspace_folder()<CR>", opts)
  vim.api.nvim_buf_set_keymap(bufnr, "n", "<space>wr", "<cmd>lua vim.lsp.buf.remove_workspace_folder()<CR>", opts)
  vim.api.nvim_buf_set_keymap(
    bufnr,
    "n",
    "<space>wl",
    "<cmd>lua print(vim.inspect(vim.lsp.buf.list_workspace_folders()))<CR>",
    opts
  )
  vim.api.nvim_buf_set_keymap(bufnr, "n", "gt", "<cmd>lua vim.lsp.buf.type_definition()<CR>", opts)
  vim.api.nvim_buf_set_keymap(bufnr, "n", "gr", "<cmd>lua vim.lsp.buf.rename()<CR>", opts)
  vim.api.nvim_buf_set_keymap(bufnr, "n", "ga", "<cmd>lua vim.lsp.buf.code_action()<CR>", opts)
  -- vim.api.nvim_buf_set_keymap(bufnr, "n", "gx", "<cmd>lua vim.lsp.diagnostic.show_line_diagnostics()<CR>", opts)
  vim.api.nvim_buf_set_keymap(bufnr, "n", "te", "<cmd>lua vim.diagnostic.open_float()<CR>", opts)
  vim.api.nvim_buf_set_keymap(bufnr, "n", "[d", "<cmd>lua vim.diagnostic.goto_prev()<CR>", opts)
  vim.api.nvim_buf_set_keymap(bufnr, "n", "]d", "<cmd>lua vim.diagnostic.goto_next()<CR>", opts)
  vim.api.nvim_buf_set_keymap(bufnr, "n", "J", "<cmd>lua vim.diagnostic.goto_next()<CR>", opts)
  vim.api.nvim_buf_set_keymap(bufnr, "n", "<space>xx", "<cmd>lua vim.lsp.diagnostic.set_loclist()<CR>", opts)
  --vim.api.nvim_buf_set_keymap(bufnr, "n", "<C-f>", "<cmd>lua vim.lsp.buf.formatting<CR>", opts)
  vim.api.nvim_buf_set_keymap(bufnr, "n", "<space>s", "<cmd>SymbolsOutline<CR>", opts)
  vim.api.nvim_buf_set_keymap(bufnr, "n", "gw", ":Telescope lsp_dynamic_workspace_symbols<CR>", opts)
  vim.api.nvim_buf_set_keymap(bufnr, "n", "gs", ":Telescope lsp_document_symbols<cr>", opts)

  vim.api.nvim_set_keymap(
    "n", "gW", "<Cmd>lua require'telescope.builtin'.lsp_workspace_symbols({ query = vim.fn.input('Symbol: ') })<CR>",
    opts
  )

  vim.api.nvim_set_keymap(
    "n", "gf", "<Cmd>lua require'telescope.builtin'.lsp_workspace_symbols({ query = vim.fn.input('Fn: '), symbols='function' })<CR>",
    opts
  )

  -- vim.api.nvim_buf_set_keymap(bufnr, "n", "xx", ":TroubleToggle<CR>", opts)
  -- vim.api.nvim_buf_set_keymap(bufnr, "n", "xw", ":TroubleToggle workspace_diagnostics<CR>", opts)
  -- vim.api.nvim_buf_set_keymap(bufnr, "n", "xq", ":TroubleToggle quickfix<CR>", opts)
  -- vim.api.nvim_buf_set_keymap(bufnr, "n", "xl", ":TroubleToggle loclist<CR>", opts)
  -- vim.api.nvim_buf_set_keymap(bufnr, "n", "xr", ":TroubleToggle lsp_references<CR>", opts)
end

return on_attach
