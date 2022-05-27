-- @monomadic neovim 0.7+ compatible config
-- requires: git

-- packer
--
local install_path = vim.fn.stdpath("data") .. "/site/pack/packer/start/packer.nvim"
if vim.fn.empty(vim.fn.glob(install_path)) > 0 then
  vim.fn.system({ "git", "clone", "--depth", "1", "https://github.com/wbthomason/packer.nvim", install_path })
  vim.cmd("packadd packer.nvim")
end

-- settings
--
vim.g.mapleader = ',' -- leader key
vim.g.tex_flavor = "latex"
vim.opt.clipboard = "unnamedplus" -- allows neovim to access the system clipboard (gnome)
vim.opt.conceallevel = 0 -- so that `` is visible in markdown files
vim.opt.cursorline = true -- highlight the current line
vim.opt.expandtab = true -- convert tabs to spaces
vim.opt.foldlevelstart = 99
vim.opt.formatoptions = vim.o.formatoptions:gsub("r", ""):gsub("o", "")
vim.opt.hidden = false -- switch buffer without unloading+saving them
vim.opt.hlsearch = false -- highlight all matches on previous search pattern
vim.opt.ignorecase = true -- ignore case when searching
vim.opt.laststatus = 2 -- 3 = global statusline (neovim 0.7+)
vim.opt.laststatus = 2 -- 3 = global statusline (neovim 0.7+)
vim.opt.lazyredraw = true -- faster macros (force update with :redraw)
vim.opt.mouse = "a" -- allow the mouse to be used in neovim
vim.opt.number = true -- set numbered lines
vim.opt.number = true -- set numbered lines
vim.opt.relativenumber = false -- set relative numbered lines
vim.opt.scrolloff = 1000 -- keep line centered (disable if scrolling past eof is enabled)
vim.opt.shiftwidth = 2 -- the number of spaces inserted for each indentation
vim.opt.showmatch = true -- matching parenthesis
vim.opt.signcolumn = "yes" -- always show the sign column, otherwise it would shift the text each time
vim.opt.smartcase = true -- searches are case insensitive unless a capital is used
vim.opt.smartindent = true -- make indenting smarter again
vim.opt.softtabstop = 2 -- number of spaces to convert a tab to
vim.opt.splitbelow = true -- force all horizontal splits to go below current window
vim.opt.splitright = true -- force all vertical splits to go to the right of current window
vim.opt.swapfile = false -- creates a swapfile
vim.opt.tabstop = 2 -- insert 2 spaces for a tab
vim.opt.termguicolors = true -- 24-bit color
vim.opt.wrap = false -- display lines as one long line
vim.wo.foldexpr = 'nvim_treesitter#foldexpr()' -- use treesitter for folding
vim.wo.foldmethod = 'expr' -- fold method (market | syntax)

-- #keymaps
--
-- split navigation
vim.keymap.set("n", "<C-j>", "<C-w><C-j>")
vim.keymap.set("i", "<C-j>", "<Esc><C-w><C-j>")
vim.keymap.set("n", "<C-k>", "<C-w><C-k>")
vim.keymap.set("i", "<C-k>", "<Esc><C-w><C-k>")
vim.keymap.set("n", "<C-l>", "<C-w><C-l>")
vim.keymap.set("i", "<C-l>", "<Esc><C-w><C-l>")
vim.keymap.set("n", "<C-h>", "<C-w><C-h>")
vim.keymap.set("i", "<C-h>", "<Esc><C-w><C-h>")
vim.keymap.set("n", "<C-w><C-d>", "<cmd>vsplit<CR>")

