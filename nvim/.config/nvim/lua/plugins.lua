local fn = vim.fn
local install_path = fn.stdpath("data") .. "/site/pack/packer/start/packer.nvim"
if fn.empty(fn.glob(install_path)) > 0 then
  fn.system({ "git", "clone", "--depth", "1", "https://github.com/wbthomason/packer.nvim", install_path })
  vim.cmd("packadd packer.nvim")
end

require("packer").startup(function(use)
  use("wbthomason/packer.nvim")

  -- Theme
  use("folke/tokyonight.nvim")
  use("bluz71/vim-nightfly-guicolors")
  use("kaicataldo/material.vim")
  use("rockerBOO/boo-colorscheme-nvim")
  use("sainnhe/sonokai")
  use("lunarvim/colorschemes") -- collection of colorschemes
  use("lunarvim/darkplus.nvim")
  --use("marko-cerovac/material.nvim")
  use("joshdick/onedark.vim")
  --use("catppuccin/nvim")
  -- use "dracula/vim"
  use({ "srcery-colors/srcery-vim", as = "srcery" })
  use("bluz71/vim-moonfly-colors")

  -- Status line
  use({ "glepnir/galaxyline.nvim", requires = "kyazdani42/nvim-web-devicons" })
  --use({ "feline-nvim/feline.nvim", requires = "kyazdani42/nvim-web-devicons" })
  --use("yamatsum/nvim-nonicons")
  use("lukas-reineke/indent-blankline.nvim")

  -- Tree
  use({
    "nvim-neo-tree/neo-tree.nvim",
    branch = "v1.x",
    requires = {
      "nvim-lua/plenary.nvim",
      "kyazdani42/nvim-web-devicons",
      "MunifTanjim/nui.nvim",
    },
  })
  use("ms-jpq/chadtree")

  use("sbdchd/neoformat")

  use({ "liuchengxu/vim-clap" })

  -- UI
  -- use "nvim-lua/popup.nvim" -- An implementation of the Popup API from vim in Neovim
  use("RishabhRD/popfix") -- popupui (required by popui)
  use("hood/popui.nvim")

  use("nvim-lua/plenary.nvim") -- useful lua functions used ny lots of plugins
  use({ "windwp/nvim-autopairs", config = "require('nvim-autopairs').setup{}" }) -- Autopairs, integrates with both cmp and treesitter

  use("numToStr/Comment.nvim") -- commenting
  use("kyazdani42/nvim-web-devicons") -- colored icons in tree and status bar

  use({ "camspiers/lens.vim" }) -- buffer autoresizing

  use({
    "noib3/nvim-cokeline", -- because bufferline sucks to configure
    requires = "kyazdani42/nvim-web-devicons",
    config = "require('plugins.cokeline')",
  })

  -- use "moll/vim-bbye"
  -- use "nvim-lualine/lualine.nvim"

  use({ "kevinhwang91/nvim-bqf", ft = "qf" })

  -- Terminal
  -- use("akinsho/toggleterm.nvim")
  use("voldikss/vim-floaterm")
  use("vimlab/split-term.vim")
  use("ternjs/tern_for_vim")

  -- use "ahmedkhalf/project.nvim"
  -- use "lewis6991/impatient.nvim"
  -- use "lukas-reineke/indent-blankline.nvim"
  -- use "goolord/alpha-nvim"
  -- use "antoinemadec/FixCursorHold.nvim" -- This is needed to fix lsp doc highlight
  use("folke/which-key.nvim") -- shows keymaps in a popup
  --use("nvim-lualine/lualine.nvim") -- status bar
  --use "ervandew/supertab" -- unknown... ?
  --use("terryma/vim-multiple-cursors") -- coule replace with visual x mode?
  -- use "akinsho/bufferline.nvim" -- not used?
  use({ "petertriho/nvim-scrollbar", config = "require'scrollbar'.setup()" }) -- side scrollbar with git support
  use({ "karb94/neoscroll.nvim", config = "require'neoscroll'.setup()" })
  use("norcalli/nvim-colorizer.lua") -- inline colors
  use("liuchengxu/vista.vim") -- symbols again?
  --use "glepnir/dashboard-nvim" -- dashboard

  -- Navigation
  use("justinmk/vim-sneak") -- 'S' followed by two characters to jump in line
  use("rinx/nvim-ripgrep") -- grep
  use("willthbill/opener.nvim") -- project manager
  -- use "brooth/far.vim" -- find and replace

  -- LSP
  use("neovim/nvim-lspconfig") -- enable LSP
  use("williamboman/nvim-lsp-installer") -- simple to use language server installer
  -- use 'onsails/lspkind-nvim' -- icons for lsp complete
  use("kosayoda/nvim-lightbulb")
  -- use 'mfussenegger/nvim-jdtls'
  use("tomlion/vim-solidity")
  --use("tamago324/nlsp-settings.nvim") -- language server settings defined in json for

  use("jose-elias-alvarez/null-ls.nvim") -- for formatters and linters
  use("ray-x/lsp_signature.nvim") -- signatures (functions etc)
  use("simrat39/symbols-outline.nvim") -- symbol outline sidebar
  -- use "mfussenegger/nvim-dap" -- debugging protocol
  use("MunifTanjim/prettier.nvim") -- formatting with Prettier

  use({
    "simrat39/rust-tools.nvim",
    --config = "require 'plugins.rust-tools'"
  }) -- extensions in addition to rust-analyzer

  use({
    "appelgriebsch/surround.nvim",
  })

  use({
    "folke/trouble.nvim",
    requires = "kyazdani42/nvim-web-devicons",
  })

  -- Autocomplete
  use("hrsh7th/nvim-cmp")
  use("hrsh7th/cmp-nvim-lsp")
  use("hrsh7th/nvim-compe")
  use("mattn/emmet-vim")
  use("hrsh7th/vim-vsnip")
  use("xabikos/vscode-javascript")
  use("dsznajder/vscode-es7-javascript-react-snippets")
  use("golang/vscode-go")
  use("rust-lang/vscode-rust")
  use("ChristianChiarulli/html-snippets")
  use("ChristianChiarulli/java-snippets")
  use("ChristianChiarulli/python-snippets")

  use({
    "saecki/crates.nvim",
    event = { "BufRead Cargo.toml" },
    requires = { { "nvim-lua/plenary.nvim" } },
    config = function()
      require("plugins.cargo")
    end,
  })

  -- Telescope
  use("nvim-telescope/telescope.nvim")
  use("nvim-telescope/telescope-symbols.nvim")
  use("nvim-telescope/telescope-project.nvim")
  --use("nvim-telescope/telescope-frecency.nvim")

  -- Treesitter
  use({
    "nvim-treesitter/nvim-treesitter",
    run = ":TSUpdate",
    requires = {
      { "p00f/nvim-ts-rainbow" },
      { "mfussenegger/nvim-treehopper" },
      -- { "folke/twilight.nvim" },
      { "folke/zen-mode.nvim" },
    },
    config = "require('plugins.treesitter')",
  })
  use("JoosepAlviste/nvim-ts-context-commentstring")

  -- -- Git
  use({ "lewis6991/gitsigns.nvim", requires = {
    "nvim-lua/plenary.nvim",
  } })
  -- airblade/vim-gitgutter
  use("ttys3/nvim-blamer.lua") -- git blame
  --
  -- Automatically set up your configuration after cloning packer.nvim
  -- Put this at the end after all plugins
  if PACKER_BOOTSTRAP then
    require("packer").sync()
  end
end)
