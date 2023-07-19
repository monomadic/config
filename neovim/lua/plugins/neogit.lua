-- magit for neovim
return {
	'NeogitOrg/neogit',
	dependencies = 'nvim-lua/plenary.nvim',
	cmd = { "Neogit" },
	config = function()
		local neogit = require('neogit')
		neogit.setup {}
	end
}
