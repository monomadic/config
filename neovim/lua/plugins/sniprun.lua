return {
	'michaelb/sniprun',
	build = 'bash ./install.sh 1',
	enabled = true,
	-- event = 'LspAttach',

	config = function()
		vim.keymap.set('n', "<leader>Rs", ":SnipRun<CR>", { desc = "file (SnipRun)", silent = true })
	end
}
