-- lsp/ts navigation
return {
	'ray-x/navigator.lua',

	dependencies = {
		{ 'ray-x/guihua.lua', run = 'cd lua/fzy && make' },
		{ 'neovim/nvim-lspconfig' },
	},
}

-- https://github.com/ray-x/navigator.lua
-- use({
-- 	'ray-x/navigator.lua',
-- 	requires = {
-- 		{ 'ray-x/guihua.lua', run = 'cd lua/fzy && make' },
-- 		{ 'neovim/nvim-lspconfig' },
-- 	},
-- 	config = function()
-- 		require('navigator').setup({
-- 			mason = true,
-- 			icons = {
-- 				icons = false,
-- 				code_action_icon = '',
-- 				code_lens_action_icon = " ",
-- 				diagnostic_err = 'E',
-- 				diagnostic_hint = [[!]],
-- 				doc_symbols = '',
-- 				diagnostic_head = '',
-- 				diagnostic_head_severity_1 = " ",
-- 				diagnostic_head_severity_2 = " ",
-- 				diagnostic_head_severity_3 = " ",
-- 				diagnostic_head_description = "",
-- 				diagnostic_virtual_text = " ",
-- 				diagnostic_file = " ",
-- 				--
-- 				-- Values
-- 				value_changed = " ",
-- 				value_definition = "𤋮 ",
-- 				side_panel = {
-- 					section_separator = '',
-- 					line_num_left = '',
-- 					line_num_right = '',
-- 					inner_node = '├○',
-- 					outer_node = '╰○',
-- 					bracket_left = '⟪',
-- 					bracket_right = '⟫',
-- 				},
-- 				-- Treesitter
-- 				match_kinds = {
-- 					var = ' ',
-- 					method = 'ƒ ',
-- 					['function'] = ' ',
-- 					parameter = "  ",
-- 					associated = "  ",
-- 					namespace = "  ",
-- 					type = "  ",
-- 					field = " ﰠ "
-- 				},
-- 				treesitter_defult = " "
-- 			},
-- 			lsp = {
-- 				enable = true,
-- 				format_on_save = true,
-- 				format_options = { async = true },
-- 				diagnostic_virtual_text = false,
-- 				diagnostic_update_in_insert = false,
-- 				display_diagnostic_qf = false,
-- 				diagnostic = {
-- 					virtual_text = false,
-- 					update_in_insert = false,
-- 				},
-- 			},
-- 		})
-- 	end
-- })
