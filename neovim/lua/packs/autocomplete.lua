return {
	'hrsh7th/nvim-cmp',
	requires = { 'dcampos/nvim-snippy', 'hrsh7th/cmp-nvim-lsp', 'dcampos/cmp-snippy' },
	config = function()
		require('snippy').setup({
			mappings = {
				is = {
					['<Tab>'] = 'expand_or_advance',
					['<S-Tab>'] = 'previous',
				},
				nx = {
					['<leader>x'] = 'cut_text',
				},
			},
		})
		local mappings = require('snippy.mapping')

		vim.keymap.set('i', '<Tab>', mappings.expand_or_advance('<Tab>'))
		vim.keymap.set('s', '<Tab>', mappings.next('<Tab>'))
		vim.keymap.set({ 'i', 's' }, '<S-Tab>', mappings.previous('<S-Tab>'))
		vim.keymap.set('x', '<Tab>', mappings.cut_text, { remap = true })
		vim.keymap.set('n', 'g<Tab>', mappings.cut_text, { remap = true })
		vim.keymap.set('n', '<C-g>', '<Cmd>LazyGit<CR>')

		local cmp = require('cmp')
		cmp.setup {
			sources = {
				{ name = 'snippy', name = 'nvim_lsp' },
			},
			preselect = cmp.PreselectMode.None,
			snippet = {
				expand = function(args)
					require('luasnip').lsp_expand(args.body)
				end,
			},
			mapping = {
				['<CR>'] = cmp.mapping.confirm({
					behavior = cmp.ConfirmBehavior.Replace,
					select = false, -- false = only complete if an item is actually selected
				}),
				['<Tab>'] = function(fallback)
					if cmp.visible() then
						cmp.select_next_item()
					else
						fallback()
					end
				end,
				['<S-Tab>'] = function(fallback)
					if cmp.visible() then
						cmp.select_prev_item()
					else
						fallback()
					end
				end,
			},
		}

	end
}
