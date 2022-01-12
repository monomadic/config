-- keys
-- modes https://github.com/nanotee/nvim-lua-guide#defining-mappings
vim.api.nvim_set_keymap('', '<C-s>', ':write<CR>', {noremap = true})
vim.api.nvim_set_keymap('', '<C-w>', ':quit<CR>', {noremap = true})
vim.api.nvim_set_keymap('', '<C-q>', ':quit!<CR>', {noremap = true})
vim.api.nvim_set_keymap('', '<C-p>', ':FZF<CR>', {})
vim.api.nvim_set_keymap('', '<C-b>', ':NERDTreeToggle<CR>', {})
vim.api.nvim_set_keymap('n', '<C-j>', '<Plug>(easymotion-bd-w)', {})

-- plugins
--
local Plug = vim.fn['plug#']
vim.call('plug#begin', '~/.config/nvim/plugged')
--Plug('roxma/nvim-completion-manager')
Plug('junegunn/fzf', {['do'] = vim.fn['fzf#install']}) -- fuzzy find
Plug('scrooloose/nerdtree', {on = 'NERDTreeToggle'}) -- file tree
Plug('easymotion/vim-easymotion') -- fast jump
--Plug('brooth/far.zim') -- find and replace
Plug('neovim/nvim-lspconfig') -- language server
Plug('Shougo/deoplete.nvim') -- autocomplete
Plug('ternjs/tern_for_vim')
Plug('carlitux/deoplete-ternjs', {['for'] = 'javascript'})
vim.call('plug#end')

-- globals
--
vim.env.FZF_DEFAULT_OPTS = '--layout=reverse'
vim.env.FZF_DEFAULT_COMMAND = 'rg --files --hidden --glob !.git'
vim.opt.tabstop = 2
vim.opt.wildignore = {'*/cache/*', '*/tmp/*'}
-- vim.opt.colorscheme "darkblue"
