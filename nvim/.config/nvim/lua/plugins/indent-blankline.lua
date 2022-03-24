require("indent_blankline").setup({
  -- for example, context is off by default, use this to turn it on
  show_current_context = true,
  show_current_context_start = true,
  filetype_exclude = { "neo-tree", "help", "floaterm", "SidebarNvim", "" },
})
-- vim.opt.termguicolors = true
-- vim.cmd([[highlight IndentBlanklineIndent1 guibg=#111111 gui=nocombine]])
-- vim.cmd([[highlight IndentBlanklineIndent2 guibg=#000000 gui=nocombine]])
--
-- require("indent_blankline").setup({
--   filetype_exclude = { "neo-tree", "help", "floaterm" },
--   char = "",
--   char_highlight_list = {
--     "IndentBlanklineIndent1",
--     "IndentBlanklineIndent2",
--   },
--   space_char_highlight_list = {
--     "IndentBlanklineIndent1",
--     "IndentBlanklineIndent2",
--   },
--   show_trailing_blankline_indent = false,
-- })
