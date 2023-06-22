-- TODO: https://github.com/chipsenkbeil/vimwiki-rs
-- NOTE: vimwiki is vimscript...

return {
	-- use { 'chipsenkbeil/vimwiki.nvim', config = function()
	-- end }

	{ 'ElPiloto/telescope-vimwiki.nvim',
		dependencies = {
			'vimwiki/vimwiki',
			'nvim-telescope/telescope.nvim',
		},
		ft = { "markdown", "vimwiki" },
		config = function()
			require('telescope').load_extension('vimwiki')
		end },

	{ 'vimwiki/vimwiki',
		-- ft = { "markdown", "vimwiki" },
		init = function()
     vim.g.vimwiki_key_mappings = { lists = 0 }
			-- vim.g.vimwiki_key_mappings = {
			-- 	all_maps = 1,
			-- 	global = 1,
			-- 	headers = 1,
			-- 	text_objs = 1,
			-- 	table_format = 1,
			-- 	table_mappings = 0,
			-- 	lists = 0,
			-- 	links = 1,
			-- 	html = 1,
			-- 	mouse = 0,
			-- }
		end,
		config = function()
			-- vim.api.nvim_create_autocmd("FileType", { pattern = "markdown", callback = function()
			-- 	vim.keymap.set("n", "gt", "<Cmd>VimwikiGoto Tasks<CR>")
			-- end })

     -- vim.g.vimwiki_key_mappings = { all_maps = 0 }

		 local keymap = vim.keymap.set
		 keymap()

			vim.g.vimwiki_list = {
				{
					path = '~/wiki/',
					syntax = 'markdown',
					ext = '.md'
				}
			}
		end }
}
