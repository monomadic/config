-- https://github.com/vimwiki/vimwiki
-- TODO: https://github.com/chipsenkbeil/vimwiki-rs

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
			local hl = vim.api.nvim_set_hl
			hl(0, "VimwikiBold", { fg = "#FF33AA" })

			local keymap = vim.keymap.set
			keymap("n", "<leader>Wb", ":VimwikiBacklinks<CR>", { desc = "backlinks" })
			keymap("n", "<leader>Wr", ":VimwikiRenameFile<CR>", { desc = "rename" })
			keymap("n", "<leader>Wd", ":VimwikiDiaryIndex<CR>", { desc = "diary" })

			vim.g.vimwiki_list = {
				{
					path = '~/wiki/',
					template_path = '~/wiki/templates/',
					template_default = 'default',
					template_ext = '.md',
					diary_rel_path = "journal/",
					diary_index = "journal",
					syntax = 'markdown',
					ext = '.md'
				}
			}
		end }
}
