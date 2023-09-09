return {
	'nat-418/tcl.nvim',
	dependencies = 'mfussenegger/nvim-lint',
	config = function()
		require('tcl').setup()
	end
}
