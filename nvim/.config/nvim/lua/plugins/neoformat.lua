-- format on save
vim.api.nvim_exec(
  [[
augroup FormatAutogroup
  autocmd!
  autocmd BufWritePre * undojoin | Neoformat
augroup END
]],
  true
)
-- autocmd BufWritePost *.js,*.rs,*.lua FormatWrite
-- autocmd BufWritePre * undojoin | Neoformat