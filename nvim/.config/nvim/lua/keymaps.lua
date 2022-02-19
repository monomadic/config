local opts = { noremap = true, silent = true }
local term_opts = { silent = true }

-- Shorten function name
local keymap = vim.api.nvim_set_keymap
--keymap("", "<C-n>", "<cmd>:NeoTreeReveal<CR>NeoTreeFocusToggle<CR>", opts)
--keymap("", "<C-b>", ":NeoTreeFocusToggle<CR>", opts)

-- emacs style shortcuts in insert mode (yes, i am like that)
vim.api.nvim_set_keymap("i", "<C-n>", "<Down>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-p>", "<Up>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-b>", "<Left>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-f>", "<Right>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-e>", "<End>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-a>", "<Home>", { noremap = true })

