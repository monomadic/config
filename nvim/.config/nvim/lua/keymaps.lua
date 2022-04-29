-- Keymaps
--  see: https://github.com/nanotee/nvim-lua-guide#defining-mappings
--  find a binding with :verbose nmap <C-]>
--
local opts = { noremap = true, silent = true }
local term_opts = { silent = true }

-- leader key
vim.g.mapleader = " "

local keymap = vim.api.nvim_set_keymap
--vim.api.nvim_set_keymap("", "<C-q>", ":bd<CR>", { noremap = true })

-- undo breakpoints (avoids large destructive undo sequences)
keymap("i", ",", ",<c-g>u", opts)
keymap("i", ".", ".<c-g>u", opts)
keymap("i", "!", "!<c-g>u", opts)
keymap("i", "(", "(<c-g>u", opts)

-- move selected text up or down lines
keymap("v", "<Down>", ":m '>+1<CR>gv=gv", opts)
keymap("v", "<Up>", ":m '<-2<CR>gv=gv", opts)

-- terminal
vim.api.nvim_set_keymap("t", "<Esc>", "<C-\\><C-n>", { noremap = true })

-- emacs style shortcuts in insert mode (yes, i am like that)
vim.keymap.set("i", "<C-n>", "<Down>");
vim.keymap.set("i", "<C-p>", "<Up>");
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

-- grep entire project with word under cursor
vim.keymap.set("n", "gF", function()
  require('telescope.builtin').live_grep({
    default_text = vim.fn.expand("<cword>")
  })
end)

-- grep entire project
vim.keymap.set("n", "<C-f>", function()
  require('telescope.builtin').live_grep()
end)

-- fuzzy file open
vim.keymap.set("n", "<Tab>", function()
  require('telescope.builtin').find_files(
    require('telescope.themes').get_dropdown({ previewer = false })
  )
end)

keymap("n", "\\", "<cmd>:Telescope git_status<CR>", { noremap = true, silent = true })
keymap("n", "<C-Tab>", "<cmd>:Term<CR>", { noremap = true, silent = true })
keymap("n", "<C-o>", "<cmd>:Telescope find_files hidden=true no_ignore=true<CR>", { noremap = true, silent = true })

vim.api.nvim_set_keymap("n", "<Esc>", ":noh<cr>", { noremap = true }) -- fix ESC confusion in normal mode
vim.api.nvim_set_keymap("n", "gf", "<Plug>(easymotion-bd-w)", {})
--vim.api.nvim_set_keymap("n", "<C-f>", ":Rg<cr>", { noremap = true, silent = true })
--nnoremap <leader>fg <cmd>lua require('telescope.builtin').live_grep()<cr>
--nnoremap <leader>fb <cmd>lua require('telescope.builtin').buffers()<cr>
--nnoremap <leader>fh <cmd>lua require('telescope.builtin').help_tags()<cr>

-- LSP
--
-- See `:help vim.lsp.*` for documentation on any of the below functions
keymap("n", "gD", "<cmd>lua vim.lsp.buf.declaration()<CR>", opts)
keymap("n", "gd", "<cmd>lua vim.lsp.buf.definition()<CR>", opts)
keymap("n", "K", "<cmd>lua vim.lsp.buf.hover()<CR>", opts)
keymap("n", "gi", "<cmd>lua vim.lsp.buf.implementation()<CR>", opts)
keymap("n", "gr", "<cmd>lua vim.lsp.buf.references()<CR>", opts)
keymap("n", "<C-k>", "<cmd>lua vim.lsp.buf.signature_help()<CR>", opts)
keymap("n", "<space>wa", "<cmd>lua vim.lsp.buf.add_workspace_folder()<CR>", opts)
keymap("n", "<space>wr", "<cmd>lua vim.lsp.buf.remove_workspace_folder()<CR>", opts)
keymap(
  "n",
  "<space>wl",
  "<cmd>lua print(vim.inspect(vim.lsp.buf.list_workspace_folders()))<CR>",
  opts
)
keymap("n", "gt", "<cmd>lua vim.lsp.buf.type_definition()<CR>", opts)
keymap("n", "gr", "<cmd>lua vim.lsp.buf.rename()<CR>", opts)
keymap("n", "ga", "<cmd>lua vim.lsp.buf.code_action()<CR>", opts)
-- keymap("n", "gx", "<cmd>lua vim.lsp.diagnostic.show_line_diagnostics()<CR>", opts)
keymap("n", "te", "<cmd>lua vim.diagnostic.open_float()<CR>", opts)
keymap("n", "[d", "<cmd>lua vim.diagnostic.goto_prev()<CR>", opts)
keymap("n", "]d", "<cmd>lua vim.diagnostic.goto_next()<CR>", opts)
keymap("n", "J", "<cmd>lua vim.diagnostic.goto_next()<CR>", opts)
keymap("n", "<space>xx", "<cmd>lua vim.lsp.diagnostic.set_loclist()<CR>", opts)
--keymap("n", "<C-f>", "<cmd>lua vim.lsp.buf.formatting<CR>", opts)
keymap("n", "<space>s", "<cmd>SymbolsOutline<CR>", opts)
keymap("n", "gw", ":Telescope lsp_dynamic_workspace_symbols<CR>", opts)
keymap("n", "gs", ":Telescope lsp_document_symbols<cr>", opts)

