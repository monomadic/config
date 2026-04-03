return {
	'nat-418/tcl.nvim',
	enabled = false,
	dependencies = 'mfussenegger/nvim-lint',
	config = function()
		require('tcl').setup()
	end
}
