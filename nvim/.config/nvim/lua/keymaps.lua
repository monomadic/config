-- Keymaps
--  see: https://github.com/nanotee/nvim-lua-guide#defining-mappings
--  find a binding with :verbose nmap <C-]>
--
local opts = { noremap = true, silent = true }
local term_opts = { silent = true }

-- leader key
vim.g.mapleader = ","

-- Shorten function name
local keymap = vim.api.nvim_set_keymap
--vim.api.nvim_set_keymap("", "<C-q>", ":bd<CR>", { noremap = true })

-- terminal
vim.api.nvim_set_keymap("t", "<Esc>", "<C-\\><C-n>", { noremap = true })

-- emacs style shortcuts in insert mode (yes, i am like that)
vim.api.nvim_set_keymap("i", "<C-n>", "<Down>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-p>", "<Up>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-b>", "<Left>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-f>", "<Right>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-e>", "<End>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-a>", "<Home>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-s>", "<Esc>:write<CR>", { noremap = true })

vim.api.nvim_set_keymap("n", "<C-s>", ":write<CR>", { noremap = true })
vim.api.nvim_set_keymap("n", "<C-a>", "ggVG", { noremap = true })

-- split navigation
vim.api.nvim_set_keymap("n", "<C-j>", "<C-w><C-j>", opts)
vim.api.nvim_set_keymap("n", "<C-k>", "<C-w><C-k>", opts)
vim.api.nvim_set_keymap("n", "<C-l>", "<C-w><C-l>", opts)
vim.api.nvim_set_keymap("n", "<C-h>", "<C-w><C-h>", opts)
--vim.api.nvim_set_keymap("n", "<C-n>", ":vsplit<CR>", opts)
vim.api.nvim_set_keymap("n", "<C-w><C-d>", ":vsplit<CR>", opts)
vim.api.nvim_set_keymap("i", "<C-j>", "<Esc><C-w><C-j>", opts)
vim.api.nvim_set_keymap("i", "<C-k>", "<Esc><C-w><C-k>", opts)
vim.api.nvim_set_keymap("i", "<C-l>", "<Esc><C-w><C-l>", opts)
vim.api.nvim_set_keymap("i", "<C-h>", "<Esc><C-w><C-h>", opts)

--vim.api.nvim_set_keymap("", "<C-m>", "<Esc>:FloatermToggle<CR>", { silent = true })
--vim.api.nvim_set_keymap("n", "<C-g>", ":FloatermNew --title=ranger ranger<CR>", { silent = true })
-- map("n", "<C-m>", ":FloatermNew --title=broot broot<CR>", { silent = true })

-- center oriented nav
-- vim.api.nvim_set_keymap("n", "j", "jzz", opts)
-- vim.api.nvim_set_keymap("n", "k", "kzz", opts)
-- vim.api.nvim_set_keymap("n", "G", "Gzz", opts)
-- vim.api.nvim_set_keymap("n", "<C-d>", "11kzz", opts)
-- vim.api.nvim_set_keymap("n", "<C-u>", "11jzz", opts)

--vim.api.nvim_buf_set_keymap(bufnr, "n", "gW", "<cmd>Rg <cexpr><cr>", opts)
--vim.api.nvim_set_keymap("n", "gW", "<cmd>Telescope grep_string<cr>", opts)
-- vim.api.nvim_buf_set_keymap(
--   bufnr,
--   "n",
--   "gW",
--   "<cmd>lua require('telescope.builtin').grep_string({search = vim.fn.expand(\"<cword>\")})<cr>",
--   opts
-- )
vim.api.nvim_set_keymap(
  "n",
  "gL",
  "<cmd>lua require('telescope.builtin').live_grep({default_text = vim.fn.expand(\"<cword>\")})<cr>",
  opts
)

--vim.api.nvim_set_keymap("", "<", ":write<CR>", { noremap = true })
--vim.api.nvim_set_keymap('', '', ':tabclose<CR>', {noremap = true})

keymap("n", "rr", "<Cmd>lua require'telescope'.extensions.project.project{display_type='full'}<cr>", opts) -- find projects
keymap(
  "n",
  "<Tab>",
  "<cmd>lua require('telescope.builtin').find_files(require('telescope.themes').get_dropdown{previewer = false})<CR>",
  { noremap = true }
)
-- keymap("n", "\\", "<cmd>:Telescope buffers<CR>", { noremap = true, silent = true })
keymap("n", "\\", "<cmd>:Telescope git_status<CR>", { noremap = true, silent = true })
keymap("n", "<C-Tab>", "<cmd>:Term<CR>", { noremap = true, silent = true })
keymap("n", "<C-o>", "<cmd>:Telescope find_files hidden=true no_ignore=true<CR>", { noremap = true, silent = true })

vim.api.nvim_set_keymap("n", "<Esc>", ":noh<cr>", { noremap = true }) -- fix ESC confusion in normal mode
vim.api.nvim_set_keymap("n", "gf", "<Plug>(easymotion-bd-w)", {})
vim.api.nvim_set_keymap("n", "<C-f>", ":Rg<cr>", { noremap = true, silent = true })
--nnoremap <leader>fg <cmd>lua require('telescope.builtin').live_grep()<cr>
--nnoremap <leader>fb <cmd>lua require('telescope.builtin').buffers()<cr>
--nnoremap <leader>fh <cmd>lua require('telescope.builtin').help_tags()<cr>
