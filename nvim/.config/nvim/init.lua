-- https://github.com/LunarVim/Neovim-from-scratch
-- https://gist.githubusercontent.com/michaelrinderle/d1bb8f18c2414953fd825b59b79fca1d/raw/8adc7bfaa60b74b28c6a96e075cd17ea249252eb/init.vim

require("plugins")
require("settings")
require("keymaps")
require("colors")

-- plugins
require("plugins.whichkey")
require("plugins.neotree")
require("plugins.barbar") -- top/tab bar
require("plugins.galaxyline") -- statusline
require("plugins.telescope")
require("plugins.cmp")
require("plugins.vsnip")
require("plugins.symbols_outline")
require("plugins.colorizer") -- inline colors
require("plugins.neoformat") -- code formatting
require("plugins.floaterm") -- floating term
require("plugins.devicons")
require("plugins.indent-blankline") -- indentation
require("plugins.harpoon") -- marks
require("plugins.neoscroll") -- smooth animations on scroll
require("plugins.hop")
require("plugins.wilder")
-- require("plugins.vgit")
--require("plugins.sidebar")
--require("scrollfix")
require 'plugins.telekasten'
require 'plugins.mkdnflow'

-- LSP
require("lsp.lsp-installer")
--require("lsp.lsp-javascript")
--require("lsp.lsp-rust")
--require("lsp.lsp-svelte")
--require("lsp.lsp-nix")
--require("lsp.lsp-solidity")
require("lsp.trouble")
--require 'lsp'

local cmd = vim.cmd
local call = vim.call

require("todo-comments").setup({})

-- Lightbulb
vim.api.nvim_create_autocmd("CursorHold,CursorHoldI", {
  pattern = "*",
  callback = function()
    require('nvim-lightbulb').update_lightbulb();
  end
})

require("null-ls").setup({
  on_attach = function(client, bufnr) end,
})

-- treehopper
require("tsht").config.hint_keys = { "h", "j", "f", "d", "n", "v", "s", "l", "a" }
vim.api.nvim_set_keymap("n", "<C-u>", ":lua require('tsht').nodes()<CR>", { noremap = true })

-- require("zk").setup({
--   -- can be "telescope", "fzf" or "select" (`vim.ui.select`)
--   -- it's recommended to use "telescope" or "fzf"
--   picker = "select",
--
--   lsp = {
--     -- `config` is passed to `vim.lsp.start_client(config)`
--     config = {
--       cmd = { "zk", "lsp" },
--       name = "zk",
-- vim.api.nvim_set_keymap("i", "<C-n>", "<Down>", { noremap = true })
--       -- on_attach = ...
--       -- etc, see `:h vim.lsp.start_client()`
--     },
--
--     -- automatically attach buffers in a zk notebook that match the given filetypes
--     auto_attach = {
--       enabled = true,
--       filetypes = { "markdown" },
--     },
--   },
-- })

-- require("nvim-blamer").setup({
--   enable = false, -- you must set this to true in order to show the blame info
--   prefix = "  ", -- you can cusomize it to any thing, unicode emoji, even disable it, just set to empty lua string
--   format = "%committer │ %committer-time %committer-tz │ %summary",
--   auto_hide = false, -- set this to true will enable delay hide even you do not have the cursor moved
--   hide_delay = 3000, -- this is the delay time in milliseconds for delay auto hide
--   show_error = false, -- set to true to show any possible error (just for debug problems)
-- })

-- Set title to PWD
-- auto BufEnter * let &titlestring = " " .. expand("%:p")
-- auto BufEnter * let &titlestring = "nvim:  " .. fnamemodify(getcwd(), ":t") .. " " .. expand("%:t")
vim.cmd([[
  auto BufEnter * let &titlestring = " " .. fnamemodify(getcwd(), ":t")
  autocmd BufReadPost,FileReadPost,BufNewFile * call system("tmux rename-window %")
  set title
]])

-- Globals
--   see :h lua-vim-variables and :h lua-vim-options
vim.opt.wildignore = { "*/cache/*", "*/tmp/*" }
vim.opt.number = true
-- tabs and spaces
vim.opt.tabstop = 2 -- size of each tab
vim.opt.shiftwidth = 2 -- spaces to shift when using << and >>
vim.opt.expandtab = true -- spaces when using tab
vim.g["noswapfile"] = true
vim.g["nocompatible"] = true
vim.g["hidden"] = true -- so buffers can hide

-- nvim-ripgrep (! means overriddes anything existing)
--vim.cmd([[command! Rg lua require'nvim-ripgrep'.grep()]])

-- Highlight on yank
vim.cmd([[
  augroup YankHighlight
    autocmd!
    autocmd TextYankPost * silent! lua vim.highlight.on_yank()
  augroup end
]])

local popui = require("popui.ui-overrider")
vim.ui.select = popui

require("prettier").setup({
  bin = "prettier",
  filetypes = { "javascript", "typescript", "json", "lua" },
})

-- vim.cmd([[let loaded_netrwPlugin = 1]])

require("plugins.gitsigns")
--require("plugins.line") -- statusbar (doesn't work?)
