require("bufferline").setup {
  options = {
    indicator_icon = ' ',
    always_show_bufferline = true,
    separator_style = 'thick',
    diagnostics = "nvim_lsp",
    -- sort_by = function(buffer_a, buffer_b)
    --   return buffer_a.modified > buffer_b.modified
    -- end
  }
}
vim.cmd([[hi BufferLineBackground guibg=#ff0000]])
