-- Automatically generated packer.nvim plugin loader code

if vim.api.nvim_call_function('has', {'nvim-0.5'}) ~= 1 then
  vim.api.nvim_command('echohl WarningMsg | echom "Invalid Neovim version for packer.nvim! | echohl None"')
  return
end

vim.api.nvim_command('packadd packer.nvim')

local no_errors, error_msg = pcall(function()

  local time
  local profile_info
  local should_profile = false
  if should_profile then
    local hrtime = vim.loop.hrtime
    profile_info = {}
    time = function(chunk, start)
      if start then
        profile_info[chunk] = hrtime()
      else
        profile_info[chunk] = (hrtime() - profile_info[chunk]) / 1e6
      end
    end
  else
    time = function(chunk, start) end
  end
  
local function save_profiles(threshold)
  local sorted_times = {}
  for chunk_name, time_taken in pairs(profile_info) do
    sorted_times[#sorted_times + 1] = {chunk_name, time_taken}
  end
  table.sort(sorted_times, function(a, b) return a[2] > b[2] end)
  local results = {}
  for i, elem in ipairs(sorted_times) do
    if not threshold or threshold and elem[2] > threshold then
      results[i] = elem[1] .. ' took ' .. elem[2] .. 'ms'
    end
  end

  _G._packer = _G._packer or {}
  _G._packer.profile_output = results
end

time([[Luarocks path setup]], true)
local package_path_str = "/home/mono/.cache/nvim/packer_hererocks/2.1.0-beta3/share/lua/5.1/?.lua;/home/mono/.cache/nvim/packer_hererocks/2.1.0-beta3/share/lua/5.1/?/init.lua;/home/mono/.cache/nvim/packer_hererocks/2.1.0-beta3/lib/luarocks/rocks-5.1/?.lua;/home/mono/.cache/nvim/packer_hererocks/2.1.0-beta3/lib/luarocks/rocks-5.1/?/init.lua"
local install_cpath_pattern = "/home/mono/.cache/nvim/packer_hererocks/2.1.0-beta3/lib/lua/5.1/?.so"
if not string.find(package.path, package_path_str, 1, true) then
  package.path = package.path .. ';' .. package_path_str
end

if not string.find(package.cpath, install_cpath_pattern, 1, true) then
  package.cpath = package.cpath .. ';' .. install_cpath_pattern
end

time([[Luarocks path setup]], false)
time([[try_loadstring definition]], true)
local function try_loadstring(s, component, name)
  local success, result = pcall(loadstring(s), name, _G.packer_plugins[name])
  if not success then
    vim.schedule(function()
      vim.api.nvim_notify('packer.nvim: Error running ' .. component .. ' for ' .. name .. ': ' .. result, vim.log.levels.ERROR, {})
    end)
  end
  return result
end

time([[try_loadstring definition]], false)
time([[Defining packer_plugins]], true)
_G.packer_plugins = {
  ["Comment.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/Comment.nvim",
    url = "https://github.com/numToStr/Comment.nvim"
  },
  ["aerial.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/aerial.nvim",
    url = "https://github.com/stevearc/aerial.nvim"
  },
  ["barbar.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/barbar.nvim",
    url = "https://github.com/romgrk/barbar.nvim"
  },
  ["boo-colorscheme-nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/boo-colorscheme-nvim",
    url = "https://github.com/rockerBOO/boo-colorscheme-nvim"
  },
  ["cmp-nvim-lsp"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/cmp-nvim-lsp",
    url = "https://github.com/hrsh7th/cmp-nvim-lsp"
  },
  ["codi.vim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/codi.vim",
    url = "https://github.com/metakirby5/codi.vim"
  },
  colorschemes = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/colorschemes",
    url = "https://github.com/lunarvim/colorschemes"
  },
  ["crates.nvim"] = {
    after_files = { "/home/mono/.local/share/nvim/site/pack/packer/opt/crates.nvim/after/plugin/cmp_crates.lua" },
    config = { "\27LJ\2\n-\0\0\3\0\2\0\0046\0\0\0'\2\1\0B\0\2\1K\0\1\0\18plugins.cargo\frequire\0" },
    loaded = false,
    needs_bufread = false,
    only_cond = false,
    path = "/home/mono/.local/share/nvim/site/pack/packer/opt/crates.nvim",
    url = "https://github.com/saecki/crates.nvim"
  },
  ["darkplus.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/darkplus.nvim",
    url = "https://github.com/lunarvim/darkplus.nvim"
  },
  ["doom-one.vim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/doom-one.vim",
    url = "https://github.com/romgrk/doom-one.vim"
  },
  ["emmet-vim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/emmet-vim",
    url = "https://github.com/mattn/emmet-vim"
  },
  ["galaxyline.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/galaxyline.nvim",
    url = "https://github.com/glepnir/galaxyline.nvim"
  },
  ["gitsigns.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/gitsigns.nvim",
    url = "https://github.com/lewis6991/gitsigns.nvim"
  },
  ["hop.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/hop.nvim",
    url = "https://github.com/phaazon/hop.nvim"
  },
  ["html-snippets"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/html-snippets",
    url = "https://github.com/ChristianChiarulli/html-snippets"
  },
  ["indent-blankline.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/indent-blankline.nvim",
    url = "https://github.com/lukas-reineke/indent-blankline.nvim"
  },
  ["java-snippets"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/java-snippets",
    url = "https://github.com/ChristianChiarulli/java-snippets"
  },
  ["lens.vim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/lens.vim",
    url = "https://github.com/camspiers/lens.vim"
  },
  ["lsp_signature.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/lsp_signature.nvim",
    url = "https://github.com/ray-x/lsp_signature.nvim"
  },
  ["material.vim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/material.vim",
    url = "https://github.com/kaicataldo/material.vim"
  },
  ["neo-tree.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/neo-tree.nvim",
    url = "https://github.com/nvim-neo-tree/neo-tree.nvim"
  },
  neoformat = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/neoformat",
    url = "https://github.com/sbdchd/neoformat"
  },
  ["neoscroll.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/neoscroll.nvim",
    url = "https://github.com/karb94/neoscroll.nvim"
  },
  ["nui.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/nui.nvim",
    url = "https://github.com/MunifTanjim/nui.nvim"
  },
  ["null-ls.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/null-ls.nvim",
    url = "https://github.com/jose-elias-alvarez/null-ls.nvim"
  },
  ["nvim-autopairs"] = {
    config = { "require('nvim-autopairs').setup{}" },
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/nvim-autopairs",
    url = "https://github.com/windwp/nvim-autopairs"
  },
  ["nvim-blamer.lua"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/nvim-blamer.lua",
    url = "https://github.com/ttys3/nvim-blamer.lua"
  },
  ["nvim-bqf"] = {
    loaded = false,
    needs_bufread = true,
    only_cond = false,
    path = "/home/mono/.local/share/nvim/site/pack/packer/opt/nvim-bqf",
    url = "https://github.com/kevinhwang91/nvim-bqf"
  },
  ["nvim-cmp"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/nvim-cmp",
    url = "https://github.com/hrsh7th/nvim-cmp"
  },
  ["nvim-colorizer.lua"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/nvim-colorizer.lua",
    url = "https://github.com/norcalli/nvim-colorizer.lua"
  },
  ["nvim-compe"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/nvim-compe",
    url = "https://github.com/hrsh7th/nvim-compe"
  },
  ["nvim-lightbulb"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/nvim-lightbulb",
    url = "https://github.com/kosayoda/nvim-lightbulb"
  },
  ["nvim-lsp-installer"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/nvim-lsp-installer",
    url = "https://github.com/williamboman/nvim-lsp-installer"
  },
  ["nvim-lspconfig"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/nvim-lspconfig",
    url = "https://github.com/neovim/nvim-lspconfig"
  },
  ["nvim-ripgrep"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/nvim-ripgrep",
    url = "https://github.com/rinx/nvim-ripgrep"
  },
  ["nvim-scrollbar"] = {
    config = { "require'scrollbar'.setup()" },
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/nvim-scrollbar",
    url = "https://github.com/petertriho/nvim-scrollbar"
  },
  ["nvim-treehopper"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/nvim-treehopper",
    url = "https://github.com/mfussenegger/nvim-treehopper"
  },
  ["nvim-treesitter"] = {
    config = { "require('plugins.treesitter')" },
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/nvim-treesitter",
    url = "https://github.com/nvim-treesitter/nvim-treesitter"
  },
  ["nvim-ts-context-commentstring"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/nvim-ts-context-commentstring",
    url = "https://github.com/JoosepAlviste/nvim-ts-context-commentstring"
  },
  ["nvim-ts-rainbow"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/nvim-ts-rainbow",
    url = "https://github.com/p00f/nvim-ts-rainbow"
  },
  ["nvim-web-devicons"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/nvim-web-devicons",
    url = "https://github.com/kyazdani42/nvim-web-devicons"
  },
  ["onedark.vim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/onedark.vim",
    url = "https://github.com/joshdick/onedark.vim"
  },
  ["opener.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/opener.nvim",
    url = "https://github.com/willthbill/opener.nvim"
  },
  ["packer.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/packer.nvim",
    url = "https://github.com/wbthomason/packer.nvim"
  },
  ["plenary.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/plenary.nvim",
    url = "https://github.com/nvim-lua/plenary.nvim"
  },
  popfix = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/popfix",
    url = "https://github.com/RishabhRD/popfix"
  },
  ["popui.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/popui.nvim",
    url = "https://github.com/hood/popui.nvim"
  },
  ["prettier.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/prettier.nvim",
    url = "https://github.com/MunifTanjim/prettier.nvim"
  },
  ["python-snippets"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/python-snippets",
    url = "https://github.com/ChristianChiarulli/python-snippets"
  },
  ["rust-tools.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/rust-tools.nvim",
    url = "https://github.com/simrat39/rust-tools.nvim"
  },
  ["sidebar.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/sidebar.nvim",
    url = "https://github.com/sidebar-nvim/sidebar.nvim"
  },
  sonokai = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/sonokai",
    url = "https://github.com/sainnhe/sonokai"
  },
  ["split-term.vim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/split-term.vim",
    url = "https://github.com/vimlab/split-term.vim"
  },
  srcery = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/srcery",
    url = "https://github.com/srcery-colors/srcery-vim"
  },
  ["surround.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/surround.nvim",
    url = "https://github.com/appelgriebsch/surround.nvim"
  },
  ["symbols-outline.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/symbols-outline.nvim",
    url = "https://github.com/simrat39/symbols-outline.nvim"
  },
  ["telescope-project.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/telescope-project.nvim",
    url = "https://github.com/nvim-telescope/telescope-project.nvim"
  },
  ["telescope-symbols.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/telescope-symbols.nvim",
    url = "https://github.com/nvim-telescope/telescope-symbols.nvim"
  },
  ["telescope.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/telescope.nvim",
    url = "https://github.com/nvim-telescope/telescope.nvim"
  },
  tern_for_vim = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/tern_for_vim",
    url = "https://github.com/ternjs/tern_for_vim"
  },
  ["todo-comments.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/todo-comments.nvim",
    url = "https://github.com/folke/todo-comments.nvim"
  },
  ["tokyonight.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/tokyonight.nvim",
    url = "https://github.com/folke/tokyonight.nvim"
  },
  ["trouble.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/trouble.nvim",
    url = "https://github.com/folke/trouble.nvim"
  },
  ["vim-clap"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/vim-clap",
    url = "https://github.com/liuchengxu/vim-clap"
  },
  ["vim-floaterm"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/vim-floaterm",
    url = "https://github.com/voldikss/vim-floaterm"
  },
  ["vim-moonfly-colors"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/vim-moonfly-colors",
    url = "https://github.com/bluz71/vim-moonfly-colors"
  },
  ["vim-nightfly-guicolors"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/vim-nightfly-guicolors",
    url = "https://github.com/bluz71/vim-nightfly-guicolors"
  },
  ["vim-solidity"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/vim-solidity",
    url = "https://github.com/tomlion/vim-solidity"
  },
  ["vim-vsnip"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/vim-vsnip",
    url = "https://github.com/hrsh7th/vim-vsnip"
  },
  ["vista.vim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/vista.vim",
    url = "https://github.com/liuchengxu/vista.vim"
  },
  ["vscode-es7-javascript-react-snippets"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/vscode-es7-javascript-react-snippets",
    url = "https://github.com/dsznajder/vscode-es7-javascript-react-snippets"
  },
  ["vscode-go"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/vscode-go",
    url = "https://github.com/golang/vscode-go"
  },
  ["vscode-javascript"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/vscode-javascript",
    url = "https://github.com/xabikos/vscode-javascript"
  },
  ["vscode-rust"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/vscode-rust",
    url = "https://github.com/rust-lang/vscode-rust"
  },
  ["which-key.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/which-key.nvim",
    url = "https://github.com/folke/which-key.nvim"
  },
  ["wilder.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/wilder.nvim",
    url = "https://github.com/gelguy/wilder.nvim"
  },
  ["zen-mode.nvim"] = {
    loaded = true,
    path = "/home/mono/.local/share/nvim/site/pack/packer/start/zen-mode.nvim",
    url = "https://github.com/folke/zen-mode.nvim"
  }
}

