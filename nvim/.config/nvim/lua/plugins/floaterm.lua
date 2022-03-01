vim.g.floaterm_width = 0.8
vim.g.floaterm_height = 0.8
-- vim.g.floaterm_position = 'topright'
vim.g.floaterm_keymap_new = "<F7>"
vim.g.floaterm_keymap_prev = "<F8>"
vim.g.floaterm_keymap_next = "<F9>"
vim.g.floaterm_keymap_toggle = "<C-j>"
-- vim.g.floaterm_autoclose = 2
vim.g.floaterm_opener = "edit" -- edit | split | vsplit | tabe | drop

local map = vim.api.nvim_set_keymap
map("n", "<C-j>", ":FloatermToggle<CR>", { silent = true })
map("n", "<C-g>", ":FloatermNew --title=ranger ranger<CR>", { silent = true })
map("n", "<C-m>", ":FloatermNew --title=broot broot<CR>", { silent = true })

vim.cmd([[command! -nargs=1 Rg :FloatermNew rg <args>]])
vim.cmd([[command! Broot :FloatermNew broot]])

vim.cmd([[
  hi Floaterm guibg=black
  hi FloatermBorder guifg=cyanA
]])
