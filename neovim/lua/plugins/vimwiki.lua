-- https://github.com/vimwiki/vimwiki
-- config: https://github.com/vimwiki/vimwiki/blob/dev/autoload/vimwiki/vars.vim
-- https://github.com/hulufei/backlinks.nvim
-- TODO: https://github.com/chipsenkbeil/vimwiki-rs

return {
	-- use { 'chipsenkbeil/vimwiki.nvim', config = function()
	-- end }

	{
		'ElPiloto/telescope-vimwiki.nvim',
		dependencies = {
			'vimwiki/vimwiki',
			'nvim-telescope/telescope.nvim',
		},
		ft = { "markdown", "vimwiki" },
		config = function()
			require('telescope').load_extension('vimwiki')
		end
	},

	{
		'vimwiki/vimwiki',
		-- cmd = { "VimwikiIndex" },
		enabled = true,
		init = function()
			vim.g.vimwiki_key_mappings = { lists = 0 }
		end,
		config = function()
			-- local hl = vim.api.nvim_set_hl
			-- hl(0, "VimwikiHeaderChar", { fg = "#F1FA8C" })

			local keymap = vim.keymap.set
			keymap("n", "<leader>Wb", ":VimwikiBacklinks<CR>", { desc = "backlinks" })
			keymap("n", "<leader>Wr", ":VimwikiRenameFile<CR>", { desc = "rename" })
			keymap("n", "<leader>Wd", ":VimwikiDiaryIndex<CR>", { desc = "diary" })

			vim.g.vimwiki_key_mappings = { lists = 0 }

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
		end
	}
}