time([[Defining packer_plugins]], false)
-- Config for: nvim-scrollbar
time([[Config for nvim-scrollbar]], true)
require'scrollbar'.setup()
time([[Config for nvim-scrollbar]], false)
-- Config for: nvim-treesitter
time([[Config for nvim-treesitter]], true)
require('plugins.treesitter')
time([[Config for nvim-treesitter]], false)
-- Config for: nvim-autopairs
time([[Config for nvim-autopairs]], true)
require('nvim-autopairs').setup{}
time([[Config for nvim-autopairs]], false)
vim.cmd [[augroup packer_load_aucmds]]
vim.cmd [[au!]]
  -- Filetype lazy-loads
time([[Defining lazy-load filetype autocommands]], true)
vim.cmd [[au FileType qf ++once lua require("packer.load")({'nvim-bqf'}, { ft = "qf" }, _G.packer_plugins)]]
time([[Defining lazy-load filetype autocommands]], false)
  -- Event lazy-loads
time([[Defining lazy-load event autocommands]], true)
vim.cmd [[au BufRead Cargo.toml ++once lua require("packer.load")({'crates.nvim'}, { event = "BufRead Cargo.toml" }, _G.packer_plugins)]]
time([[Defining lazy-load event autocommands]], false)
vim.cmd("augroup END")
if should_profile then save_profiles() end

end)

if not no_errors then
  error_msg = error_msg:gsub('"', '\\"')
  vim.api.nvim_command('echohl ErrorMsg | echom "Error in packer_compiled: '..error_msg..'" | echom "Please check your config for correctness" | echohl None')
end
