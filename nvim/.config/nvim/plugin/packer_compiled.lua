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
  ["neo-tree.nvim"] = {
    config = { "\27LJ\2\n�\14\0\0\6\0!\0-6\0\0\0009\0\1\0'\2\2\0B\0\2\0016\0\3\0'\2\4\0B\0\2\0029\0\5\0005\2\6\0005\3\b\0005\4\a\0=\4\t\0035\4\n\0=\4\v\0035\4\f\0=\4\r\0035\4\14\0=\4\15\3=\3\16\0025\3\18\0005\4\17\0=\4\19\0035\4\20\0005\5\21\0=\5\22\4=\4\23\3=\3\24\0025\3\25\0005\4\26\0005\5\27\0=\5\22\4=\4\23\3=\3\28\0025\3\31\0005\4\29\0005\5\30\0=\5\22\4=\4\23\3=\3\15\2B\0\2\0016\0\0\0009\0\1\0'\2 \0B\0\2\1K\0\1\0\"nnoremap \\ :NeoTreeReveal<cr>\1\0\0\1\0\19\t<CR>\topen\6x\21cut_to_clipboard\6d\vdelete\6R\frefresh\6c\22copy_to_clipboard\6p\25paste_from_clipboard\6r\vrename\agc\15git_commit\agp\rgit_push\6C\15close_node\agg\24git_commit_and_push\6A\16git_add_all\18<1-LeftMouse>\topen\aga\17git_add_file\6s\16open_vsplit\agu\21git_unstage_file\6S\15open_split\6l\topen\agr\20git_revert_file\1\0\1\rposition\nfloat\fbuffers\1\0\15\6.\rset_root\t<bs>\16navigate_up\6a\badd\6R\frefresh\6s\16open_vsplit\6p\25paste_from_clipboard\6r\vrename\6c\22copy_to_clipboard\abd\18buffer_delete\6x\21cut_to_clipboard\6S\15open_split\6l\topen\6d\vdelete\t<cr>\topen\18<1-LeftMouse>\topen\1\0\1\rposition\tleft\1\0\1\18show_unloaded\2\15filesystem\vwindow\rmappings\1\0\21\6/\17fuzzy_finder\n<c-x>\17clear_filter\6I\21toggle_gitignore\6a\badd\6c\22copy_to_clipboard\6p\25paste_from_clipboard\6r\vrename\6H\18toggle_hidden\6R\frefresh\6C\15close_node\t<bs>\16navigate_up\6x\21cut_to_clipboard\6l\topen\6q\17close_window\t<cr>\topen\18<1-LeftMouse>\topen\6s\16open_vsplit\6f\21filter_on_submit\6S\15open_split\6.\rset_root\6d\vdelete\1\0\2\nwidth\3(\rposition\tleft\ffilters\1\0\3\26hijack_netrw_behavior\17open_default\27use_libuv_file_watcher\1\24follow_current_file\1\1\0\2\22respect_gitignore\2\16show_hidden\1\30default_component_configs\15git_status\1\0\1\14highlight\19NeoTreeDimText\tname\1\0\2\26use_git_status_colors\2\19trailing_slash\1\ticon\1\0\4\16folder_open\b\17folder_empty\bﰊ\18folder_closed\b\fdefault\6*\vindent\1\0\0\1\0\6\16indent_size\3\2\17with_markers\2\14highlight\24NeoTreeIndentMarker\23last_indent_marker\b└\18indent_marker\b│\fpadding\3\1\1\0\4\23enable_diagnostics\2\22enable_git_status\2\23popup_border_style\frounded\25close_if_last_window\2\nsetup\rneo-tree\frequirez          hi link NeoTreeDirectoryName Directory\n          hi link NeoTreeDirectoryIcon NeoTreeDirectoryName\n        \bcmd\bvim\0" },
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/neo-tree.nvim",
    url = "https://github.com/nvim-neo-tree/neo-tree.nvim"
  },
  ["nui.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/nui.nvim",
    url = "https://github.com/MunifTanjim/nui.nvim"
  },
  ["nvim-web-devicons"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/nvim-web-devicons",
    url = "https://github.com/kyazdani42/nvim-web-devicons"
  },
  ["plenary.nvim"] = {
    loaded = true,
    path = "/Users/monomadic/.local/share/nvim/site/pack/packer/start/plenary.nvim",
    url = "https://github.com/nvim-lua/plenary.nvim"
  }
}