-- #plugins
--
vim.cmd [[packadd packer.nvim]]
require('packer').startup(function(use)
  use {'wbthomason/packer.nvim' }
  use {'tpope/vim-fugitive'}
  use {'dylanaraps/wal.vim'}
  use {'morhetz/gruvbox'} -- theme
  use {'bluz71/vim-nightfly-guicolors'}
  use {'lunarvim/darkplus.nvim'}
  use {'neovim/nvim-lspconfig'}
  use {'williamboman/nvim-lsp-installer'} -- install lsp clients
  use {'nvim-treesitter/nvim-treesitter'}
  use {'nvim-lua/popup.nvim'}
  use {'nvim-lua/plenary.nvim'}
  use {'nvim-telescope/telescope.nvim', config = function()
    require('telescope').setup{}
  end}
  use {'hrsh7th/nvim-cmp'} -- completion
  use {'hrsh7th/cmp-nvim-lsp'}
  use {'honza/vim-snippets'}
  use {'L3MON4D3/LuaSnip'}
  use {'norcalli/nvim-colorizer.lua', config = [[require"colorizer".setup()]]}

  -- custom tabline framework
  use { "rafcamlet/tabline-framework.nvim",  requires = "kyazdani42/nvim-web-devicons" }

  use 'stevearc/aerial.nvim' -- aerial view / overview of lsp structure

  -- jump/sneak
  use({
    "phaazon/hop.nvim", -- alternative to sneak
    branch = "v1", -- optional but strongly recommended
    config = function()
      require("hop").setup({ keys = "etovxqpdygfblzhckisuran" })
      vim.keymap.set("n", "s", "<Cmd>HopWord<CR>")
      vim.keymap.set("n", "S", "<Cmd>HopPattern<CR>")
    end
  })

  use {'chentoast/marks.nvim', config = function()
    require'marks'.setup {
      sign_priority = { lower=10, upper=15, builtin=8, bookmark=20 },
    }
  end}

  use { 'junegunn/goyo.vim' } -- distraction-free

  use {
    "folke/trouble.nvim",
    requires = "kyazdani42/nvim-web-devicons",
    config = function()
      require("trouble").setup {
      }
    end
  }

  use {'numToStr/Comment.nvim',
    config = function()
      -- `gcc` line comment
      -- `gcA` line comment at eol
      -- `gc0` line comment at bol
      -- `gco` line comment at line-open
      -- `gbc` block comment
      require('Comment').setup()
    end
  }
  use {'nvim-telescope/telescope-bibtex.nvim', config = [[require"telescope".load_extension("bibtex")]], ft = 'tex'}
  use {'nvim-neo-tree/neo-tree.nvim',
    requires = {
      "nvim-lua/plenary.nvim",
      "kyazdani42/nvim-web-devicons",
      "MunifTanjim/nui.nvim",
    },
  }

  use {'renerocksai/telekasten.nvim'}
  use {'preservim/vim-markdown'}

  use {
    -- surround completion
    "appelgriebsch/surround.nvim",
    -- surround inline change
    "tpope/vim-surround",
    config = function()
      require('surround').setup {}
    end
  }

  use {'tamago324/nlsp-settings.nvim'}

  -- git
  use { "lewis6991/gitsigns.nvim", requires = {"nvim-lua/plenary.nvim"}, config = function()
    require('gitsigns').setup {}
  end}

  use { 'numToStr/FTerm.nvim', config = function()
    require'FTerm'.setup({
        border = 'none',
        hl = "Term",
    })
  end}

  -- #lsp
  -- cargo
  use({
    "saecki/crates.nvim",
    event = { "BufRead Cargo.toml" },
    requires = { { "nvim-lua/plenary.nvim" } },
    config = function()
      require('crates').setup {}
    end,
  })
  -- null
  use({
      "jose-elias-alvarez/null-ls.nvim",
      config = function()
          require("null-ls").setup()
      end,
      requires = { "nvim-lua/plenary.nvim" },
  })
  use { 'simrat39/rust-tools.nvim' } -- rust lsp

  -- formatting
  use { 'lukas-reineke/lsp-format.nvim', config = function()
    require("lsp-format").setup {}
  end}

  use { "petertriho/nvim-scrollbar", config = "require'scrollbar'.setup()" } -- side scrollbar with git support

  use { "lukas-reineke/indent-blankline.nvim", config = function()
    require("indent_blankline").setup({
      show_current_context = true,
      show_current_context_start = true,
      filetype_exclude = { "neo-tree", "help", "floaterm", "SidebarNvim", "" },
    })
  end}

  -- Todo
  use {
    "folke/todo-comments.nvim",
    requires = "nvim-lua/plenary.nvim",
    config = function()
      require('todo-comments').setup {}
    end
  }
end)
-- compile packer plugins on plugins.lua change
vim.cmd([[autocmd BufWritePost plugins.lua PackerCompile]])

require('telescope').setup{
  defaults = {
    prompt_prefix = " ",
    file_ignore_patterns = {".git/", ".cache", "%.o", "%.a", "%.out", "%.class", "%.pdf", "%.mkv", "%.mp4", "%.zip", "*.lock"},
    buffer_previewer_maker = small_file_preview_only,
    mappings = {
      i = {
        ["<Esc>"] = "close",
        ["<Tab>"] = "close",
        ["<C-l>"] = require("telescope.actions.layout").toggle_preview,
        ["<C-u>"] = false,
      },
    },
  }
};

