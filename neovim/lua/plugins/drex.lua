return {
	'theblob42/drex.nvim',
	cmd = { "Drex", "DrexDrawerOpen", "DrexDrawerToggle" },
	dependencies = 'kyazdani42/nvim-web-devicons', -- optional

	config = function()
		-- local utils = require('drex.utils')
		-- local elements = require('drex.elements')

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
					['<C-l>'] = { '<C-w><C-l>', {} },
					['<C-h>'] = { '<C-w><C-h>', {} },
				}
			}
		}
	end
}
