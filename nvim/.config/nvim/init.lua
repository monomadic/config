-- Plugins
-- {{{
--   see https://github.com/junegunn/vim-plug
--
--   `do`: executed on plugin install or update.
--   `on`: executed on-demand when another function is called
--   `for`:
--
local Plug = vim.fn["plug#"]
vim.call("plug#begin", "~/.config/nvim/plugged")
Plug("scrooloose/nerdtree", { on = "NERDTreeToggle" }) -- file tree
--Plug('roxma/nvim-completion-manager')
Plug("junegunn/fzf", { ["do"] = vim.fn["fzf#install"] }) -- fuzzy find
Plug("easymotion/vim-easymotion") -- fast jumplocal nvim_lsp = require('lspconfig')
Plug("brooth/far.vim") -- find and replace
Plug("neovim/nvim-lspconfig") -- language server protocol
Plug("mfussenegger/nvim-dap") -- debugging protocol
Plug("simrat39/symbols-outline.nvim")
--Plug('Shougo/deoplete.nvim') -- autocomplete
Plug("ternjs/tern_for_vim")
Plug("carlitux/deoplete-ternjs", { ["for"] = "javascript" })
--Plug 'dracula/vim' -- colorscheme
Plug("evturn/cosmic-barf") -- colorscheme
Plug("ryanoasis/vim-devicons") -- icons
Plug("vim-airline/vim-airline") -- status bar
Plug("ervandew/supertab") -- tab complete? check this more
Plug("terryma/vim-multiple-cursors")
Plug("jose-elias-alvarez/null-ls.nvim")
Plug("nvim-lua/plenary.nvim")
Plug("MunifTanjim/prettier.nvim")
Plug("evanleck/vim-svelte")
Plug("sumneko/lua-language-server")
-- Plug 'vim-airline/vim-airline-themes' -- status bar themes
Plug("kyazdani42/nvim-web-devicons")
Plug("nvim-telescope/telescope.nvim")
Plug("nvim-treesitter/nvim-treesitter")
Plug("akinsho/toggleterm.nvim")
Plug("fatih/vim-go", { ["do"] = ":GoUpdateBinaries" })
Plug("RishabhRD/popfix") -- popup ui (required by popui)
Plug("hood/popui.nvim") -- popups to replace vim-ui selects
Plug("catppuccin/nvim", { ["as"] = "catppuccin" }) -- themes?
Plug("norcalli/nvim-colorizer.lua") -- inline colors
Plug 'justinmk/vim-sneak' -- fast jump
--Plug('mj-hd/vim-picomap', {["do"] = "bash install.sh" }) -- minimap
--Plug 'hisaknown/nanomap.vim' -- minimap

-- rust
Plug("simrat39/rust-tools.nvim")
-- rust (debugging)
Plug("nvim-lua/plenary.nvim")
Plug("mfussenegger/nvim-dap")

vim.call("plug#end")
-- }}}

-- Keymaps
-- {{{
--  see: https://github.com/nanotee/nvim-lua-guide#defining-mappings
local function keymap(...)
	vim.api.nvim_buf_set_keymap(bufnr, ...)
end
local opts = { noremap = true, silent = true }
keymap("", "<C-s>", ":write<CR>", { noremap = true })
--vim.api.nvim_set_keymap('', '<C-w>', ':tabclose<CR>', {noremap = true})
vim.api.nvim_set_keymap("", "<C-q>", ":quit!<CR>", { noremap = true })
vim.api.nvim_set_keymap("", "<C-[>", ":bprev<CR>", {})
vim.api.nvim_set_keymap("", "<C-]>", ":bnext<CR>", {})
vim.api.nvim_set_keymap("", "<C-t>", ":Telescope<cr>", { noremap = true })
vim.api.nvim_set_keymap("n", "<C-o>", ":FZF<CR>", {})
vim.api.nvim_set_keymap("n", "<Esc>", ":noh<cr>", { noremap = true }) -- fix ESC confusion in normal mode
vim.api.nvim_set_keymap("n", "<C-f>", "<Plug>(easymotion-bd-w)", {})
vim.api.nvim_set_keymap("n", "<C-p>", ":Telescope find_files<cr>", { noremap = true })
--nnoremap <leader>fg <cmd>lua require('telescope.builtin').live_grep()<cr>
--nnoremap <leader>fb <cmd>lua require('telescope.builtin').buffers()<cr>
--nnoremap <leader>fh <cmd>lua require('telescope.builtin').help_tags()<cr>
-- emacs style shortcuts in insert mode (yes, I am like that)
vim.api.nvim_set_keymap("i", "<C-n>", "<Down>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-p>", "<Up>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-b>", "<Left>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-f>", "<Right>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-e>", "<End>", { noremap = true })
vim.api.nvim_set_keymap("i", "<C-a>", "<Home>", { noremap = true })
-- }}}

