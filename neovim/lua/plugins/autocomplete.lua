return {
	'hrsh7th/nvim-cmp',

	dependencies = {
		'dcampos/nvim-snippy', -- snipmate and lsp snippets
		'dcampos/cmp-snippy', -- cmp support for snippy
		'hrsh7th/cmp-nvim-lsp', -- cmp support for LSP (needed?)
		'onsails/lspkind.nvim', -- icons for lsp
	},

	config = function()
		local snippy = require 'snippy'
		snippy.setup({
			mappings = {
				-- is = {
				-- 	['<Tab>'] = 'expand_or_advance',
				-- 	['<S-Tab>'] = 'previous',
				-- },
				-- nx = {
				-- 	['<leader>x'] = 'cut_text',
				-- },
			},
		})

		-- local mappings = require('snippy.mapping')
		-- vim.keymap.set('i', '<Tab>', mappings.expand_or_advance('<Tab>'))
		-- vim.keymap.set('i', '<C-n>', mappings.expand_or_advance('<Tab>'))
		-- vim.keymap.set('s', '<Tab>', mappings.next('<Tab>'))
		-- vim.keymap.set('s', '<C-n>', mappings.next('<Tab>'))
		-- vim.keymap.set({ 'i', 's' }, '<S-Tab>', mappings.previous('<S-Tab>'))
		-- vim.keymap.set('x', '<Tab>', mappings.cut_text, { remap = true })
		-- vim.keymap.set('n', 'g<Tab>', mappings.cut_text, { remap = true })

		local cmp = require('cmp')
		cmp.setup {
			--view = 'native',
			sources = {
				{ name = 'snippy' },
				{ name = 'nvim_lsp' },
			},
			preselect = cmp.PreselectMode.None,

			snippet = {
				expand = function(args)
					snippy.expand_snippet(args.body)
				end,
			},

			formatting = {
				format = function(entry, vim_item)
					if vim.tbl_contains({ 'path' }, entry.source.name) then
						local icon, hl_group = require('nvim-web-devicons').get_icon(entry:get_completion_item().label)
						if icon then
							vim_item.kind = icon
							vim_item.kind_hl_group = hl_group
							return vim_item
						end
					end
					return require('lspkind').cmp_format({
						with_text = true,
						-- mode = "symbol_text",
						-- menu = ({
						-- 	buffer = "Buffer",
						-- 	nvim_lsp = "LSP",
						-- 	luasnip = "LuaSnip",
						-- 	nvim_lua = "Lua",
						-- 	latex_symbols = "Latex",
						-- })
					})(entry, vim_item)
				end
			},

			mapping = cmp.mapping.preset.insert {
				['<CR>'] = cmp.mapping.confirm({
					behavior = cmp.ConfirmBehavior.Replace,
					select = false, -- false = only complete if an item is actually selected
				}),

				['<C-n>'] = function(fallback)
					if cmp.visible() then
						cmp.select_next_item()
					else
						fallback()
					end
				end,

				["<Tab>"] = cmp.mapping(function(fallback)
					if snippy.can_jump(1) then
						snippy.next()
					elseif cmp.visible() then
						cmp.select_next_item()
					else
						fallback()
					end
				end, { "i", "s" }),

				["<S-Tab>"] = cmp.mapping(function(fallback)
					if snippy.can_expand_or_advance() then
						snippy.previous()
					elseif cmp.visible() then
						cmp.select_prev_item()
					else
						fallback()
					end
				end, { "i", "s" }),

				['<C-p>'] = function(fallback)
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
