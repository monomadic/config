local map = vim.api.nvim_set_keymap
local opts = { noremap = true, silent = true }

vim.cmd([[
  hi BufferCurrent guibg=#c8fc0c guifg=black gui=bold
  hi BufferCurrentSign guibg=#c8fc0c
  hi BufferCurrentMod guibg=#c8fc0c guifg=#FF00CC
  hi BufferVisible guifg=#AAAAAA guibg=none gui=bold
  hi BufferVisibleSign guibg=none
  hi BufferInactive guifg=#555555 guibg=none gui=bold
  hi BufferInactiveSign guibg=none
  hi BufferInactiveMod guibg=none guifg=#FF00CC
  hi BufferTabpageFill guibg=none
]])

-- Move to previous/next
map("n", "gz", ":BufferMove 0<CR>", opts)
map("n", "<C-0>", ":BufferMove 0<CR>", opts)
map("n", "H", ":BufferPrevious<CR>", opts)
map("n", "L", ":BufferNext<CR>", opts)
-- Re-order to previous/next
map("n", "<A-<>", ":BufferMovePrevious<CR>", opts)
map("n", "<A->>", " :BufferMoveNext<CR>", opts)
-- Goto buffer in position...
map("n", "g1", ":BufferGoto 1<CR>", opts)
map("n", "g2", ":BufferGoto 2<CR>", opts)
map("n", "g3", ":BufferGoto 3<CR>", opts)
map("n", "g4", ":BufferGoto 4<CR>", opts)
map("n", "g5", ":BufferGoto 5<CR>", opts)
map("n", "g6", ":BufferGoto 6<CR>", opts)
map("n", "g7", ":BufferGoto 7<CR>", opts)
map("n", "g8", ":BufferGoto 8<CR>", opts)
map("n", "g9", ":BufferGoto 9<CR>", opts)

-- map("n", "m1", ":BufferMove 1<CR>", opts)
-- map("n", "m2", ":BufferMove 2<CR>", opts)
-- map("n", "m3", ":BufferMove 3<CR>", opts)
-- map("n", "m4", ":BufferMove 4<CR>", opts)
-- map("n", "m5", ":BufferMove 5<CR>", opts)
-- map("n", "m6", ":BufferMove 6<CR>", opts)
-- map("n", "m7", ":BufferMove 7<CR>", opts)
-- map("n", "m8", ":BufferMove 8<CR>", opts)
-- map("n", "m9", ":BufferMove 9<CR>", opts)

map("n", "<A-0>", ":BufferLast<CR>", opts)
-- Close buffer
map("n", "<C-q>", ":BufferClose<CR>", opts)
-- Wipeout buffer
--                 :BufferWipeout<CR>
-- Close commands
--                 :BufferCloseAllButCurrent<CR>
--                 :BufferCloseBuffersLeft<CR>
--                 :BufferCloseBuffersRight<CR>
-- Magic buffer-picking mode
--map('n', '<C-p>', ':BufferPick<CR>', opts)
-- Sort automatically by...
-- map("n", "<Space>bb", ":BufferOrderByBufferNumber<CR>", opts)
-- map("n", "<Space>bd", ":BufferOrderByDirectory<CR>", opts)
-- map("n", "<Space>bl", ":BufferOrderByLanguage<CR>", opts)

vim.g.bufferline = {
  -- Enable/disable animations
  animation = true,

  -- Enable/disable auto-hiding the tab bar when there is a single buffer
  auto_hide = false,

  -- Enable/disable current/total tabpages indicator (top right corner)
  tabpages = true,

  -- Enable/disable close button
  closable = false,

  -- Enables/disable clickable tabs
  --  - left-click: go to buffer
  --  - middle-click: delete buffer
  clickable = true,

  -- Excludes buffers from the tabline
  -- exclude_ft = { "javascript" },
  -- exclude_name = { "package.json" },

  -- Enable/disable icons
  -- if set to 'numbers', will show buffer index in the tabline
  -- if set to 'both', will show buffer index and icons in the tabline
  icons = true,

  -- If set, the icon color will follow its corresponding buffer
  -- highlight group. By default, the Buffer*Icon group is linked to the
  -- Buffer* group (see Highlighting below). Otherwise, it will take its
  -- default value as defined by devicons.
  icon_custom_colors = true,

  -- Configure icons on the bufferline.
  icon_separator_active = " ",
  icon_separator_inactive = " ",
  icon_close_tab = "",
  icon_close_tab_modified = "●",
  icon_pinned = "車",

  -- If true, new buffers will be inserted at the start/end of the list.
  -- Default is to insert after current buffer.
  insert_at_end = false,
  insert_at_start = false,

  -- Sets the maximum padding width with which to surround each tab
  maximum_padding = 2,

  -- Sets the maximum buffer name length.
  maximum_length = 30,

  -- If set, the letters for each buffer in buffer-pick mode will be
  -- assigned based on their name. Otherwise or in case all letters are
  -- already assigned, the behavior is to assign letters in order of
  -- usability (see order below)
  semantic_letters = true,

  -- New buffer letters are assigned in this order. This order is
  -- optimal for the qwerty keyboard layout but might need adjustement
  -- for other layouts.
  letters = "asdfjkl;ghnmxcvbziowerutyqpASDFJKLGHNMXCVBZIOWERUTYQP",

  -- Sets the name of unnamed buffers. By default format is "[Buffer X]"
  -- where X is the buffer number. But only a static string is accepted here.
  no_name_title = nil,
}