-- LSP
-- {{{
local nvim_lsp = require("lspconfig")

--nvim_lsp.typescript.setup {}
vim.cmd([[set foldmethod=marker]])

-- LSP keymaps
--   only map these keys if an lsp client is attached
local attach_lsp_keymaps = function(client, bufnr)
	print("attaching LSP keymaps")
	-- See `:help vim.lsp.*` for documentation on any of the below functions
	keymap("n", "gD", "<cmd>lua vim.lsp.buf.declaration()<CR>", opts)
	keymap("n", "gd", "<cmd>lua vim.lsp.buf.definition()<CR>", opts)
	keymap("n", "K", "<cmd>lua vim.lsp.buf.hover()<CR>", opts)
	keymap("n", "gi", "<cmd>lua vim.lsp.buf.implementation()<CR>", opts)
	keymap("n", "gr", "<cmd>lua vim.lsp.buf.references()<CR>", opts)
	keymap("n", "<C-k>", "<cmd>lua vim.lsp.buf.signature_help()<CR>", opts)
	keymap("n", "<space>wa", "<cmd>lua vim.lsp.buf.add_workspace_folder()<CR>", opts)
	keymap("n", "<space>wr", "<cmd>lua vim.lsp.buf.remove_workspace_folder()<CR>", opts)
	keymap("n", "<space>wl", "<cmd>lua print(vim.inspect(vim.lsp.buf.list_workspace_folders()))<CR>", opts)
	keymap("n", "td", "<cmd>lua vim.lsp.brf.type_definition()<CR>", opts)
	keymap("n", "tr", "<cmd>lua vim.lsp.buf.rename()<CR>", opts)
	keymap("n", "ga", "<cmd>lua vim.lsp.buf.code_action()<CR>", opts)
	keymap("n", "te", "<cmd>lua vim.diagnostic.open_float()<CR>", opts)
	keymap("n", "[d", "<cmd>lua vim.diagnostic.goto_prev()<CR>", opts)
	keymap("n", "]d", "<cmd>lua vim.diagnostic.goto_next()<CR>", opts)
	keymap("n", "<space>q", "<cmd>lua vim.diagnostic.setloclist()<CR>", opts)
	keymap("n", "<C-f>", "<cmd>lua vim.lsp.buf.formatting()<CR>", opts)

	-- Enable completion triggered by <c-x><c-o>
	--vim.api.nvim_buf_set_option('omnifunc', 'v:lua.vim.lsp.omnifunc', opts)

	-- symbols outline:
	vim.api.nvim_set_keymap("n", "<C-i>", ":SymbolsOutline<CR>", {})
end

vim.g.symbols_outline = {
	highlight_hovered_item = true,
	show_guides = true,
	auto_preview = true,
	position = "right",
	relative_width = true,
	width = 64,
	auto_close = false,
	show_numbers = false,
	show_relative_numbers = false,
	show_symbol_details = true,
	preview_bg_highlight = "Pmenu",
	keymaps = { -- These keymaps can be a string or a table for multiple keys
		close = { "<Esc>", "q" },
		goto_location = "<Cr>",
		focus_location = "o",
		hover_symbol = "<C-space>",
		toggle_preview = "K",
		rename_symbol = "r",
		code_actions = "a",
	},
	lsp_blacklist = {},
	symbol_blacklist = {},
	symbols = {
		File = { icon = "", hl = "TSURI" },
		Module = { icon = "", hl = "TSNamespace" },
		Namespace = { icon = "", hl = "TSNamespace" },
		Package = { icon = "", hl = "TSNamespace" },
		Class = { icon = "𝓒", hl = "TSType" },
		Method = { icon = "ƒ", hl = "TSMethod" },
		Property = { icon = "", hl = "TSMethod" },
		Field = { icon = "", hl = "TSField" },
		Constructor = { icon = "", hl = "TSConstructor" },
		Enum = { icon = "ℰ", hl = "TSType" },
		Interface = { icon = "ﰮ", hl = "TSType" },
		Function = { icon = "", hl = "TSFunction" },
		Variable = { icon = "", hl = "TSConstant" },
		Constant = { icon = "", hl = "TSConstant" },
		String = { icon = "𝓐", hl = "TSString" },
		Number = { icon = "#", hl = "TSNumber" },
		Boolean = { icon = "⊨", hl = "TSBoolean" },
		Array = { icon = "", hl = "TSConstant" },
		Object = { icon = "⦿", hl = "TSType" },
		Key = { icon = "🔐", hl = "TSType" },
		Null = { icon = "NULL", hl = "TSType" },
		EnumMember = { icon = "", hl = "TSField" },
		Struct = { icon = "𝓢", hl = "TSType" },
		Event = { icon = "🗲", hl = "TSType" },
		Operator = { icon = "+", hl = "TSOperator" },
		TypeParameter = { icon = "𝙏", hl = "TSParameter" },
	},
}

-- svelte
nvim_lsp.svelte.setup({
	cmd = { "/home/nom/.nvm/versions/node/v17.3.1/bin/svelteserver", "--stdio" },
})

nvim_lsp.rust_analyzer.setup({
	on_attach = attach_lsp_keymaps,
})
-- }}}

