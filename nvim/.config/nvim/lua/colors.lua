vim.g.material_style = "deep ocean"
vim.g.sonokai_style = "espresso"
vim.g.tokyonight_style = "night"

vim.cmd([[colorscheme moonfly]])
vim.cmd([[
  hi Visual guibg=#c8fc0c guifg=#555555
  hi String guifg=#50fc0c
  hi TSString guifg=#50fc0c
  hi Function guifg=#0cb0fc
  hi Type guifg=#0cfcd0
  hi Conditional guifg=#0c84fc
  hi Keyword guifg=#99CCCC
  hi LineNr guifg=#444444
  hi CursorLineNr guifg=#AAAAAA gui=bold

  hi ActiveWindow guibg=#000000 | hi InactiveWindow guibg=#090909
  set winhighlight=Normal:ActiveWindow,NormalNC:InactiveWindow
]])