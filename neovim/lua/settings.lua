-- SETTINGS
--
--	sudo command: :w !sudo tee %
--

local o = vim.o -- vim options
local g = vim.g -- global scoped options
local w = vim.wo -- window scoped options
-- local b = vim.b -- buffer scoped options

--vim.g.vim_markdown_new_list_item_indent = 2 -- markdown list indent
--vim.opt.formatoptions = vim.o.formatoptions:gsub("r", ""):gsub("o", "")

-- vim options
-- o.foldcolumn=2 -- show folds
o.autowriteall = true -- ensure write upon leaving a page
o.clipboard = "unnamedplus" -- allows neovim to access the system clipboard (gnome)
o.conceallevel = 0 -- so that `` is visible in markdown files
o.cursorline = true -- highlight the current line
o.expandtab = false -- insert spaces when tab is pressed
o.foldlevelstart = 99
o.hidden = false -- switch buffer without unloading+saving them
o.hlsearch = false -- highlight all matches on previous search pattern
o.ignorecase = true -- ignore case when searching
o.laststatus = 3 -- 2 = local, 3 = global statusline (neovim 0.7+)
o.lazyredraw = true -- faster macros (force update with :redraw)
o.mouse = "a" -- allow the mouse to be used in neovim
o.title = true -- set window title
o.titlestring = vim.fn.fnamemodify(vim.fn.getcwd(), ":~:t")

-- global options
g.mapleader = " " -- leader key
g.tex_flavor = "latex"
g.vim_markdown_edit_url_in = 'current' -- open md links as (vplit | current)

-- window options
w.number = false -- show numbered lines

-- vim.g.vim_markdown_new_list_item_indent = 1 -- indent new items on 'o' from n mode
-- vim.cmd "let g:clipboard = {'copy': {'+': 'pbcopy', '*': 'pbcopy'}, 'paste': {'+': 'pbpaste', '*': 'pbpaste'}, 'name': 'pbcopy', 'cache_enabled': 0}" -- hack for macos
vim.wo.relativenumber = false -- set relative numbered lines
vim.opt.scrolloff = 1000 -- keep line centered (disable if scrolling past eof is enabled)
vim.opt.scroll = 3 -- number of lines to scroll
vim.opt.shiftwidth = 2 -- the number of spaces inserted for each indentation
vim.opt.showmatch = true -- matching parenthesis
vim.opt.signcolumn = "yes" -- always show the sign column, otherwise it would shift the text each time
vim.opt.smartcase = true -- searches are case insensitive unless a capital is used
vim.opt.smartindent = true -- make indenting smarter again
vim.opt.softtabstop = 2 -- number of spaces to convert a tab to
vim.opt.splitbelow = true -- force all horizontal splits to go below current window
vim.opt.splitright = true -- force all vertical splits to go to the right of current window
vim.opt.swapfile = false -- creates a swapfile
vim.opt.tabstop = 2 -- insert 2 spaces for a tab
vim.opt.termguicolors = true -- 24-bit color
vim.opt.wrap = false -- display lines as one long line
vim.wo.foldexpr = 'nvim_treesitter#foldexpr()' -- use treesitter for folding
vim.wo.foldmethod = 'expr' -- fold method (market | syntax)
vim.o.completeopt = "menuone,noinsert,noselect"
vim.g.completion_matching_strategy_list = { 'exact', 'substring', 'fuzzy' }
vim.g.completion_matching_ignore_case = 1
vim.g.completion_trigger_keyword_length = 3
vim.opt.showmode = false
vim.opt.regexpengine = 2

vim.opt.guicursor = "n-v-c:block,i-ci-ve:ver25,r-cr:hor20,o:hor50,a:blinkon250-Cursor/lCursor,sm:block-blinkwait175-blinkoff150-blinkon175"
-- vim.opt.guicursor = "n-v-c:block,i-ci-ve:ver25,r-cr:hor20,o:hor50,a:blinkon250-Cursor/lCursor,sm:block"
-- vim.opt.guicursor = "n-v-c:block,i-ci-ve:ver25"

-- set guicursor=a:blinkon1 -- blinking cursor
		-- vim.g.timeoutlen=0
		-- vim.g.ttimeoutlen=0

-- netrw
vim.g.netrw_banner = 0 -- hide banner
vim.g.netrw_localcopydircmd = 'cp -r' -- recursive copy
vim.g.netrw_liststyle = 3 -- tree view
vim.g.netrw_winsize = -28 -- absolute width
-- vim.g.netrw_sort_sequence = '[\/]$,*' -- sort dirs first

vim.api.nvim_set_option('tabstop', 2)

vim.diagnostic.config({
	virtual_text = false,
})

-- split border chars
vim.opt.fillchars = {
	vert = " ",
	fold = "⠀",
	horiz = " ",
	vertleft = " ",
	vertright = " ",
	stl = "⠀", -- statusline
	stlnc = " ", -- statusline (inactive)
	eob = " ", -- suppress ~ at EndOfBuffer
	--diff = "⣿", -- alternatives = ⣿ ░ ─ ╱
	msgsep = "‾",
	foldopen = "▾",
	foldsep = "│",
	foldclose = "▸",
}
