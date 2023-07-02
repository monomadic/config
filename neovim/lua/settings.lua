-- SETTINGS
--
--	sudo command: :w !sudo tee %
--

local g = vim.g -- global variables
local o = vim.o -- global neovim built-in options
local w = vim.wo -- window scoped neovim options
-- local b = vim.b -- buffer scoped neovim options

-- neovim built-in options
o.autowriteall = true -- ensure write upon leaving a page
o.clipboard = "unnamedplus" -- allows neovim to access the system clipboard (gnome)
o.completeopt = "menuone,noinsert,noselect"
o.conceallevel = 0 -- so that `` is visible in markdown files
o.cursorline = true -- highlight the current line
o.expandtab = false -- insert spaces when tab is pressed
o.guicursor = "n-v-c:block,i-ci-ve:ver25,r-cr:hor20,o:hor50,a:blinkon250-Cursor/lCursor,sm:block-blinkwait175-blinkoff150-blinkon175"
o.hidden = false -- switch buffer without unloading+saving them
o.hlsearch = false -- highlight all matches on previous search pattern
o.ignorecase = true -- ignore case when searching
o.laststatus = 3 -- 2 = local, 3 = global statusline (neovim 0.7+)
o.lazyredraw = true -- faster macros (force update with :redraw)
o.mouse = "a" -- allow the mouse to be used in neovim
o.regexpengine = 2
o.scroll = 3 -- number of lines to scroll
o.scrolloff = 1000 -- keep line centered (disable if scrolling past eof is enabled)
o.shiftwidth = 2 -- the number of spaces inserted for each indentation
o.showmatch = true -- matching parenthesis
o.showmode = false
o.signcolumn = "yes" -- always show the sign column, otherwise it would shift the text each time
o.smartcase = true -- searches are case insensitive unless a capital is used
o.smartindent = true -- make indenting smarter again
o.softtabstop = 2 -- number of spaces to convert a tab to
o.splitbelow = true -- force all horizontal splits to go below current window
o.splitright = true -- force all vertical splits to go to the right of current window
o.swapfile = false -- creates a swapfile
o.tabstop = 2
o.tabstop = 2 -- insert 2 spaces for a tab
o.termguicolors = true -- 24-bit color
o.title = true -- set window title
o.titlestring = vim.fn.fnamemodify(vim.fn.getcwd(), ":~:t")
o.wrap = false -- display lines as one long line

-- global variables
g.completion_matching_ignore_case = 1
g.completion_matching_strategy_list = { 'exact', 'substring', 'fuzzy' }
g.completion_trigger_keyword_length = 3
g.mapleader = " " -- leader key
g.tex_flavor = "latex"
g.vim_markdown_edit_url_in = 'current' -- open md links as (vplit | current)

g.template_directory = "~/config/neovim/templates"
g.wiki_directory = "~/wiki/"
g.neovim_config_directory = "~/config/neovim"

-- use newer filetype.lua instead of filetype.vim
g.do_filetype_lua = 1
-- g.did_load_filetypes = 0 -- do not use filetype.vim at all

-- folds
--o.foldlevelstart = 99
w.foldlevel = 99
o.foldcolumn="0" -- show folds
w.foldexpr = 'nvim_treesitter#foldexpr()' -- use treesitter for folding
w.foldmethod = 'expr' -- fold method (market | syntax | expr)
w.foldminlines = 5 -- minimum lines before fold
--w.foldnestmax = 3 -- maximum nested folds

-- window options
-- vim.g.vim_markdown_new_list_item_indent = 1 -- indent new items on 'o' from n mode
-- vim.cmd "let g:clipboard = {'copy': {'+': 'pbcopy', '*': 'pbcopy'}, 'paste': {'+': 'pbpaste', '*': 'pbpaste'}, 'name': 'pbcopy', 'cache_enabled': 0}" -- hack for macos
-- o.guicursor = "n-v-c:block,i-ci-ve:ver25,r-cr:hor20,o:hor50,a:blinkon250-Cursor/lCursor,sm:block"
-- o.guicursor = "n-v-c:block,i-ci-ve:ver25"
w.number = false -- show line number on current line
w.relativenumber = false -- relative numbered lines

-- vim.g.timeoutlen=0
-- vim.g.ttimeoutlen=0
--vim.g.vim_markdown_new_list_item_indent = 2 -- markdown list indent
--o.formatoptions = vim.o.formatoptions:gsub("r", ""):gsub("o", "")

-- netrw
g.netrw_banner = 0 -- hide banner
g.netrw_localcopydircmd = 'cp -r' -- recursive copy
g.netrw_liststyle = 3 -- tree view
g.netrw_winsize = -28 -- absolute width
-- vim.g.netrw_sort_sequence = '[\/]$,*' -- sort dirs first

vim.diagnostic.config({ virtual_text = false })

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

--
-- NeoVide Settings
--
-- Helper function for transparency formatting
local alpha = function()
  return string.format("%x", math.floor(255 * (vim.g.neovide_transparency_point or 0.8)))
end
g.neovide_transparency = 0.0
g.transparency = 0.8
g.neovide_background_color = "#0f1117" .. alpha()
g.neovide_fullscreen = true
g.neovide_cursor_vfx_mode = "sonicboom"
