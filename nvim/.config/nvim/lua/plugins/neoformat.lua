-- format on save
vim.api.nvim_exec(
  [[
augroup FormatAutogroup
  autocmd!
  autocmd BufWritePre * undojoin | Prettier
augroup END
]],
  true
)
-- autocmd BufWritePost *.js,*.rs,*.lua FormatWrite
-- autocmd BufWritePre * undojoin | Neoformat
