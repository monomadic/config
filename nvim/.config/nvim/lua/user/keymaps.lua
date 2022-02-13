local opts = { noremap = true, silent = true }
local term_opts = { silent = true }

-- Shorten function name
local keymap = vim.api.nvim_set_keymap
--keymap("", "<C-n>", "<cmd>:NeoTreeReveal<CR>NeoTreeFocusToggle<CR>", opts)
keymap("", "<C-b>", ":NeoTreeFocusToggle<CR>", opts)