vim.keymap.set('n', '<C-Space>', '<CMD>lua require("FTerm").toggle()<CR>')
vim.keymap.set('t', '<C-Space>', '<C-\\><C-n><CMD>lua require("FTerm").toggle()<CR>')

vim.cmd("hi Term guibg=black")

-- #tabline
require('tabline_framework').setup {
  render = function(f)
    f.add(' %{fnamemodify(getcwd(), ":t")}')
  end
}
vim.opt.showtabline=2
vim.cmd("hi TablineFramework_1 gui=bold")

-- tree
--
require('neo-tree').setup({
  popup_border_style = "single",
  window = {
    mappings = {
      ["l"] = "open",
      ["<C-l>"] = "open_vsplit",
    }
  }
})

local home = vim.fn.expand("/mnt/data/notes/zk")
require('telekasten').setup {
  home = home,
  dailies      = home .. '/' .. 'daily',
  weeklies     = home .. '/' .. 'weekly',
  templates    = home .. '/' .. 'templates',
  new_note_filename = "title",
  auto_set_filetype = false,
}

-- #colors
-- #00FF99 #FF00CC #FFFF00 #00CCFF
--
vim.cmd('syntax on')
vim.g.gruvbox_contrast_dark="hard"
vim.cmd("colorscheme {{colorscheme}}");
vim.cmd("highlight WinSeparator guifg=none");
vim.cmd("hi TodoBgTODO guibg=#FFFF00 guifg=black");
vim.cmd("hi TodoFgTODO guifg=#FFFF00");

-- active window
-- vim.cmd("set winhighlight=Normal:ActiveWindow,NormalNC:InactiveWindow");
-- vim.cmd("hi ActiveWindow guibg=#092236");
-- vim.cmd("hi InactiveWindow guibg=#000001");

