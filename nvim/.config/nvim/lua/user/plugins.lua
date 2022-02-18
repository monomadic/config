local fn = vim.fn
local install_path = fn.stdpath("data") .. "/site/pack/packer/start/packer.nvim"
if fn.empty(fn.glob(install_path)) > 0 then
  PACKER_BOOTSTRAP = fn.system({
    "git",
    "clone",
    "--depth",
    "1",
    "https://github.com/wbthomason/packer.nvim",
    install_path,
  })
end

require("packer").startup(function(use)
  use("wbthomason/packer.nvim") -- Have packer manage itself

  -- Tree
  use({
    "nvim-neo-tree/neo-tree.nvim",
    requires = {
      "nvim-lua/plenary.nvim",
      "kyazdani42/nvim-web-devicons",
      "MunifTanjim/nui.nvim",
    },
    --config = require("user.neotree"),
  })
  --use("kyazdani42/nvim-tree.lua")
  --use("MunifTanjim/nui.nvim") -- neotree dep

  -- UI
  -- use "nvim-lua/popup.nvim" -- An implementation of the Popup API from vim in Neovim
  use("RishabhRD/popfix") -- popupui (required by popui)
  use("hood/popui.nvim")

  use("nvim-lua/plenary.nvim") -- useful lua functions used ny lots of plugins
  use {"windwp/nvim-autopairs",
    config = "require('nvim-autopairs').setup{}"
  } -- Autopairs, integrates with both cmp and treesitter
  
  use("numToStr/Comment.nvim") -- commenting
  use("kyazdani42/nvim-web-devicons") -- colored icons in tree and status bar

  use { 'camspiers/lens.vim' } -- buffer autoresizing

  use {
    'noib3/nvim-cokeline', -- because bufferline sucks to configure
    requires = 'kyazdani42/nvim-web-devicons',
    config = "require('user.coke')",
  }

  -- use "moll/vim-bbye"
  -- use "nvim-lualine/lualine.nvim"
  use("akinsho/toggleterm.nvim")
  use("ternjs/tern_for_vim")
  -- use "ahmedkhalf/project.nvim"
  -- use "lewis6991/impatient.nvim"
  -- use "lukas-reineke/indent-blankline.nvim"
  -- use "goolord/alpha-nvim"
  -- use "antoinemadec/FixCursorHold.nvim" -- This is needed to fix lsp doc highlight
  use("folke/which-key.nvim") -- shows keymaps in a popup
  use("nvim-lualine/lualine.nvim") -- status bar
  --use "ervandew/supertab" -- unknown... ?
  use("terryma/vim-multiple-cursors") -- coule replace with visual x mode?
  -- use "akinsho/bufferline.nvim" -- not used?
  use({"petertriho/nvim-scrollbar",
    config = "require'scrollbar'.setup()"
  }) -- side scrollbar with git support
  use("norcalli/nvim-colorizer.lua") -- inline colors
  use("liuchengxu/vista.vim") -- symbols again?
  --use "glepnir/dashboard-nvim" -- dashboard

  -- Navigation
  use("justinmk/vim-sneak") -- 'S' followed by two characters to jump in line
  use("rinx/nvim-ripgrep") -- grep
  use("willthbill/opener.nvim") -- project manager
  -- use "brooth/far.vim" -- find and replace

  -- Colorschemes
  use("sainnhe/sonokai")
  use("lunarvim/colorschemes") -- collection of colorschemes
  use("lunarvim/darkplus.nvim")
  use("folke/tokyonight.nvim")
  use("marko-cerovac/material.nvim")
  use("joshdick/onedark.vim")
  use("catppuccin/nvim")
  -- use "dracula/vim"

  -- LSP
  use("neovim/nvim-lspconfig") -- enable LSP
  use("williamboman/nvim-lsp-installer") -- simple to use language server installer
  use("tamago324/nlsp-settings.nvim") -- language server settings defined in json for
  use("jose-elias-alvarez/null-ls.nvim") -- for formatters and linters
  use("ray-x/lsp_signature.nvim") -- signatures (functions etc)
  use("simrat39/symbols-outline.nvim") -- symbol outline sidebar
  -- use "mfussenegger/nvim-dap" -- debugging protocol
  use("MunifTanjim/prettier.nvim") -- formatting with Prettier

  -- Completion
  use({"hrsh7th/nvim-cmp",
    config = "require'user.autocomplete'"
  }) -- The completion plugin
  use("hrsh7th/cmp-buffer") -- buffer completions
  use({"hrsh7th/cmp-path",
    config = require'cmp'.setup {
    sources = {
      { name = 'path' }
    }
  }}) -- path completions for filesystem
  use("hrsh7th/cmp-cmdline") -- cmdline completions
  use("saadparwaiz1/cmp_luasnip") -- snippet completions
  use("hrsh7th/cmp-nvim-lsp") -- lsp completions
  use {"simrat39/rust-tools.nvim",
    config = "require 'user.rust-tools'"
  } -- extensions in addition to rust-analyzer

  -- snippets
  use("L3MON4D3/LuaSnip") --snippet engine
  use("rafamadriz/friendly-snippets") -- a bunch of snippets to use

  use {
    'saecki/crates.nvim',
    event = { "BufRead Cargo.toml" },
    requires = { { 'nvim-lua/plenary.nvim' } },
    config = function()
      require('user.cargo')
    end,
  }

  -- Telescope
  use("nvim-telescope/telescope.nvim")
  use("nvim-telescope/telescope-symbols.nvim")
  use("nvim-telescope/telescope-project.nvim")

  -- Treesitter
  use({
    "nvim-treesitter/nvim-treesitter",
    run = ":TSUpdate",
    config = "require('user.treesitter')"
  })
  use("JoosepAlviste/nvim-ts-context-commentstring")

  -- -- Git
  -- use "lewis6991/gitsigns.nvim"
  -- use "ttys3/nvim-blamer.lua" -- git blame
  --
  -- Automatically set up your configuration after cloning packer.nvim
  -- Put this at the end after all plugins
  if PACKER_BOOTSTRAP then
    require("packer").sync()
  end
end)
