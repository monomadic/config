-- COLORS
--

-- https://gist.github.com/romainl/379904f91fa40533175dfaec4c833f2f
vim.cmd("colorscheme {{colorscheme}}");

vim.api.nvim_set_hl(0, "EndOfBuffer", { fg = "#444444" })
vim.api.nvim_set_hl(0, "TabLineFill", { bg = "None" })
vim.api.nvim_set_hl(0, "Title", { fg = "#CCFF00" })
vim.api.nvim_set_hl(0, "VimwikiHeaderChar", { fg = "#44FF00" })
vim.api.nvim_set_hl(0, "VimwikiLink", { fg = "#44FFFF" })
vim.api.nvim_set_hl(0, "LineNr", { fg = "#222222" }) -- active
vim.api.nvim_set_hl(0, "StatusLine", {}) -- active
vim.api.nvim_set_hl(0, "StatusLineNC", {}) -- inactive

vim.cmd("hi WinSeparator guifg=none"); -- I think this is the split column
vim.cmd("hi TodoBgTODO guibg=#FFFF00 guifg=black");
vim.cmd("hi TodoFgTODO guifg=#FFFF00");
vim.cmd("hi DiagnosticVirtualTextHint guifg=#F0F0AA")
vim.cmd("hi DiagnosticVirtualTextError guifg=#F02282")
