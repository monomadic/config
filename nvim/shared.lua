-- settings
--
vim.g.mapleader = ',' -- leader key
vim.o.formatoptions = vim.o.formatoptions:gsub("r", ""):gsub("o", "")
vim.opt.cursorline = true -- highlight the current line
vim.opt.expandtab = true -- convert tabs to spaces
vim.opt.foldlevelstart = 99
vim.opt.hidden = false -- switch buffer without unloading+saving them
vim.opt.hlsearch = false -- highlight all matches on previous search pattern
vim.opt.ignorecase = true -- ignore case when searching
vim.opt.laststatus = 2 -- 3 = global statusline (neovim 0.7+)
vim.opt.lazyredraw = true -- faster macros (force update with :redraw)
vim.opt.mouse = "a" -- allow the mouse to be used in neovim
vim.opt.number = true -- set numbered lines
vim.opt.relativenumber = true -- set relative numbered lines
vim.opt.shiftwidth = 2 -- the number of spaces inserted for each indentation
vim.opt.showmatch = true -- matching parenthesis
vim.opt.smartcase = true -- searches are case insensitive unless a capital is used
vim.opt.smartindent = true -- make indenting smarter again
vim.opt.softtabstop = 2 -- number of spaces to convert a tab to
vim.opt.splitbelow = true -- force all horizontal splits to go below current window
vim.opt.splitright = true -- force all vertical splits to go to the right of current window
vim.opt.swapfile = false -- creates a swapfile
vim.opt.tabstop = 2 -- insert 2 spaces for a tab
vim.wo.foldexpr = 'nvim_treesitter#foldexpr()' -- use treesitter for folding
vim.wo.foldmethod = 'expr' -- fold method (market | syntax)

-- set guicursor=n-v-c:block,i-ci-ve:ver25,r-cr:hor20,o:hor50,a:blinkwait700-blinkoff400-blinkon250-Cursor/lCursor,sm:block-blinkwait175-blinkoff150-blinkon175
