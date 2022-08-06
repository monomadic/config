-- @monomadic neovim 0.7+
-- requires: git

-- packer
--
local install_path = vim.fn.stdpath("data") .. "/site/pack/packer/start/packer.nvim"
if vim.fn.empty(vim.fn.glob(install_path)) > 0 then
	vim.fn.system({ "git", "clone", "--depth", "1", "https://github.com/wbthomason/packer.nvim", install_path })
	vim.cmd("packadd packer.nvim")
end

-- settings
--
--vim.g.vim_markdown_new_list_item_indent = 2 -- markdown list indent
--vim.opt.formatoptions = vim.o.formatoptions:gsub("r", ""):gsub("o", "")
vim.g.mapleader = " " -- leader key
vim.g.tex_flavor = "latex"
vim.g.vim_markdown_edit_url_in = 'current' -- open md links as (vplit | current)
vim.g.vim_markdown_new_list_item_indent = 1 -- indent new items on 'o' from n mode
-- vim.cmd "let g:clipboard = {'copy': {'+': 'pbcopy', '*': 'pbcopy'}, 'paste': {'+': 'pbpaste', '*': 'pbpaste'}, 'name': 'pbcopy', 'cache_enabled': 0}" -- hack for macos
vim.opt.clipboard = "unnamedplus" -- allows neovim to access the system clipboard (gnome)
vim.opt.conceallevel = 0 -- so that `` is visible in markdown files
vim.opt.cursorline = true -- highlight the current line
vim.opt.expandtab = false -- insert spaces when tab is pressed
vim.opt.foldlevelstart = 99
vim.opt.hidden = false -- switch buffer without unloading+saving them
vim.opt.hlsearch = false -- highlight all matches on previous search pattern
vim.opt.ignorecase = true -- ignore case when searching
vim.opt.laststatus = 2 -- 3 = global statusline (neovim 0.7+)
vim.opt.lazyredraw = true -- faster macros (force update with :redraw)
vim.opt.mouse = "a" -- allow the mouse to be used in neovim
vim.opt.number = true -- set numbered lines
vim.opt.number = true -- set numbered lines
vim.opt.relativenumber = true -- set relative numbered lines
vim.opt.scrolloff = 1000 -- keep line centered (disable if scrolling past eof is enabled)
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

vim.api.nvim_set_option('tabstop', 2)

-- https://gist.github.com/romainl/379904f91fa40533175dfaec4c833f2f
vim.cmd("colorscheme {{colorscheme}}");

-- tabline
vim.opt.showtabline = 2 -- show the global tab line at the top of neovim
vim.opt.tabline = ' /%{fnamemodify(getcwd(), ":t")}'
vim.api.nvim_create_autocmd("VimEnter", { pattern = "*", callback = function()
	vim.api.nvim_set_hl(0, "EndOfBuffer", { fg = "#444444" })
	vim.api.nvim_set_hl(0, "TabLineFill", { bg = "None" })

	vim.api.nvim_set_hl(0, "Title", { fg = "#CCFF00" })
	vim.api.nvim_set_hl(0, "VimwikiHeaderChar", { fg = "#44FF00" })
	vim.api.nvim_set_hl(0, "VimwikiLink", { fg = "#44FFFF" })
end })
-- vim.opt.tabline = "%!render_tabline()"

-- floats
vim.g.floaterm_borderchars = '        '
vim.g.floaterm_opener = 'edit'
vim.g.floaterm_title = ''
vim.api.nvim_create_autocmd("VimEnter", { pattern = "*", callback = function()
	vim.api.nvim_set_hl(0, "NormalFloat", {})
	vim.api.nvim_set_hl(0, "Floaterm", { bg = "Black" })
	vim.api.nvim_set_hl(0, "FloatermBorder", { bg = "Black" })
end })

-- vim.keymap.set({ 'n', 't' }, '<C-Space>', function()
-- 	if (vim.api.nvim_win_get_config(0).relative ~= '') then
-- 		vim.api.nvim_input('<ESC>')
-- 	end
-- 	vim.cmd("FloatermToggle")
-- end)

