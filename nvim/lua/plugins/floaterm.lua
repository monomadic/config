vim.cmd([[
  hi Floaterm guibg=black
  hi FloatermBorder guifg=cyan
]])

require('floaterm').setup({
  keymaps = {
    exit = '<C-q>',
    normal = '<Esc>',
    name = 'terminal'
  },
})
