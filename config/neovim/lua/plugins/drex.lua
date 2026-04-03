-- https://github.com/TheBlob42/drex.nvim
return {
	'theblob42/drex.nvim',

	cmd = {
		"Drex",
		"DrexDrawerOpen",
		"DrexDrawerToggle",
		"DrexDrawerFindFileAndFocus"
	},

	dependencies = 'nvim-tree/nvim-web-devicons', -- optional

	config = function()
		local drex = require('drex')
		local elements = require('drex.elements')
		-- open the home directory
		-- vim.keymap.set('n', '~', '<CMD>Drex ~<CR>', {})
		-- open parent DREX buffer and focus current file
		vim.keymap.set('n', '-', function()
			local path = vim.fn.expand('%:p')
			if path == '' then
				drex.open_directory_buffer() -- open at cwd
			else
				drex.open_directory_buffer(vim.fn.fnamemodify(path, ':h'))
				elements.focus_element(0, path)
			end
		end, {})

		require('drex.config').configure {
			icons = {
				file_default = "",
				dir_open = "",
				dir_closed = "",
				link = "",
				others = "",
			},
			disable_default_keybindings = false,
			keybindings = {
				['n'] = {
					['~'] = { ':Drex ~<CR>', {} },
					['<C-l>'] = { '<C-w><C-l>', {} },
					['<C-h>'] = { '<C-w><C-h>', {} },
					['<C-b>'] = { '<C-w><C-q>', {} },
				}
			}
		}
	end
}
