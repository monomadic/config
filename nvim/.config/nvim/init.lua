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

vim.g.symbols_outline = {
  highlight_hovered_item = true,
  show_guides = true,
  auto_preview = true,
  position = "right",
  relative_width = true,
  width = 64,
  auto_close = false,
  show_numbers = false,
  show_relative_numbers = false,
  show_symbol_details = true,
  preview_bg_highlight = "Normal",
  keymaps = { -- These keymaps can be a string or a table for multiple keys
    close = { "<Esc>", "q" },
    goto_location = "<Cr>",
    focus_location = "o",
    hover_symbol = "<C-space>",
    toggle_preview = "K",
    rename_symbol = "r",
    code_actions = "a",
  },
  lsp_blacklist = {},
  symbol_blacklist = {},
  symbols = {
    File = { icon = "", hl = "TSURI" },
    Module = { icon = "", hl = "TSNamespace" },
    Namespace = { icon = "", hl = "TSNamespace" },
    Package = { icon = "", hl = "TSNamespace" },
    Class = { icon = "𝓒", hl = "TSType" },
    Method = { icon = "ƒ", hl = "TSMethod" },
    Property = { icon = "", hl = "TSMethod" },
    Field = { icon = "", hl = "TSField" },
    Constructor = { icon = "", hl = "TSConstructor" },
    Enum = { icon = "ℰ", hl = "TSType" },
    Interface = { icon = "ﰮ", hl = "TSType" },
    Function = { icon = "", hl = "TSFunction" },
    Variable = { icon = "", hl = "TSConstant" },
    Constant = { icon = "", hl = "TSConstant" },
    String = { icon = "𝓐", hl = "TSString" },
    Number = { icon = "#", hl = "TSNumber" },
    Boolean = { icon = "⊨", hl = "TSBoolean" },
    Array = { icon = "", hl = "TSConstant" },
    Object = { icon = "⦿", hl = "TSType" },
    Key = { icon = "🔐", hl = "TSType" },
    Null = { icon = "NULL", hl = "TSType" },
    EnumMember = { icon = "", hl = "TSField" },
    Struct = { icon = "𝓢", hl = "TSType" },
    Event = { icon = "🗲", hl = "TSType" },
    Operator = { icon = "+", hl = "TSOperator" },
    TypeParameter = { icon = "𝙏", hl = "TSParameter" },
  },
}

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
--vim.cmd([[:set nowrap]])

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

-- inline colors
--require'colorizer'.setup()

require("nvim-web-devicons").setup()

-- Keymaps
--  see: https://github.com/nanotee/nvim-lua-guide#defining-mappings
local keymap = vim.api.nvim_set_keymap
local opts = { noremap = true, silent = true }

vim.api.nvim_set_keymap("", "<C-s>", ":write<CR>", { noremap = true })
--vim.api.nvim_set_keymap("", "<", ":write<CR>", { noremap = true })
--vim.api.nvim_set_keymap('', '', ':tabclose<CR>', {noremap = true})
vim.api.nvim_set_keymap("", "<C-q>", ":bdelete<CR>", { noremap = true })

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

