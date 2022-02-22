banner = {
  "██▄   ▄███▄   ██     ▄▄▄▄▀ ▄  █ ██▄   ▄█    ▄▄▄▄▄   ▄█▄    ████▄ ",
  "█  █  █▀   ▀  █ █ ▀▀▀ █   █   █ █  █  ██   █     ▀▄ █▀ ▀▄  █   █ ",
  "█   █ ██▄▄    █▄▄█    █   ██▀▀█ █   █ ██ ▄  ▀▀▀▀▄   █   ▀  █   █ ",
  "█  █  █▄   ▄▀ █  █   █    █   █ █  █  ▐█  ▀▄▄▄▄▀    █▄  ▄▀ ▀████ ",
  "███▀  ▀███▀      █  ▀        █  ███▀   ▐            ▀███▀        ",
  "                █           ▀                                    ",
  "               ▀                                   ",
}

require 'plugins'
require 'settings'
require 'keymaps'

-- plugins
require "plugins.whichkey"
require 'plugins.cmp'
require 'plugins.telescope'
require 'plugins.vsnip'
require 'plugins.galaxyline'
require 'plugins.symbols_outline'
require 'plugins.colorizer' -- inline colors

local cmd = vim.cmd
local call = vim.call

require("user.colors")
require("user.comment")
require("user.neotree")

-- LSP
require 'lsp.lsp-javascript'
require 'lsp.lsp-svelte'
require 'lsp.lsp-rust-tools' -- provides type-hints, rust-runnables
require 'lsp.keymaps'()
--require 'lsp'

-- https://github.com/LunarVim/Neovim-from-scratch

-- Lightbulb
vim.cmd [[autocmd CursorHold,CursorHoldI * lua require'nvim-lightbulb'.update_lightbulb()]]

-- Terminal
require("toggleterm").setup({
  open_mapping = [[<C-j>]],
  shading_factor = "1",
  -- shell = "bash",
})

require("null-ls").setup({
  on_attach = function(client, bufnr) end,
})

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

-- nvim-ripgrep
vim.cmd([[command! Rg lua require'nvim-ripgrep'.grep()]])

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

vim.cmd([[let loaded_netrwPlugin = 1]])

require("nvim-web-devicons").setup()

