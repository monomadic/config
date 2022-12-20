-- PLUGINS
--
--   PackerCompile: compile plugins
--   PackerClean: remove unused plugs
--   PackerInstall: add new plugins
--   PackerUpdate: PackerClean, PackerUpdate, PackerInstall
--   PackerSync: PackerUpdate, PackerCompile
--

-- TODO: https://github.com/MunifTanjim/nui.nvim

-- -- autoinstall packer:
-- local packer_exists = pcall(require, "packer")
-- if not packer_exists then
-- 	local install_path = vim.fn.stdpath("data") .. "/site/pack/packer/start/packer.nvim"
-- 	if vim.fn.empty(vim.fn.glob(install_path)) > 0 then
-- 		print("downloading packer...")
-- 		vim.fn.system({ "git", "clone", "--depth", "1", "https://github.com/wbthomason/packer.nvim", install_path })
-- 		vim.cmd 'packadd packer.nvim'
-- 	end
-- end
--
-- vim.cmd 'packadd packer.nvim' -- only required if packer is opt

require('packer').startup(function(use)
	-- speed up lua modules
	use { 'lewis6991/impatient.nvim', config = function()
		require('impatient')
	end }

	-- packer package manager
	use 'wbthomason/packer.nvim'

	use {
		-- require 'plugin.comments', -- commenting
		require 'plugin.themes', -- colorschemes
		require 'plugin.fzf', -- fuzzy finder
		require 'plugin.menu', -- whichkey
		-- require 'plugin.filetree', -- drex
	}

	-- better % motion using treesitter - vimscript

	-- notifications
	-- use 'rcarriga/nvim-notify'

	-- convenience file operations (new, rename, etc)
	use { "chrisgrieser/nvim-genghis",
		requires = { "stevearc/dressing.nvim", "rcarriga/nvim-notify" },
		config = function()
			local keymap = vim.keymap.set
			local genghis = require("genghis")
			keymap("n", "<leader>fc", genghis.copyFilepath, { desc = " copy path" })
			keymap("n", "<leader>fC", genghis.copyFilename, { desc = " copy filename" })
			keymap("n", "<leader>fr", genghis.renameFile, { desc = " rename" })
			keymap("n", "<leader>fn", genghis.createNewFile, { desc = " new" })
			keymap("n", "<leader>fd", genghis.duplicateFile, { desc = " duplicate" })
			keymap("n", "<leader>fx", genghis.chmodx, { desc = " chmod" })
			keymap("n", "<leader>ft", function() genghis.trashFile { trashLocation = "your/path" } end, { desc = "﬒ trash" }) -- default: '$HOME/.Trash'.
			keymap("x", "<leader>x", genghis.moveSelectionToNewFile)
		end }

	-- also see: https://github.com/VonHeikemen/lsp-zero.nvim
	-- lspconfig (with mason)
	use { "williamboman/mason.nvim", config = function()
		require("mason").setup {}
	end }

	use { "williamboman/mason-lspconfig.nvim",
		requires = { "neovim/nvim-lspconfig" },
		after = "mason.nvim",
		ft = { "lua", "solidity" },
		config = function()
			require("mason-lspconfig").setup {
				-- ensure_installed = { 'sumneko_lua' },
				--automatic_installation = true,
			}
			-- require('lspconfig').sumneko_lua.setup {}
			require('lspconfig').solidity.setup {
				-- on_attach = on_attach, -- probably you will need this.
				-- capabilities = capabilities,
				settings = {
					-- example of global remapping
					solidity = { includePath = '', remapping = { ["@OpenZeppelin/"] = 'OpenZeppelin/openzeppelin-contracts@4.6.0/' } }
				},
			}
			require 'lspconfig'.solidity_ls.setup {}
		end
	}

	use 'andymass/vim-matchup'

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

	-- show lsp progress
	use {
		'j-hui/fidget.nvim',
		config = function() require("fidget").setup {} end
	}

	-- cargo
	use {
		"saecki/crates.nvim",
		event = { "BufRead Cargo.toml" },
		requires = { "nvim-lua/plenary.nvim" },
		config = function()
			require('crates').setup()
		end
	}

	-- async formatting
	-- https://github.com/lukas-reineke/lsp-format.nvim
	use { 'lukas-reineke/lsp-format.nvim', config = function()
		require("lsp-format").setup()
	end }

	-- lsp naviation
	-- https://github.com/DNLHC/glance.nvim
	use({
		"dnlhc/glance.nvim",
		config = function()
			require('glance').setup({
				winbar = { enable = true }
			})
			vim.keymap.set("n", "gR", "<CMD>Glance references<CR>")
			vim.keymap.set("n", "gD", "<CMD>Glance definitions<CR>")
			vim.keymap.set("n", "gY", "<CMD>Glance type_definitions<CR>")
			vim.keymap.set("n", "gM", "<CMD>Glance implementations<CR>")
		end,
	})

	use { 'jose-elias-alvarez/typescript.nvim',
		ft = 'typescript',
		reqires = { 'lvimuser/lsp-inlayhints.nvim' },
		config = function()
			require("lsp-inlayhints").setup()
			require("typescript").setup({
				-- disable_commands = false, -- prevent the plugin from creating Vim commands
				debug = false, -- enable debug logging for commands
				go_to_source_definition = {
					fallback = true, -- fall back to standard LSP definition on failure
				},
				server = {
					on_attach = function(client, bufnr)
						require("lsp-format").on_attach(client, bufnr)
						require("lsp-inlayhints").on_attach(client, bufnr)
						require("lsp")
					end
				},
			})
		end }

	use { 'lvimuser/lsp-inlayhints.nvim',
		ft = 'typescript',
		config = function()
			vim.api.nvim_create_autocmd("LspAttach", {
				callback = function(args)
					if not (args.data and args.data.client_id) then
						return
					end

					local bufnr = args.buf
					local client = vim.lsp.get_client_by_id(args.data.client_id)
					require("lsp-inlayhints").on_attach(client, bufnr)
				end
			})

		end
	}

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


	-- lsp
	use {
		require('lsp.rust'),
		require('lsp.lua'),
	}

	-- highlight TODO comments
	use {
		"folke/todo-comments.nvim",
		requires = "nvim-lua/plenary.nvim",
		config = function()
			require('todo-comments').setup {}
			-- local Search = require("todo-comments.search")
			-- Search.search(function(results)
			-- 	print(vim.inspect(results))
			-- end)
		end
	}

	use { 'ray-x/lsp_signature.nvim',
		config = function()
			require "lsp_signature".setup {}
		end
	}

	use {
		require 'plugin.syntax',
		require 'plugin.scrollbar',
		require 'plugin.autocomplete',
		require 'plugin.telescope',
		require 'plugin.wiki', -- vimwiki
		require 'plugin.reading-mode', -- flowstate, zen modes
	}

	use { 'glepnir/template.nvim',
		after = "telescope",
		config = function()
			local temp    = require('template')
			temp.temp_dir = '~/.config/nvim/templates' -- template directory
			temp.author   = 'monomadic' -- your name
			temp.email    = 'monomadic@localhost' -- email address

			require("telescope").load_extension('find_template')
		end }

	use { "folke/neodev.nvim",
		-- after = "nvim-lspconfig",
		ft = "lua",
		config = function()
			require("neodev").setup {
				lspconfig = false
			}
			vim.api.nvim_create_autocmd("FileType", { pattern = "lua", callback = function()
				vim.lsp.start {
					name = "neodev",
					cmd = { "lua-language-server" },
					before_init = require("neodev.lsp").before_init,
					root_dir = vim.fn.getcwd(),
					settings = { Lua = {} },
				}
			end })
		end
	}

	-- vim.api.nvim_create_autocmd("FileType", { pattern = "solidity", callback = function()
	-- 	vim.lsp.start {
	-- 		cmd = { 'solidity-ls', '--stdio' },
	-- 		filetypes = { 'solidity' },
	-- 		root_dir = vim.fn.getcwd(),
	-- 		settings = { solidity = { includePath = '', remapping = {} } },
	-- 	}
	-- end })

	-- lsp/ts navigation
	use({
		'ray-x/navigator.lua',
		requires = {
			{ 'ray-x/guihua.lua', run = 'cd lua/fzy && make' },
			{ 'neovim/nvim-lspconfig' },
		},
	})

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
