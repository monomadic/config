	-- https://github.com/vijaymarupudi/nvim-fzf
	return { 'ibhagwan/fzf-lua',
		requires = { 'vijaymarupudi/nvim-fzf', 'nvim-tree/nvim-web-devicons' },
		config = function()
			require('fzf-lua').setup{
				winopts = {
					border = 'solid',
					preview = {
						delay = 20,
						hidden = 'hidden',
					}
				}
			}

			vim.keymap.set('n', '<c-P>', function()
				require 'fzf-lua'.files({
					cmd = "fd",
					prompt = "   ",
				})
			end)
		end
	}
