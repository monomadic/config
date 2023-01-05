-- git status in git gutter
return {
	"lewis6991/gitsigns.nvim",

	dependencies = { "nvim-lua/plenary.nvim" },

	config = function()
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
	end
}
