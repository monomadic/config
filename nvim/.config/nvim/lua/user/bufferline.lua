vim.cmd([[
  hi BufferLineBackground guibg=#ff0000
]])

vim.api.nvim_set_keymap("n", "H", ":BufferLineCyclePrev<CR>", { noremap = true, silent = true })
vim.api.nvim_set_keymap("n", "L", ":BufferLineCycleNext<CR>", { noremap = true, silent = true })

require("bufferline").setup {
  options = {
    indicator_icon = ' ',
    always_show_bufferline = true,
    separator_style = 'thick',
    diagnostics = "nvim_lsp",
  }
}