-- LSP: Rust
-- {{{
require("rust-tools").setup({

	tools = { -- rust-tools options
		-- Automatically set inlay hints (type hints)
		autoSetHints = true,
		-- Whether to show hover actions inside the hover window
		-- This overrides the default hover handler
		hover_with_actions = true,

		-- how to execute terminal commands
		-- options right now: termopen / quickfix
		executor = require("rust-tools/executors").termopen,
		runnables = {
			-- whether to use telescope for selection menu or not
			use_telescope = true,
			-- rest of the opts are forwarded to telescope
		},
		debuggables = {
			-- whether to use telescope for selection menu or not
			use_telescope = true,
			-- rest of the opts are forwarded to telescope
		},

		-- These apply to the default RustSetInlayHints command
		inlay_hints = {
			-- Only show inlay hints for the current line
			only_current_line = false,
			-- Event which triggers a refersh of the inlay hints.
			-- You can make this "CursorMoved" or "CursorMoved,CursorMovedI" but
			-- not that this may cause  higher CPU usage.
			-- This option is only respected when only_current_line and
			-- autoSetHints both are true.
			only_current_line_autocmd = "CursorHold",
			-- wheter to show parameter hints with the inlay hints or not
			show_parameter_hints = true,
			-- prefix for parameter hints
			parameter_hints_prefix = "<- ",
			-- prefix for all the other hints (type, chaining)
			other_hints_prefix = "=> ",
			-- whether to align to the length of the longest line in the file
			max_len_align = false,
			-- padding from the left if max_len_align is true
			max_len_align_padding = 1,
			-- whether to align to the extreme right or not
			right_align = false,
			-- padding from the right if right_align is true
			right_align_padding = 7,
			-- The color of the hints
			highlight = "Comment",
		},

		hover_actions = {
			-- the border that is used for the hover window
			-- see vim.api.nvim_open_win()
			border = {
				{ "╭", "FloatBorder" },
				{ "─", "FloatBorder" },
				{ "╮", "FloatBorder" },
				{ "│", "FloatBorder" },
				{ "╯", "FloatBorder" },
				{ "─", "FloatBorder" },
				{ "╰", "FloatBorder" },
				{ "│", "FloatBorder" },
			},
			-- whether the hover action window gets automatically focused
			auto_focus = false,
		},
		-- settings for showing the crate graph based on graphviz and the dot
		-- command
		crate_graph = {
			-- Backend used for displaying the graph
			-- see: https://graphviz.org/docs/outputs/
			-- default: x11
			backend = "x11",
			-- where to store the output, nil for no output stored (relative
			-- path from pwd)
			-- default: nil
			output = nil,
			-- true for all crates.io and external crates, false only the local
			-- crates
			-- default: true
			full = true,
		},
	},

	-- all the opts to send to nvim-lspconfig
	-- these override the defaults set by rust-tools.nvim
	-- see https://github.com/neovim/nvim-lspconfig/blob/master/doc/server_configurations.md#rust_analyzer
	server = {
		-- standalone file support
		-- setting it to false may improve startup time
		standalone = true,
	}, -- rust-analyer options

	-- debugging stuff
	dap = {
		adapter = {
			type = "executable",
			command = "lldb-vscode",
			name = "rt_lldb",
		},
	},
})
-- }}}

