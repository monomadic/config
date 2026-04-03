-- https://github.com/vijaymarupudi/nvim-fzf
-- https://github.com/ibhagwan/fzf-lua/wiki/Advanced#preview-nvim-builtin

return {
	'ibhagwan/fzf-lua',

	dependencies = {
		'vijaymarupudi/nvim-fzf',
		'nvim-tree/nvim-web-devicons'
	},

	config = function()
		local fzf = require('fzf-lua')

		fzf.setup {
			winopts = {
				border = 'solid',
				preview = {
					delay = 20,
					hidden = 'hidden',
				}
			}
		}

		vim.keymap.set('n', '<c-P>', function()
			fzf.files {
				cmd = "fd",
				prompt = "   ",
			}
		end)
	end
}