keymap(
  "n", "gW", "<Cmd>lua require'telescope.builtin'.lsp_workspace_symbols({ query = vim.fn.input('Symbol: ') })<CR>",
  opts
)

keymap(
  "n", "gf", "<Cmd>lua require'telescope.builtin'.lsp_workspace_symbols({ query = vim.fn.input('Fn: '), symbols='function' })<CR>",
  opts
)

-- keymap("n", "xx", ":TroubleToggle<CR>", opts)
-- keymap("n", "xw", ":TroubleToggle workspace_diagnostics<CR>", opts)
-- keymap("n", "xq", ":TroubleToggle quickfix<CR>", opts)
-- keymap("n", "xl", ":TroubleToggle loclist<CR>", opts)
-- keymap("n", "xr", ":TroubleToggle lsp_references<CR>", opts)

-- Syntax Tree Surfer
-- Normal Mode Swapping
vim.api.nvim_set_keymap("n", "vd", '<cmd>lua require("syntax-tree-surfer").move("n", false)<cr>', {noremap = true, silent = true})
vim.api.nvim_set_keymap("n", "vu", '<cmd>lua require("syntax-tree-surfer").move("n", true)<cr>', {noremap = true, silent = true})
-- -- .select() will show you what you will be swapping with .move(), you'll get used to .select() and .move() behavior quite soon!
vim.api.nvim_set_keymap("n", "vx", '<cmd>lua require("syntax-tree-surfer").select()<cr>', {noremap = true, silent = true})
-- -- .select_current_node() will select the current node at your cursor
vim.api.nvim_set_keymap("n", "vn", '<cmd>lua require("syntax-tree-surfer").select_current_node()<cr>', {noremap = true, silent = true})
-- NAVIGATION: Only change the keymap to your liking. I would not recommend changing anything about the .surf() parameters!
vim.api.nvim_set_keymap("x", "J", '<cmd>lua require("syntax-tree-surfer").surf("next", "visual")<cr>', {noremap = true, silent = true})
vim.api.nvim_set_keymap("x", "K", '<cmd>lua require("syntax-tree-surfer").surf("prev", "visual")<cr>', {noremap = true, silent = true})
vim.api.nvim_set_keymap("x", "H", '<cmd>lua require("syntax-tree-surfer").surf("parent", "visual")<cr>', {noremap = true, silent = true})
vim.api.nvim_set_keymap("x", "L", '<cmd>lua require("syntax-tree-surfer").surf("child", "visual")<cr>', {noremap = true, silent = true})
-- SWAPPING WITH VISUAL SELECTION: Only change the keymap to your liking. Don't change the .surf() parameters!
vim.api.nvim_set_keymap("x", "<A-j>", '<cmd>lua require("syntax-tree-surfer").surf("next", "visual", true)<cr>', {noremap = true, silent = true})
vim.api.nvim_set_keymap("x", "<A-k>", '<cmd>lua require("syntax-tree-surfer").surf("prev", "visual", true)<cr>', {noremap = true, silent = true})
