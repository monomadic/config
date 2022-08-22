-- @monomadic
-- requires: git, neovim 0.7+

--
-- PLUGINS
--
--   PackerCompile: compile plugins
--   PackerClean: remove unused plugs
--   PackerInstall: add new plugins
--   PackerUpdate: PackerClean, PackerUpdate, PackerInstall
--   PackerSync: PackerUpdate, PackerCompile
--
-- autoinstall packer:
local install_path = vim.fn.stdpath("data") .. "/site/pack/packer/start/packer.nvim"
if vim.fn.empty(vim.fn.glob(install_path)) > 0 then
	print("downloading packer...")
	vim.fn.system({ "git", "clone", "--depth", "1", "https://github.com/wbthomason/packer.nvim", install_path })
	vim.cmd 'packadd packer.nvim'
end

vim.cmd 'packadd packer.nvim' -- only required if packer is opt

require('packer').startup(function(use)
	use 'wbthomason/packer.nvim'

	--
	-- COLORSCHEMES
	--
	use 'bluz71/vim-nightfly-guicolors'
	use 'lunarvim/darkplus.nvim'
	use 'projekt0n/github-nvim-theme'
	use { 'Mofiqul/vscode.nvim', config = function()
		vim.o.background = 'dark'
		require('vscode').setup({
			transparent = true, -- transparent bg
			color_overrides = {
				vscGreen = '#555555',
			},
		})
	end }
	use {
		"olimorris/onedarkpro.nvim",
		config = function()
			require("onedarkpro").setup({
				theme = "onedark_dark"
			})
		end
	}

	-- better % using treesiter - vimscript
	use { 'andymass/vim-matchup', event = 'VimEnter' }

	-- show filename in winbar
	use { "b0o/incline.nvim", config = function()
		require('incline').setup()
	end }

	use { 'neovim/nvim-lspconfig', config = function()
		local custom_attach = function(client, bufnr)
			require "lsp-format".on_attach(client)

			vim.lsp.handlers["textDocument/publishDiagnostics"] = vim.lsp.with(
				vim.lsp.diagnostic.on_publish_diagnostics, { virtual_text = false, update_in_insert = false }
			)
			vim.api.nvim_command('autocmd CursorHold <buffer> lua vim.diagnostic.show()') -- automatic diagnostics popup
			vim.o.updatetime = 500 -- time before diagnostics popup in ms
			vim.diagnostic.config({
				virtual_text = true,
				signs = true, -- sidebar signs
				underline = false,
				severity_sort = true,
			})

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

			--vim.keymap.set("n", 'gD', '<cmd>lua vim.lsp.buf.declaration()<CR>') -- not used by most lsp servers
			--vim.keymap.set("n", 'K', '<cmd>lua vim.lsp.buf.hover()<CR>')
			vim.keymap.set("n", 'gr', '<cmd>lua vim.lsp.buf.references()<CR>')
			vim.keymap.set("n", 'gs', '<cmd>lua vim.lsp.buf.signature_help()<CR>')
			--vim.keymap.set("n", 'gi', '<cmd>lua vim.lsp.buf.implementation()<CR>')
			vim.keymap.set("n", 'ga', '<cmd>lua vim.lsp.buf.code_action()<CR>')
			--vim.keymap.set("n", '<leader>r', '<cmd>lua vim.lsp.buf.rename()<CR>')
			vim.keymap.set("n", '<leader>=', '<cmd>lua vim.lsp.buf.formatting()<CR>')
			-- vim.keymap.set("n", '<C-]>', '<cmd>lua vim.diagnostic.goto_next()<CR>')
			-- vim.keymap.set("n", '<C-[>', '<cmd>lua vim.diagnostic.goto_prev()<CR>')
			vim.keymap.set("n", 'gt', '<cmd>TroubleToggle<CR>')
			vim.keymap.set("n", 'gD', '<cmd>TroubleToggle<CR>')

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
		local capabilities = require('cmp_nvim_lsp').update_capabilities(
			vim.lsp.protocol.make_client_capabilities()
		);
		local lspconfig = require('lspconfig')
		for _, lsp in ipairs({ 'bashls', 'rnix', 'tsserver' }) do
			lspconfig[lsp].setup {
				on_attach = custom_attach,
				capabilities = capabilities,
			}
		end

		lspconfig.eslint.setup({
			on_attach = custom_attach,
			capabilities = capabilities,
		})

		lspconfig.tsserver.setup({
			on_attach = function(client, _)
				require('nvim-lsp-ts-utils').setup({
					filter_out_diagnostics_by_code = { 80001 },
				})
				require('nvim-lsp-ts-utils').setup_client(client)
			end,
		})

		-- lspconfig.denols.setup {
		-- 	root_dir = lspconfig.util.root_pattern("mod.ts", "mod.js")
		-- }

		lspconfig.prosemd_lsp.setup {
			on_attach = custom_attach,
			capabilities = capabilities,
			filetypes = { "markdown" }
		}

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
	end }

	-- lspconfig (with mason)
	use {
		{ "williamboman/mason.nvim",
			requires = { "neovim/nvim-lspconfig" },
			config = function() require("mason").setup() end
		},
		{ "williamboman/mason-lspconfig.nvim",
			requires = { "williamboman/mason.nvim" },
			config = function() require("mason-lspconfig").setup {
					ensure_installed = { 'prosemd' },
					automatic_installation = true,
				}
			end
		}
	}

	-- treesitter
	use { 'nvim-treesitter/nvim-treesitter',
		requires = { "p00f/nvim-ts-rainbow" },
		config = function()
			require 'nvim-treesitter.configs'.setup {
				ensure_installed = { "rust", "bash", "yaml", "typescript", "javascript", "markdown", "lua" },
				auto_install = true, -- install missing when entering buffer
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
				matchup = {
					enable = true, -- mandatory, false will disable the whole extension
					disable = {}, -- optional, list of language that will be disabled
				},
			}
		end }

	use { 'nvim-telescope/telescope.nvim',
		config = function()
			require('telescope').setup {
				defaults = {
					prompt_prefix = "   ",
					selection_caret = "  ",
					entry_prefix = "  ",
					initial_mode = "insert",
					selection_strategy = "reset",
					sorting_strategy = "ascending",
					layout_strategy = "horizontal",
					prompt_title = "",
					results_title = "",
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
					winblend = 10,
					-- border = false,
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

			-- telescope keymaps
			vim.keymap.set("n", "go", function()
				require('telescope.builtin').find_files()
			end)
			vim.keymap.set("n", "<leader>o", function()
				require('telescope.builtin').find_files(
					require('telescope.themes').get_dropdown()
				)
			end)
			vim.keymap.set("n", 'tb', '<cmd>Telescope buffers<cr>')
			vim.keymap.set("n", 'tc', '<cmd>Telescope commands<cr>')
			vim.keymap.set("n", '<leader>f', function()
				require('telescope.builtin').find_files { path_display = { "truncate" }, prompt_title = "", preview_title = "" }
			end)
			vim.keymap.set("n", 'to', '<cmd>Telescope oldfiles<cr>')
			vim.keymap.set("n", '<leader>c', '<cmd>Telescope commands<cr>')
			vim.keymap.set("n", '<leader>ch', '<cmd>Telescope command_history<cr>')
			vim.keymap.set("n", '<leader>g', '<cmd>Telescope live_grep<cr>')
			vim.keymap.set("n", 'ts', '<cmd>Telescope spell_suggest<cr>')
			vim.keymap.set('n', 'td', '<Cmd>Telescope diagnostics<cr>')
			vim.keymap.set('n', 'tgb', '<Cmd>Telescope git_branches<cr>')
			vim.keymap.set('n', 'tgc', '<Cmd>Telescope git_bcommits<cr>')
			vim.keymap.set('n', 'tgd', '<Cmd>Telescope git_status<cr>')
			vim.keymap.set('n', 'tk', '<Cmd>Telescope keymaps<cr>')
			vim.keymap.set('n', 'tld', '<Cmd>Telescope lsp_definitions<cr>')
			vim.keymap.set('n', 'tli', '<Cmd>Telescope lsp_implementations<cr>')
			vim.keymap.set('n', 'tls', '<Cmd>Telescope lsp_document_symbols<cr>')
			vim.keymap.set('n', 'tlw', function()
				require('telescope.builtin').lsp_workspace_symbols { path_display = "hidden", prompt_title = "", preview_title = "" }
			end)
			vim.keymap.set('n', 'tlf', function()
				require('telescope.builtin').lsp_document_symbols { symbols = "function", prompt_title = "", preview_title = "",
					borderchars = { " ", " ", " ", " ", " ", " ", " ", " " } }
			end)

			vim.keymap.set('n', 'tm', '<Cmd>Telescope marks<cr>')
			vim.keymap.set('n', 'tr', '<Cmd>Telescope resume<cr>')
			vim.keymap.set('n', 'tt', '<Cmd>TodoTelescope<cr>')
			vim.keymap.set('n', 'tz', '<Cmd>Telekasten find_notes<cr>')
			vim.keymap.set("n", "ts", function()
				require("luasnip.loaders.from_snipmate").lazy_load()
				require('telescope').load_extension('luasnip')
				vim.api.nvim_command('Telescope luasnip')
			end)

			local prompt_bg = "#000000"
			local results_bg = "#000000"
			local preview_bg = "#000000"

			vim.api.nvim_set_hl(0, "TelescopeBorder", { fg = prompt_bg, bg = prompt_bg })

			vim.api.nvim_set_hl(0, "TelescopePromptBorder", { fg = prompt_bg, bg = prompt_bg })
			vim.api.nvim_set_hl(0, "TelescopePromptNormal", { fg = "White", bg = prompt_bg })
			vim.api.nvim_set_hl(0, "TelescopePromptPrefix", { fg = "White" }) -- the icon
			vim.api.nvim_set_hl(0, "TelescopePromptTitle", { fg = prompt_bg, bg = prompt_bg })

			vim.api.nvim_set_hl(0, "TelescopePreviewTitle", { fg = preview_bg, bg = preview_bg })
			vim.api.nvim_set_hl(0, "TelescopePreviewBorder", { fg = preview_bg, bg = preview_bg })
			vim.api.nvim_set_hl(0, "TelescopePreviewNormal", { bg = preview_bg })

			vim.api.nvim_set_hl(0, "TelescopeResultsTitle", { fg = results_bg, bg = results_bg })
			vim.api.nvim_set_hl(0, "TelescopeResultsBorder", { fg = results_bg, bg = results_bg })
			vim.api.nvim_set_hl(0, "TelescopeResultsNormal", { bg = results_bg })
		end }

	-- use { 'vijaymarupudi/nvim-fzf' }

	-- completion TODO: clear out useless plugs, maybe custom one
	use {
		'hrsh7th/nvim-cmp',
		wants = { "LuaSnip" },
		requires = { "L3MON4D3/LuaSnip", 'hrsh7th/cmp-nvim-lsp' },
		config = function()
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

		end
	}

	-- snippy
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
		module = "telescope._extensions.luasnip"
	}

	-- inline colors
	use { 'norcalli/nvim-colorizer.lua', config = function()
		require("colorizer").setup()
	end }

	use { 'ggandor/leap.nvim', config = function()
		require('leap').set_default_keymaps()
	end }

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
	end } -- distraction-free / zen mode (VIMSCRIPT)

	-- Lua
	use {
		"folke/zen-mode.nvim",
		config = function()
			require("zen-mode").setup {}
		end
	}

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

	-- surround completion
	-- use { "numToStr/Surround.nvim" }
	-- use {
	-- 	"ur4ltz/surround.nvim",
	-- 	config = function()
	-- 		require "surround".setup { mappings_style = "sandwich" }
	-- 	end
	-- }

	-- inline diagnostics
	use({
		"https://git.sr.ht/~whynothugo/lsp_lines.nvim",
		config = function()
			require("lsp_lines").setup()
		end,
	})

	-- lsp progress
	use {
		'j-hui/fidget.nvim',
		requires = { 'neovim/nvim-lspconfig' },
		config = function() require("fidget").setup {} end
	}

	-- #lsp
	-- cargo
	use {
		"saecki/crates.nvim",
		event = { "BufRead Cargo.toml" },
		requires = { "nvim-lua/plenary.nvim" },
		config = function()
			require('crates').setup {}
		end,
	}

	-- null lsp
	use {
		"jose-elias-alvarez/null-ls.nvim",
		requires = { "nvim-lua/plenary.nvim" },
		config = function()
			local null_ls = require("null-ls")
			null_ls.setup {
				sources = {
					null_ls.builtins.code_actions.gitsigns,
				}
			}
		end,
	}

	use { 'simrat39/rust-tools.nvim', config = function()
		require('rust-tools').setup({
			tools = {
				autoSetHints = true,
				runnables = {
					use_telescope = true
				},
				inlay_hints = {
					show_parameter_hints = true,
					-- parameter_hints_prefix = "",
					-- other_hints_prefix = "",
				},
			},

			-- see https://github.com/neovim/nvim-lspconfig/blob/master/CONFIG.md#rust_analyzer
			server = {
				--on_attach = custom_attach,
				settings = {
					-- to enable rust-analyzer settings visit:
					-- https://github.com/rust-analyzer/rust-analyzer/blob/master/docs/user/generated_config.adoc
					["rust-analyzer"] = {
						-- hover = {
						-- },
						checkOnSave = {
							enable = true,
							command = "clippy",
							features = 'all',
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
	end }

	-- formatting
	use { 'lukas-reineke/lsp-format.nvim', config = function()
		require("lsp-format").setup {}
	end }

	-- use { "lukas-reineke/indent-blankline.nvim", config = function()
	-- 	require("indent_blankline").setup({
	-- 		show_current_context = true,
	-- 		show_current_context_start = true,
	-- 		filetype_exclude = { "neo-tree", "help", "floaterm", "SidebarNvim", "" },
	-- 	})
	-- end }

	-- better lsp ui
	use { "glepnir/lspsaga.nvim",
		requires = { 'neovim/nvim-lspconfig' },
		config = function()
			local lsp_saga = require('lspsaga')

			vim.keymap.set("n", "<leader>lf", "<cmd>Lspsaga lsp_finder<CR>")
			vim.keymap.set("n", "<leader>la", "<cmd>Lspsaga code_action<CR>")
			vim.keymap.set("n", "<leader>lr", "<cmd>Lspsaga rename<CR>")
			vim.keymap.set("n", "<leader>ld", "<cmd>Lspsaga preview_definition<CR>")
			vim.keymap.set("n", 'K', '<cmd>Lspsaga hover_doc<CR>')
			vim.keymap.set("n", "\\", '<cmd>LSoutlineToggle<CR>', { silent = true })
			vim.keymap.set("n", ']d', '<cmd>Lspsaga diagnostic_jump_next<cr>')
			vim.keymap.set("n", '[d', '<cmd>Lspsaga diagnostic_jump_prev<cr>')

			lsp_saga.init_lsp_saga {
				show_outline = {
					saga_winblend = 30,
					jump_key = '<CR>',
				}
			}
		end }

	-- lua formatting
	use { "ckipp01/stylua-nvim", ft = { 'lua' } }

	-- highlight TODO comments
	use {
		"folke/todo-comments.nvim",
		requires = "nvim-lua/plenary.nvim",
		config = function()
			require('todo-comments').setup {}
		end
	}

	-- NOTE: vimwiki is vimscript...
	-- use { 'chipsenkbeil/vimwiki.nvim', config = function()
	-- end }
	use { 'ElPiloto/telescope-vimwiki.nvim', config = function()
		require('telescope').load_extension('vimwiki')
		vim.keymap.set("n", 'tw', '<cmd>Telescope vimwiki<cr>')
	end }
	use { 'vimwiki/vimwiki', config = function()
		vim.keymap.set("n", "gi", "<Cmd>VimwikiIndex<CR>") -- TODO: lsp variants (eg rust will look for lib.rs, main.rs etc)
		vim.keymap.set("n", "gw", "<Cmd>VimwikiGoto ")

		vim.api.nvim_create_autocmd("FileType", { pattern = "markdown", callback = function()
			vim.keymap.set("n", "gt", "<Cmd>VimwikiGoto Tasks<CR>")
		end })

		vim.g.vimwiki_list = {
			{
				path = '~/wiki/',
				syntax = 'markdown',
				ext = '.md'
			}
		}
	end }

	-- side scrollbar with git support
	use { "petertriho/nvim-scrollbar",
		--event = "VimEnter",
		config = function()
			require('scrollbar').setup()
		end
	}

	-- make background highlight groups transparent
	use { 'xiyaowong/nvim-transparent',
		config = function()
			require("transparent").setup {
				enable = true,
				extra_groups = { "NvimTreeNormal", "ModeMsg" },
			}
		end
	}
end)

--
-- SETTINGS
--
-- sudo command: :w !sudo tee %
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
vim.opt.title = true -- set window title
vim.opt.titlestring = vim.fn.fnamemodify(vim.fn.getcwd(), ":~:t")
vim.opt.cursorline = true -- highlight the current line
vim.opt.expandtab = false -- insert spaces when tab is pressed
vim.opt.foldlevelstart = 99
vim.opt.hidden = false -- switch buffer without unloading+saving them
vim.opt.hlsearch = false -- highlight all matches on previous search pattern
vim.opt.ignorecase = true -- ignore case when searching
vim.opt.laststatus = 3 -- 2 = local, 3 = global statusline (neovim 0.7+)
vim.opt.lazyredraw = true -- faster macros (force update with :redraw)
vim.opt.mouse = "a" -- allow the mouse to be used in neovim
vim.wo.number = false -- show numbered lines
vim.wo.relativenumber = false -- set relative numbered lines
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
vim.opt.showmode = false
vim.opt.regexpengine = 2

vim.api.nvim_set_option('tabstop', 2)

require 'floats'

-- local ICONS = {
-- 	"file" = ""
-- }

-- https://gist.github.com/romainl/379904f91fa40533175dfaec4c833f2f
-- vim.cmd("colorscheme {{colorscheme}}");

vim.api.nvim_create_autocmd("VimEnter", { pattern = "*", callback = function()
	vim.api.nvim_set_hl(0, "EndOfBuffer", { fg = "#444444" })
	vim.api.nvim_set_hl(0, "TabLineFill", { bg = "None" })
	vim.api.nvim_set_hl(0, "Title", { fg = "#CCFF00" })
	vim.api.nvim_set_hl(0, "VimwikiHeaderChar", { fg = "#44FF00" })
	vim.api.nvim_set_hl(0, "VimwikiLink", { fg = "#44FFFF" })
	vim.api.nvim_set_hl(0, "LineNr", { fg = "#222222" }) -- active
	vim.api.nvim_set_hl(0, "StatusLine", {}) -- active
	vim.api.nvim_set_hl(0, "StatusLineNC", {}) -- inactive
end })
-- vim.opt.tabline = "%!render_tabline()"

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
vim.keymap.set("n", "<C-]>", "i<C-t><C-f><Esc>")
vim.keymap.set("v", "<C-]>", "<Esc><C-]>")

vim.keymap.set("i", "<C-[>", "<C-d>")
vim.keymap.set("n", "<C-[>", "i<C-d><C-f><Esc>")
vim.keymap.set("v", "<C-[>", "<Esc><C-[>")

vim.keymap.set("n", "gd", function() vim.lsp.buf.definition() end)
vim.keymap.set("n", "gD", function() vim.lsp.buf.declaration() end)
vim.keymap.set("n", "gr", function() vim.lsp.buf.references() end)

-- use ; for commands instead of :
vim.keymap.set("n", ";", ":")
-- vim.keymap.set("n", "<Space>", ":")

-- go back
vim.keymap.set('n', '<bs>', ':edit #<cr>', { silent = true })

-- #colors
-- #00FF99 #FF00CC #FFFF00 #00CCFF
--
vim.cmd('syntax on')
vim.cmd("hi WinSeparator guifg=none"); -- I think this is the split column
vim.cmd("hi TodoBgTODO guibg=#FFFF00 guifg=black");
vim.cmd("hi TodoFgTODO guifg=#FFFF00");
vim.cmd("hi DiagnosticVirtualTextHint guifg=#F0F0AA")
vim.cmd("hi DiagnosticVirtualTextError guifg=#F02282")

vim.opt.fillchars = {
	vert = " ",
	fold = "⠀",
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
-- EVENTS
--

-- on document write
vim.api.nvim_create_autocmd("BufWrite", { pattern = "*", callback = function()
	vim.cmd [[%s/\s\+$//e]] -- remove trailing whitespace
	vim.cmd [[%s/\n\+\%$//e]] -- remove trailing newlines
end })

-- -- when opening vim
-- vim.api.nvim_create_autocmd("VimEnter", { pattern = "*", callback = function()
-- 	-- if no args are passed
-- 	if vim.fn.argc() == 0 then
-- 		vim.cmd "enew"
-- 		vim.cmd "setlocal bufhidden=wipe buftype=nofile nocursorcolumn nocursorline nolist nonumber noswapfile norelativenumber"
-- 		vim.cmd([[call append('$', "")]])
-- 	end
-- end })

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
vim.keymap.set("n", "<leader>n", function()
	vim.wo.relativenumber = false -- turn off line numbers
	vim.wo.number = false
end)
--
-- split navigation
vim.keymap.set("n", "<C-j>", "<C-w><C-j>")
vim.keymap.set("n", "<C-k>", "<C-w><C-k>")
vim.keymap.set("n", "<C-l>", "<C-w><C-l>")
vim.keymap.set("n", "<C-h>", "<C-w><C-h>")
vim.keymap.set("n", "<C-s>", "<Cmd>write<CR>");
vim.keymap.set({ "v", "i" }, "<C-s>", "<Esc><Cmd>write<CR>");
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
-- move lines up and down in visual mode
vim.keymap.set("x", "K", ":move '<-2<CR>gv-gv")
vim.keymap.set("x", "J", ":move '>+1<CR>gv-gv")
--
-- quote quickly
--vim.keymap.set("i", '<leader>"', '<Esc>viw<Esc>a"<Esc>bi"<Esc>leli')
vim.keymap.set("v", '"', '<Esc>`<i"<Esc>`>ea"<Esc>')
-- substitute shortcut
-- vim.keymap.set("n", "S", ":%s//g<Left><Left>")
-- vim.keymap.set("v", "S", ":s//g<Left><Left>")
-- more reachable line start/end
vim.keymap.set("n", "H", "^")
vim.keymap.set("n", "L", "$")

--
-- STATUSLINE
--
-- get(b:,'gitsigns_status','')
-- local stl = {
-- 	-- ' %{getcwd()}',
-- 	' %{pathshorten(expand("%:p"))}',
-- 	-- ' %{fnamemodify(expand("%"), ":~:.")}', -- current file
-- 	-- ' %{pathshorten(expand("%"), ":~:.")}',
-- 	'%=',
-- 	-- '  %{FugitiveStatusline()}',
-- 	--' %M', ' %y', ' %r'
-- }

local function lsp_connections()
	local status = ''
	local ids = vim.lsp.buf_get_clients(0)
	vim.cmd "highlight LspIcon guifg=#00DD88"
	for _, client in ipairs(ids) do
		status = status .. "  %#LspIcon#" .. client.name
	end
	return status
end

function StatusLine()
	local filetype = vim.api.nvim_buf_get_option(0, 'filetype')
	return table.concat {
		vim.fn.fnamemodify(vim.fn.expand("%"), ":."), -- project directory
		"%=",
		filetype,
		lsp_connections()
	}
end

vim.opt.statusline = "%!v:lua.StatusLine()"
-- vim.api.nvim_create_autocmd("VimEnter", { pattern = "*", callback = function()
-- end })

--
-- TABLINE
--
vim.opt.showtabline = 2 -- show the global tab line at the top of neovim
local function git_branch()
	local git_info = vim.b.gitsigns_status_dict
	if git_info then
		return " " .. git_info.head
	else
		return ""
	end
end

function TabLine()
	--vim.cmd "highlight PWD guifg=white guibg=#222222"
	return table.concat {
		"%#PWD#",
		vim.fn.fnamemodify(vim.fn.getcwd(), ":~"), -- project directory
		"%#Normal#",
		"%=",
		git_branch(),
	}
end

vim.opt.tabline = "%!v:lua.TabLine()"

--
-- SESSIONS
--
-- local session_dir = vim.fn.stdpath('data') .. '/sessions/'
-- vim.keymap.set("n", '<leader>mks', ':mks! ' .. session_dir)
-- vim.keymap.set("n", '<leader>lds', ':%bd | so ' .. session_dir)

-- on markdown
-- vim.api.nvim_create_autocmd("FileType", { pattern = "markdown", callback = function()
-- 	vim.opt.autowriteall = true -- ensure write upon leaving a page
-- 	vim.opt.wrap = true -- display lines as one long line
-- end })
--
-- vim.api.nvim_create_autocmd("FileType", { pattern = "rust", callback = function()
-- 	vim.keymap.set("n", "gi", function()
-- 		vim.cmd ':edit src/lib.rs'
-- 	end)
-- end })

vim.keymap.set("n", "<leader>ri", function()
	vim.cmd ':edit src/lib.rs'
end)
