-- COLORS
--

-- https://gist.github.com/romainl/379904f91fa40533175dfaec4c833f2f
vim.cmd("colorscheme {{colorscheme}}");

vim.api.nvim_set_hl(0, "Float", { bg = "#111111" })
-- vim.api.nvim_set_hl(0, "NormalFloat", { bg = "red" })

vim.api.nvim_set_hl(0, "TabLine", { fg = "white", bg = "black" })
vim.api.nvim_set_hl(0, "WinBar", { fg = "white", bg = "#2222FF" })

--vim.api.nvim_set_hl(0, "TabLine", { fg = "white", bg = "#2222FF" })
--vim.api.nvim_set_hl(0, "WinBar", { fg = "white", bg = "#7740FF" })
-- vim.api.nvim_set_hl(0, "WinBar", { fg = "white" })

vim.api.nvim_set_hl(0, "LspActive", { fg = "#00FF00" })

vim.api.nvim_set_hl(0, "EndOfBuffer", { fg = "#444444" })
vim.api.nvim_set_hl(0, "TabLineFill", { bg = "None" })
vim.api.nvim_set_hl(0, "Title", { fg = "#CCFF00" })
vim.api.nvim_set_hl(0, "VimwikiHeaderChar", { fg = "#44FF00" })
vim.api.nvim_set_hl(0, "VimwikiLink", { fg = "#44FFFF" })
-- vim.api.nvim_set_hl(0, "LineNr", { fg = "#222222" }) -- active
vim.api.nvim_set_hl(0, "StatusLine", {}) -- active
vim.api.nvim_set_hl(0, "StatusLineNC", {}) -- inactive

vim.api.nvim_set_hl(0, "WinSeparator", { fg = "bg", bg = "bg" }) -- inactive
-- vim.cmd("hi WinSeparator guifg=none"); -- I think this is the split column
vim.api.nvim_set_hl(0, "TodoBgTODO", { bg = "#FFFF00", fg = "black" })
vim.api.nvim_set_hl(0, "TodoFgTODO", { fg = "#FFFF00" })
vim.api.nvim_set_hl(0, "DiagnosticVirtualTextHint", { fg = "#F0F0AA" })
vim.api.nvim_set_hl(0, "DiagnosticVirtualTextError", { fg = "#F02282" })

-- text input borders
vim.api.nvim_set_hl(0, "FloatBorder", { fg = "#CCFF00" })
vim.api.nvim_set_hl(0, "NormalFloat", { bg = "black" })
