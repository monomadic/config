return {
	'saecki/crates.nvim',
	dependencies = { 'nvim-lua/plenary.nvim' },
	event = { 'BufRead Cargo.toml' },

	config = function()
		require('crates').setup()
	end
}