-- custom terminal float
vim.keymap.set('n', '<C-p>', function()
	local buf = vim.api.nvim_create_buf(false, true) -- new buffer for the term
	local selected_file = vim.fn.expand('%:p') -- the currently open filename

	vim.api.nvim_buf_set_option(buf, "filetype", "terminal")
	vim.api.nvim_buf_set_option(buf, "buflisted", false) -- don't show in bufferlist
	vim.api.nvim_open_win(buf, true, { -- true here focuses the buffer
		relative = 'editor',
		row = math.floor(0.05 * vim.o.lines),
		col = math.floor(0.1 * vim.o.columns),
		width = math.ceil(0.8 * vim.o.columns),
		height = math.ceil(0.7 * vim.o.lines),
		border = 'single'
	})

	local win = vim.api.nvim_get_current_win()
	vim.api.nvim_win_set_buf(win, buf)
	vim.wo.relativenumber = false -- turn off line numbers
	vim.wo.number = false

	-- local job_id = vim.fn.termopen(vim.o.shell)
	-- vim.api.nvim_chan_send(job_id, "lf\n")

	vim.cmd "startinsert" -- start in insert mode

	local lf_tmpfile = vim.fn.tempname()
	local lf_tmpdir = vim.fn.tempname()

	local process_cmd = 'lf -last-dir-path="' ..
			lf_tmpdir .. '" -selection-path="' .. lf_tmpfile .. '" '

	if selected_file ~= "" then
		process_cmd = process_cmd .. '"' .. selected_file .. '"'
	end
	--print(process_cmd)

	-- launch lf process
	vim.fn.termopen(process_cmd, {
		on_exit = function() -- job_id, exit_code, event_type

			-- -- if window is a float, close the window
			-- if vim.api.nvim_win_get_config(win).zindex then
			-- 	vim.api.nvim_win_close(win, true)
			-- end
			-- if lf correctly left us a tempfile
			if vim.loop.fs_stat(lf_tmpfile) then
				local contents = {}
				-- grab the entries that were selected (one per line)
				for line in io.lines(lf_tmpfile) do
					table.insert(contents, line)
				end
				if not vim.tbl_isempty(contents) then
					--vim.api.nvim_win_close(0, true) -- close current (0) with force

					for _, fname in pairs(contents) do
						-- and open them for editing
						vim.cmd(("%s %s"):format('edit', fname))
					end
				end
			end
		end,
	})
end)


-- #keymaps
-- to view current mappings: :verbose nmap <C-]>
-- split navigation
vim.keymap.set("n", "<C-j>", "<C-w><C-j>")
vim.keymap.set("i", "<C-j>", "<Esc><C-w><C-j>")
vim.keymap.set("n", "<C-k>", "<C-w><C-k>")
vim.keymap.set("i", "<C-k>", "<Esc><C-w><C-k>")
vim.keymap.set("n", "<C-l>", "<C-w><C-l>")
vim.keymap.set("i", "<C-l>", "<Esc><C-w><C-l>")
vim.keymap.set("n", "<C-h>", "<C-w><C-h>")
vim.keymap.set("i", "<C-h>", "<Esc><C-w><C-h>")
vim.keymap.set("n", "<C-w><C-d>", "<cmd>vsplit<CR>")

vim.keymap.set("n", "}", "}j")
vim.keymap.set("n", "{", "k{j")

-- indent in insert mode
vim.keymap.set("i", "<C-]>", "<C-t>")
vim.keymap.set("i", "<C-[>", "<C-d>")

vim.keymap.set("n", "gd", function() vim.lsp.buf.definition() end)
vim.keymap.set("n", "gD", function() vim.lsp.buf.declaration() end)
vim.keymap.set("n", "gr", function() vim.lsp.buf.references() end)

-- use ; for commands instead of :
vim.keymap.set("n", ";", ":")
-- vim.keymap.set("n", "<Space>", ":")

