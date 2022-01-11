-- keys
-- see modes here https://github.com/nanotee/nvim-lua-guide#defining-mappings
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
 Plug('junegunn/fzf')
 Plug('scrooloose/nerdtree', {on = 'NERDTreeToggle'})
 Plug('easymotion/vim-easymotion')
 Plug('neovim/nvim-lspconfig')
 Plug('Shougo/deoplete.nvim')
 Plug('ternjs/tern_for_vim')
 Plug('carlitux/deoplete-ternjs')
vim.call('plug#end')

-- globals
--
vim.env.FZF_DEFAULT_OPTS = '--layout=reverse'
vim.env.FZF_DEFAULT_COMMAND = 'rg --files --hidden --glob !.git'
vim.opt.tabstop = 2
vim.opt.wildignore = {'*/cache/*', '*/tmp/*'}
-- vim.opt.colorscheme "darkblue"