-- loop servers
local servers = { "rust_analyzer", "gopls" }
for _, lsp in ipairs(servers) do
	nvim_lsp[lsp].setup({
		on_attach = attach_lsp_keymaps,
		flags = {
			debounce_text_changes = 150,
		},
	})
end

LaunchLuaLSP = function()
	local client_id = vim.lsp.start_client({ cmd = { "lua-language-server", "--stdio" } })
	vim.lsp = require("vim.lsp")
	vim.lsp.buf_attach_client(0, client_id)
end

vim.cmd([[
  command! -range LaunchLuaLSP  execute 'lua LaunchLuaLSP()'
]])

-- Terminal
require("toggleterm").setup({
	open_mapping = [[<C-j>]],
	shading_factor = "1",
	shell = "bash",
})

-- Fuzzy Find
require("telescope").setup({
	defaults = {
		mappings = {
			i = {
				["<esc>"] = "close",
			},
		},
	},
})

require("null-ls").setup({
	on_attach = function(client, bufnr) end,
})

-- Formatting
require("prettier").setup({
	bin = "prettier",
	filetypes = { "javascript", "json" },
})

-- Globals
--   see :h lua-vim-variables and :h lua-vim-options
-- {{{
vim.env.FZF_DEFAULT_OPTS = "--layout=reverse"
vim.env.FZF_DEFAULT_COMMAND = "rg --files --hidden --glob !.git"
vim.opt.wildignore = { "*/cache/*", "*/tmp/*" }
vim.opt.number = true
-- tabs and spaces
vim.opt.tabstop = 2 -- size of each tab
vim.opt.shiftwidth = 2 -- spaces to shift when using << and >>
vim.opt.expandtab = true -- spaces when using tab
-- vim.g['ctrlp_prompt_mappings'] = {['AcceptSelection("t")'] = '<cr>'}
vim.g["airline#extensions#tabline#show_devicons"] = "true"
vim.g["airline#extensions#tabline#enabled"] = 1
vim.g["airline#extensions#tabline#buffer_nr_show"] = 1
--vim.g['airline#extensions#tabline#left_alt_sep'] = ' '
vim.g["airline#extensions#tabline#left_sep"] = " "
vim.g["noswapfile"] = true
vim.g["nocompatible"] = true
vim.g["hidden"] = true -- so buffers can hide
-- syntax enable
-- filetype plugin on

vim.opt.mouse = "a" -- basic obvious mouse behavior. wtf
vim.opt.cursorline = true
vim.opt.relativenumber = true
-- }}}

local popui = require("popui.ui-overrider");
vim.ui.select = popui;

require("prettier").setup({
	filetypes = { "javascript", "typescript" },
})

vim.cmd([[let loaded_netrwPlugin = 1]])

-- NERDTree
-- {{{
vim.api.nvim_set_keymap("", "<C-b>", ":NERDTreeToggle<CR>", {})
vim.g["NERDTreeMapActivateNode"] = "l" -- note: vim.g are globals
vim.g["NERDTreeWinPos"] = "left"
vim.g["NERDTreeMinimalUI"] = 1
-- vim.g['NERDTreeIgnore'] = ['^\.DS_Store$', '^tags$', '\.git$[[dir]]']
vim.g["NERDTreeShowHidden"] = 1
--vim.g['NERDTreeMapOpenInTab'] = '<ENTER>'
-- }}}


-- Colors
-- {{{
-- 36 forest green
-- 46 bright green
-- 62 purple
-- 120 pale neon green
-- 234 dark grey
vim.opt.termguicolors = false
vim.api.nvim_exec(
	[[
  colorscheme cosmic-barf
  hi Normal guibg=none ctermbg=none
  hi VertSplit ctermfg=234 gui=none
  hi Pmenu ctermfg=white guibg=#222222 ctermbg=234 ctermfg=246
  hi Folded ctermbg=DarkGrey ctermfg=White
  hi NERDTREEDir gui=none ctermfg=120 cterm=none
  hi NERDTreeCWD cterm=none ctermfg=62
]],
	false
)

-- inline colors
--require'colorizer'.setup()

--highlight LineNr ctermfg=darkgrey
--vim.cmd [[hi Comment ctermfg=darkgrey]]
-- cursor line bar
vim.cmd([[hi Cursor ctermbg=0 ctermfg=none]])
vim.cmd([[hi Comment ctermfg=darkgrey]])

local catppuccin = require("catppuccin")
catppuccin.setup({
  term_colors = false,
  transparent_background = true,
  colorscheme = "neon_latte"
})

-- }}}

