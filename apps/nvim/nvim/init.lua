-- By default title is off. Needed for detecting window as neovim instance (sworkstyle)
-- require('plugins/packer')
--require('settings')
require('keymaps')

vim.cmd "set title"
vim.call('plug#begin', '~/.config/nvim/plugins')

