return {
	'bluz71/vim-nightfly-guicolors',
	'lunarvim/darkplus.nvim',
	'projekt0n/github-nvim-theme',
	'olimorris/onedarkpro.nvim',
	'Mofiqul/vscode.nvim',
	'notken12/base46-colors',
	{ 'NvChad/base46',
		config = function()
			local ok, base46 = pcall(require, "base46")

			if ok then
				base46.load_theme()
			end

			vim.cmd("colorscheme {{colorscheme}}");
		end
	}
}