time([[Defining packer_plugins]], false)
-- Config for: neo-tree.nvim
time([[Config for neo-tree.nvim]], true)
try_loadstring("\27LJ\2\n�\14\0\0\6\0!\0-6\0\0\0009\0\1\0'\2\2\0B\0\2\0016\0\3\0'\2\4\0B\0\2\0029\0\5\0005\2\6\0005\3\b\0005\4\a\0=\4\t\0035\4\n\0=\4\v\0035\4\f\0=\4\r\0035\4\14\0=\4\15\3=\3\16\0025\3\18\0005\4\17\0=\4\19\0035\4\20\0005\5\21\0=\5\22\4=\4\23\3=\3\24\0025\3\25\0005\4\26\0005\5\27\0=\5\22\4=\4\23\3=\3\28\0025\3\31\0005\4\29\0005\5\30\0=\5\22\4=\4\23\3=\3\15\2B\0\2\0016\0\0\0009\0\1\0'\2 \0B\0\2\1K\0\1\0\"nnoremap \\ :NeoTreeReveal<cr>\1\0\0\1\0\19\t<CR>\topen\6x\21cut_to_clipboard\6d\vdelete\6R\frefresh\6c\22copy_to_clipboard\6p\25paste_from_clipboard\6r\vrename\agc\15git_commit\agp\rgit_push\6C\15close_node\agg\24git_commit_and_push\6A\16git_add_all\18<1-LeftMouse>\topen\aga\17git_add_file\6s\16open_vsplit\agu\21git_unstage_file\6S\15open_split\6l\topen\agr\20git_revert_file\1\0\1\rposition\nfloat\fbuffers\1\0\15\6.\rset_root\t<bs>\16navigate_up\6a\badd\6R\frefresh\6s\16open_vsplit\6p\25paste_from_clipboard\6r\vrename\6c\22copy_to_clipboard\abd\18buffer_delete\6x\21cut_to_clipboard\6S\15open_split\6l\topen\6d\vdelete\t<cr>\topen\18<1-LeftMouse>\topen\1\0\1\rposition\tleft\1\0\1\18show_unloaded\2\15filesystem\vwindow\rmappings\1\0\21\6/\17fuzzy_finder\n<c-x>\17clear_filter\6I\21toggle_gitignore\6a\badd\6c\22copy_to_clipboard\6p\25paste_from_clipboard\6r\vrename\6H\18toggle_hidden\6R\frefresh\6C\15close_node\t<bs>\16navigate_up\6x\21cut_to_clipboard\6l\topen\6q\17close_window\t<cr>\topen\18<1-LeftMouse>\topen\6s\16open_vsplit\6f\21filter_on_submit\6S\15open_split\6.\rset_root\6d\vdelete\1\0\2\nwidth\3(\rposition\tleft\ffilters\1\0\3\26hijack_netrw_behavior\17open_default\27use_libuv_file_watcher\1\24follow_current_file\1\1\0\2\22respect_gitignore\2\16show_hidden\1\30default_component_configs\15git_status\1\0\1\14highlight\19NeoTreeDimText\tname\1\0\2\26use_git_status_colors\2\19trailing_slash\1\ticon\1\0\4\16folder_open\b\17folder_empty\bﰊ\18folder_closed\b\fdefault\6*\vindent\1\0\0\1\0\6\16indent_size\3\2\17with_markers\2\14highlight\24NeoTreeIndentMarker\23last_indent_marker\b└\18indent_marker\b│\fpadding\3\1\1\0\4\23enable_diagnostics\2\22enable_git_status\2\23popup_border_style\frounded\25close_if_last_window\2\nsetup\rneo-tree\frequirez          hi link NeoTreeDirectoryName Directory\n          hi link NeoTreeDirectoryIcon NeoTreeDirectoryName\n        \bcmd\bvim\0", "config", "neo-tree.nvim")
time([[Config for neo-tree.nvim]], false)
if should_profile then save_profiles() end

end)

if not no_errors then
  vim.api.nvim_command('echohl ErrorMsg | echom "Error in packer_compiled: '..error_msg..'" | echom "Please check your config for correctness" | echohl None')
end
