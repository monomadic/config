	-- treesitter
	return {
		'nvim-treesitter/nvim-treesitter',

		dependencies = { "p00f/nvim-ts-rainbow" },

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
