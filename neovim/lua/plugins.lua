-- PLUGINS
--
--   PackerCompile: compile plugins
--   PackerClean: remove unused plugs
--   PackerInstall: add new plugins
--   PackerUpdate: PackerClean, PackerUpdate, PackerInstall
--   PackerSync: PackerUpdate, PackerCompile
--
-- autoinstall packer:
local packer_exists = pcall(require, "packer")
if not packer_exists then
	local install_path = vim.fn.stdpath("data") .. "/site/pack/packer/start/packer.nvim"
	if vim.fn.empty(vim.fn.glob(install_path)) > 0 then
		print("downloading packer...")
		vim.fn.system({ "git", "clone", "--depth", "1", "https://github.com/wbthomason/packer.nvim", install_path })
		vim.cmd 'packadd packer.nvim'
	end
end

vim.cmd 'packadd packer.nvim' -- only required if packer is opt

require('packer').startup(function(use)
	use 'wbthomason/packer.nvim'

	use {
		require 'packs.comments',
		require 'packs.themes',
		require 'packs.leader',
	}

	-- lspconfig (with mason)
	use { "williamboman/mason.nvim", config = function()
		require("mason").setup {}
	end }

	-- better % motion using treesiter - vimscript

	-- flowstate reading
	-- https://github.com/nullchilly/fsread.nvim
	use { "nullchilly/fsread.nvim", ft = { 'markdown', 'text', 'vimwiki' } }

	-- use {'glepnir/template.nvim', config = function()
	-- 	local temp = require('template')
	-- 	temp.temp_dir = '~/.config/nvim/templates' -- template directory
	-- 	temp.author   = 'monomadic' -- your name
	-- 	temp.email    = 'monomadic@localhost' -- email address
	-- end}

	-- notifications
	-- use 'rcarriga/nvim-notify'

	-- convenience file operations (new, rename, etc)
	use { "chrisgrieser/nvim-genghis",
		requires = { "stevearc/dressing.nvim", "rcarriga/nvim-notify" },
		config = function()
			local keymap = vim.keymap.set
			local genghis = require("genghis")
			keymap("n", "<leader>fp", genghis.copyFilepath)
			-- keymap("n", "<leader>fn", genghis.copyFilename)
			keymap("n", "<leader>fx", genghis.chmodx)
			keymap("n", "<leader>fr", genghis.renameFile)
			keymap("n", "<leader>fn", genghis.createNewFile)
			-- keymap("n", "<leader>fd", genghis.duplicateFile)
			keymap("n", "<leader>fd", function() genghis.trashFile { trashLocation = "your/path" } end) -- default: '$HOME/.Trash'.
			keymap("x", "<leader>x", genghis.moveSelectionToNewFile)
		end }


	use { "williamboman/mason-lspconfig.nvim",
		requires = { "neovim/nvim-lspconfig" },
		after = "mason.nvim",
		ft = "lua",
		config = function()
			require("mason-lspconfig").setup {
				-- ensure_installed = { 'sumneko_lua' },
				--automatic_installation = true,
			}
			-- require('lspconfig').sumneko_lua.setup {}
		end
	}


	use 'andymass/vim-matchup'


	-- https://github.com/vijaymarupudi/nvim-fzf
	use { 'vijaymarupudi/nvim-fzf' }
	-- vim.keymap.set("n", 'tb', function ()
	-- end)

	use { 'ibhagwan/fzf-lua',
		requires = { 'nvim-tree/nvim-web-devicons' },
		config = function()
			vim.keymap.set('n', '<c-P>', function()
				require('fzf-lua').files()
			end)
		end
	}

	-- -- completion
	-- use {
	-- 	'hrsh7th/nvim-cmp',
	-- 	wants = { "LuaSnip" },
	-- 	requires = { "L3MON4D3/LuaSnip", 'hrsh7th/cmp-nvim-lsp' },
	-- 	config = function()
	-- 		local cmp = require('cmp')
	-- 		cmp.setup {
	-- 			sources = {
	-- 				{ name = 'nvim_lsp' },
	-- 				{ name = 'snippy' },
	-- 			},
	-- 			preselect = cmp.PreselectMode.None,
	-- 			snippet = {
	-- 				expand = function(args)
	-- 					require('luasnip').lsp_expand(args.body)
	-- 				end,
	-- 			},
	-- 			mapping = {
	-- 				['<CR>'] = cmp.mapping.confirm({
	-- 					behavior = cmp.ConfirmBehavior.Replace,
	-- 					select = false, -- false = only complete if an item is actually selected
	-- 				}),
	-- 				['<Tab>'] = function(fallback)
	-- 					if cmp.visible() then
	-- 						cmp.select_next_item()
	-- 					else
	-- 						fallback()
	-- 					end
	-- 				end,
	-- 				['<S-Tab>'] = function(fallback)
	-- 					if cmp.visible() then
	-- 						cmp.select_prev_item()
	-- 					else
	-- 						fallback()
	-- 					end
	-- 				end,
	-- 			},
	-- 		}
	--
	-- 	end
	-- }
	--

	-- -- snippy
	-- use { 'dcampos/nvim-snippy', config = function()
	-- 	require('snippy').setup({
	-- 		mappings = {
	-- 			is = {
	-- 				['<Tab>'] = 'expand_or_advance',
	-- 				['<S-Tab>'] = 'previous',
	-- 			},
	-- 			nx = {
	-- 				['<leader>x'] = 'cut_text',
	-- 			},
	-- 		},
	-- 	})
	-- 	local mappings = require('snippy.mapping')
	-- 	vim.keymap.set('i', '<Tab>', mappings.expand_or_advance('<Tab>'))
	-- 	vim.keymap.set('s', '<Tab>', mappings.next('<Tab>'))
	-- 	vim.keymap.set({ 'i', 's' }, '<S-Tab>', mappings.previous('<S-Tab>'))
	-- 	vim.keymap.set('x', '<Tab>', mappings.cut_text, { remap = true })
	-- 	vim.keymap.set('n', 'g<Tab>', mappings.cut_text, { remap = true })
	-- 	vim.keymap.set('n', '<C-g>', '<Cmd>LazyGit<CR>')
	-- end }

	-- use { 'honza/vim-snippets' }
	-- use { 'dcampos/cmp-snippy' }
	-- -- for luasnip and cmp
	-- use { 'saadparwaiz1/cmp_luasnip' }
	-- use {
	-- 	"benfowler/telescope-luasnip.nvim",
	-- 	module = "telescope._extensions.luasnip"
	-- }

	-- inline colors
	use { 'norcalli/nvim-colorizer.lua', config = function()
		require("colorizer").setup()
	end }

	-- color picker
	-- nvim-colortils/colortils.nvim

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

	-- surround completion
	-- use { "numToStr/Surround.nvim" }
	-- use {
	-- 	"ur4ltz/surround.nvim",
	-- 	config = function()
	-- 		require "surround".setup { mappings_style = "sandwich" }
	-- 	end
	-- }

	-- lsp progress
	use {
		'j-hui/fidget.nvim',
		-- requires = { 'neovim/nvim-lspconfig' },
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

	-- async formatting
	-- https://github.com/lukas-reineke/lsp-format.nvim
	use { 'lukas-reineke/lsp-format.nvim', config = function()
		require("lsp-format").setup {}
	end }

	-- null-lsp: a generic lsp server providing lsp functions to neovim on behalf of various tools
	use {
		"jose-elias-alvarez/null-ls.nvim",
		requires = { "nvim-lua/plenary.nvim", "lukas-reineke/lsp-format.nvim" },
		config = function()
			local null_ls = require("null-ls")
			-- https://github.com/jose-elias-alvarez/null-ls.nvim/tree/main/lua/null-ls/builtins/formatting

			-- null_ls.register {
			-- 	name = "markdown_source",
			-- 	filetypes = { "markdown", "vimwiki" },
			-- 	sources = {
			-- 		null_ls.builtins.formatting.prettier,
			-- 		-- null_ls.builtins.diagnostics.proselint, -- prosemd is better
			-- 		-- null_ls.builtins.code_actions.proselint,
			-- 	},
			-- }

			-- null_ls.register {
			-- 	name = "rustfmt",
			-- 	filetypes = { "rust" },
			-- 	sources = { formatting.rustfmt },
			-- }

			null_ls.setup {
				sources = {
					null_ls.builtins.formatting.taplo, -- cargo install taplo-cli --locked
					null_ls.builtins.formatting.prettier.with({
						filetypes = { "html", "json", "yaml", "markdown", "graphql", "solidity" },
					}),
					null_ls.builtins.diagnostics.jsonlint, -- brew install jsonlint
					null_ls.builtins.hover.dictionary.with {
						filetypes = { "markdown", "vimwiki" }
					}, -- markdown spellcheck
				},
				on_attach = function(client, bufnr)
					require("lsp-format").on_attach(client)

					-- disable this dumb mapping
					-- local augroup = vim.api.nvim_create_augroup("LspFormatting", {})
					-- if client.supports_method("textDocument/formatting") then
					-- 	vim.api.nvim_clear_autocmds({ group = augroup, buffer = bufnr })
					-- 	vim.api.nvim_create_autocmd("BufWritePre", {
					-- 		group = augroup,
					-- 		buffer = bufnr,
					-- 		callback = function()
					-- 			-- on 0.8, you should use vim.lsp.buf.format({ bufnr = bufnr }) instead
					-- 			vim.lsp.buf.formatting_sync()
					-- 		end,
					-- 	})
					-- end
				end,
			}
		end,
	}

	-- lsp naviation
	-- https://github.com/DNLHC/glance.nvim
	use({
		"dnlhc/glance.nvim",
		config = function()
			require('glance').setup({
				winbar = { enable = true }
			})
			vim.keymap.set("n", "gr", "<CMD>Glance references<CR>")
			vim.keymap.set("n", "gD", "<CMD>Glance definitions<CR>")
			vim.keymap.set("n", "gY", "<CMD>Glance type_definitions<CR>")
			vim.keymap.set("n", "gM", "<CMD>Glance implementations<CR>")
		end,
	})

	use { 'jose-elias-alvarez/typescript.nvim',
		ft = 'typescript',
		config = function()
			require("typescript").setup({
				-- disable_commands = false, -- prevent the plugin from creating Vim commands
				debug = false, -- enable debug logging for commands
				go_to_source_definition = {
					fallback = true, -- fall back to standard LSP definition on failure
				},
				server = {
					on_attach = require("lsp-format").on_attach
				},
			})
		end }

	-- lsp navigation plug
	-- https://github.com/ray-x/navigator.lua
	-- use({
	-- 	'ray-x/navigator.lua',
	-- 	requires = {
	-- 		{ 'ray-x/guihua.lua', run = 'cd lua/fzy && make' },
	-- 		{ 'neovim/nvim-lspconfig' },
	-- 	},
	-- 	config = function()
	-- 		require('navigator').setup({
	-- 			mason = true,
	-- 			icons = {
	-- 				icons = false,
	-- 				code_action_icon = '',
	-- 				code_lens_action_icon = " ",
	-- 				diagnostic_err = 'E',
	-- 				diagnostic_hint = [[!]],
	-- 				doc_symbols = '',
	-- 				diagnostic_head = '',
	-- 				diagnostic_head_severity_1 = " ",
	-- 				diagnostic_head_severity_2 = " ",
	-- 				diagnostic_head_severity_3 = " ",
	-- 				diagnostic_head_description = "",
	-- 				diagnostic_virtual_text = " ",
	-- 				diagnostic_file = " ",
	-- 				--
	-- 				-- Values
	-- 				value_changed = " ",
	-- 				value_definition = "𤋮 ",
	-- 				side_panel = {
	-- 					section_separator = '',
	-- 					line_num_left = '',
	-- 					line_num_right = '',
	-- 					inner_node = '├○',
	-- 					outer_node = '╰○',
	-- 					bracket_left = '⟪',
	-- 					bracket_right = '⟫',
	-- 				},
	-- 				-- Treesitter
	-- 				match_kinds = {
	-- 					var = ' ',
	-- 					method = 'ƒ ',
	-- 					['function'] = ' ',
	-- 					parameter = "  ",
	-- 					associated = "  ",
	-- 					namespace = "  ",
	-- 					type = "  ",
	-- 					field = " ﰠ "
	-- 				},
	-- 				treesitter_defult = " "
	-- 			},
	-- 			lsp = {
	-- 				enable = true,
	-- 				format_on_save = true,
	-- 				format_options = { async = true },
	-- 				diagnostic_virtual_text = false,
	-- 				diagnostic_update_in_insert = false,
	-- 				display_diagnostic_qf = false,
	-- 				diagnostic = {
	-- 					virtual_text = false,
	-- 					update_in_insert = false,
	-- 				},
	-- 			},
	-- 		})
	-- 	end
	-- })

	-- rust-tools: a rust lsp server specific to rust
	-- https://github.com/simrat39/rust-tools.nvim
	use {
		'simrat39/rust-tools.nvim',
		ft = 'rust',
		requires = { 'neovim/nvim-lspconfig', 'jubnzv/virtual-types.nvim', 'nvim-lua/plenary.nvim', 'mfussenegger/nvim-dap' }, -- last 2 for debug
		config = function()
			local rust_tools = require('rust-tools')

			rust_tools.setup({
				tools = {
					autosethints = true,
					runnables = {
						use_telescope = true
					},
					inlay_hints = {
						show_parameter_hints = true,
						-- parameter_hints_prefix = "",
						-- other_hints_prefix = "",
					},
					hover_actions = { auto_focus = false },
				},

				-- see https://github.com/neovim/nvim-lspconfig/blob/master/CONFIG.md#rust_analyzer
				server = {
					on_attach = function(client, bufnr)
						require("lsp-format").on_attach(client)

						vim.keymap.set("n", "K", rust_tools.hover_actions.hover_actions, { buffer = bufnr })
						vim.keymap.set("n", "<leader>a", "<cmd>Lspsaga code_action<CR>", { buffer = bufnr })
						-- vim.keymap.set("n", "<leader>a", rust_tools.code_action_group.code_action_group, { buffer = bufnr })
						vim.keymap.set('n', 'gP', rust_tools.open_cargo_toml.open_cargo_toml, { buffer = bufnr })
						vim.keymap.set('n', 'gp', rust_tools.parent_module.parent_module, { buffer = bufnr })
						-- vim.keymap.set('v', '<C-j>', rust_tools.move_item.move_item(false), { buffer = bufnr }) -- down
						-- vim.keymap.set('v', '<C-k>', rust_tools.move_item.move_item(true), { buffer = bufnr }) -- up
						-- vim.keymap.set("n", "gi", function()
						-- 	vim.cmd ':edit src/lib.rs'
						-- end)
						require("virtualtypes").on_attach(client, bufnr)
					end,
					settings = {
						-- to enable rust-analyzer settings visit:
						-- https://github.com/rust-analyzer/rust-analyzer/blob/master/docs/user/generated_config.adoc
						["rust-analyzer"] = {
							lens = { enable = true },
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
		end
	}

	-- use { "lukas-reineke/indent-blankline.nvim", config = function()
	-- 	require("indent_blankline").setup({
	-- 		show_current_context = true,
	-- 		show_current_context_start = true,
	-- 		filetype_exclude = { "neo-tree", "help", "floaterm", "SidebarNvim", "" },
	-- 	})
	-- end }

	-- better lsp ui
	use { "glepnir/lspsaga.nvim",
		-- requires = { 'neovim/nvim-lspconfig' },
		ft = { 'rust', 'typescript', 'javascript', 'lua' },
		config = function()
			local lsp_saga = require('lspsaga')

			vim.keymap.set("n", "gR", "<cmd>Lspsaga lsp_finder<CR>")
			vim.keymap.set("n", "<leader>a", "<cmd>Lspsaga code_action<CR>")
			vim.keymap.set("n", "<leader>r", "<cmd>Lspsaga rename<CR>")
			vim.keymap.set("n", "<leader>d", "<cmd>Lspsaga preview_definition<CR>")
			vim.keymap.set("n", 'K', '<cmd>Lspsaga hover_doc<CR>')
			vim.keymap.set("n", "|", '<cmd>LSoutlineToggle<CR>', { silent = true })
			vim.keymap.set("n", ']d', '<cmd>Lspsaga diagnostic_jump_next<cr>')
			vim.keymap.set("n", '[d', '<cmd>Lspsaga diagnostic_jump_prev<cr>')
			-- vim.keymap.set("n", '\\', '<cmd>Lspsaga open_floaterm<cr>')
			-- vim.keymap.set("t", '\\', '<cmd>Lspsaga close_floaterm<cr>')

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
	use { 'ElPiloto/telescope-vimwiki.nvim', ft = { "markdown", "vimwiki" }, config = function()
		require('telescope').load_extension('vimwiki')
		vim.keymap.set("n", 'tw', '<cmd>Telescope vimwiki<cr>')
	end }
	use { 'vimwiki/vimwiki', ft = { "markdown", "vimwiki" }, config = function()
		--vim.keymap.set("n", "gi", "<Cmd>VimwikiIndex<CR>") -- TODO: lsp variants (eg rust will look for lib.rs, main.rs etc)
		vim.keymap.set("n", "gw", "<Cmd>VimwikiGoto ")
		vim.cmd 'nmap <Leader>nl <Plug>VimwikiToggleListItem' -- unset this shit, it conflicts with term. see also: g:vimwiki_key_mappings

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


	use { 'ray-x/lsp_signature.nvim',
		config = function()
			require "lsp_signature".setup {}
		end
	}

	use {
		require 'packs.syntax',
		require 'packs.scrollbar',
		require 'packs.autocomplete',
		require 'packs.telescope',
	}

	use { "folke/neodev.nvim",
		-- after = "nvim-lspconfig",
		ft = "lua",
		config = function()
			require("neodev").setup {
				lspconfig = false
			}
			vim.lsp.start({
				name = "neodev",
				cmd = { "lua-language-server" },
				before_init = require("neodev.lsp").before_init,
				root_dir = vim.fn.getcwd(),
				settings = { Lua = {} },
			})
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