vim.cmd([[set fillchars+=vert:\ ]]) -- remove awful vertical split character
--
-- remove trailing whitespaces on save
vim.cmd([[autocmd BufWritePre * %s/\s\+$//e]])
--
-- remove trailing newline on save
vim.cmd([[autocmd BufWritePre * %s/\n\+\%$//e]])

-- hide line-bar in insert-mode
vim.api.nvim_create_autocmd("InsertEnter", { pattern = "*", callback = function()
  vim.o.cursorline = false
end})
vim.api.nvim_create_autocmd("InsertLeave", { pattern = "*", callback = function()
  vim.o.cursorline = true
end})

-- only show line-bar on current buffer, on active window
vim.api.nvim_create_autocmd("BufLeave", { pattern = "*", callback = function()
  vim.o.cursorline = false
end})
vim.api.nvim_create_autocmd("BufEnter", { pattern = "*", callback = function()
  vim.o.cursorline = true
end})
vim.api.nvim_create_autocmd("WinLeave", { pattern = "*", callback = function()
  vim.o.cursorline = false
end})
vim.api.nvim_create_autocmd("WinEnter", { pattern = "*", callback = function()
  vim.o.cursorline = true
end})

-- keymaps
--
-- split resize
--
vim.keymap.set("n", "<leader>-", "<cmd>vertical resize -10<CR>")
vim.keymap.set("n", "<leader>+", "<cmd>vertical resize +10<CR>")
vim.keymap.set("n", "<leader>_", "<cmd>resize -10<CR>")
vim.keymap.set("n", "<leader>*", "<cmd>resize +10<CR>")
--
-- split navigation
--
vim.keymap.set("n", "<C-j>", "<C-w><C-j>")
vim.keymap.set("n", "<C-k>", "<C-w><C-k>")
vim.keymap.set("n", "<C-l>", "<C-w><C-l>")
vim.keymap.set("n", "<C-h>", "<C-w><C-h>")
vim.keymap.set("", "<C-p>", "<Cmd>NeoTreeFloatToggle<CR>")
vim.keymap.set("n", "<C-s>", "<Cmd>write<CR>");
vim.keymap.set("i", "<C-s>", "<Esc><Cmd>write<CR>");
--
-- grep entire project
vim.keymap.set("n", "<C-f>", function()
  require('telescope.builtin').live_grep()
end)
--
-- fuzzy file open
vim.keymap.set("n", "<Tab>", function()
  require('telescope.builtin').find_files(
    require('telescope.themes').get_dropdown({ previewer = false })
  )
end)
--
--telekasten
vim.keymap.set("n", "z", "<Cmd>Telekasten panel<CR>")
vim.keymap.set("n", "zn", "<Cmd>Telekasten new_note<CR>")
vim.keymap.set("n", "zN", "<Cmd>Telekasten new_templated_note<CR>")
vim.keymap.set("n", "zt", "<Cmd>Telekasten show_tags<CR>")
vim.keymap.set("n", "zo", "<Cmd>Telekasten toggle_todo<CR>")
vim.keymap.set("n", "zT", "<Cmd>Telekasten find_weekly_notes<CR>")
vim.keymap.set("n", "zf", "<Cmd>Telekasten find_notes<CR>")
vim.keymap.set("n", "zr", "<Cmd>Telekasten rename_note<CR>")
vim.keymap.set("n", "zg", "<Cmd>Telekasten follow_link<CR>")
vim.keymap.set("n", "zr", "<Cmd>Telekasten show_backlinks<CR>")
vim.keymap.set("n", "gz", "<Cmd>Telekasten follow_link<CR>")
vim.cmd("hi tkLink ctermfg=Magenta cterm=bold,underline guifg=#FF00DF gui=bold,underline")
vim.cmd("hi tkBrackets ctermfg=gray guifg=gray")

-- run command in current line and paste stout into current buffer
vim.keymap.set("n", "Q", "!!$SHELL<CR>")
--
-- move lines up and down in visual mode
vim.keymap.set("x", "K", ":move '<-2<CR>gv-gv")
vim.keymap.set("x", "J", ":move '>+1<CR>gv-gv")
--
-- useful bindings
-- vim.keymap.set("i", "kj", "<Esc>")
-- vim.keymap.set("", "<Space>", ":")
vim.keymap.set("n", "<leader>ev", "<cmd>vs $MYVIMRC<CR>")
vim.keymap.set("n", "<leader>sv", "<cmd>source $MYVIMRC<CR>")
--
-- quote quickly
vim.keymap.set("i", '<leader>"', '<Esc>viw<Esc>a"<Esc>bi"<Esc>leli')
vim.keymap.set("v", '<leader>"', '<Esc>`<i"<Esc>`>ea"<Esc>')
-- substitute shortcut
vim.keymap.set("n", "S", ":%s//g<Left><Left>")
vim.keymap.set("v", "S", ":s//g<Left><Left>")
-- quickfix navigation
vim.keymap.set("n", "<leader>q", "<cmd>cnext<cr>")
vim.keymap.set("n", "<leader>Q", "<cmd>cprev<cr>")
-- spellcheck
vim.keymap.set("n", "<leader>sp", ":setlocal spell spelllang=de")
-- more reachable line start/end
vim.keymap.set("n", "H", "^")
vim.keymap.set("n", "L", "$")
-- write to ----READONLY---- files
vim.keymap.set("c", "w!!",  "execute 'silent! write !sudo tee % >/dev/null' <bar> edit!")
-- nvim-commenter
vim.keymap.set("v", "<leader>x", "<cmd>MultiCommenterToggle<cr>")
vim.keymap.set("n", "<leader>x", "<cmd>SingleCommenterToggle<cr>")

-- terminal mappings
vim.keymap.set("t", "<Esc>", "<C-\\><C-n>")

vim.keymap.set("n", "<leader>t", "<cmd>sp | term<cr>")
-- termdebugger
vim.keymap.set("n", "<leader>dd", ":TermdebugCommand")

-- ===== find project root for quick cd =====
local api = vim.api
function find_project_root()
  local id = [[.git]]
  local file = api.nvim_buf_get_name(0)
  local root = vim.fn.finddir(id, file .. ';')
  if root ~= "" then
    root = root:gsub(id, '')
    print(root)
    vim.api.nvim_set_current_dir(root)
  else
    print("No repo found.")
  end
end

-- I always type :Qa accidentally... should be using ZZ
vim.keymap.set("n", "Qa", "<cmd>qa<cr>");

-- smart cwd
vim.keymap.set("n", "cf", "<cmd>cd %:p:h | pwd<cr>")
vim.keymap.set("n", "cr", "<cmd>lua find_project_root()<cr>")
-- tab for completion menu
vim.keymap.set("i", "<Tab>", 'pumvisible() ? "\\<C-n>" : "\\<Tab>"', {expr = true})
vim.keymap.set("i", "<S-Tab>", 'pumvisible() ? "\\<C-p>" : "\\<Tab>"', {expr = true})

-- #statusline
--
local stl = {
  -- ' %{fnamemodify(getcwd(), ":t")}',
  -- ' %{pathshorten(expand("%:p"))}',
  ' %{fnamemodify(expand("%"), ":~:.")}',
  -- ' %{pathshorten(expand("%"), ":~:.")}',
  '%=',
  -- '  %{FugitiveStatusline()}',
  ' %M', ' %y', ' %r'
}
vim.o.statusline = table.concat(stl)
vim.cmd("hi StatusLine guibg=#002233 guifg=#00FFAA"); --active
vim.cmd("hi StatusLineNC guibg=none"); --inactive

-- ===== telescope setup =====
vim.keymap.set("n", '<leader>b', '<cmd>Telescope buffers<cr>')
vim.keymap.set("n", '<leader>o', '<cmd>Telescope find_files<cr>')
vim.keymap.set("n", '<leader>h', '<cmd>Telescope oldfiles<cr>')
vim.keymap.set("n", '<leader>c', '<cmd>Telescope commands<cr>')
vim.keymap.set("n", '<leader>ch', '<cmd>Telescope command_history<cr>')
vim.keymap.set("n", '<leader>f', '<cmd>Telescope live_grep<cr>')
vim.keymap.set("n", '<leader>z', '<cmd>Telescope spell_suggest<cr>')
vim.keymap.set('','<F1>', '<cmd>Telescope help_tags<cr>')

vim.keymap.set('n', 'td', '<Cmd>Telescope diagnostics<cr>')
vim.keymap.set('n', 'tgb', '<Cmd>Telescope git_branches<cr>')
vim.keymap.set('n', 'tgc', '<Cmd>Telescope git_bcommits<cr>')
vim.keymap.set('n', 'tgd', '<Cmd>Telescope git_status<cr>')
vim.keymap.set('n', 'tk', '<Cmd>Telescope keymaps<cr>')
vim.keymap.set('n', 'tld', '<Cmd>Telescope lsp_definitions<cr>')
vim.keymap.set('n', 'tli', '<Cmd>Telescope lsp_implementations<cr>')
vim.keymap.set('n', 'tls', '<Cmd>Telescope lsp_document_symbols<cr>')
vim.keymap.set('n', 'tlw', '<Cmd>Telescope lsp_workspace_symbols<cr>')
vim.keymap.set('n', 'tm', '<Cmd>Telescope marks<cr>')
vim.keymap.set('n', 'tr', '<Cmd>Telescope live_grep<cr>')
vim.keymap.set('n', 'tt', '<Cmd>TodoTelescope<cr>')
vim.keymap.set('n', 'tz', '<Cmd>Telekasten find_notes<cr>')

-- ===== simple session management =====
local session_dir = vim.fn.stdpath('data') .. '/sessions/'
vim.keymap.set("n", '<leader>ss', ':mks! ' .. session_dir)
vim.keymap.set("n", '<leader>sr', ':%bd | so ' .. session_dir)

-- ===== completion settings =====
vim.o.completeopt="menuone,noinsert,noselect"
vim.g.completion_matching_strategy_list = {'exact', 'substring', 'fuzzy'}
vim.g.completion_matching_ignore_case = 1
vim.g.completion_trigger_keyword_length = 3

-- #lsp
--
local custom_attach = function(client, bufnr)
	print("lsp started");

  require "lsp-format".on_attach(client)

	--require('cmp-lsp').on_attach(client);
  vim.lsp.handlers["textDocument/publishDiagnostics"] = vim.lsp.with(
    vim.lsp.diagnostic.on_publish_diagnostics, { virtual_text = false, update_in_insert = false }
  )
  -- automatic diagnostics popup
  vim.api.nvim_command('autocmd CursorHold <buffer> lua vim.diagnostic.show()')
  -- speedup diagnostics popup
  vim.o.updatetime=500
  -- diagnostic settings
  vim.diagnostic.config({
    virtual_text = false,
    signs = true, -- sidebar signs
    underline = true,
    severity_sort = true,
  })

  require("aerial").setup({
    backends = { "lsp", "treesitter", "markdown" },
  })
  require("aerial").on_attach(client, bufnr)
  require('telescope').load_extension('aerial')

  -- diagnostics icon
  local signs = { Error = "┃ ", Warn = "┃ ", Hint = "┃ ", Info = "┃ " }
  for type, icon in pairs(signs) do
    local hl = "DiagnosticSign" .. type
    -- vim.cmd("hi " .. hl .. " guibg=none")
    vim.fn.sign_define(hl, { text = icon, texthl = hl })
  end

  -- diagnostics float on hover
  vim.api.nvim_create_autocmd("CursorHold", {
    buffer = bufnr,
    callback = function()
      local opts = {
        focusable = false,
        close_events = { "BufLeave", "CursorMoved", "InsertEnter", "FocusLost" },
        border = 'rounded',
        source = 'always',
        prefix = ' ',
        scope = 'cursor',
      }
      vim.diagnostic.open_float(nil, opts)
    end
  })

  vim.keymap.set("n", 'gD','<cmd>lua vim.lsp.buf.declaration()<CR>')
  vim.keymap.set("n", '<c-]>','<cmd>lua vim.lsp.buf.definition()<CR>')
  vim.keymap.set("n", 'K','<cmd>lua vim.lsp.buf.hover()<CR>')
  vim.keymap.set("n", 'gr','<cmd>lua vim.lsp.buf.references()<CR>')
  vim.keymap.set("n", 'gs','<cmd>lua vim.lsp.buf.signature_help()<CR>')
  vim.keymap.set("n", 'gi','<cmd>lua vim.lsp.buf.implementation()<CR>')
  vim.keymap.set("n", 'ga','<cmd>lua vim.lsp.buf.code_action()<CR>')
  vim.keymap.set("n", '<leader>r','<cmd>lua vim.lsp.buf.rename()<CR>')
  vim.keymap.set("n", '<leader>=', '<cmd>lua vim.lsp.buf.formatting()<CR>')
  vim.keymap.set("n", '<C-]>', '<cmd>lua vim.diagnostic.goto_next()<CR>')
  vim.keymap.set("n", '<C-[>', '<cmd>lua vim.diagnostic.goto_prev()<CR>')
  vim.keymap.set("n", '\\', '<cmd>TroubleToggle<CR>')
end

-- cmp/lsp config
--
-- tsserver: npm install -g typescript typescript-language-server
local lspconfig = require('lspconfig')
local servers = { 'bashls', 'rnix', 'zk', 'tsserver' }
local capabilities = require('cmp_nvim_lsp').update_capabilities(
  vim.lsp.protocol.make_client_capabilities()
);
for _, lsp in ipairs(servers) do
  lspconfig[lsp].setup {
    on_attach = custom_attach,
    capabilities = capabilities,
  }
end

-- #rust-tools
--
require('rust-tools').setup({
    tools = {
        autoSetHints = true,
        hover_with_actions = true,
        runnables = {
            use_telescope = true
        },
        inlay_hints = {
            show_parameter_hints = false,
            parameter_hints_prefix = "",
            other_hints_prefix = "",
        },
    },

    -- see https://github.com/neovim/nvim-lspconfig/blob/master/CONFIG.md#rust_analyzer
    server = {
        on_attach = custom_attach,
        settings = {
            -- to enable rust-analyzer settings visit:
            -- https://github.com/rust-analyzer/rust-analyzer/blob/master/docs/user/generated_config.adoc
            ["rust-analyzer"] = {
                -- enable clippy on save
                checkOnSave = {
                    command = "clippy"
                },
            }
        }
    },
})

-- treesitter
--
require'nvim-treesitter.configs'.setup {
  ensure_installed = {"rust", "bash", "yaml", "typescript", "javascript", "markdown"},
  highlight = { enable = true },
}

-- cmp
local cmp = require('cmp')
cmp.setup {
  sources = {
    { name = 'nvim_lsp' }
  },
  preselect = cmp.PreselectMode.None,
  snippet = {
    expand = function(args)
      require('luasnip').lsp_expand(args.body) -- For `luasnip` users.
    end,
  },
  mapping = {
    ['<CR>'] = cmp.mapping.confirm({
      behavior = cmp.ConfirmBehavior.Replace,
      select = false, -- false = only complete if an item is actually selected
    }),
    ['<Tab>'] = function(fallback)
        if cmp.visible() then
            cmp.select_next_item()
        else
            fallback()
        end
    end,
    ['<S-Tab>'] = function(fallback)
        if cmp.visible() then
            cmp.select_prev_item()
        else
            fallback()
        end
    end,
  },
}
