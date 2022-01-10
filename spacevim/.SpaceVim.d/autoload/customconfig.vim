function! customconfig#after() abort
  nnoremap <silent> <C-p> :FZF<CR>
  nnoremap <silent> <C-b> :NERDTreeToggle<CR>

  nnoremap <C-w> :q<CR>
  nnoremap <C-s> :w<CR>

  " Move to line
  map <C-l> <Plug>(easymotion-bd-jk)
  nmap <C-L> <Plug>(easymotion-overwin-line)

  " move to word
  map  <C-j> <Plug>(easymotion-bd-w)
endfunction
