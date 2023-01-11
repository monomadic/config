-- treesitter
return {
	'nvim-treesitter/nvim-treesitter',

	dependencies = { "p00f/nvim-ts-rainbow", 'nvim-treesitter/nvim-treesitter-textobjects' },

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
			textobjects = {
				select = {
					enable = true,
					lookahead = true,
					-- The keymaps are defined in the configuration table, no way to get our Mapper in there !
					keymaps = {
						["af"] = "@function.outer",
						["if"] = "@function.inner",
						["ac"] = "@class.outer",
						["ic"] = "@class.inner"
					}
				}
			}
		}
	end
}
