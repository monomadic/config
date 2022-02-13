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
local package_path_str = "/Users/monomadic/.cache/nvim/packer_hererocks/2.1.0-beta3/share/lua/5.1/?.lua;/Users/monomadic/.cache/nvim/packer_hererocks/2.1.0-beta3/share/lua/5.1/?/init.lua;/Users/monomadic/.cache/nvim/packer_hererocks/2.1.0-beta3/lib/luarocks/rocks-5.1/?.lua;/Users/monomadic/.cache/nvim/packer_hererocks/2.1.0-beta3/lib/luarocks/rocks-5.1/?/init.lua"
local install_cpath_pattern = "/Users/monomadic/.cache/nvim/packer_hererocks/2.1.0-beta3/lib/lua/5.1/?.so"
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
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/Comment.nvim",
    url = "https://github.com/numToStr/Comment.nvim"
  },
  LuaSnip = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/LuaSnip",
    url = "https://github.com/L3MON4D3/LuaSnip"
  },
  ["bufferline.nvim"] = {
    config = { "\27LJ\2\n\v\0\0\1\0\0\0\1K\0\1\0\0" },
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/bufferline.nvim",
    url = "https://github.com/akinsho/bufferline.nvim"
  },
  ["cmp-buffer"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/cmp-buffer",
    url = "https://github.com/hrsh7th/cmp-buffer"
  },
  ["cmp-cmdline"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/cmp-cmdline",
    url = "https://github.com/hrsh7th/cmp-cmdline"
  },
  ["cmp-nvim-lsp"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/cmp-nvim-lsp",
    url = "https://github.com/hrsh7th/cmp-nvim-lsp"
  },
  ["cmp-path"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/cmp-path",
    url = "https://github.com/hrsh7th/cmp-path"
  },
  cmp_luasnip = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/cmp_luasnip",
    url = "https://github.com/saadparwaiz1/cmp_luasnip"
  },
  colorschemes = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/colorschemes",
    url = "https://github.com/lunarvim/colorschemes"
  },
  ["darkplus.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/darkplus.nvim",
    url = "https://github.com/lunarvim/darkplus.nvim"
  },
  ["friendly-snippets"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/friendly-snippets",
    url = "https://github.com/rafamadriz/friendly-snippets"
  },
  ["lsp_signature.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/lsp_signature.nvim",
    url = "https://github.com/ray-x/lsp_signature.nvim"
  },
  ["lualine.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/lualine.nvim",
    url = "https://github.com/nvim-lualine/lualine.nvim"
  },
  ["material.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/material.nvim",
    url = "https://github.com/marko-cerovac/material.nvim"
  },
  ["neo-tree.nvim"] = {
    config = { "\27LJ\2\n�\14\0\0\b\0'\00096\0\0\0009\0\1\0'\2\2\0B\0\2\0016\0\3\0'\2\4\0B\0\2\0029\0\5\0005\2\6\0005\3\b\0005\4\a\0=\4\t\0035\4\n\0=\4\v\0035\4\f\0=\4\r\0034\4\0\0=\4\14\3=\3\15\0025\3\17\0005\4\16\0=\4\18\0035\4\19\0005\5\20\0=\5\21\4=\4\22\3=\3\23\0025\3\24\0005\4\25\0005\5\26\0=\5\21\4=\4\22\3=\3\27\0025\3\30\0005\4\28\0005\5\29\0=\5\21\4=\4\22\3=\3\14\2B\0\2\0016\0\0\0009\0\31\0009\0 \0005\1!\0\18\2\0\0'\4\"\0'\5#\0'\6$\0\18\a\1\0B\2\5\1\18\2\0\0'\4\"\0'\5%\0'\6&\0\18\a\1\0B\2\5\1K\0\1\0\27:NeoTreeShowToggle<CR>\n<C-e>\28:NeoTreeFocusToggle<CR>\n<C-b>\5\1\0\2\fnoremap\2\vsilent\2\20nvim_set_keymap\bapi\1\0\0\1\0\19\6r\vrename\6C\15close_node\6x\21cut_to_clipboard\agc\15git_commit\6S\15open_split\6A\16git_add_all\18<1-LeftMouse>\topen\agu\21git_unstage_file\aga\17git_add_file\6s\16open_vsplit\6c\22copy_to_clipboard\6p\25paste_from_clipboard\6l\topen\agg\24git_commit_and_push\agp\rgit_push\agr\20git_revert_file\6R\frefresh\6d\vdelete\t<CR>\topen\1\0\1\rposition\nfloat\fbuffers\1\0\15\6r\vrename\6d\vdelete\6c\22copy_to_clipboard\6.\rset_root\6S\15open_split\t<cr>\topen\18<1-LeftMouse>\topen\6x\21cut_to_clipboard\6p\25paste_from_clipboard\6s\16open_vsplit\abd\18buffer_delete\6R\frefresh\6l\topen\6a\badd\t<bs>\16navigate_up\1\0\1\rposition\tleft\1\0\1\18show_unloaded\2\15filesystem\vwindow\rmappings\1\0\21\6r\vrename\6C\15close_node\6f\21filter_on_submit\6a\badd\6S\15open_split\t<cr>\topen\18<1-LeftMouse>\topen\6x\21cut_to_clipboard\6q\17close_window\6s\16open_vsplit\6p\25paste_from_clipboard\6l\topen\6/\17fuzzy_finder\6.\rset_root\n<c-x>\17clear_filter\6c\22copy_to_clipboard\6I\21toggle_gitignore\6R\frefresh\6H\18toggle_hidden\6d\vdelete\t<bs>\16navigate_up\1\0\2\rposition\tleft\nwidth\3(\ffilters\1\0\3\26hijack_netrw_behavior\17open_default\27use_libuv_file_watcher\1\24follow_current_file\2\1\0\2\16show_hidden\1\22respect_gitignore\2\30default_component_configs\15git_status\tname\1\0\2\26use_git_status_colors\2\19trailing_slash\1\ticon\1\0\4\18folder_closed\b\17folder_empty\bﰊ\16folder_open\b\fdefault\6*\vindent\1\0\0\1\0\6\14highlight\24NeoTreeIndentMarker\23last_indent_marker\b└\18indent_marker\b│\17with_markers\2\fpadding\3\1\16indent_size\3\2\1\0\4\23enable_diagnostics\2\22enable_git_status\2\23popup_border_style\frounded\25close_if_last_window\2\nsetup\rneo-tree\frequirez          hi link NeoTreeDirectoryName Directory\n          hi link NeoTreeDirectoryIcon NeoTreeDirectoryName\n        \bcmd\bvim\0" },
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/neo-tree.nvim",
    url = "https://github.com/nvim-neo-tree/neo-tree.nvim"
  },
  ["nlsp-settings.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/nlsp-settings.nvim",
    url = "https://github.com/tamago324/nlsp-settings.nvim"
  },
  ["nui.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/nui.nvim",
    url = "https://github.com/MunifTanjim/nui.nvim"
  },
  ["null-ls.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/null-ls.nvim",
    url = "https://github.com/jose-elias-alvarez/null-ls.nvim"
  },
  nvim = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/nvim",
    url = "https://github.com/catppuccin/nvim"
  },
  ["nvim-cmp"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/nvim-cmp",
    url = "https://github.com/hrsh7th/nvim-cmp"
  },
  ["nvim-colorizer.lua"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/nvim-colorizer.lua",
    url = "https://github.com/norcalli/nvim-colorizer.lua"
  },
  ["nvim-lsp-installer"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/nvim-lsp-installer",
    url = "https://github.com/williamboman/nvim-lsp-installer"
  },
  ["nvim-lspconfig"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/nvim-lspconfig",
    url = "https://github.com/neovim/nvim-lspconfig"
  },
  ["nvim-ripgrep"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/nvim-ripgrep",
    url = "https://github.com/rinx/nvim-ripgrep"
  },
  ["nvim-scrollbar"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/nvim-scrollbar",
    url = "https://github.com/petertriho/nvim-scrollbar"
  },
  ["nvim-tree.lua"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/nvim-tree.lua",
    url = "https://github.com/kyazdani42/nvim-tree.lua"
  },
  ["nvim-treesitter"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/nvim-treesitter",
    url = "https://github.com/nvim-treesitter/nvim-treesitter"
  },
  ["nvim-ts-context-commentstring"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/nvim-ts-context-commentstring",
    url = "https://github.com/JoosepAlviste/nvim-ts-context-commentstring"
  },
  ["nvim-web-devicons"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/nvim-web-devicons",
    url = "https://github.com/kyazdani42/nvim-web-devicons"
  },
  ["onedark.vim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/onedark.vim",
    url = "https://github.com/joshdick/onedark.vim"
  },
  ["opener.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/opener.nvim",
    url = "https://github.com/willthbill/opener.nvim"
  },
  ["packer.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/packer.nvim",
    url = "https://github.com/wbthomason/packer.nvim"
  },
  ["plenary.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/plenary.nvim",
    url = "https://github.com/nvim-lua/plenary.nvim"
  },
  popfix = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/popfix",
    url = "https://github.com/RishabhRD/popfix"
  },
  ["popui.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/popui.nvim",
    url = "https://github.com/hood/popui.nvim"
  },
  ["prettier.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/prettier.nvim",
    url = "https://github.com/MunifTanjim/prettier.nvim"
  },
  sonokai = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/sonokai",
    url = "https://github.com/sainnhe/sonokai"
  },
  ["symbols-outline.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/symbols-outline.nvim",
    url = "https://github.com/simrat39/symbols-outline.nvim"
  },
  ["telescope-project.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/telescope-project.nvim",
    url = "https://github.com/nvim-telescope/telescope-project.nvim"
  },
  ["telescope-symbols.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/telescope-symbols.nvim",
    url = "https://github.com/nvim-telescope/telescope-symbols.nvim"
  },
  ["telescope.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/telescope.nvim",
    url = "https://github.com/nvim-telescope/telescope.nvim"
  },
  tern_for_vim = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/tern_for_vim",
    url = "https://github.com/ternjs/tern_for_vim"
  },
  ["toggleterm.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/toggleterm.nvim",
    url = "https://github.com/akinsho/toggleterm.nvim"
  },
  ["tokyonight.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/tokyonight.nvim",
    url = "https://github.com/folke/tokyonight.nvim"
  },
  ["vim-multiple-cursors"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/vim-multiple-cursors",
    url = "https://github.com/terryma/vim-multiple-cursors"
  },
  ["vim-sneak"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/vim-sneak",
    url = "https://github.com/justinmk/vim-sneak"
  },
  ["vista.vim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/vista.vim",
    url = "https://github.com/liuchengxu/vista.vim"
  },
  ["which-key.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/which-key.nvim",
    url = "https://github.com/folke/which-key.nvim"
  }
}

