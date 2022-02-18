local cmd = vim.cmd
local call = vim.call
vim.g["dashboard_custom_header"] = {
  "██▄   ▄███▄   ██     ▄▄▄▄▀ ▄  █ ██▄   ▄█    ▄▄▄▄▄   ▄█▄    ████▄ ",
  "█  █  █▀   ▀  █ █ ▀▀▀ █   █   █ █  █  ██   █     ▀▄ █▀ ▀▄  █   █ ",
  "█   █ ██▄▄    █▄▄█    █   ██▀▀█ █   █ ██ ▄  ▀▀▀▀▄   █   ▀  █   █ ",
  "█  █  █▄   ▄▀ █  █   █    █   █ █  █  ▐█  ▀▄▄▄▄▀    █▄  ▄▀ ▀████ ",
  "███▀  ▀███▀      █  ▀        █  ███▀   ▐            ▀███▀        ",
  "                █           ▀                                    ",
  "               ▀                                   ",
}

require("user.options")
require("user.keymaps")
require("user.plugins")
require("user.colors")
require("user.whichkey")
require("user.comment")
--require 'user.nvimtree'
require("user.neotree")
--require("user.telescope")

-- https://github.com/LunarVim/Neovim-from-scratch

dropdown_theme = require("telescope.themes").get_dropdown({
  previewer = false,
  prompt_title = "",
  results_height = 16,
  width = 0.6,
  borderchars = {
    { "─", "│", "─", "│", "╭", "╮", "╯", "╰" },
    prompt = { "─", "│", " ", "│", "╭", "╮", "│", "│" },
    results = { "─", "│", "─", "│", "├", "┤", "╯", "╰" },
    preview = { "─", "│", "─", "│", "╭", "╮", "╯", "╰" },
  },
})

-- LSP
local nvim_lsp = require("lspconfig")
local lsp_installer = require("nvim-lsp-installer")

lsp_installer.on_server_ready(function(server)
  local keymap = vim.api.nvim_set_keymap

  -- LSP Keymaps
  keymap("n", "gD", "<cmd>lua vim.lsp.buf.declaration()<CR>", { noremap = true, silent = true })
  keymap("n", "gd", "<cmd>lua vim.lsp.buf.definition()<CR>", { noremap = true, silent = true })
  keymap("n", "K", "<cmd>lua vim.lsp.buf.hover()<CR>", { noremap = true, silent = true })
  keymap("n", "gi", "<cmd>lua vim.lsp.buf.implementation()<CR>", { noremap = true, silent = true })
  keymap("n", "gr", "<cmd>lua vim.lsp.buf.references()<CR>", { noremap = true, silent = true })
  keymap("n", "<C-k>", "<cmd>lua vim.lsp.buf.signature_help()<CR>", { noremap = true, silent = true })
  keymap("n", "<space>wa", "<cmd>lua vim.lsp.buf.add_workspace_folder()<CR>", { noremap = true, silent = true })
  keymap("n", "<space>wr", "<cmd>lua vim.lsp.buf.remove_workspace_folder()<CR>", { noremap = true, silent = true })
  keymap(
    "n",
    "<space>wl",
    "<cmd>lua print(vim.inspect(vim.lsp.buf.list_workspace_folders()))<CR>",
    { noremap = true, silent = true }
  )
  keymap("n", "td", "<cmd>lua vim.lsp.brf.type_definition()<CR>", { noremap = true, silent = true })
  keymap("n", "tr", "<cmd>lua vim.lsp.buf.rename()<CR>", { noremap = true, silent = true })
  keymap("n", "ga", "<cmd>lua vim.lsp.buf.code_action()<CR>", { noremap = true, silent = true })
  keymap("n", "te", "<cmd>lua vim.diagnostic.open_float()<CR>", { noremap = true, silent = true })
  keymap("n", "[d", "<cmd>lua vim.diagnostic.goto_prev()<CR>", { noremap = true, silent = true })
  keymap("n", "]d", "<cmd>lua vim.diagnostic.goto_next()<CR>", { noremap = true, silent = true })
  --keymap("n", "gl", "<cmd>lua vim.diagnostic.setloclist()<CR>", { noremap = true, silent = true})
  keymap("n", "gf", "<cmd>lua vim.lsp.buf.formatting()<CR>", { noremap = true, silent = true })
  keymap("n", "gs", "<cmd>:SymbolsOutline<CR>", { noremap = true, silent = true })
  -- vim.api.nvim_set_keymap("n", "<C-i>", ":TagbarOpenAutoClose<CR>", {})

  keymap(
    "n",
    "gw",
    "<cmd>lua require('telescope.builtin').lsp_workspace_symbols(require('telescope.themes').get_dropdown{previewer = false})<cr>",
    { noremap = true }
  )
  keymap(
    "n",
    "gd",
    "<cmd>lua require('telescope.builtin').lsp_document_symbols(require('telescope.themes').get_dropdown{previewer = false})<cr>",
    { noremap = true }
  )

  -- Enable completion triggered by <c-x><c-o>

  local opts = {}

  -- signatures (eg function completion etc)
  require("lsp_signature").setup({})
  --require('user.autocomplete')

  -- (optional) Customize the options passed to the server
  -- if server.name == "rust_analyzer" then
  --     -- opts.root_dir = function()
  --     --   cmd([[colorscheme sonokai]])
  --     -- end
  -- end

  -- This setup() function will take the provided server configuration and decorate it with the necessary properties
  -- before passing it onwards to lspconfig.
  -- Refer to https://github.com/neovim/nvim-lspconfig/blob/master/doc/server_configurations.md
  --server:setup(opts)
  --vim.cmd [[ do User LspAttach Buffers ]]
end)

