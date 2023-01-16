-- treesitter
return {
	'nvim-treesitter/nvim-treesitter',

	dependencies = {
		"p00f/nvim-ts-rainbow",
		'nvim-treesitter/nvim-treesitter-textobjects',
		'nvim-treesitter/playground',
	},

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

			playground = {
				enable = true,
				disable = {},
				updatetime = 25, -- Debounced time for highlighting nodes in the playground from source code
				persist_queries = false, -- Whether the query persists across vim sessions
				keybindings = {
					toggle_query_editor = 'o',
					toggle_hl_groups = 'i',
					toggle_injected_languages = 't',
					toggle_anonymous_nodes = 'a',
					toggle_language_display = 'I',
					focus_language = 'f',
					unfocus_language = 'F',
					update = 'R',
					goto_node = '<cr>',
					show_help = '?',
				},
			},

			textobjects = {
				lookahead = true, -- Automatically jump forward to textobj, similar to targets.vim
				select = {
					enable = true,
					lookahead = true,
					-- The keymaps are defined in the configuration table, no way to get our Mapper in there !
					keymaps = { -- prefix v
						["af"] = "@function.outer",
						["if"] = "@function.inner",
						["ac"] = "@class.outer",
						["ic"] = "@class.inner"
					}
				},
				move = {
					enable = true,
					set_jumps = true,
					goto_next_start = {
						["]f"] = { query = "@function.outer", desc = "Next function start" },
						["]]"] = { query = "@function.outer", desc = "Next function start" },
					},
					goto_next_end = {
						["]F"] = "@function.outer",
						["]["] = "@function.outer",
					},
					goto_previous_start = {
						["[f"] = "@function.outer",
						["[["] = "@function.outer",
					},
					goto_previous_end = {
						["[F"] = "@function.outer",
						["[]"] = "@function.outer",
					},
				},
			},
			incremental_selection = {
				enable = true,
				keymaps = {
					init_selection = "vv",
					node_incremental = "v",
					scope_incremental = "<CR>",
					node_decremental = "V",
				},
			},
		}

		-- vim.keymap.set("n", "", function()
		-- 	local tsparser = vim.treesitter.get_parser()
		--
		-- end)

	end
}
