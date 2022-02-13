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
  use({
    "nvim-neo-tree/neo-tree.nvim",
    requires = {
      "nvim-lua/plenary.nvim",
      "kyazdani42/nvim-web-devicons",
      "MunifTanjim/nui.nvim",
    },
    config = require("user.neotree"),
  })
  use("MunifTanjim/nui.nvim") -- neotree dep

  -- UI
  -- use "nvim-lua/popup.nvim" -- An implementation of the Popup API from vim in Neovim
  use("RishabhRD/popfix") -- popupui (required by popui)
  use("hood/popui.nvim")

  use("nvim-lua/plenary.nvim") -- useful lua functions used ny lots of plugins
  -- use "windwp/nvim-autopairs" -- Autopairs, integrates with both cmp and treesitter
  use("numToStr/Comment.nvim") -- commenting
  use("kyazdani42/nvim-web-devicons") -- colored icons in tree and status bar
  use("kyazdani42/nvim-tree.lua") -- directory structure tree
  -- use "akinsho/bufferline.nvim"
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
  use("petertriho/nvim-scrollbar") -- side scrollbar with git support
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

  -- Completion
  use("hrsh7th/nvim-cmp") -- The completion plugin
  use("hrsh7th/cmp-buffer") -- buffer completions
  use("hrsh7th/cmp-path") -- path completions
  use("hrsh7th/cmp-cmdline") -- cmdline completions
  use("saadparwaiz1/cmp_luasnip") -- snippet completions
  use("hrsh7th/cmp-nvim-lsp")
  -- use "simrat39/rust-tools.vim" -- needed?

  -- -- snippets
  use("L3MON4D3/LuaSnip") --snippet engine
  use("rafamadriz/friendly-snippets") -- a bunch of snippets to use

  -- LSP
  use("neovim/nvim-lspconfig") -- enable LSP
  use("williamboman/nvim-lsp-installer") -- simple to use language server installer
  use("tamago324/nlsp-settings.nvim") -- language server settings defined in json for
  use("jose-elias-alvarez/null-ls.nvim") -- for formatters and linters
  use("ray-x/lsp_signature.nvim") -- signatures (functions etc)
  use("simrat39/symbols-outline.nvim") -- symbol outline sidebar
  -- use "mfussenegger/nvim-dap" -- debugging protocol
  use("MunifTanjim/prettier.nvim") -- formatting with Prettier

  -- Telescope
  use("nvim-telescope/telescope.nvim")
  use("nvim-telescope/telescope-symbols.nvim")
  use("nvim-telescope/telescope-project.nvim")

  -- Treesitter
  use({
    "nvim-treesitter/nvim-treesitter",
    run = ":TSUpdate",
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