vim.cmd([[set foldmethod=marker]]) -- marker | syntax

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

-- LSP: Completion
-- {{{
local cmp = require("cmp")

cmp.setup({
  snippet = {
    -- REQUIRED - you must specify a snippet engine
    expand = function(args)
      vim.fn["vsnip#anonymous"](args.body) -- For `vsnip` users.
      -- require('luasnip').lsp_expand(args.body) -- For `luasnip` users.
      -- require('snippy').expand_snippet(args.body) -- For `snippy` users.
      -- vim.fn["UltiSnips#Anon"](args.body) -- For `ultisnips` users.
    end,
  },
  mapping = {
    ["<C-b>"] = cmp.mapping(cmp.mapping.scroll_docs(-4), { "i", "c" }),
    ["<C-f>"] = cmp.mapping(cmp.mapping.scroll_docs(4), { "i", "c" }),
    ["<C-Space>"] = cmp.mapping(cmp.mapping.complete(), { "i", "c" }),
    ["<C-y>"] = cmp.config.disable, -- Specify `cmp.config.disable` if you want to remove the default `<C-y>` mapping.
    ["<C-e>"] = cmp.mapping({
      i = cmp.mapping.abort(),
      c = cmp.mapping.close(),
    }),
    ["<Cr>"] = cmp.mapping.confirm({ select = false }), -- Accept currently selected item. Set `select` to `false` to only confirm explicitly selected items.
  },
  sources = cmp.config.sources({
    { name = "nvim_lsp" },
    { name = "vsnip" }, -- For vsnip users.
    -- { name = 'luasnip' }, -- For luasnip users.
    -- { name = 'ultisnips' }, -- For ultisnips users.
    -- { name = 'snippy' }, -- For snippy users.
  }, {
    { name = "buffer" },
  }),
})

-- Use buffer source for `/` (if you enabled `native_menu`, this won't work anymore).
cmp.setup.cmdline("/", {
  sources = {
    { name = "buffer" },
  },
})

-- Use cmdline & path source for ':' (if you enabled `native_menu`, this won't work anymore).
cmp.setup.cmdline(":", {
  sources = cmp.config.sources({
    { name = "path" },
  }, {
    { name = "cmdline" },
  }),
})

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
-- {{{
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
vim.cmd([[:set nowrap]])

-- Highlight on yank
vim.cmd([[
  augroup YankHighlight
    autocmd!
    autocmd TextYankPost * silent! lua vim.highlight.on_yank()
  augroup end
]])

--vim.o.completeopt = "menu,menuone,noselect" -- completion

local popui = require("popui.ui-overrider")
vim.ui.select = popui

require("prettier").setup({
  bin = "prettier",
  filetypes = { "javascript", "typescript", "json", "lua" },
})

vim.cmd([[let loaded_netrwPlugin = 1]])

-- Colors
-- {{{
-- 36 forest green
-- 46 bright green
-- 62 purple
-- 120 pale neon green
-- 234 dark grey
-- vim.g.tokyonight_terminal_colors = true
-- vim.g.tokyonight_transparent = true
-- vim.g.transparent_sidebar = true
-- vim.g.dark_sidebar = true

-- vim.g.tokyonight_sidebars = { "terminal" }
--vim.g.tokyonight_day_brightness = 1

local catppuccin = require("catppuccin")
catppuccin.setup({
  term_colors = false,
  transparent_background = true,
  colorscheme = "neon_latte",
  styles = {
    comments = "NONE",
    functions = "italic",
    keywords = "NONE",
    strings = "NONE",
    variables = "NONE",
  },
  integrations = {
    treesitter = true,
    native_lsp = {
      enabled = true,
      virtual_text = {
        errors = "italic",
        hints = "italic",
        warnings = "italic",
        information = "italic",
      },
      underlines = {
        errors = "underline",
        hints = "underline",
        warnings = "underline",
        information = "underline",
      },
    },
    lsp_trouble = false,
    cmp = true,
    lsp_saga = false,
    gitgutter = false,
    gitsigns = true,
    telescope = true,
    which_key = false,
    indent_blankline = {
      enabled = true,
      colored_indent_levels = false,
    },
    dashboard = false,
    neogit = false,
    vim_sneak = true,
    fern = false,
    barbar = false,
    bufferline = false,
    markdown = true,
    lightspeed = false,
    ts_rainbow = false,
    hop = false,
    notify = true,
    telekasten = true,
  },
})

require("material").setup({
  contrast = {
    sidebars = false, -- Enable contrast for sidebar-like windows ( for example Nvim-Tree )
    floating_windows = false, -- Enable contrast for floating windows
    line_numbers = false, -- Enable contrast background for line numbers
    sign_column = false, -- Enable contrast background for the sign column
    cursor_line = false, -- Enable darker background for the cursor line
    non_current_windows = false, -- Enable darker background for non-current windows
    popup_menu = true, -- Enable lighter background for the popup menu
  },

  italics = {
    comments = false, -- Enable italic comments
    keywords = false, -- Enable italic keywords
    functions = false, -- Enable italic functions
    strings = false, -- Enable italic strings
    variables = false, -- Enable italic variables
  },

  contrast_filetypes = { -- Specify which filetypes get the contrasted (darker) background
    "terminal", -- Darker terminal background
    "packer", -- Darker packer background
    "qf", -- Darker qf list background
  },

  high_visibility = {
    lighter = false, -- Enable higher contrast text for lighter style
    darker = false, -- Enable higher contrast text for darker style
  },

  disable = {
    borders = false, -- Disable borders between verticaly split windows
    background = false, -- Prevent the theme from setting the background (NeoVim then uses your teminal background)
    term_colors = false, -- Prevent the theme from setting terminal colors
    eob_lines = false, -- Hide the end-of-buffer lines
  },

  lualine_style = "default", -- Lualine style ( can be 'stealth' or 'default' )

  async_loading = true, -- Load parts of the theme asyncronously for faster startup (turned on by default)

  custom_highlights = {}, -- Overwrite highlights with your own
})

-- cmd([[hi NvimTreeVertSplit guibg=none]])

-- vim.api.nvim_exec(
--   [[
--   hi Normal guibg=none ctermbg=none
--   hi Pmenu ctermfg=white guibg=#222222 ctermbg=234 ctermfg=246
--   hi Folded ctermbg=DarkGrey ctermfg=White guibg=#222222 guifg=#FFFFFF
--   hi VertSplit ctermfg=none gui=none guibg=none
--   hi Comment guifg=#666666 ctermfg=grey
--   hi Cursor ctermbg=0 ctermfg=none guibg=0 guibg=none guifg=#FFFFFF
--   hi Special ctermfg=white
-- ]], false)
--
-- inline colors
--require'colorizer'.setup()

require("nvim-web-devicons").setup()

-- StatusBar
-- require("lualine").setup({
--   options = {
--     icons_enabled = true,
--     theme = "auto",
--     component_separators = { left = "", right = "" },
--     section_separators = { left = "", right = "" },
--     disabled_filetypes = {},
--     always_divide_middle = true,
--   },
--   sections = {
--     lualine_a = { "mode" },
--     lualine_b = { "branch", "diff", "diagnostics" },
--     lualine_c = { "filename" },
--     lualine_x = { "encoding", "fileformat", "filetype" },
--     lualine_y = { "progress" },
--     lualine_z = { "location" },
--   },
--   inactive_sections = {
--     lualine_a = {},
--     lualine_b = {},
--     lualine_c = { "filename" },
--     lualine_x = { "location" },
--     lualine_y = {},
--     lualine_z = {},
--   },
--   tabline = {},
--   extensions = {},
-- })

-- Keymaps
-- {{{
--  see: https://github.com/nanotee/nvim-lua-guide#defining-mappings
local keymap = vim.api.nvim_set_keymap
local opts = { noremap = true, silent = true }

vim.api.nvim_set_keymap("", "S", ":write<CR>", { noremap = true })
--vim.api.nvim_set_keymap("", "<", ":write<CR>", { noremap = true })
--vim.api.nvim_set_keymap('', '', ':tabclose<CR>', {noremap = true})
vim.api.nvim_set_keymap("", "<C-q>", ":bdelete<CR>", { noremap = true })
vim.api.nvim_set_keymap("", "<C-t>", ":Telescope<cr>", { noremap = true })

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

-- emacs style shortcuts in insert mode (yes, i am like that)
vim.api.nvim_set_keymap("i", "<C-n>", "<Down>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-p>", "<Up>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-b>", "<Left>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-f>", "<Right>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-e>", "<End>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-a>", "<Home>", { noremap = true })

-- move page with cursor
-- keymap("n", "k", "kzz", { noremap = true, silent = true })
-- keymap("n", "j", "jzz", { noremap = true, silent = true })
-- keymap("n", "p", "pzz", { noremap = true, silent = true })
-- keymap("n", "P", "Pzz", { noremap = true, silent = true })
-- keymap("n", "{", "{zz", { noremap = true, silent = true})
-- keymap("n", "}", "}zz", { noremap = true, silent = true})
-- keymap("n", "G", "Gzz", { noremap = true, silent = true})
-- keymap("n", "n", "nzz", { noremap = true, silent = true})
-- keymap("n", "N", "Nzz", { noremap = true, silent = true})
-- keymap("n", "o", "o<ESC>zza", { noremap = true, silent = true})
-- keymap("n", "O", "O<ESC>zza", { noremap = true, silent = true})
-- keymap("n", "a", "a<ESC>zza", { noremap = true, silent = true})
-- keymap("n", "<ENTER>", "<ENTER>zz", { noremap = true, silent = true})
-- keymap("i", "<ESC>", "<ESC>zz", { noremap = true, silent = true})
-- keymap("i", "<ENTER>", "<ENTER><ESC>zzi", { noremap = true, silent = true})

vim.opt.scrolloff = 100
