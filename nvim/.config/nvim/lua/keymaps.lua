-- Keymaps
--  see: https://github.com/nanotee/nvim-lua-guide#defining-mappings
local opts = { noremap = true, silent = true }
local term_opts = { silent = true }

-- Shorten function name
local keymap = vim.api.nvim_set_keymap
vim.api.nvim_set_keymap("", "<C-q>", ":bd<CR>", { noremap = true })

-- emacs style shortcuts in insert mode (yes, i am like that)
vim.api.nvim_set_keymap("i", "<C-n>", "<Down>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-p>", "<Up>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-b>", "<Left>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-f>", "<Right>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-e>", "<End>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-a>", "<Home>", { noremap = true })
vim.api.nvim_set_keymap("", "<C-s>", ":write<CR>", { noremap = true })
--vim.api.nvim_set_keymap("", "<", ":write<CR>", { noremap = true })
--vim.api.nvim_set_keymap('', '', ':tabclose<CR>', {noremap = true})

keymap("n", "rr", "<Cmd>lua require'telescope'.extensions.project.project{display_type='full'}<cr>", opts) -- find projects
keymap(
  "n",
  "<C-i>",
  "<cmd>lua require('telescope.builtin').find_files(require('telescope.themes').get_dropdown{previewer = false})<cr>",
  { noremap = true }
)
keymap("n", "<C-o>", "<cmd>:Telescope find_files<CR>", { noremap = true, silent = true })

vim.api.nvim_set_keymap("n", "<Esc>", ":noh<cr>", { noremap = true }) -- fix ESC confusion in normal mode
vim.api.nvim_set_keymap("n", "gf", "<Plug>(easymotion-bd-w)", {})
vim.api.nvim_set_keymap("n", "<C-f>", ":Rg<cr>", { noremap = true, silent = true })
--nnoremap <leader>fg <cmd>lua require('telescope.builtin').live_grep()<cr>
--nnoremap <leader>fb <cmd>lua require('telescope.builtin').buffers()<cr>
--nnoremap <leader>fh <cmd>lua require('telescope.builtin').help_tags()<cr>