-- #plugins
--
vim.cmd [[packadd packer.nvim]]
require('packer').startup(function(use)

	--use { 'wbthomason/packer.nvim', opt = true }

	use {
		"NvChad/nvterm",
		config = function()
			require("nvterm").setup()
		end,
	}

	-- mini utility plugins
	-- https://github.com/echasnovski/mini.nvim#general-principles
	--
	-- use { 'echasnovski/mini.nvim', branch = 'stable', config = function()
	-- 	-- require('mini.cursorword').setup({}) -- highlight word under cursor
	-- 	require('mini.fuzzy').setup({}) -- fuzzy search (like rg)
	-- end }

	--use {'voldikss/vim-floaterm'}
	use { 'kdheepak/lazygit.nvim' }
	use { 'lambdalisue/suda.vim' } -- sudo

	-- #colorschemes
	-- use { 'ellisonleao/gruvbox.nvim', config = function()
	-- 	vim.o.background = "dark"
	-- end } -- theme
	use 'bluz71/vim-nightfly-guicolors'
	use { 'Mofiqul/vscode.nvim', config = function()
		vim.o.background = 'dark'
		require('vscode').setup({
			transparent = true, -- transparent bg
			color_overrides = {
				vscGreen = '#555555',
			},
		})
	end }
	use { 'lunarvim/darkplus.nvim' }
	use {
		"olimorris/onedarkpro.nvim",
		config = function()
			require("onedarkpro").setup({
				theme = "onedark_dark"
			})
		end
	}
	use { 'projekt0n/github-nvim-theme' }

	-- lspconfig (with mason)
	use {
		"williamboman/mason.nvim",
		"williamboman/mason-lspconfig.nvim",
		"neovim/nvim-lspconfig",
		config = function()
			-- require("mason").setup()
			-- require("mason-lspconfig").setup()
		end
	}
	require("mason").setup()
	require("mason-lspconfig").setup()

	-- treesitter
	use { 'nvim-treesitter/nvim-treesitter', requires = { "p00f/nvim-ts-rainbow" }, config = function()
		require 'nvim-treesitter.configs'.setup {
			ensure_installed = { "rust", "bash", "yaml", "typescript", "javascript", "markdown" },
			highlight = { enable = true },
			rainbow = { enable = true, colors = {
				"#9944FF",
				"#45F588",
				"#FFFF00",
				"#FF44FF",
				"#00BBFF",
				"#FFAACC",
				"#AAFF66",
			} },
		}
	end }

	use { 'nvim-lua/popup.nvim' }
	use { 'nvim-lua/plenary.nvim' }

	use { 'nvim-telescope/telescope.nvim', config = function()
		require('telescope').setup {
			defaults = {
				prompt_prefix = "   ",
				selection_caret = "  ",
				entry_prefix = "  ",
				initial_mode = "insert",
				selection_strategy = "reset",
				sorting_strategy = "ascending",
				layout_strategy = "horizontal",
				layout_config = {
					horizontal = {
						prompt_position = "top",
						preview_width = 0.55,
						results_width = 0.8,
					},
					vertical = {
						mirror = false,
					},
					width = 0.87,
					height = 0.80,
					preview_cutoff = 120,
				},
				file_sorter = require("telescope.sorters").get_fuzzy_file,
				set_env = { ["COLORTERM"] = "truecolor" },
				file_ignore_patterns = { ".git/", ".cache", "%.o", "%.a", "%.out", "%.class", "%.pdf", "%.mkv", "%.mp4", "%.zip",
					"*.lock", "node_modules", "target" },
				generic_sorter = require("telescope.sorters").get_generic_fuzzy_sorter,
				path_display = { "truncate" },
				winblend = 0,
				border = {},
				mappings = {
					i = {
						["<Esc>"] = "close",
						["<Tab>"] = "close",
						["<C-l>"] = require("telescope.actions.layout").toggle_preview,
						["<C-u>"] = false,
					},
				},
				extensions_list = { "themes", "terms" },
			},
		}
	end }

	--use { 'vijaymarupudi/nvim-fzf' }

	-- completion TODO: clear out useless plugs, maybe custom one
	use {
		'hrsh7th/nvim-cmp',
		wants = { "LuaSnip" },
		requires = { "L3MON4D3/LuaSnip" }
	}
	use { 'hrsh7th/cmp-nvim-lsp' }

	-- inline colors
	use { 'norcalli/nvim-colorizer.lua', config = function()
		require "colorizer".setup()
	end }

	use 'stevearc/aerial.nvim' -- aerial view / overview of lsp structure

	-- jump/sneak
	use {
		"phaazon/hop.nvim", -- alternative to sneak
		branch = "v1", -- optional but strongly recommended
		config = function()
			require("hop").setup({ keys = "etovxqpdygfblzhckisuran" })
			vim.keymap.set("n", "s", "<Cmd>HopWord<CR>")
			vim.keymap.set("n", "S", "<Cmd>HopPattern<CR>")
		end
	}

	-- git status in git gutter
	use { "lewis6991/gitsigns.nvim", requires = { "nvim-lua/plenary.nvim" }, config = function()
		require('gitsigns').setup {
			on_attach = function()
				local gs = package.loaded.gitsigns
				-- jump between git hunks
				vim.keymap.set('n', ']g', function()
					if vim.wo.diff then return ']g' end
					vim.schedule(function() gs.next_hunk() end)
					return '<Ignore>'
				end)
				vim.keymap.set('n', '[g', function()
					if vim.wo.diff then return '[g' end
					vim.schedule(function() gs.prev_hunk() end)
					return '<Ignore>'
				end)

				vim.api.nvim_set_hl(0, "GitSignsAdd", { fg = "#44FF00" })
				vim.api.nvim_set_hl(0, "GitSignsChange", { fg = "#FFFF00" })
				vim.api.nvim_set_hl(0, "GitSignsDelete", { fg = "#FF0088" })
			end
		}
	end }

	-- interacting with marks, including putting them in the gutter / sign column
	use { 'chentoast/marks.nvim', config = function()
		require 'marks'.setup {
			sign_priority = { lower = 10, upper = 15, builtin = 8, bookmark = 20 },
		}
	end }

	use { 'junegunn/goyo.vim', config = function()
		vim.g.goyo_width = "65%"
	end } -- distraction-free / zen mode

	use {
		"folke/trouble.nvim",
		requires = "kyazdani42/nvim-web-devicons",
		config = function()
			require("trouble").setup {
			}
		end
	}

	use { 'numToStr/Comment.nvim',
		config = function()
			-- `gcc` line comment
			-- `gcA` line comment at eol
			-- `gc0` line comment at bol
			-- `gco` line comment at line-open
			-- `gbc` block comment
			require('Comment').setup()
			--vim.keymap.set("n", "<C-.>", "gcc")
		end
	}

	-- nvim-tree
	use {
		'kyazdani42/nvim-tree.lua',
		requires = {
			'kyazdani42/nvim-web-devicons', -- optional, for file icons
		},
		config = function()
			require('nvim-tree').setup {}
		end,
		tag = 'nightly' -- optional, updated every week. (see issue #1193)
	}

	-- -- neotree
	-- use { 'nvim-neo-tree/neo-tree.nvim',
	-- 	requires = {
	-- 		"nvim-lua/plenary.nvim",
	-- 		"kyazdani42/nvim-web-devicons",
	-- 		"MunifTanjim/nui.nvim",
	-- 	},
	-- 	config = function()
	-- 		require('neo-tree').setup {
	-- 			close_if_last_window = true,
	-- 			popup_border_style = "solid",
	-- 			window = {
	-- 				mappings = {
	-- 					["l"] = "open",
	-- 					["<C-l>"] = "open_vsplit",
	-- 				}
	-- 			}
	-- 		}
	--
	-- 		vim.api.nvim_create_autocmd("VimEnter", { pattern = "*", callback = function()
	-- 			vim.api.nvim_set_hl(0, "NeoTreeFloatBorder", { bg = "None", fg = "None" })
	-- 			vim.api.nvim_set_hl(0, "NeoTreeFloatTitle", { bg = "None", fg = "None" })
	-- 		end })
	--
	-- 		vim.keymap.set("n", "<leader>t", "<cmd>Neotree<CR>")
	-- 		vim.keymap.set("n", "<leader>b", "<cmd>Neotree buffers<CR>")
	-- 		vim.keymap.set("n", "<C-b>", "<Cmd>NeoTreeFloatToggle<CR>")
	-- 		--vim.api.nvim_set_hl(0, "NeoTreeFloatBorder", { bg = "Red" })
	--
	-- 		-- vim.cmd("hi NeoTreeFloatBorder guifg=bg guibg=bg");
	-- 		-- vim.cmd("hi NeoTreeFloatBorder guifg=bg guibg=bg");
	-- 		-- vim.cmd("hi NeoTreeFloatTitle guifg=bg guibg=bg");
	-- 	end
	-- }

	use { 'preservim/vim-markdown' }

	-- surround completion
	use {
		"numToStr/Surround.nvim"
	}

	-- use{
	--   -- surround inline change
	--   "tpope/vim-surround",
	--   config = function()
	--     require('surround').setup {}
	--   end
	-- }

	-- inline diagnostics
	use({
		"https://git.sr.ht/~whynothugo/lsp_lines.nvim",
		config = function()
			require("lsp_lines").setup()
		end,
	})


	-- #lsp
	-- cargo
	use {
		"saecki/crates.nvim",
		event = { "BufRead Cargo.toml" },
		requires = { { "nvim-lua/plenary.nvim" } },
		config = function()
			require('crates').setup {}
		end,
	}

	-- null lsp
	use {
		"jose-elias-alvarez/null-ls.nvim",
		config = function()
			local null_ls = require("null-ls")
			null_ls.setup {
				sources = {
					null_ls.builtins.code_actions.gitsigns,
				}
			}
		end,
		requires = { "nvim-lua/plenary.nvim" },
	}

	use { 'simrat39/rust-tools.nvim', config = function()
	end }

	-- formatting
	use { 'lukas-reineke/lsp-format.nvim', config = function()
		require("lsp-format").setup {}
	end }

	use { "petertriho/nvim-scrollbar", config = function()
		require('scrollbar').setup() -- side scrollbar with git support
	end }

	-- use { "lukas-reineke/indent-blankline.nvim", config = function()
	-- 	require("indent_blankline").setup({
	-- 		show_current_context = true,
	-- 		show_current_context_start = true,
	-- 		filetype_exclude = { "neo-tree", "help", "floaterm", "SidebarNvim", "" },
	-- 	})
	-- end }

	-- snippets
	use {
		'L3MON4D3/LuaSnip',
	}

	-- nvim-snippy
	use { 'dcampos/nvim-snippy', config = function()
		require('snippy').setup({
			mappings = {
				is = {
					['<Tab>'] = 'expand_or_advance',
					['<S-Tab>'] = 'previous',
				},
				nx = {
					['<leader>x'] = 'cut_text',
				},
			},
		})
		local mappings = require('snippy.mapping')
		vim.keymap.set('i', '<Tab>', mappings.expand_or_advance('<Tab>'))
		vim.keymap.set('s', '<Tab>', mappings.next('<Tab>'))
		vim.keymap.set({ 'i', 's' }, '<S-Tab>', mappings.previous('<S-Tab>'))
		vim.keymap.set('x', '<Tab>', mappings.cut_text, { remap = true })
		vim.keymap.set('n', 'g<Tab>', mappings.cut_text, { remap = true })
		vim.keymap.set('n', '<C-g>', '<Cmd>LazyGit<CR>')
	end }

	use { 'honza/vim-snippets' }
	use { 'dcampos/cmp-snippy' }
	-- for luasnip and cmp
	use { 'saadparwaiz1/cmp_luasnip' }
	use {
		"benfowler/telescope-luasnip.nvim",
		module = "telescope._extensions.luasnip",
	}

	-- better lsp ui
	use { "glepnir/lspsaga.nvim", config = function()
		local lsp_saga = require('lspsaga')

		vim.keymap.set("n", "<leader>lo", "<cmd>Lspsaga lsp_finder<CR>")
		vim.keymap.set("n", "<leader>a", ":Lspsaga code_action")
		vim.keymap.set("n", "<leader>r", ":Lspsaga rename")
		vim.keymap.set("n", 'K', '<cmd>lua vim.lsp.buf.hover()<CR>')

		lsp_saga.init_lsp_saga {
			show_outline = {
				jump_key = '<CR>',
			}
		}
	end }

	-- lua formatting
	use({ "ckipp01/stylua-nvim" })

	-- todo
	use {
		"folke/todo-comments.nvim",
		requires = "nvim-lua/plenary.nvim",
		config = function()
			require('todo-comments').setup {}
		end
	}

	use { 'vimwiki/vimwiki', config = function()
		vim.keymap.set("n", "gi", "<Cmd>VimwikiIndex<CR>")
		vim.keymap.set("n", "gw", "<Cmd>VimwikiGoto ")
		vim.g.vimwiki_list = {
			{
				path = '~/wiki/',
				syntax = 'markdown',
				ext = '.md'
			}
		}
	end }

	-- make background highlight groups transparent
	use { 'xiyaowong/nvim-transparent',
		config = function()
			require("transparent").setup {
				enable = true,
				extra_groups = { "NvimTreeNormal" },
			}
		end
	}
end)



-- go back
vim.keymap.set('n', '<bs>', ':edit #<cr>', { silent = true })

vim.api.nvim_set_hl(0, "Term", { bg = "Black" })


-- tree
--


-- #colors
-- #00FF99 #FF00CC #FFFF00 #00CCFF
--
vim.cmd('syntax on')
vim.cmd("hi WinSeparator guifg=none"); -- I think this is the split column
vim.cmd("hi TodoBgTODO guibg=#FFFF00 guifg=black");
vim.cmd("hi TodoFgTODO guifg=#FFFF00");
vim.cmd("hi DiagnosticVirtualTextHint guifg=#F0F0AA")
vim.cmd("hi DiagnosticVirtualTextError guifg=#F02282")

-- active window
-- vim.cmd("set winhighlight=Normal:ActiveWindow,NormalNC:InactiveWindow");
-- vim.cmd("hi ActiveWindow guibg=#092236");
-- vim.cmd("hi InactiveWindow guibg=#000001");

vim.cmd([[set fillchars+=vert:\ ]]) -- remove awful vertical split character
--
-- remove trailing whitespaces on save
vim.cmd([[autocmd BufWritePre * %s/\s\+$//e]])
--
-- remove trailing newline on save
vim.cmd([[autocmd BufWritePre * %s/\n\+\%$//e]])

-- when opening vim
vim.api.nvim_create_autocmd("VimEnter", { pattern = "*", callback = function()
	-- if no args are passed
	if vim.fn.argc() == 0 then
		vim.cmd "enew"

		--vim.cmd "setlocal bufhidden=wipe buftype=nofile nocursorline nonumber nolist"
		vim.cmd "setlocal bufhidden=wipe buftype=nofile nocursorcolumn nocursorline nolist nonumber noswapfile norelativenumber"

		--local buf = vim.api.nvim_create_buf(false, true) -- new buffer for the term

		--vim.cmd "setlocal bufhidden=wipe buftype=nofile nobuflisted nocursorcolumn nocursorline nolist nonumber noswapfile norelativenumber"
		vim.cmd([[call append('$', "")]])
	end
end })

-- hide line-bar in insert-mode
vim.api.nvim_create_autocmd("InsertEnter", { pattern = "*", callback = function()
	vim.o.cursorline = false
end })
vim.api.nvim_create_autocmd("InsertLeave", { pattern = "*", callback = function()
	vim.o.cursorline = true
end })

-- only show line-bar on current buffer, on active window
vim.api.nvim_create_autocmd("BufLeave", { pattern = "*", callback = function()
	vim.o.cursorline = false
end })
vim.api.nvim_create_autocmd("BufEnter", { pattern = "*", callback = function()
	vim.o.cursorline = true
end })
vim.api.nvim_create_autocmd("WinLeave", { pattern = "*", callback = function()
	vim.o.cursorline = false
end })
vim.api.nvim_create_autocmd("WinEnter", { pattern = "*", callback = function()
	vim.o.cursorline = true
end })

--
-- KEYMAPS
--
-- leader keys
vim.keymap.set("n", "<leader>s", "<cmd>write<CR>")
vim.keymap.set("n", "<leader>ww", "<cmd>wq!<CR>")
vim.keymap.set("n", "<leader>wq", "<cmd>wq<CR>")
vim.keymap.set("n", "<leader>q", "<cmd>quit<CR>")
vim.keymap.set("n", "<leader>!", "<cmd>quit!<CR>")
vim.keymap.set("n", "Q", "<cmd>quit<CR>")
vim.keymap.set("n", "WQ", "<cmd>wq<CR>")
vim.keymap.set("n", "<leader>lf", "<cmd>Lf<CR>")
vim.keymap.set("n", "<leader>lg", "<cmd>LazyGit<CR>")
vim.keymap.set("n", "<leader>h1", "<Esc>/1.<CR>")
vim.keymap.set("n", "<leader>h2", "<Esc>/2.<CR>")
--
-- split navigation
vim.keymap.set("n", "<C-j>", "<C-w><C-j>")
vim.keymap.set("n", "<C-k>", "<C-w><C-k>")
vim.keymap.set("n", "<C-l>", "<C-w><C-l>")
vim.keymap.set("n", "<C-h>", "<C-w><C-h>")
vim.keymap.set("n", "<C-s>", "<Cmd>write<CR>");
vim.keymap.set("i", "<C-s>", "<Esc><Cmd>write<CR>");
--
-- grep entire project
vim.keymap.set("n", "<C-f>", function()
	require('telescope.builtin').live_grep()
end)
--
-- emacs style shortcuts in insert mode (yes, i am like that)
vim.keymap.set("i", "<C-n>", "<Down>")
vim.keymap.set("i", "<C-p>", "<Up>")
vim.keymap.set("i", "<C-b>", "<Left>")
vim.keymap.set("i", "<C-f>", "<Right>")
vim.keymap.set("i", "<C-e>", "<End>")
vim.keymap.set("i", "<C-a>", "<Home>")
vim.keymap.set("i", "<C-s>", "<Esc>:write<CR>")
vim.keymap.set("n", "<leader>o", function()
	require('telescope.builtin').find_files(
		require('telescope.themes').get_dropdown()
	)
end)
-- vim.keymap.set("n", "<C-g>", function()
-- 	vim.cmd "FloatermNew --height=0.9 --width=0.9 --wintype=float --name=lazygit --position=center --autoclose=2 lazygit"
-- end)

--telekasten
-- vim.keymap.set("n", "z", "<Cmd>Telekasten panel<CR>")
-- vim.keymap.set("n", "zn", "<Cmd>Telekasten new_note<CR>")
-- vim.keymap.set("n", "zN", "<Cmd>Telekasten new_templated_note<CR>")
-- vim.keymap.set("n", "zt", "<Cmd>Telekasten show_tags<CR>")
-- vim.keymap.set("n", "zo", "<Cmd>Telekasten toggle_todo<CR>")
-- vim.keymap.set("n", "zT", "<Cmd>Telekasten find_weekly_notes<CR>")
-- vim.keymap.set("n", "zf", "<Cmd>Telekasten find_notes<CR>")
-- vim.keymap.set("n", "zr", "<Cmd>Telekasten rename_note<CR>")
-- vim.keymap.set("n", "zg", "<Cmd>Telekasten follow_link<CR>")
-- vim.keymap.set("n", "zr", "<Cmd>Telekasten show_backlinks<CR>")
-- vim.keymap.set("n", "gz", "<Cmd>Telekasten follow_link<CR>")

-- move lines up and down in visual mode
vim.keymap.set("x", "K", ":move '<-2<CR>gv-gv")
vim.keymap.set("x", "J", ":move '>+1<CR>gv-gv")
--
-- useful bindings
-- vim.keymap.set("i", "kj", "<Esc>")
vim.keymap.set("n", "<leader>sv", "<cmd>source $MYVIMRC<CR>")
--vim.keymap.set("n", "<leader>ev", "<cmd>vs $MYVIMRC<CR>")
--
-- quote quickly
--vim.keymap.set("i", '<leader>"', '<Esc>viw<Esc>a"<Esc>bi"<Esc>leli')
vim.keymap.set("v", '<leader>"', '<Esc>`<i"<Esc>`>ea"<Esc>')
-- substitute shortcut
vim.keymap.set("n", "S", ":%s//g<Left><Left>")
vim.keymap.set("v", "S", ":s//g<Left><Left>")
-- more reachable line start/end
vim.keymap.set("n", "H", "^")
vim.keymap.set("n", "L", "$")
-- write to ----READONLY---- files
-- vim.keymap.set("c", "<C-w>", "execute 'silent! write !sudo tee % >/dev/null' <bar> edit!")

-- ===== find project root for quick cd =====
-- function find_project_root()
--   local id = [[.git]]
--   local file = api.nvim_buf_get_name(0)
--   local root = vim.fn.finddir(id, file .. ';')
--   if root ~= "" then
--     root = root:gsub(id, '')
--     print(root)
--     vim.api.nvim_set_current_dir(root)
--   else
--     print("No repo found.")
--   end
-- end

--
-- STATUSLINE
--
-- get(b:,'gitsigns_status','')
local stl = {
	-- ' %{fnamemodify(getcwd(), ":t")}',
	-- ' %{pathshorten(expand("%:p"))}',
	' %{fnamemodify(expand("%"), ":~:.")}', -- current file
	-- ' %{pathshorten(expand("%"), ":~:.")}',
	'%=',
	-- '  %{FugitiveStatusline()}',
	' %M', ' %y', ' %r'
}
vim.o.statusline = table.concat(stl)
vim.api.nvim_create_autocmd("VimEnter", { pattern = "*", callback = function()
	vim.api.nvim_set_hl(0, "StatusLine", {}) -- active
	vim.api.nvim_set_hl(0, "StatusLineNC", {}) -- active
end })

-- #telescope
vim.keymap.set("n", 'tb', '<cmd>Telescope buffers<cr>')
vim.keymap.set("n", 'tc', '<cmd>Telescope commands<cr>')
vim.keymap.set("n", '<leader>f', '<cmd>Telescope find_files<cr>')
vim.keymap.set("n", '<leader>h', '<cmd>Telescope oldfiles<cr>')
vim.keymap.set("n", '<leader>c', '<cmd>Telescope commands<cr>')
vim.keymap.set("n", '<leader>ch', '<cmd>Telescope command_history<cr>')
vim.keymap.set("n", '<leader>g', '<cmd>Telescope live_grep<cr>')
vim.keymap.set("n", 'ts', '<cmd>Telescope spell_suggest<cr>')
vim.keymap.set('', '<F1>', '<cmd>Telescope help_tags<cr>')
vim.keymap.set('n', 'td', '<Cmd>Telescope diagnostics<cr>')
vim.keymap.set('n', 'tgb', '<Cmd>Telescope git_branches<cr>')
vim.keymap.set('n', 'tgc', '<Cmd>Telescope git_bcommits<cr>')
vim.keymap.set('n', 'tgd', '<Cmd>Telescope git_status<cr>')
vim.keymap.set('n', 'tk', '<Cmd>Telescope keymaps<cr>')
vim.keymap.set('n', 'tld', '<Cmd>Telescope lsp_definitions<cr>')
vim.keymap.set('n', 'tli', '<Cmd>Telescope lsp_implementations<cr>')
vim.keymap.set('n', 'tls', '<Cmd>Telescope lsp_document_symbols<cr>')
vim.keymap.set('n', 'tlw', '<Cmd>Telescope lsp_workspace_symbols<cr>')
vim.keymap.set('n', 'tm', '<Cmd>Telescope marks<cr>')
vim.keymap.set('n', 'tr', '<Cmd>Telescope live_grep<cr>')
vim.keymap.set('n', 'tt', '<Cmd>TodoTelescope<cr>')
vim.keymap.set('n', 'tz', '<Cmd>Telekasten find_notes<cr>')
vim.keymap.set("n", "ts", function()
	require("luasnip.loaders.from_snipmate").lazy_load()
	require('telescope').load_extension('luasnip')
	vim.api.nvim_command('Telescope luasnip')
end)

-- ===== simple session management =====
local session_dir = vim.fn.stdpath('data') .. '/sessions/'
vim.keymap.set("n", '<leader>mks', ':mks! ' .. session_dir)
vim.keymap.set("n", '<leader>lds', ':%bd | so ' .. session_dir)

--
-- LSP
--
local custom_attach = function(client, bufnr)
	print("lsp started");

	require "lsp-format".on_attach(client)

	--require('cmp-lsp').on_attach(client);
	vim.lsp.handlers["textDocument/publishDiagnostics"] = vim.lsp.with(
		vim.lsp.diagnostic.on_publish_diagnostics, { virtual_text = false, update_in_insert = false }
	)
	-- automatic diagnostics popup
	vim.api.nvim_command('autocmd CursorHold <buffer> lua vim.diagnostic.show()')
	-- speedup diagnostics popup
	vim.o.updatetime = 500
	-- diagnostic settings
	vim.diagnostic.config({
		virtual_text = false,
		signs = true, -- sidebar signs
		underline = true,
		severity_sort = true,
	})

	require("aerial").setup({
		backends = { "lsp", "treesitter", "markdown" },
	})
	require("aerial").on_attach(client, bufnr)
	require('telescope').load_extension('aerial')

	-- diagnostics icon
	local signs = { Error = "┃ ", Warn = "┃ ", Hint = "┃ ", Info = "┃ " }
	for type, icon in pairs(signs) do
		local hl = "DiagnosticSign" .. type
		-- vim.cmd("hi " .. hl .. " guibg=none")
		vim.fn.sign_define(hl, { text = icon, texthl = hl })
	end

	-- diagnostics float on hover
	vim.api.nvim_create_autocmd("CursorHold", {
		buffer = bufnr,
		callback = function()
			local opts = {
				focusable = false,
				close_events = { "BufLeave", "CursorMoved", "InsertEnter", "FocusLost" },
				border = 'rounded',
				source = 'always',
				prefix = ' ',
				scope = 'cursor',
			}
			vim.diagnostic.open_float(nil, opts)
		end
	})

	vim.keymap.set("n", 'gD', '<cmd>lua vim.lsp.buf.declaration()<CR>')
	--vim.keymap.set("n", '<c-]>', '<cmd>lua vim.lsp.buf.definition()<CR>')
	--vim.keymap.set("n", 'K', '<cmd>lua vim.lsp.buf.hover()<CR>')
	vim.keymap.set("n", 'gr', '<cmd>lua vim.lsp.buf.references()<CR>')
	vim.keymap.set("n", 'gs', '<cmd>lua vim.lsp.buf.signature_help()<CR>')
	vim.keymap.set("n", 'gi', '<cmd>lua vim.lsp.buf.implementation()<CR>')
	vim.keymap.set("n", 'ga', '<cmd>lua vim.lsp.buf.code_action()<CR>')
	--vim.keymap.set("n", '<leader>r', '<cmd>lua vim.lsp.buf.rename()<CR>')
	vim.keymap.set("n", '<leader>=', '<cmd>lua vim.lsp.buf.formatting()<CR>')
	-- vim.keymap.set("n", '<C-]>', '<cmd>lua vim.diagnostic.goto_next()<CR>')
	-- vim.keymap.set("n", '<C-[>', '<cmd>lua vim.diagnostic.goto_prev()<CR>')
	vim.keymap.set("n", ']d', '<cmd>lua vim.diagnostic.goto_next()<CR>')
	vim.keymap.set("n", '[d', '<cmd>lua vim.diagnostic.goto_prev()<CR>')
	vim.keymap.set("n", '\\', '<cmd>TroubleToggle<CR>')

	-- vim.keymap.set("n", '<leader>a', function()
	-- 	vim.lsp.buf.code_action()
	-- end)

	vim.keymap.set("n", ']e', function()
		vim.diagnostic.goto_next({
			severity = vim.diagnostic.severity.ERROR,
		})
	end)

	vim.keymap.set("n", '[e', function()
		vim.diagnostic.goto_prev({
			severity = vim.diagnostic.severity.ERROR,
		})
	end)
end

-- cmp/lsp config
--
-- tsserver: npm install -g typescript typescript-language-server

local capabilities = require('cmp_nvim_lsp').update_capabilities(
	vim.lsp.protocol.make_client_capabilities()
);
local lspconfig = require('lspconfig')
for _, lsp in ipairs({ 'bashls', 'rnix', 'zk', 'tsserver' }) do
	lspconfig[lsp].setup {
		on_attach = custom_attach,
		capabilities = capabilities,
	}
end

-- lspconfig.tsserver.setup({
-- 	on_attach = function(client, _)
-- 		require('nvim-lsp-ts-utils').setup({
-- 			filter_out_diagnostics_by_code = { 80001 },
-- 		})
-- 		require('nvim-lsp-ts-utils').setup_client(client)
-- 	end,
-- })

-- lspconfig.denols.setup {
-- 	root_dir = lspconfig.util.root_pattern("mod.ts", "mod.js")
-- }

lspconfig.sumneko_lua.setup {
	on_attach = custom_attach,
	capabilities = capabilities,
	settings = {
		Lua = {
			diagnostics = {
				globals = { 'vim' }
			}
		}
	}
}

-- #rust-tools
--
require('rust-tools').setup({
	tools = {
		autoSetHints = true,
		hover_with_actions = true,
		runnables = {
			use_telescope = true
		},
		inlay_hints = {
			show_parameter_hints = false,
			parameter_hints_prefix = "",
			other_hints_prefix = "",
		},
	},

	-- see https://github.com/neovim/nvim-lspconfig/blob/master/CONFIG.md#rust_analyzer
	server = {
		on_attach = custom_attach,
		settings = {
			-- to enable rust-analyzer settings visit:
			-- https://github.com/rust-analyzer/rust-analyzer/blob/master/docs/user/generated_config.adoc
			["rust-analyzer"] = {
				-- enable clippy on save
				checkOnSave = {
					command = "clippy"
				},
			}
		}
	},
})

vim.g.rust_recommended_style = 0 -- don't use default rust styles (causes indent problems)
vim.g.rust_fold = 2
vim.g.rustfmt_autosave = true
vim.g.rust_conceal_mod_path = true
vim.g.rust_conceal = true

-- markdown
--
vim.cmd('autocmd FileType markdown set autowriteall') -- ensure write upon leaving a page
vim.cmd('autocmd FileType markdown set wrap') -- wrap only markdown

-- cmp
local cmp = require('cmp')
cmp.setup {
	sources = {
		{ name = 'nvim_lsp' },
		{ name = 'snippy' },
	},
	preselect = cmp.PreselectMode.None,
	snippet = {
		expand = function(args)
			require('luasnip').lsp_expand(args.body)
		end,
	},
	mapping = {
		['<CR>'] = cmp.mapping.confirm({
			behavior = cmp.ConfirmBehavior.Replace,
			select = false, -- false = only complete if an item is actually selected
		}),
		['<Tab>'] = function(fallback)
			if cmp.visible() then
				cmp.select_next_item()
			else
				fallback()
			end
		end,
		['<S-Tab>'] = function(fallback)
			if cmp.visible() then
				cmp.select_prev_item()
			else
				fallback()
			end
		end,
	},
}
