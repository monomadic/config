

-- keys
-- modes https://github.com/nanotee/nvim-lua-guide#defining-mappings
vim.api.nvim_set_keymap('', '<C-s>', ':write<CR>', {noremap = true})
vim.api.nvim_set_keymap('', '<C-w>', ':tabclose<CR>', {noremap = true})
vim.api.nvim_set_keymap('', '<C-q>', ':quit!<CR>', {noremap = true})
vim.api.nvim_set_keymap('', '<C-p>', ':FZF<CR>', {})
vim.api.nvim_set_keymap('', '<C-[>', ':bprev<CR>', {})
vim.api.nvim_set_keymap('', '<C-]>', ':bnext<CR>', {})
vim.api.nvim_set_keymap('n', '<C-j>', '<Plug>(easymotion-bd-w)', {})

-- plugins
--
local Plug = vim.fn['plug#']
vim.call('plug#begin', '~/.config/nvim/plugged')
Plug('scrooloose/nerdtree', {on = 'NERDTreeToggle'}) -- file tree
--Plug('roxma/nvim-completion-manager')
Plug('junegunn/fzf', {['do'] = vim.fn['fzf#install']}) -- fuzzy find
Plug('easymotion/vim-easymotion') -- fast jump
--Plug('brooth/far.zim') -- find and replace
Plug('neovim/nvim-lspconfig') -- language server
--Plug('Shougo/deoplete.nvim') -- autocomplete
Plug('ternjs/tern_for_vim')
Plug('carlitux/deoplete-ternjs', {['for'] = 'javascript'})
Plug 'dracula/vim' -- colorscheme
Plug 'ryanoasis/vim-devicons' -- icons
Plug 'vim-airline/vim-airline' -- status bar
Plug 'ervandew/supertab' -- tab complete? check this more
Plug 'terryma/vim-multiple-cursors'
-- Plug 'vim-airline/vim-airline-themes' -- status bar themes
Plug 'kyazdani42/nvim-web-devicons'
vim.call('plug#end')

-- globals
--
-- see: :h lua-vim-variables and :h lua-vim-options
vim.env.FZF_DEFAULT_OPTS = '--layout=reverse'
vim.env.FZF_DEFAULT_COMMAND = 'rg --files --hidden --glob !.git'
vim.opt.tabstop = 2
vim.opt.wildignore = {'*/cache/*', '*/tmp/*'}
vim.opt.number = true
-- vim.g['ctrlp_prompt_mappings'] = {['AcceptSelection("t")'] = '<cr>'}

-- colors
--
vim.opt.termguicolors = false
vim.cmd [[colorscheme torte]]
vim.cmd [[hi Normal guibg=none ctermbg=none]]
vim.cmd [[hi VertSplit cterm=none gui=none]]
vim.cmd [[hi LineNr ctermfg=darkgrey]]

-- file tree
--
vim.api.nvim_set_keymap('', '<C-b>', ':NERDTreeToggle<CR>', {})
vim.g['NERDTreeMapActivateNode'] = 'l' -- note: vim.g are globals
vim.g['NERDTreeWinPos'] = 'left'
vim.g['NERDTreeMinimalUI'] = 1
-- vim.g['NERDTreeIgnore'] = ['^\.DS_Store$', '^tags$', '\.git$[[dir]]']
vim.g['NERDTreeShowHidden'] = 1
--vim.g['NERDTreeMapOpenInTab'] = '<ENTER>'
vim.g['airline#extensions#tabline#show_devicons'] = "true"
vim.g['airline#extensions#tabline#enabled'] = 1
vim.g['airline#extensions#tabline#buffer_nr_show'] = 1
--vim.g['airline#extensions#tabline#left_alt_sep'] = ' '
vim.g['airline#extensions#tabline#left_sep'] = ' '

