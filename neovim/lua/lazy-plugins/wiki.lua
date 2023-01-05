-- TODO: https://github.com/chipsenkbeil/vimwiki-rs
return {
	-- NOTE: vimwiki is vimscript...
	-- use { 'chipsenkbeil/vimwiki.nvim', config = function()
	-- end }
	{ 'ElPiloto/telescope-vimwiki.nvim',
		ft = { "markdown", "vimwiki" }, config = function()
			require('telescope').load_extension('vimwiki')
			vim.keymap.set("n", 'tw', '<cmd>Telescope vimwiki<cr>')
		end },

	{ 'vimwiki/vimwiki',
		ft = { "markdown", "vimwiki" }, config = function()
			vim.api.nvim_create_autocmd("FileType", { pattern = "markdown", callback = function()
				vim.keymap.set("n", "gt", "<Cmd>VimwikiGoto Tasks<CR>")
			end })
			vim.g.vimwiki_list = {
				{
					path = '~/wiki/',
					syntax = 'markdown',
					ext = '.md'
				}
			}
		end }
}