time([[Defining packer_plugins]], false)
-- Config for: neo-tree.nvim
time([[Config for neo-tree.nvim]], true)
try_loadstring("\27LJ\2\n�\14\0\0\b\0'\00096\0\0\0009\0\1\0'\2\2\0B\0\2\0016\0\3\0'\2\4\0B\0\2\0029\0\5\0005\2\6\0005\3\b\0005\4\a\0=\4\t\0035\4\n\0=\4\v\0035\4\f\0=\4\r\0034\4\0\0=\4\14\3=\3\15\0025\3\17\0005\4\16\0=\4\18\0035\4\19\0005\5\20\0=\5\21\4=\4\22\3=\3\23\0025\3\24\0005\4\25\0005\5\26\0=\5\21\4=\4\22\3=\3\27\0025\3\30\0005\4\28\0005\5\29\0=\5\21\4=\4\22\3=\3\14\2B\0\2\0016\0\0\0009\0\31\0009\0 \0005\1!\0\18\2\0\0'\4\"\0'\5#\0'\6$\0\18\a\1\0B\2\5\1\18\2\0\0'\4\"\0'\5%\0'\6&\0\18\a\1\0B\2\5\1K\0\1\0\27:NeoTreeShowToggle<CR>\n<C-e>\28:NeoTreeFocusToggle<CR>\n<C-b>\5\1\0\2\fnoremap\2\vsilent\2\20nvim_set_keymap\bapi\1\0\0\1\0\19\6r\vrename\6C\15close_node\6x\21cut_to_clipboard\agc\15git_commit\6S\15open_split\6A\16git_add_all\18<1-LeftMouse>\topen\agu\21git_unstage_file\aga\17git_add_file\6s\16open_vsplit\6c\22copy_to_clipboard\6p\25paste_from_clipboard\6l\topen\agg\24git_commit_and_push\agp\rgit_push\agr\20git_revert_file\6R\frefresh\6d\vdelete\t<CR>\topen\1\0\1\rposition\nfloat\fbuffers\1\0\15\6r\vrename\6d\vdelete\6c\22copy_to_clipboard\6.\rset_root\6S\15open_split\t<cr>\topen\18<1-LeftMouse>\topen\6x\21cut_to_clipboard\6p\25paste_from_clipboard\6s\16open_vsplit\abd\18buffer_delete\6R\frefresh\6l\topen\6a\badd\t<bs>\16navigate_up\1\0\1\rposition\tleft\1\0\1\18show_unloaded\2\15filesystem\vwindow\rmappings\1\0\21\6r\vrename\6C\15close_node\6f\21filter_on_submit\6a\badd\6S\15open_split\t<cr>\topen\18<1-LeftMouse>\topen\6x\21cut_to_clipboard\6q\17close_window\6s\16open_vsplit\6p\25paste_from_clipboard\6l\topen\6/\17fuzzy_finder\6.\rset_root\n<c-x>\17clear_filter\6c\22copy_to_clipboard\6I\21toggle_gitignore\6R\frefresh\6H\18toggle_hidden\6d\vdelete\t<bs>\16navigate_up\1\0\2\rposition\tleft\nwidth\3(\ffilters\1\0\3\26hijack_netrw_behavior\17open_default\27use_libuv_file_watcher\1\24follow_current_file\2\1\0\2\16show_hidden\1\22respect_gitignore\2\30default_component_configs\15git_status\tname\1\0\2\26use_git_status_colors\2\19trailing_slash\1\ticon\1\0\4\18folder_closed\b\17folder_empty\bﰊ\16folder_open\b\fdefault\6*\vindent\1\0\0\1\0\6\14highlight\24NeoTreeIndentMarker\23last_indent_marker\b└\18indent_marker\b│\17with_markers\2\fpadding\3\1\16indent_size\3\2\1\0\4\23enable_diagnostics\2\22enable_git_status\2\23popup_border_style\frounded\25close_if_last_window\2\nsetup\rneo-tree\frequirez          hi link NeoTreeDirectoryName Directory\n          hi link NeoTreeDirectoryIcon NeoTreeDirectoryName\n        \bcmd\bvim\0", "config", "neo-tree.nvim")
time([[Config for neo-tree.nvim]], false)
-- Config for: bufferline.nvim
time([[Config for bufferline.nvim]], true)
try_loadstring("\27LJ\2\n\v\0\0\1\0\0\0\1K\0\1\0\0", "config", "bufferline.nvim")
time([[Config for bufferline.nvim]], false)
if should_profile then save_profiles() end

end)

if not no_errors then
  vim.api.nvim_command('echohl ErrorMsg | echom "Error in packer_compiled: '..error_msg..'" | echom "Please check your config for correctness" | echohl None')
end
